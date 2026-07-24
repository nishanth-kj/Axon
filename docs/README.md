# Axon Documentation

Welcome to the comprehensive documentation for the Axon Decentralized Proxy Network!

This folder contains deep-dive explanations of the entire codebase. Please refer to the following documents for specific sub-systems:

- [ARCHITECTURE.md](file:///c:/Projects/Axon/docs/ARCHITECTURE.md) - A full overview of how the decentralized P2P proxy system routes traffic.
- [CORE_ENGINE.md](file:///c:/Projects/Axon/docs/CORE_ENGINE.md) - Deep dive into the Rust proxy engine (`core/` crate), the SOCKS5 protocols, and the dual-role system.
- [WEB_REGISTRY.md](file:///c:/Projects/Axon/docs/WEB_REGISTRY.md) - Explanation of the Next.js Web Coordinator that manages active nodes.

## Repository Layout

```text
c:\Projects\Axon\
├── docs/             # You are here!
├── web/              # Next.js Central Web Coordinator (Registry)
├── core/             # Rust Library (The Brains of the Proxy)
├── cli/              # Rust CLI Wrapper using `core/`
├── desktop/          # Rust Desktop GUI Wrapper using `core/`
└── axon-android/     # Rust Android Service Wrapper using `core/`
```
