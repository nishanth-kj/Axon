# Axon security and development status

## Current implementation notes

Axon is a work in progress. The registry, Rust core, CLI, desktop shell, Android scaffolding, and documentation site exist, but they should not be treated as a production-ready anonymity network.

### Proxy safety

- The SOCKS5 relay supports CONNECT and no-authentication mode. Do not expose it directly to the public internet without adding peer authentication, abuse controls, rate limits, and operational monitoring.
- Set `use_tor = true` only when a Tor SOCKS5 daemon is reachable at `127.0.0.1:9050`. The relay now fails closed when the Tor daemon cannot be reached instead of silently making a direct connection.
- Without Tor mode, outbound connections are direct. This is not anonymous.
- SOCKS5 does not encrypt application traffic by itself. Use HTTPS for end-to-end transport encryption.
- A relay operator may see destination metadata and unencrypted traffic. Do not use the network for sensitive activity until the threat model and implementation have been independently reviewed.
- The registry's default `SECRET_KEY` is for local development only. Configure a long, random secret before deployment and never commit credentials.
- The node verification endpoint is currently unauthenticated and should be restricted or protected before untrusted nodes can reach it.

## Local development

Start the registry:

```bash
# Use a unique development secret; production deployments must set a strong secret.
SECRET_KEY="replace-with-a-long-random-development-secret" cargo run -p server
```

In another terminal, start a node using the same registry secret:

```bash
REGISTRY_URL="http://127.0.0.1:7400" \
SECRET_KEY="replace-with-a-long-random-development-secret" \
cargo run -p cli
```

The CLI currently uses fixed local ports: verification listener `8080`, relay `8081`, and local client proxy `1080`. Ensure they are unused. The default registry URL can be overridden with `REGISTRY_URL`.

Run tests:

```bash
cargo test --workspace
```

For Tor routing, run a local Tor daemon that exposes a SOCKS5 listener on `127.0.0.1:9050`, then enable Tor mode in the node startup configuration. Confirm routing from a test environment before relying on it.

## Before public deployment

1. Remove development-secret defaults and require explicit configuration.
2. Add authentication and verification for peers, not just the registry.
3. Add rate limits, destination/port policy, relay opt-in, abuse reporting, and resource limits.
4. Review DNS, IPv6, DNS rebinding, local-network and loopback destination protections.
5. Add integration tests for SOCKS5 handshake, domain-name requests, error replies, direct mode, and Tor mode.
6. Test from clean environments and conduct an independent security review.

These are security requirements, not optional optimizations for a public proxy network.
