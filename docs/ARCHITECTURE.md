# Architecture Overview

Axon is a Peer-to-Peer (P2P) proxy network designed to provide censorship-resistant, anonymous internet access. The architecture consists of a centralized discovery coordinator (Web Registry) and decentralized traffic nodes (Rust Proxy).

## The Dual-Role Proxy Model

Unlike traditional VPNs where a client connects directly to a central server, Axon utilizes a mesh-like approach. When a user runs the Axon Rust application on their device, their device becomes **both a Client and a Server**.

### 1. The Client (Entry Point)
When an Axon node starts, it opens a local port (e.g. `127.0.0.1:1080`). 
The user points their browser's SOCKS5 configuration to this local port. 
When the browser requests a website, the node:
1. Queries the Web Registry to find a list of other users currently running Axon.
2. Randomly selects one of those users (an "Active Peer").
3. Connects to that peer's public Server Port.
4. Tunnels the encrypted SOCKS5 request through that peer.

### 2. The Server (Relay Node)
Simultaneously, the user's Axon node listens on a public port (e.g. `8081`).
1. It registers this public port with the Web Registry so other users can find it.
2. When another Axon user selects this node from the registry, they connect to this port.
3. The node accepts the incoming SOCKS5 connection and acts as a standard proxy, forwarding the traffic out to the open internet and returning the result to the connected peer.

### 3. The Tor Integration
To provide anonymity at the exit node layer, if the Axon node is configured with `use_tor = true`, the Server logic will NOT forward the traffic directly to the open internet. Instead, it forwards the traffic to the local Tor daemon (`127.0.0.1:9050`). This chains the Axon network directly into the Tor network, ensuring the exit traffic is untraceable back to the Relay Node!

## Sequence Diagram: A Proxy Request

1. **Browser** -> `SOCKS5 request` -> **Local Axon Node (Client)**
2. **Local Axon Node** -> `GET /api/v1/nodes` -> **Web Registry** (Receives Peer B's IP)
3. **Local Axon Node** -> `SOCKS5 tunnel` -> **Peer B's Axon Node (Server)**
4. **Peer B** -> `TCP connection` -> **Target Website (e.g. Google)**
5. **Target Website** -> `Response` -> **Peer B** -> **Local Axon Node** -> **Browser**
