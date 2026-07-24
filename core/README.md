# Axon Core Library

The `core` crate contains the fundamental logic for the Axon decentralized proxy network. It serves as the primary engine for the Rust-based proxy nodes and provides cross-platform functionality that is consumed by the `cli`, `desktop`, and `axon-android/rust` crates.

## Architecture: Dual-Role Proxy

The Axon node is designed to operate in a dual-role architecture, meaning it can function simultaneously as a **Server (Relay)** and a **Client (Entry Node)** in the P2P proxy network.

### 1. Server Mode (Public Relay)
When the node starts, it registers itself with the central Web Registry.
- **Registration & Heartbeat**: The node POSTs its public details (including its `server_port`) to the web coordinator. It maintains an "ACTIVE" status by sending periodic heartbeats.
- **SOCKS5 Server**: It spins up a `tokio`-based SOCKS5 proxy server listening on the `server_port` (e.g., `8081`).
- **Relaying Traffic**: When *other* Axon peers connect to this port, the node proxies their traffic to the broader internet. Optionally, it can route this outbound traffic through the Tor network if Tor is enabled.

### 2. Client Mode (Local Entry Proxy)
The node provides a local entry point for the user to securely browse the internet through the Axon network.
- **Local Listener**: It spins up a SOCKS5 server on `127.0.0.1:<client_port>` (e.g., `1080`) that local applications (like a web browser) can connect to.
- **Node Discovery**: When a connection is received, it reaches out to the central Web Registry (`GET /api/v1/nodes`) to fetch a list of currently active Axon relay nodes.
- **Decentralized Routing**: It picks a random, active peer from the list and securely tunnels the user's SOCKS5 request to that peer's `server_port`.

## Module Structure

- **`logger/mod.rs`**: Handles cross-platform logging initialization (`env_logger` for CLI/Desktop, `android_logger` for Android).
- **`node/mod.rs`**: Orchestrates the node lifecycle. Spawns asynchronous background tasks for registration, the heartbeat loop, and both the Server and Client proxies.
- **`proxy/server.rs`**: The implementation of the SOCKS5 relay server that accepts remote connections.
- **`proxy/client.rs`**: The implementation of the local entry proxy that fetches active peers and routes local traffic through the Axon network. Includes testing utilities.

## Environment Variables

For testing and local development, the `core` library expects certain environment variables (which can be loaded via a `.env` file):

- `REGISTRY_URL`: The URL of the central web coordinator (default: `http://localhost:3000`).

## Testing

The core library includes an integration test to verify proxy routing.
To run the test:
```bash
cargo test
```
*Note: The test relies on loading `.env` variables and will attempt to perform a real outbound request to verify the proxy connection. On strictly locked-down Windows environments (e.g., Application Control Policies), `cargo test` execution might be blocked.*
