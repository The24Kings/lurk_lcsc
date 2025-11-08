# lurk_lcsc

A Rust library for building **Lurk protocol** servers and clients.  
This crate implements the [Lurk Protocol](https://github.com/The24Kings/LurkProtocol/wiki), enabling MUD text-based MMORPG games over the network.

---

## Features

- Provides core data structures for Lurk messages (character, room, etc.)
- Handles parsing and serialization of Lurk protocol messages from TcpStream
- Ready to be used in both **client** and **server** implementations

### Optional Features

- `tracing`: Enables logging via the [tracing](https://crates.io/crates/tracing) crate

---

## Installation

Add `lurk_lcsc` to your `Cargo.toml`:

```toml
[dependencies]
lurk_lcsc = { version = "2.3.11" }
```
