use axon_core::node;
use eframe::egui;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::panic::AssertUnwindSafe;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

const ACCENT: egui::Color32 = egui::Color32::from_rgb(45, 212, 191);
const OK: egui::Color32 = egui::Color32::from_rgb(52, 211, 153);
const WARN: egui::Color32 = egui::Color32::from_rgb(251, 191, 36);
const MUTED: egui::Color32 = egui::Color32::from_rgb(148, 163, 184);

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

struct Launch {
    registry_url: String,
    secret: String,
    verify_port: u16,
    relay_port: u16,
    socks_port: u16,
    use_tor: bool,
    use_vpn: bool,
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
    fn spawn(launch: Launch) -> Self {
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
                                &launch.registry_url,
                                launch.verify_port,
                                launch.relay_port,
                                launch.socks_port,
                                &launch.secret,
                                launch.use_tor,
                                launch.use_vpn,
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

struct AxonApp {
    config: DesktopConfig,
    phase: Phase,
    notice: String,
    session: Option<NodeSession>,
    lines: Arc<Mutex<VecDeque<String>>>,
}

impl AxonApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        apply_style(&cc.egui_ctx);
        let lines = Arc::new(Mutex::new(VecDeque::new()));
        install_logger(lines.clone());
        log::info!("Axon desktop ready");
        Self {
            config: load_config(),
            phase: Phase::Stopped,
            notice: String::new(),
            session: None,
            lines,
        }
    }

    fn poll_session(&mut self) {
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
        self.notice = error.unwrap_or_else(|| {
            if was_stopping {
                "Node stopped.".to_string()
            } else {
                "Node exited. The log has the reason.".to_string()
            }
        });
    }

    fn start(&mut self) {
        let launch = match validate(&self.config) {
            Ok(launch) => launch,
            Err(error) => {
                self.notice = error;
                return;
            }
        };
        save_config(&self.config);
        self.notice.clear();
        log::info!(
            "Starting node. Registry {} · local SOCKS 127.0.0.1:{} · relay 0.0.0.0:{}",
            launch.registry_url,
            launch.socks_port,
            launch.relay_port
        );
        self.session = Some(NodeSession::spawn(launch));
        self.phase = Phase::Running;
    }

    fn stop(&mut self) {
        if let Some(session) = self.session.as_mut() {
            self.phase = Phase::Stopping;
            self.notice = "Stopping the node…".to_string();
            session.request_stop();
        }
    }

    fn settings_ui(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("Connection").strong());
        ui.add_space(4.0);
        let editable = self.phase == Phase::Stopped;
        ui.add_enabled_ui(editable, |ui| {
            labeled(ui, "Registry", &mut self.config.registry_url);
            ui.label(egui::RichText::new("Master secret").weak());
            ui.add(
                egui::TextEdit::singleline(&mut self.config.secret)
                    .password(true)
                    .desired_width(f32::INFINITY),
            );
            ui.columns(3, |columns| {
                port_field(&mut columns[0], "Verify", &mut self.config.verify_port);
                port_field(&mut columns[1], "Relay", &mut self.config.relay_port);
                port_field(&mut columns[2], "SOCKS", &mut self.config.socks_port);
            });
            ui.checkbox(&mut self.config.use_tor, "Exit relay traffic through Tor");
            ui.label(
                egui::RichText::new("Requires a Tor daemon on 127.0.0.1:9050.")
                    .weak()
                    .size(12.0),
            );
            ui.checkbox(&mut self.config.use_vpn, "Capture traffic with the TUN adapter");
            ui.label(
                egui::RichText::new("Needs an elevated process and wintun.dll on Windows.")
                    .weak()
                    .size(12.0),
            );
        });
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if self.phase == Phase::Stopped {
                let button = egui::Button::new(egui::RichText::new("Start node").strong())
                    .fill(egui::Color32::from_rgb(13, 148, 136));
                if ui.add(button).clicked() {
                    self.start();
                }
            } else {
                let button = egui::Button::new(egui::RichText::new("Stop node").strong())
                    .fill(egui::Color32::from_rgb(127, 29, 29));
                if ui.add(button).clicked() {
                    self.stop();
                }
            }
        });
    }

    fn status_ui(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("This machine").strong());
        ui.add_space(4.0);
        let socks = format!("127.0.0.1:{}", self.config.socks_port.trim());
        let relay = format!("0.0.0.0:{}", self.config.relay_port.trim());
        let verify = format!("0.0.0.0:{}", self.config.verify_port.trim());
        row(ui, "Local proxy", &socks);
        if ui.button("Copy SOCKS address").clicked() {
            ui.ctx().copy_text(socks);
            self.notice = "Copied the local SOCKS address.".to_string();
        }
        row(ui, "Relay", &relay);
        row(ui, "Verification", &verify);
        row(ui, "Registry", self.config.registry_url.trim());
        ui.add_space(8.0);
        ui.label(egui::RichText::new("Browser").strong());
        ui.label("Set the browser SOCKS5 host to 127.0.0.1 and the port to the SOCKS value above. Turn remote DNS on so names are resolved by the relay, not by this machine.");
        ui.add_space(6.0);
        ui.label(
            egui::RichText::new("The registry must already be running. Docker: docker compose up --build registry")
                .weak()
                .size(12.0),
        );
    }
}

