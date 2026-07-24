pub fn init() {
    #[cfg(target_os = "android")]
    {
        android_logger::init_once(
            android_logger::Config::default().with_max_level(log::LevelFilter::Debug),
        );
        log::info!("Initialized android_logger in core");
    }

    #[cfg(not(target_os = "android"))]
    {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
            .try_init()
            .ok();
        log::info!("Initialized env_logger in core");
    }
}
