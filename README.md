# Axon: Decentralized Proxy Network

**Live Environment:**

- 🌍 **Production**: [https://nishanth-kj.github.io/Axon](https://nishanth-kj.github.io/Axon)

Axon is a decentralized, peer-to-peer proxy network built with Rust and Next.js. It allows users to route their internet traffic securely and anonymously through a network of volunteer nodes, similar to Tor, but with a custom architecture.

The project is split into a **central registry** (`server/`, an Actix Web service) and **proxy nodes** (Core/CLI/Desktop/Android). The Next.js app in `web/` is documentation and a read-only view of the nodes.

---

## 🏗️ High-Level Architecture

The system relies on two main components:

1. **The central registry (`server/`)**: An Actix Web service and the directory of nodes that are currently willing to relay traffic. Run it with Docker or `cargo run -p server`.
2. **The Proxy Nodes (`core/`)**: Rust-based applications that run on user devices (Desktop, Mobile, Servers). These nodes do the actual heavy lifting of proxying traffic.

Every proxy node in the Axon network acts as **both a Client and a Server simultaneously**.

---

## ⚙️ How It Works (The Node Lifecycle)

### 1. Node Registration & Verification

When a user launches an Axon node (e.g., via the CLI or Desktop app):

1. **POST to Registry**: The node sends a `POST /api/v1/registry` request to the Actix registry, providing its public `server_port`. **Important**: The server does *not* return a token in the response!
2. **Verification Callback**: To prove the node is reachable, the registry reaches out and sends an HTTP POST callback to the node's local Axum server (`/api/v1/verify`), providing a unique `secret_key` (Token). The node receives and saves this token.
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
- **Peer Discovery**: When the user tries to load a webpage, the local node asks the Actix registry: *"Give me a list of all active nodes in the network"* (`GET /api/v1/nodes`).
- **Traffic Routing**: The local node picks a random active peer from the list and forwards the user's SOCKS5 request to that remote peer's Server Port.
- The remote peer fetches the data and sends it back through the tunnel.

---

## Development and security status

Axon is an active work in progress, not yet a production-ready anonymity network. Review [Security and Development](docs/SECURITY_AND_DEVELOPMENT.md) before running nodes, especially before exposing relay ports to the public internet. Tor mode requires a local Tor SOCKS5 listener at `127.0.0.1:9050`; the relay now fails closed when that service is unavailable instead of silently falling back to direct connections.

## 📁 Directory Structure & How to Run

Because Axon is designed to run anywhere, the logic is heavily modularized into several directories:

### 🖥 Central registry

- **`server/`**: The Actix Web registry. Nodes register, heartbeat, and discover peers here. Proxy traffic does not flow through this process.
  - Listens on `0.0.0.0:7400` unless `AXON_BIND` is set.
  - `SECRET_KEY` is the registration token. The CLI uses `Nishanth` when the variable is unset, which matches the Docker Compose default. Change both together before a public deployment.
  - `DEBUG=true` includes nodes that are still `PENDING` in `GET /api/v1/nodes`.
  - Log lines are appended to `server.log` in the working directory (`AXON_LOG_FILE` overrides the path) and copied to stderr.
  - **How to run with Docker**:

    ```bash
    docker compose up --build registry
    ```

  - **How to run on the host**:

    ```bash
    cargo run -p server
    ```

### 🌐 Web dashboard

- **`web/`**: Documentation pages and the read-only **/nodes** view. It does not implement the registry.
  - The nodes page reads `NEXT_PUBLIC_REGISTRY_URL` (default `http://127.0.0.1:7400`).
  - **Environment Variables** (`web/.env`):
    - `NEXT_PUBLIC_REGISTRY_URL` - Actix registry base URL
    - `STATIC_EXPORT=true|false` - Set to `true` to export a purely static UI (for GitHub Pages).
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

- **`cli/`**: A terminal node. Flags override `REGISTRY_URL` and `SECRET_KEY`.
  - **How to run**:

    ```bash
    cargo run -p cli -- --help
    cargo run -p cli
    cargo run -p cli -- --registry http://127.0.0.1:7400 --socks-port 1080
    ```

- **`desktop/`**: An Iced window for the node. It registers with the Actix registry, opens the local SOCKS proxy, and shows the node log. Settings are saved under the Axon config directory.
  - **How to run**:

    ```bash
    cargo run -p desktop
    ```

- **`axon-android/`**: An Android screen for the same node. The activity stores the registry, secret, and ports, and a foreground service keeps the Rust node running.
  - Build the native library into the app, then open `axon-android/` in Android Studio:

    ```bash
    cargo install cargo-ndk
    cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 -o axon-android/app/src/main/jniLibs build -p axon_android --release
    ```

### 📦 Utilities & Others

- **`docs/`**: Project documentation, architecture diagrams, and guides.
- **`lib/`**: Language bindings (contains `npm` and `pypip` folders) to interact with Axon from Node.js or Python.
- **`tor/`**: An empty directory created for standalone Tor experiments (the actual Tor integration now lives in `core/src/tor/`).

### Cargo Workspace

All the Rust crates (`core`, `cli`, `desktop`, `server`) are linked together using a single root `Cargo.toml` workspace. This ensures they all share the exact same dependencies and versions, making compilation significantly faster. You can run `cargo build` in the root directory to compile all Rust components at once.
