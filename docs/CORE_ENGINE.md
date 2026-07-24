# Axon Core Engine

The `core` crate is the heart of the Axon Proxy Network. It is a cross-platform Rust library that contains the networking, SOCKS5 protocols, Tor routing, and registry orchestration. 

It is designed to be compiled into:
- Native binaries (Windows/Linux/macOS) via the `cli` and `desktop` wrappers.
- An Android Background Service via JNI bindings in `axon-android/rust`.

## Module Breakdown

### 1. `node/mod.rs`
The lifecycle manager. When `run_node` is called:
1. **HTTP Client Initialization**: It spins up a `reqwest::Client`. If `use_tor = true`, it binds this client to the local Tor proxy (`socks5h://127.0.0.1:9050`).
2. **Registration**: It sends a POST request to the Web Registry with its public Server Port.
3. **Verification (Axum)**: It starts a small Axum HTTP server to listen for the web registry's callback. The callback contains the `secret_key` necessary to authenticate heartbeats.
4. **Heartbeat Loop**: A `tokio` task that pings the registry every 10 seconds to keep the node "Active" in the database.
5. **Proxy Spawning**: It concurrently launches both the Client Proxy and Server Proxy tasks.

### 2. `proxy/server.rs`
The public relay logic.
- Listens on `server_port` (e.g. 8081).
- Expects incoming standard SOCKS5 connections from *other Axon peers*.
- Parses the SOCKS5 address (IPv4, IPv6, or Domain).
- Connects to the requested destination. (If Tor is enabled, it chains the request).
- Uses `tokio::io::copy_bidirectional` to blast data between the Axon peer and the destination.

### 3. `proxy/client.rs`
The local entry point logic.
- Listens on `client_port` (e.g. 1080).
- Expects incoming SOCKS5 connections from the *local user* (e.g. their browser).
- Before forwarding, it hits the Web Registry (`GET /api/v1/nodes`) to fetch all active nodes in the mesh.
- It randomly selects an active peer and tunnels the SOCKS5 request *to that peer's server_port*.

### 4. `logger/mod.rs`
Provides a unified logging interface across different operating systems. It conditionally initializes `android_logger` (for Android via JNI) or `env_logger` (for Desktop/CLI).
