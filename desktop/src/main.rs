use axon_core::node::{self, NodeConfig};
use iced::widget::{
    button, column, container, row, scrollable, space, text, text_input, toggler,
};
use iced::{
    Alignment, Background, Border, Color, Element, Fill, Font, Subscription, Task, Theme,
};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::panic::AssertUnwindSafe;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

const TEXT: Color = Color::from_rgb8(250, 250, 250);
const MUTED: Color = Color::from_rgb8(161, 161, 170);
const FAINT: Color = Color::from_rgb8(113, 113, 122);
const CARD: Color = Color::from_rgb8(16, 16, 18);
const LINE: Color = Color::from_rgb8(39, 39, 42);
const FIELD: Color = Color::from_rgb8(9, 9, 11);
const OK: Color = Color::from_rgb8(74, 222, 128);
const WARN: Color = Color::from_rgb8(250, 204, 21);
const DANGER: Color = Color::from_rgb8(248, 113, 113);

#[derive(Clone, Serialize, Deserialize)]
struct DesktopConfig {
    registry_url: String,
    secret: String,
    verify_port: String,
    relay_port: String,
    socks_port: String,
    use_tor: bool,
    use_vpn: bool,
}

impl Default for DesktopConfig {
    fn default() -> Self {
        Self {
            registry_url: "http://127.0.0.1:7400".to_string(),
            secret: "Nishanth".to_string(),
            verify_port: "8080".to_string(),
            relay_port: "8081".to_string(),
            socks_port: "1080".to_string(),
            use_tor: false,
            use_vpn: false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Stopped,
    Running,
    Stopping,
}

struct NodeSession {
    stop: Option<tokio::sync::oneshot::Sender<()>>,
    thread: Option<std::thread::JoinHandle<Option<String>>>,
}

impl NodeSession {
    fn spawn(config: NodeConfig) -> Self {
        let (stop, stop_rx) = tokio::sync::oneshot::channel();
        let thread = std::thread::Builder::new()
            .name("axon-node".into())
            .spawn(move || {
                let joined = std::panic::catch_unwind(AssertUnwindSafe(|| {
                    let runtime = tokio::runtime::Builder::new_multi_thread()
                        .enable_all()
                        .build()
                        .expect("tokio runtime");
                    runtime.block_on(async move {
                        tokio::select! {
                            _ = node::run_node(
                                &config.registry_url,
                                config.verify_port,
                                config.relay_port,
                                config.socks_port,
                                &config.secret,
                                config.use_tor,
                                config.use_vpn,
                            ) => {}
                            _ = stop_rx => {
                                log::info!("Stop requested");
                            }
                        }
                    });
                }));
                match joined {
                    Ok(()) => None,
                    Err(payload) => Some(panic_message(payload)),
                }
            })
            .expect("axon node thread");
        Self {
            stop: Some(stop),
            thread: Some(thread),
        }
    }

    fn request_stop(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
    }

    fn is_finished(&self) -> bool {
        self.thread.as_ref().is_none_or(|thread| thread.is_finished())
    }

    fn join(&mut self) -> Option<String> {
        self.request_stop();
        self.thread.take().and_then(|thread| thread.join().ok().flatten())
    }
}

struct UiLogger {
    lines: Arc<Mutex<VecDeque<String>>>,
}

impl log::Log for UiLogger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= log::Level::Debug
    }

    fn log(&self, record: &log::Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let mut lines = self.lines.lock().expect("log buffer");
        lines.push_back(format!("{:<5} {}", record.level(), record.args()));
        while lines.len() > 300 {
            lines.pop_front();
        }
    }

    fn flush(&self) {}
}

struct App {
    config: DesktopConfig,
    phase: Phase,
    notice: String,
    notice_warn: bool,
    session: Option<NodeSession>,
    lines: Arc<Mutex<VecDeque<String>>>,
    log_lines: Vec<String>,
}

#[derive(Debug, Clone)]
enum Message {
    RegistryChanged(String),
    SecretChanged(String),
    VerifyChanged(String),
    RelayChanged(String),
    SocksChanged(String),
    TorToggled(bool),
    VpnToggled(bool),
    Start,
    Stop,
    CopySocks,
    Tick,
}

