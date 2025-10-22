//! # Lurk LCSC Protocol
//!
//! A lightweight Rust implementation of the **Lurk** protocol created by S. Seth Long, Ph.D.
//! It provides the fundamental logic for building multiplayer game servers that communicate using the Lurk protocol.
//!
//! This crate provides types and utilities for working with Lurk protocol packets,
//! including parsing, error handling, and protocol definitions.
//!
//! ## Features
//! - Optional `tracing` support for structured logging and diagnostics. Also adds the [`PCap`] type for capturing and debugging packet data.
//!
//! For more details about the protocol itself, see the [Lurk Protocol Wiki](https://github.com/The24Kings/LurkProtocol/wiki).
//!
//! ## Where to start
//!
//! A good starting point is the [`Protocol`] enum, which defines the various packet types and their associated data structures.
//! You can also explore the [`packet`] module for detailed packet definitions and parsing logic.
//!
//! # Basic Usage
//!
//! Server thread uses `Protocol::recv()` to receive packets from connected clients over `TcpStream`
//! and the packet's corresponding struct is returned for processing.
//!
//! Each connected client can be sent packets using the provided macros, such as `send_accept!`, `send_error!`, and `send_character!`.
//!
//! ### Server Example
//!
//! ```no_run
//! use lurk_lcsc::Protocol;
//! use std::net::TcpStream;
//! use std::sync::{Arc, mpsc, Mutex};
//!
//! let (_tx, rx) = mpsc::channel();
//!
//! let receiver = Arc::new(Mutex::new(rx));
//!
//! std::thread::spawn(move || {
//!     loop {
//!         let packet = receiver.lock().unwrap().recv().unwrap();
//!
//!         match packet {
//!             Protocol::Start(author, content) => {
//!                 // Handle start packet
//!             },
//!             _ => {
//!                // Handle other packet types
//!             },
//!        }
//!    }
//! });
//! ```
//!
//! ### Client Example
//!
//! ```no_run
//! use lurk_lcsc::Protocol;
//! use std::net::TcpStream;
//! use std::sync::{Arc, mpsc};
//!
//! let (tx, _rx) = mpsc::channel();
//!
//! let stream = Arc::new(TcpStream::connect("127.0.0.1:8080").unwrap());
//! let sender = Arc::new(tx);
//!
//! std::thread::spawn(move || {
//!     loop {
//!         match Protocol::recv(&stream) {
//!             Ok(packet) => {
//!                 // Send the received packet to the main thread for processing
//!                 sender.send(packet).unwrap();
//!             },
//!             Err(e) => eprintln!("Error receiving packet: {}", e),
//!         }
//!     }
//! });
//! ```

/////////////////////////////////////////////////////////////////////////////////////////////////

// Lurk types in rustdoc of other crates get linked to here.
#![doc(html_root_url = "https://docs.rs/lurk_lcsc/2.3.9")]
// Show which crate feature enables conditionally compiled APIs in documentation.
#![cfg_attr(docsrs, feature(doc_cfg, rustdoc_internals))]
#![cfg_attr(docsrs, allow(internal_features))]
// Ignored clippy and clippy_pedantic lints
#![allow(
    // clippy bug: https://github.com/rust-lang/rust-clippy/issues/5704
    clippy::unnested_or_patterns,
    // clippy bug: https://github.com/rust-lang/rust-clippy/issues/7768
    clippy::semicolon_if_nothing_returned,
    // not available in our oldest supported compiler
    clippy::empty_enum,
    clippy::type_repetition_in_bounds, // https://github.com/rust-lang/rust-clippy/issues/8772
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    // things are often more readable this way
    clippy::cast_lossless,
    clippy::module_name_repetitions,
    clippy::single_match_else,
    clippy::type_complexity,
    clippy::use_self,
    clippy::zero_prefixed_literal,
    // correctly used
    clippy::derive_partial_eq_without_eq,
    clippy::enum_glob_use,
    clippy::explicit_auto_deref,
    clippy::incompatible_msrv,
    clippy::let_underscore_untyped,
    clippy::map_err_ignore,
    clippy::new_without_default,
    clippy::result_unit_err,
    clippy::wildcard_imports,
    // not practical
    clippy::needless_pass_by_value,
    clippy::similar_names,
    clippy::too_many_lines,
    // preference
    clippy::doc_markdown,
    clippy::needless_lifetimes,
    clippy::unseparated_literal_suffix,
    // false positive
    clippy::needless_doctest_main,
    // noisy
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
)]
// Rustc lints.
#![deny(missing_docs, unused_imports)]

////////////////////////////////////////////////////////////////////////////////

pub use flags::CharacterFlags;
pub use lurk_error::LurkError;
#[doc(hidden)]
pub use packet::Packet;
pub use packet::Parser;
pub use packet::{
    accept::PktAccept, change_room::PktChangeRoom, character::PktCharacter,
    connection::PktConnection, error::PktError, fight::PktFight, game::PktGame, leave::PktLeave,
    loot::PktLoot, message::PktMessage, pvp_fight::PktPVPFight, room::PktRoom, start::PktStart,
    version::PktVersion,
};
pub use pkt_type::PktType;
pub use protocol::Protocol;

/// Flags representing the state of a character in the game.
///
/// When a client uses [`PktType::CHARACTER`] to describe a new player, the server may (should) ignore the client's initial specification for flags, health, gold, etc.
/// using [`CharacterFlags::reset()`].
/// > Since the character packet is shared between players and monsters, the server is responsible for setting these values correctly.
pub mod flags;
/// Error types for the Lurk protocol.
pub mod lurk_error;
/// Module for handling various packet types in the Lurk protocol.
///
/// This module defines the [`Parser`] trait for serializing and deserializing packets,
/// as well as the various packet structures used in the protocol.
pub mod packet;
#[cfg(feature = "tracing")]
/// Packet capture and tracing utilities.
pub mod pcap;
/// Packet type definitions.
pub mod pkt_type;
/// The Protocol.
pub mod protocol;

#[cfg(feature = "tracing")]
pub use pcap::PCap;

/// Testing utilities and common setup for tests.
#[doc(hidden)]
#[allow(dead_code)] // Suppress unused warning as this module is used in multiple test modules.
pub(crate) mod test_common;
