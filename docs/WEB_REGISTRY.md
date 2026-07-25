# Axon Web Registry

The `web/` directory contains a **Next.js** application that acts as the central brain and coordinator for the decentralized Axon network.

## Purpose

Since Axon relies on peer-to-peer proxying, nodes need a way to discover each other. The Web Registry maintains a real-time database of all active nodes, allowing Client nodes to fetch the list and randomly select peers for routing traffic.

## API Endpoints (`/app/api/v1/`)

The registry relies on a strict handshake and heartbeat protocol to ensure nodes are actually alive before distributing their IPs to clients.

### 1. Registration (`POST /api/v1/registry`)
When a node starts up, it hits this endpoint to declare its presence. The response does *not* contain the token.
- **Payload**: `{ status: 1, port: <server_port> }`
- **Action**: The registry records the node's IP address (from the incoming HTTP request header) and its designated `server_port`. It marks the node as "PENDING".
- **Callback**: To verify reachability, the registry immediately makes an outbound POST request *back* to the node's IP on port 8080 (`/api/v1/verify`), providing a unique `secretKey`. The node receives and saves this token.

### 2. Heartbeat (`POST /api/v1/heartbeat`)
Nodes must prove they are still alive by pinging this endpoint periodically (every 5 to 30 seconds).
- **Headers**: `Authorization: Token <secretKey>`
- **Action**: If the `secretKey` is valid, the node's status is updated to "ACTIVE". If the token is invalid or the node misses too many heartbeats, it is marked as "OFFLINE" and removed from the active peer list. If the client receives an error, it deletes its token and re-registers.

### 3. Node Discovery (`GET /api/v1/nodes`)
This is the endpoint queried by the Client Proxy component of an Axon node.
- **Action**: Returns a JSON array of all nodes currently marked as "ACTIVE".
- **Usage**: The local Client Proxy uses this list to randomly select a remote Axon peer and route the user's SOCKS5 traffic to it.

## Architecture & Database

The Web Registry uses a service-repository pattern:
- **`lib/services/NodeService.ts`**: Contains the business logic for verifying nodes, tracking heartbeats, and fetching active instances.
- **`lib/repository/NodeRepository.ts`**: The data access layer.

By keeping the Web Registry completely decoupled from the data plane (the actual proxy traffic never flows through the web server), the registry remains incredibly lightweight while the Rust nodes handle the heavy bandwidth lifting!
