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
1. **POST to Registry**: The node sends a `POST /api/v1/registry` request to the Web Coordinator, providing its public `server_port`. **Important**: The server does *not* return a token in the response!
2. **Verification Callback**: To prove the node is reachable, the Web Coordinator reaches out and sends an HTTP POST callback to the node's local Axum server (`/api/v1/verify`), providing a unique `secret_key` (Token). The node receives and saves this token.
3. **Heartbeat Loop**: The node periodically sends a `POST /api/v1/heartbeat` request, authenticating with the saved `secret_key`.
4. **Re-registration**: If the heartbeat is rejected (e.g., the token is invalid or the node was kicked), the node deletes the token and automatically loops back to Step 1 to re-register.

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

## 📁 Directory Structure & How to Run

Because Axon is designed to run anywhere, the logic is heavily modularized into several directories:

### 🌐 Web Coordinator
- **`web/`**: The Next.js Web Coordinator, Dashboard, and Registry API.
  - Contains the **/nodes** dashboard which provides real-time telemetry on active relay nodes.
  - **Environment Variables** (`web/.env`):
    - `DEBUG=true` - Enables verbose logging
    - `SECRET_KEY=...` - Required token secret for node verification
    - `STATIC_EXPORT=true|false` - Set to `true` to export a purely static UI (for GitHub Pages). Note that enabling this breaks the Next.js API route backends. Use `false` if deploying to Vercel/Render.
  - **How to run**: 
    ```bash
    cd web
    npm install
    npm run dev
    ```

### 🧠 Core Engine
- **`core/`**: The Rust library crate containing 100% of the proxy, heartbeat, and our new Tor integration logic. It is the engine of Axon.
  - **How to run**: Since this is a library, you don't run it directly, but you can run its tests:
    ```bash
    cargo test -p core
    ```

### 💻 End-User Applications
- **`cli/`**: A lightweight Rust wrapper that runs the `core` engine in the terminal.
  - **How to run**:
    ```bash
    cargo run -p cli
    ```
- **`desktop/`**: A GUI wrapper (Tauri/Iced/etc.) that runs the `core` engine for desktop users.
  - **How to run**:
    ```bash
    cargo run -p desktop
    ```
- **`axon-android/`**: The Android application and Rust/JNI bindings that allow the `core` engine to run as a background service on mobile.
  - **How to run**: Open the `axon-android` directory in Android Studio and run it on a device or emulator.

### 📦 Utilities & Others
- **`docs/`**: Project documentation, architecture diagrams, and guides.
- **`lib/`**: Language bindings (contains `npm` and `pypip` folders) to interact with Axon from Node.js or Python.
- **`server/`**: An empty directory reserved for a future standalone proxy server implementation.
- **`tor/`**: An empty directory created for standalone Tor experiments (the actual Tor integration now lives in `core/src/tor/`).

### Cargo Workspace
All the Rust crates (`core`, `cli`, `desktop`) are linked together using a single root `Cargo.toml` workspace. This ensures they all share the exact same dependencies and versions, making compilation significantly faster. You can run `cargo build` in the root directory to compile all Rust components at once.