impl App {
    fn boot() -> Self {
        let lines = Arc::new(Mutex::new(VecDeque::new()));
        install_logger(lines.clone());
        log::info!("Axon desktop ready");
        Self {
            config: load_config(),
            phase: Phase::Stopped,
            notice: String::new(),
            notice_warn: false,
            session: None,
            lines,
            log_lines: Vec::new(),
        }
    }

    fn title(_app: &Self) -> String {
        "Axon".to_string()
    }

    fn theme(_app: &Self) -> Theme {
        Theme::custom(
            "Axon",
            iced::theme::Palette {
                background: Color::BLACK,
                text: TEXT,
                primary: Color::WHITE,
                success: OK,
                warning: WARN,
                danger: DANGER,
            },
        )
    }

    fn style(_app: &Self, _theme: &Theme) -> iced::theme::Style {
        iced::theme::Style {
            background_color: Color::BLACK,
            text_color: TEXT,
        }
    }

    fn subscription(app: &Self) -> Subscription<Message> {
        let interval = if app.phase == Phase::Stopped {
            Duration::from_millis(1000)
        } else {
            Duration::from_millis(400)
        };
        iced::time::every(interval).map(|_| Message::Tick)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::RegistryChanged(value) => self.config.registry_url = value,
            Message::SecretChanged(value) => self.config.secret = value,
            Message::VerifyChanged(value) => self.config.verify_port = value,
            Message::RelayChanged(value) => self.config.relay_port = value,
            Message::SocksChanged(value) => self.config.socks_port = value,
            Message::TorToggled(value) => self.config.use_tor = value,
            Message::VpnToggled(value) => self.config.use_vpn = value,
            Message::Start => self.start(),
            Message::Stop => self.stop(),
            Message::CopySocks => {
                let socks = format!("127.0.0.1:{}", self.config.socks_port.trim());
                self.notice = "Copied the local SOCKS address.".to_string();
                self.notice_warn = false;
                return iced::clipboard::write(socks);
            }
            Message::Tick => self.poll_session(),
        }
        Task::none()
    }

    fn poll_session(&mut self) {
        self.log_lines = self
            .lines
            .lock()
            .expect("log buffer")
            .iter()
            .cloned()
            .collect();
        let finished = self.session.as_ref().is_some_and(|session| session.is_finished());
        if !finished {
            return;
        }
        let Some(mut session) = self.session.take() else {
            return;
        };
        let error = session.join();
        let was_stopping = self.phase == Phase::Stopping;
        self.phase = Phase::Stopped;
        match error {
            Some(error) => {
                self.notice = error;
                self.notice_warn = true;
            }
            None if was_stopping => {
                self.notice = "Node stopped.".to_string();
                self.notice_warn = false;
            }
            None => {
                self.notice = "Node exited. The log has the reason.".to_string();
                self.notice_warn = true;
            }
        }
    }

    fn start(&mut self) {
        let config = match prepare(&self.config) {
            Ok(config) => config,
            Err(error) => {
                self.notice = error;
                self.notice_warn = true;
                return;
            }
        };
        save_config(&self.config);
        self.notice.clear();
        log::info!(
            "Starting node. Registry {} · local SOCKS 127.0.0.1:{} · relay 0.0.0.0:{}",
            config.registry_url,
            config.socks_port,
            config.relay_port
        );
        self.session = Some(NodeSession::spawn(config));
        self.phase = Phase::Running;
    }

    fn stop(&mut self) {
        if self.session.is_some() {
            self.phase = Phase::Stopping;
            self.notice = "Stopping the node…".to_string();
            self.notice_warn = false;
            if let Some(session) = self.session.as_mut() {
                session.request_stop();
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let editable = self.phase == Phase::Stopped;
        let socks = format!("127.0.0.1:{}", self.config.socks_port.trim());

        let mut page = column![self.header()].spacing(16).padding(24).width(Fill).height(Fill);
        if !self.notice.is_empty() {
            page = page.push(notice_banner(&self.notice, self.notice_warn));
        }
        page = page.push(
            row![self.connection_card(editable), self.machine_card(socks)]
                .spacing(16)
                .width(Fill),
        );
        page = page.push(self.log_card());

        container(page).width(Fill).height(Fill).into()
    }

    fn header(&self) -> Element<'_, Message> {
        row![
            mark(),
            column![
                text("Axon").size(24),
                text("Local proxy node").size(13).color(MUTED),
            ]
            .spacing(2),
            space::horizontal(),
            status_pill(self.phase),
        ]
        .align_y(Alignment::Center)
        .spacing(14)
        .width(Fill)
        .into()
    }

    fn connection_card(&self, editable: bool) -> Element<'_, Message> {
        let toggle = |message: fn(bool) -> Message| editable.then_some(message);
        let body = column![
            field(
                "Registry",
                "http://127.0.0.1:7400",
                &self.config.registry_url,
                editable.then_some(Message::RegistryChanged),
                false,
            ),
            field(
                "Master secret",
                "Matches SECRET_KEY",
                &self.config.secret,
                editable.then_some(Message::SecretChanged),
                true,
            ),
            row![
                field(
                    "Verify",
                    "8080",
                    &self.config.verify_port,
                    editable.then_some(Message::VerifyChanged),
                    false,
                ),
                field(
                    "Relay",
                    "8081",
                    &self.config.relay_port,
                    editable.then_some(Message::RelayChanged),
                    false,
                ),
                field(
                    "SOCKS",
                    "1080",
                    &self.config.socks_port,
                    editable.then_some(Message::SocksChanged),
                    false,
                ),
            ]
            .spacing(10),
            option_row(
                "Tor exit",
                "Relay traffic goes through a Tor daemon on 127.0.0.1:9050.",
                self.config.use_tor,
                toggle(Message::TorToggled),
            ),
            option_row(
                "TUN capture",
                "Needs an elevated process and wintun.dll on Windows.",
                self.config.use_vpn,
                toggle(Message::VpnToggled),
            ),
            self.action_button(),
        ]
        .spacing(12);

        card("Connection", body.into())
    }

    fn action_button(&self) -> Element<'_, Message> {
        match self.phase {
            Phase::Stopped => button(text("Start node").size(15))
                .width(Fill)
                .padding([12, 16])
                .on_press(Message::Start)
                .style(primary_button)
                .into(),
            Phase::Running => button(text("Stop node").size(15))
                .width(Fill)
                .padding([12, 16])
                .on_press(Message::Stop)
                .style(stop_button)
                .into(),
            Phase::Stopping => button(text("Stopping…").size(15))
                .width(Fill)
                .padding([12, 16])
                .style(stop_button)
                .into(),
        }
    }

    fn machine_card(&self, socks: String) -> Element<'_, Message> {
        let relay = format!("0.0.0.0:{}", self.config.relay_port.trim());
        let verify = format!("0.0.0.0:{}", self.config.verify_port.trim());
        let registry = self.config.registry_url.trim().to_string();
        let body = column![
            text("SOCKS").size(12).color(FAINT),
            text(socks).size(28).font(Font::MONOSPACE),
            button(text("Copy address").size(13))
                .padding([8, 14])
                .on_press(Message::CopySocks)
                .style(ghost_button),
            column![
                stat("Relay", relay),
                stat("Verify", verify),
                stat("Registry", registry),
            ]
            .spacing(8),
            text("Point the browser SOCKS5 host at 127.0.0.1 and this port. Turn remote DNS on so names are resolved by the relay.")
                .size(13)
                .color(MUTED),
            text("The registry must already be running.")
                .size(12)
                .color(FAINT),
        ]
        .spacing(10);
        card("This machine", body.into())
    }

    fn log_card(&self) -> Element<'_, Message> {
        let lines: Element<'_, Message> = if self.log_lines.is_empty() {
            text("No log lines yet.").size(13).color(FAINT).into()
        } else {
            column(self.log_lines.iter().map(|line| {
                text(line)
                    .size(12)
                    .font(Font::MONOSPACE)
                    .color(log_color(line))
                    .into()
            }))
            .spacing(2)
            .width(Fill)
            .into()
        };

        container(
            column![
                text("Log").size(12).color(MUTED),
                scrollable(lines).height(Fill).width(Fill).anchor_bottom(),
            ]
            .spacing(10),
        )
        .padding(18)
        .width(Fill)
        .height(Fill)
        .style(card_style)
        .into()
    }
}

