//! JNI bridge the Android app uses to start and stop one Axon node.
//!
//! The node runs on its own thread and Tokio runtime. Java calls are cheap:
//! they record a stop signal or copy the latest log lines.

use axon_core::node::{self, NodeConfig};
use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jint, jstring};
use jni::JNIEnv;
use std::collections::VecDeque;
use std::panic::AssertUnwindSafe;
use std::sync::{Mutex, Once};
use std::thread::JoinHandle;

const LOG_LIMIT: usize = 300;

struct NodeSession {
    stop: Option<tokio::sync::oneshot::Sender<()>>,
    thread: Option<JoinHandle<Option<String>>>,
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Stopped,
    Running,
    Stopping,
}

struct Bridge {
    phase: Phase,
    notice: String,
    session: Option<NodeSession>,
}

struct LogBuffer {
    lines: VecDeque<String>,
}

struct UiLogger;

impl log::Log for UiLogger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= log::Level::Debug
    }

    fn log(&self, record: &log::Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let line = format!("{:<5} {}", record.level(), record.args());
        platform_log(record.level(), &line);
        if let Ok(mut buffer) = LOG.lock() {
            buffer.lines.push_back(line);
            while buffer.lines.len() > LOG_LIMIT {
                buffer.lines.pop_front();
            }
        }
    }

    fn flush(&self) {}
}

static BRIDGE: Mutex<Bridge> = Mutex::new(Bridge {
    phase: Phase::Stopped,
    notice: String::new(),
    session: None,
});
static LOG: Mutex<LogBuffer> = Mutex::new(LogBuffer {
    lines: VecDeque::new(),
});
static LOGGER: Once = Once::new();

fn install_logger() {
    LOGGER.call_once(|| {
        let logger = Box::leak(Box::new(UiLogger));
        if log::set_logger(logger).is_ok() {
            log::set_max_level(log::LevelFilter::Debug);
        }
        log::info!("Axon Android ready");
    });
}

fn lock_bridge() -> std::sync::MutexGuard<'static, Bridge> {
    BRIDGE.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn reap(bridge: &mut Bridge) {
    let finished = bridge
        .session
        .as_ref()
        .is_some_and(|session| session.is_finished());
    if !finished {
        return;
    }
    let Some(mut session) = bridge.session.take() else {
        return;
    };
    match session.join() {
        Some(error) => bridge.notice = error,
        None if bridge.phase == Phase::Stopping => bridge.notice = "Node stopped.".to_string(),
        None => {
            if bridge.notice.is_empty() {
                bridge.notice = "Node exited. The log has the reason.".to_string();
            }
        }
    }
    bridge.phase = Phase::Stopped;
}

fn start_node(
    registry: &str,
    secret: &str,
    verify_port: u16,
    relay_port: u16,
    socks_port: u16,
    use_tor: bool,
) -> String {
    install_logger();
    let config = match NodeConfig::parse(
        registry,
        secret,
        verify_port,
        relay_port,
        socks_port,
        use_tor,
        false,
    ) {
        Ok(config) => config,
        Err(error) => return error,
    };

    let mut bridge = lock_bridge();
    reap(&mut bridge);
    if bridge.phase != Phase::Stopped {
        return "The node is already running.".to_string();
    }
    log::info!(
        "Starting node. Registry {} · local SOCKS 127.0.0.1:{} · relay 0.0.0.0:{}",
        config.registry_url,
        config.socks_port,
        config.relay_port
    );
    bridge.session = Some(NodeSession::spawn(config));
    bridge.phase = Phase::Running;
    bridge.notice.clear();
    "ok".to_string()
}

fn stop_node() {
    install_logger();
    let mut bridge = lock_bridge();
    if bridge.session.is_some() {
        bridge.phase = Phase::Stopping;
        bridge.notice = "Stopping the node…".to_string();
        if let Some(session) = bridge.session.as_mut() {
            session.request_stop();
        }
    }
}

fn status_word() -> &'static str {
    let mut bridge = lock_bridge();
    reap(&mut bridge);
    match bridge.phase {
        Phase::Stopped => "stopped",
        Phase::Running => "running",
        Phase::Stopping => "stopping",
    }
}

fn notice_text() -> String {
    let mut bridge = lock_bridge();
    reap(&mut bridge);
    bridge.notice.clone()
}

fn log_text() -> String {
    let lines = LOG.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    if lines.lines.is_empty() {
        "No log lines yet.".to_string()
    } else {
        lines.lines.iter().cloned().collect::<Vec<_>>().join("\n")
    }
}

fn java_result(env: &mut JNIEnv, value: String) -> jstring {
    env.new_string(value)
        .map(|value| value.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

fn read_java_string(env: &mut JNIEnv, value: &JString) -> Result<String, String> {
    env.get_string(value)
        .map(|value| value.into())
        .map_err(|_| "Could not read the settings.".to_string())
}

fn read_port(value: jint, name: &str) -> Result<u16, String> {
    if !(1..=65535).contains(&value) {
        return Err(format!("{name} port must be a number from 1 to 65535."));
    }
    Ok(value as u16)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_axon_1android_NodeBridge_nativeStart<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    registry: JString<'local>,
    secret: JString<'local>,
    verify_port: jint,
    relay_port: jint,
    socks_port: jint,
    use_tor: jboolean,
) -> jstring {
    let result = (|| {
        let registry = read_java_string(&mut env, &registry)?;
        let secret = read_java_string(&mut env, &secret)?;
        let verify_port = read_port(verify_port, "Verify")?;
        let relay_port = read_port(relay_port, "Relay")?;
        let socks_port = read_port(socks_port, "SOCKS")?;
        Ok(start_node(
            &registry,
            &secret,
            verify_port,
            relay_port,
            socks_port,
            use_tor != 0,
        ))
    })()
    .unwrap_or_else(|error| error);
    java_result(&mut env, result)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_axon_1android_NodeBridge_nativeStop(
    _env: JNIEnv,
    _class: JClass,
) {
    stop_node();
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_axon_1android_NodeBridge_nativeStatus<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jstring {
    java_result(&mut env, status_word().to_string())
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_axon_1android_NodeBridge_nativeNotice<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jstring {
    java_result(&mut env, notice_text())
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_axon_1android_NodeBridge_nativeLogs<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jstring {
    java_result(&mut env, log_text())
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

#[cfg(target_os = "android")]
fn platform_log(level: log::Level, message: &str) {
    let priority = match level {
        log::Level::Error => 6,
        log::Level::Warn => 5,
        log::Level::Info => 4,
        log::Level::Debug => 3,
        log::Level::Trace => 2,
    };
    let tag = std::ffi::CString::new("axon").unwrap_or_default();
    let text = std::ffi::CString::new(message.replace('\0', " "))
        .unwrap_or_else(|_| std::ffi::CString::new("log").unwrap_or_default());
    unsafe {
        __android_log_write(priority, tag.as_ptr(), text.as_ptr());
    }
}

#[cfg(target_os = "android")]
unsafe extern "C" {
    fn __android_log_write(priority: i32, tag: *const i8, text: *const i8) -> i32;
}

#[cfg(not(target_os = "android"))]
fn platform_log(_level: log::Level, _message: &str) {}
