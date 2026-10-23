# Axon Web Registry

The central registry is the Actix Web service in `server/`. The Next.js app in `web/` is documentation and a read-only node view. The browser calls the Actix server directly.

## Purpose

Since Axon relies on peer-to-peer proxying, nodes need a way to discover each other. The Web Registry maintains a real-time database of all active nodes, allowing Client nodes to fetch the list and randomly select peers for routing traffic.

## API Endpoints

The Actix process serves these routes. The registry relies on a strict handshake and heartbeat protocol to ensure nodes are actually alive before distributing their IPs to clients.

### 1. Registration (`POST /api/v1/registry`)

When a node starts up, it hits this endpoint to declare its presence. The response does *not* contain the token.

- **Payload**: `{ status: 1, port: <server_port> }`
- **Action**: The registry records the node's IP address (from the incoming HTTP request header) and its designated `server_port`. It marks the node as "PENDING".
- **Callback**: To verify reachability, the registry immediately makes an outbound POST request *back* to the node's IP on its verification port (`/api/v1/verify`), providing a unique `secretKey`. The node receives and saves this token. The registration request must send `Authorization: Token <SECRET_KEY>`.

### 2. Heartbeat (`POST /api/v1/heartbeat`)

Nodes must prove they are still alive by pinging this endpoint periodically (every 5 to 30 seconds).

- **Headers**: `Authorization: Token <secretKey>`
- **Action**: If the `secretKey` is valid, the node's status is updated to "ACTIVE". If the token is invalid or the node misses too many heartbeats, it is marked as "OFFLINE" and removed from the active peer list. If the client receives an error, it deletes its token and re-registers.

### 3. Node Discovery (`GET /api/v1/nodes`)

This is the endpoint queried by the Client Proxy component of an Axon node.

- **Action**: Returns `{ status, data: { nodes } }` for nodes marked ACTIVE. With `DEBUG=true`, PENDING nodes are included and each entry also has `status`.
- **Usage**: The local Client Proxy uses this list to randomly select a remote Axon peer and route the user's SOCKS5 traffic to it.

## Where the code lives

- **`server/src/controller/`**: HTTP routes. Each handler reads the request and calls the service.
- **`server/src/service/`**: Registration, heartbeats, discovery, and the health probe.
- **`server/src/model/request.rs`** and **`response.rs`**: Request and response bodies.
- **`server/src/model/custom/`**: Node, config, and page data structs.
- **`server/src/utils/logger.rs`**: Writes `server.log` and mirrors the same lines to stderr.
- **`server/Dockerfile`** and **`docker-compose.yml`**: Container build. The published port is `7400`.

The registry stays off the data plane. Proxy traffic goes from node to node. The Actix process only stores who is online.

Run it with `docker compose up --build registry` or `cargo run -p server`. `SECRET_KEY` must match the token a node sends on `POST /api/v1/registry`. The development default is `Nishanth`.
