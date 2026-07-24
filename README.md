# Axon: Decentralized Proxy Network

Axon is a decentralized, peer-to-peer proxy network built with Rust and Next.js. It allows users to route their internet traffic securely and anonymously through a network of volunteer nodes, similar to Tor, but with a custom architecture. 

The project is split into a **Web Coordinator** (Registry) and **Proxy Nodes** (Core/CLI/Desktop/Android).

---

## 🏗️ High-Level Architecture

The system relies on two main components:
1. **The Web Registry (`web/`)**: A centralized coordinator built with Next.js. It acts as a directory of all currently active nodes in the network.
2. **The Proxy Nodes (`core/`)**: Rust-based applications that run on user devices (Desktop, Mobile, Servers). These nodes do the actual heavy lifting of proxying traffic.

Every proxy node in the Axon network acts as **both a Client and a Server simultaneously**.

---

## ⚙️ How It Works (The Node Lifecycle)

### 1. Node Registration & Verification
When a user launches an Axon node (e.g., via the CLI or Desktop app):
1. **POST to Registry**: The node sends a `POST /api/v1/registry` request to the Web Coordinator, providing its public `server_port` and declaring itself available.
2. **Verification Callback**: The Web Coordinator generates a unique `secret_key` and sends it back to the node via an HTTP POST callback to the node's local Axum server (`/api/v1/verify`).
3. **Heartbeat Loop**: To stay "Active" in the registry, the node periodically sends a `POST /api/v1/heartbeat` request to the Web Coordinator, authenticating with the `secret_key` it received.

### 2. Server Mode (Acting as a Relay)
While the node is running, it spins up a **SOCKS5 Server** on a public port (e.g., `8081`).
- When *other* Axon users want to access a website, their traffic is routed to this port.
- The node receives the encrypted SOCKS5 request and forwards it to the final destination (e.g., `google.com`).
- **Tor Integration**: If the node is configured with `use_tor = true`, instead of fetching the website directly, it forwards the traffic through the local Tor daemon (`127.0.0.1:9050`), providing an extra layer of anonymity!

### 3. Client Mode (Acting as an Entry Point)
At the same time, the node spins up a **Local SOCKS5 Proxy** (e.g., `127.0.0.1:1080`) for the local user.
- The user configures their web browser (e.g., Firefox) to use `127.0.0.1:1080` as its SOCKS5 proxy.
- **Peer Discovery**: When the user tries to load a webpage, the local node asks the Web Coordinator: *"Give me a list of all active nodes in the network"* (`GET /api/v1/nodes`).
- **Traffic Routing**: The local node picks a random active peer from the list and forwards the user's SOCKS5 request to that remote peer's Server Port.
- The remote peer fetches the data and sends it back through the tunnel.

---

## 📁 Directory Structure

Because Axon is designed to run anywhere, the Rust logic is heavily modularized:

- **`web/`**: The Next.js Web Coordinator and Dashboard.
- **`core/`**: The Rust library crate containing 100% of the proxy, heartbeat, and Tor integration logic. It is the engine of Axon.
- **`cli/`**: A lightweight Rust wrapper that runs the `core` engine in the terminal.
- **`desktop/`**: A GUI wrapper (Tauri/Iced/etc) that runs the `core` engine for desktop users.
- **`axon-android/rust/`**: The JNI/Rust bindings that allow the `core` engine to run as a background service on Android phones.

### Cargo Workspace
All the Rust crates (`core`, `cli`, `desktop`, `axon-android/rust`) are linked together using a single root `Cargo.toml` workspace. This ensures they all share the exact same dependencies and versions, making compilation significantly faster.