impl Drop for App {
    fn drop(&mut self) {
        save_config(&self.config);
        if let Some(mut session) = self.session.take() {
            let _ = session.join();
        }
    }
}

fn mark() -> Element<'static, Message> {
    container(text("A").size(16).color(Color::BLACK))
        .width(36)
        .height(36)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .style(|_theme| container::Style {
            text_color: Some(Color::BLACK),
            background: Some(Background::Color(Color::WHITE)),
            border: Border {
                color: Color::WHITE,
                width: 0.0,
                radius: 10.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn status_pill(phase: Phase) -> Element<'static, Message> {
    let (label, dot) = match phase {
        Phase::Stopped => ("Stopped", FAINT),
        Phase::Running => ("Running", OK),
        Phase::Stopping => ("Stopping", WARN),
    };
    container(
        row![
            container("")
                .width(8)
                .height(8)
                .style(move |_theme| container::Style {
                    background: Some(Background::Color(dot)),
                    border: Border {
                        color: dot,
                        width: 0.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                }),
            text(label).size(13),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .padding([6.0, 12.0])
    .style(|_theme| container::Style {
        background: Some(Background::Color(Color::from_rgb8(24, 24, 27))),
        border: Border {
            color: LINE,
            width: 1.0,
            radius: 999.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn notice_banner(message: &str, warn: bool) -> Element<'_, Message> {
    let color = if warn { WARN } else { TEXT };
    container(text(message).size(13).color(color))
        .padding([12.0, 14.0])
        .width(Fill)
        .style(move |_theme| container::Style {
            background: Some(Background::Color(Color::from_rgb8(24, 24, 27))),
            border: Border {
                color: if warn { Color::from_rgb8(82, 68, 24) } else { LINE },
                width: 1.0,
                radius: 12.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn card<'a>(title: &'a str, body: Element<'a, Message>) -> Element<'a, Message> {
    container(column![text(title).size(12).color(MUTED), body].spacing(14))
        .padding(18)
        .width(Fill)
        .style(card_style)
        .into()
}

fn field<'a>(
    label: &'a str,
    placeholder: &'a str,
    value: &'a str,
    on_input: Option<fn(String) -> Message>,
    secure: bool,
) -> Element<'a, Message> {
    let mut input = text_input(placeholder, value)
        .padding(12)
        .size(15)
        .secure(secure)
        .style(input_style);
    if let Some(on_input) = on_input {
        input = input.on_input(on_input);
    }
    column![text(label).size(12).color(MUTED), input]
        .spacing(6)
        .width(Fill)
        .into()
}

fn option_row<'a>(
    title: &'a str,
    detail: &'a str,
    on: bool,
    toggle: Option<fn(bool) -> Message>,
) -> Element<'a, Message> {
    let mut switch = toggler(on);
    if let Some(toggle) = toggle {
        switch = switch.on_toggle(toggle);
    }
    container(
        row![
            column![
                text(title).size(14),
                text(detail).size(12).color(FAINT),
            ]
            .spacing(2)
            .width(Fill),
            switch,
        ]
        .align_y(Alignment::Center)
        .spacing(12),
    )
    .padding(12)
    .width(Fill)
    .style(move |_theme| {
        let mut style = card_style(_theme);
        style.background = Some(Background::Color(if on {
            Color::from_rgb8(24, 24, 27)
        } else {
            FIELD
        }));
        style.border.radius = 10.0.into();
        style
    })
    .into()
}

fn stat(label: &'static str, value: String) -> Element<'static, Message> {
    row![
        text(label).size(13).color(MUTED).width(72),
        text(value).size(13).font(Font::MONOSPACE),
    ]
    .align_y(Alignment::Center)
    .spacing(8)
    .into()
}

fn card_style(_theme: &Theme) -> container::Style {
    container::Style {
        text_color: Some(TEXT),
        background: Some(Background::Color(CARD)),
        border: Border {
            color: LINE,
            width: 1.0,
            radius: 14.0.into(),
        },
        ..Default::default()
    }
}

fn input_style(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let border_color = match status {
        text_input::Status::Focused { .. } => Color::WHITE,
        text_input::Status::Hovered => Color::from_rgb8(82, 82, 91),
        text_input::Status::Disabled => LINE,
        _ => LINE,
    };
    text_input::Style {
        background: Background::Color(FIELD),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 8.0.into(),
        },
        icon: MUTED,
        placeholder: FAINT,
        value: TEXT,
        selection: Color::from_rgb8(63, 63, 70),
    }
}

fn primary_button(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::primary(theme, status);
    let (background, foreground) = match status {
        button::Status::Disabled => (Color::from_rgb8(63, 63, 70), MUTED),
        button::Status::Hovered => (Color::from_rgb8(228, 228, 231), Color::BLACK),
        button::Status::Pressed => (Color::from_rgb8(212, 212, 216), Color::BLACK),
        _ => (Color::WHITE, Color::BLACK),
    };
    style.background = Some(Background::Color(background));
    style.text_color = foreground;
    style.border.radius = 10.0.into();
    style.border.width = 0.0;
    style
}

fn stop_button(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::danger(theme, status);
    style.background = Some(Background::Color(Color::from_rgb8(127, 29, 29)));
    style.text_color = Color::WHITE;
    style.border.radius = 10.0.into();
    style.border.width = 0.0;
    style
}

fn ghost_button(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::secondary(theme, status);
    let background = match status {
        button::Status::Hovered => Color::from_rgb8(39, 39, 42),
        button::Status::Pressed => Color::from_rgb8(52, 52, 56),
        _ => Color::from_rgb8(24, 24, 27),
    };
    style.background = Some(Background::Color(background));
    style.text_color = TEXT;
    style.border = Border {
        color: Color::from_rgb8(63, 63, 70),
        width: 1.0,
        radius: 10.0.into(),
    };
    style
}

fn log_color(line: &str) -> Color {
    if line.starts_with("ERROR") {
        DANGER
    } else if line.starts_with("WARN") {
        WARN
    } else if line.starts_with("DEBUG") {
        FAINT
    } else {
        Color::from_rgb8(212, 212, 216)
    }
}

fn prepare(config: &DesktopConfig) -> Result<NodeConfig, String> {
    let verify_port = parse_port("Verify", &config.verify_port)?;
    let relay_port = parse_port("Relay", &config.relay_port)?;
    let socks_port = parse_port("SOCKS", &config.socks_port)?;
    NodeConfig::parse(
        &config.registry_url,
        &config.secret,
        verify_port,
        relay_port,
        socks_port,
        config.use_tor,
        config.use_vpn,
    )
}

fn parse_port(name: &str, value: &str) -> Result<u16, String> {
    let port: u16 = value
        .trim()
        .parse()
        .map_err(|_| format!("{name} port must be a number from 1 to 65535."))?;
    if port == 0 {
        return Err(format!("{name} port must be a number from 1 to 65535."));
    }
    Ok(port)
}

fn config_path() -> PathBuf {
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("Axon").join("desktop.json")
}

fn load_config() -> DesktopConfig {
    let path = config_path();
    let Ok(text) = std::fs::read_to_string(&path) else {
        return DesktopConfig::default();
    };
    serde_json::from_str(&text).unwrap_or_default()
}

fn save_config(config: &DesktopConfig) {
    let path = config_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(text) = serde_json::to_string_pretty(config) {
        let _ = std::fs::write(path, text);
    }
}

fn install_logger(lines: Arc<Mutex<VecDeque<String>>>) {
    let logger = Box::leak(Box::new(UiLogger { lines }));
    if log::set_logger(logger).is_ok() {
        log::set_max_level(log::LevelFilter::Debug);
    }
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(text) = payload.downcast_ref::<&str>() {
        return (*text).to_string();
    }
    if let Some(text) = payload.downcast_ref::<String>() {
        return text.clone();
    }
    "The node thread panicked.".to_string()
}

fn window_settings() -> iced::window::Settings {
    iced::window::Settings {
        size: iced::Size::new(1100.0, 780.0),
        min_size: Some(iced::Size::new(920.0, 640.0)),
        ..Default::default()
    }
}

fn main() -> iced::Result {
    let mut application = iced::application(App::boot, App::update, App::view)
        .title(App::title)
        .theme(App::theme)
        .style(App::style)
        .subscription(App::subscription)
        .window(window_settings())
        .centered()
        .antialiasing(true);

    if let Ok(bytes) = std::fs::read(r"C:\Windows\Fonts\segoeui.ttf") {
        application = application
            .font(bytes)
            .default_font(Font::with_name("Segoe UI"));
    }

    application.run()
}