impl eframe::App for AxonApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_session();
        if self.phase != Phase::Stopped {
            ctx.request_repaint_after(Duration::from_millis(400));
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(egui::RichText::new("Axon").color(ACCENT));
                ui.label(egui::RichText::new("desktop node").color(MUTED));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let (label, color) = match self.phase {
                        Phase::Stopped => ("Stopped", MUTED),
                        Phase::Running => ("Running", OK),
                        Phase::Stopping => ("Stopping", WARN),
                    };
                    ui.label(egui::RichText::new(label).color(color).strong());
                });
            });
            ui.separator();
            if !self.notice.is_empty() {
                ui.label(egui::RichText::new(&self.notice).color(WARN));
                ui.add_space(4.0);
            }

            ui.columns(2, |columns| {
                self.settings_ui(&mut columns[0]);
                self.status_ui(&mut columns[1]);
            });

            ui.add_space(8.0);
            ui.separator();
            ui.label(egui::RichText::new("Log").strong());
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .max_height(220.0)
                .show(ui, |ui| {
                    let lines = self.lines.lock().expect("log buffer");
                    if lines.is_empty() {
                        ui.label(egui::RichText::new("No log lines yet.").color(MUTED));
                    }
                    for line in lines.iter() {
                        ui.monospace(line);
                    }
                });
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        save_config(&self.config);
        if let Some(mut session) = self.session.take() {
            let _ = session.join();
        }
    }
}

fn labeled(ui: &mut egui::Ui, name: &str, value: &mut String) {
    ui.label(egui::RichText::new(name).weak());
    ui.add(egui::TextEdit::singleline(value).desired_width(f32::INFINITY));
}

fn port_field(ui: &mut egui::Ui, name: &str, value: &mut String) {
    ui.label(egui::RichText::new(name).weak());
    ui.add(egui::TextEdit::singleline(value).desired_width(f32::INFINITY));
}

fn row(ui: &mut egui::Ui, name: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(name).color(MUTED).size(13.0));
        ui.label(egui::RichText::new(value).monospace());
    });
}

fn validate(config: &DesktopConfig) -> Result<Launch, String> {
    let registry_url = config.registry_url.trim().trim_end_matches('/').to_string();
    if !(registry_url.starts_with("http://") || registry_url.starts_with("https://")) {
        return Err("Registry URL must start with http:// or https://.".into());
    }
    let secret = config.secret.trim().to_string();
    if secret.is_empty() {
        return Err("Master secret is required. It must match the registry SECRET_KEY.".into());
    }
    let verify_port = parse_port("Verify", &config.verify_port)?;
    let relay_port = parse_port("Relay", &config.relay_port)?;
    let socks_port = parse_port("SOCKS", &config.socks_port)?;
    if verify_port == relay_port || verify_port == socks_port || relay_port == socks_port {
        return Err("Verify, relay, and SOCKS ports must be different.".into());
    }
    Ok(Launch {
        registry_url,
        secret,
        verify_port,
        relay_port,
        socks_port,
        use_tor: config.use_tor,
        use_vpn: config.use_vpn,
    })
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
        .or_else(|| {
            std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config"))
        })
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

fn apply_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(14.0, 7.0);
    ctx.set_style(style);

    let mut visuals = egui::Visuals::dark();
    visuals.window_fill = egui::Color32::from_rgb(10, 14, 20);
    visuals.panel_fill = egui::Color32::from_rgb(16, 22, 32);
    visuals.extreme_bg_color = egui::Color32::from_rgb(8, 11, 16);
    visuals.faint_bg_color = egui::Color32::from_rgb(22, 30, 42);
    visuals.code_bg_color = egui::Color32::from_rgb(8, 11, 16);
    visuals.hyperlink_color = ACCENT;
    visuals.selection.bg_fill = egui::Color32::from_rgb(15, 90, 84);
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(24, 33, 48);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(32, 50, 62);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(15, 90, 84);
    ctx.set_visuals(visuals);
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([980.0, 680.0])
            .with_min_inner_size([760.0, 520.0])
            .with_title("Axon"),
        ..Default::default()
    };
    eframe::run_native(
        "Axon",
        options,
        Box::new(|cc| Ok(Box::new(AxonApp::new(cc)))),
    )
}
