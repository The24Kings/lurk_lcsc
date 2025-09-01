# lurk_lcsc

A Rust library for building **LURK protocol** servers and clients.  
This crate implements the [LURK Protocol](https://github.com/The24Kings/LurkProtocol/wiki), enabling MUD text-based MMORPG games over the network.

> [!NOTE]
> I probably won't be adding anything more to this, or creating documentation, go read the wiki.

---

## Features

- Provides core data structures for LURK messages (character, room, etc.)
- Handles parsing and serialization of LURK protocol messages from TcpStream
- Ready to be used in both **client** and **server** implementations

### Optional Features

- `tracing`: Enables logging via the [tracing](https://crates.io/crates/tracing) crate
- `commands`: Adds an additional Protocol Message `Command` that currently implements `HELP`, `BROADCAST`, `MESSAGE`, `NUKE`, `OTHER` commands

---

## Installation

Add `lurk_lcsc` to your `Cargo.toml`:

```toml
[dependencies]
lurk_lcsc = { version = "2.3.5" }
```
