use axon_core::logger;
use axon_core::node::NodeConfig;
use clap::Parser;

#[derive(Parser)]
#[command(
    name = "axon",
    bin_name = "axon",
    about = "Run an Axon proxy node",
    version,
    after_help = "REGISTRY_URL and SECRET_KEY are used when the matching flag is omitted."
)]
struct Args {
    /// Registry base URL.
    #[arg(long, env = "REGISTRY_URL", default_value = "http://127.0.0.1:7400")]
    registry: String,

    /// Master secret. It must match the registry SECRET_KEY.
    #[arg(long, env = "SECRET_KEY", default_value = "Nishanth")]
    secret: String,

    /// Port that receives the registry verification callback.
    #[arg(long, default_value_t = 8080)]
    verify_port: u16,

    /// Public SOCKS5 port other nodes connect to.
    #[arg(long, default_value_t = 8081)]
    relay_port: u16,

    /// Local SOCKS5 port for this machine.
    #[arg(long, default_value_t = 1080)]
    socks_port: u16,

    /// Send relay traffic through a Tor daemon on 127.0.0.1:9050.
    #[arg(long)]
    tor: bool,

    /// Capture traffic with a TUN adapter. Needs an elevated process.
    #[arg(long)]
    vpn: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    logger::init();
    let config = match NodeConfig::parse(
        &args.registry,
        &args.secret,
        args.verify_port,
        args.relay_port,
        args.socks_port,
        args.tor,
        args.vpn,
    ) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("error: {error}");
            std::process::exit(2);
        }
    };

    println!("Axon node");
    println!("  Registry     {}", config.registry_url);
    println!("  Local SOCKS  127.0.0.1:{}", config.socks_port);
    println!("  Relay        0.0.0.0:{}", config.relay_port);
    println!("  Verify       0.0.0.0:{}", config.verify_port);
    if config.use_tor {
        println!("  Tor          on (127.0.0.1:9050)");
    }
    if config.use_vpn {
        println!("  VPN          on");
    }
    println!("Point the browser SOCKS5 host at 127.0.0.1 and the port above. Turn remote DNS on.");
    println!("Press Ctrl+C to stop.");

    tokio::select! {
        _ = config.run() => {
            eprintln!("The node stopped. The log has the reason.");
        }
        result = tokio::signal::ctrl_c() => {
            if let Err(error) = result {
                eprintln!("Could not listen for Ctrl+C: {error}");
            }
            println!("Stopping the node.");
        }
    }
}
