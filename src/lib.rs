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
//! Use `Protocol::recv()` to receive packets from a connected `TcpStream`.
//! The returned [`Protocol`] variant contains the deserialized packet data for processing.
//!
//! Each connected client can be sent packets using the provided macros, such as `send_accept!`, `send_error!`, and `send_character!`.
//!
//! ### Server Example
//!
//! ```no_run
//! use lurk_lcsc::Protocol;
//! use std::net::TcpStream;
//! use std::sync::Arc;
//!
//! let stream = Arc::new(TcpStream::connect("127.0.0.1:8080").unwrap());
//!
//! loop {
//!     let packet = match Protocol::recv(&stream) {
//!         Ok(pkt) => pkt,
//!         Err(e) => {
//!             eprintln!("Error receiving packet: {}", e);
//!             break;
//!         },
//!     };
//!
//!     match packet {
//!         Protocol::Start(start) => {
//!             // Handle start packet
//!         },
//!         _ => {
//!            // Handle other packet types
//!         },
//!     }
//! }
//! ```

/////////////////////////////////////////////////////////////////////////////////////////////////

// Lurk types in rustdoc of other crates get linked to here.
#![doc(html_root_url = "https://docs.rs/lurk_lcsc/2.3.20")]
// Show which crate feature enables conditionally compiled APIs in documentation.
#![cfg_attr(docsrs, feature(doc_cfg, rustdoc_internals))]
#![cfg_attr(docsrs, allow(internal_features))]
// Ignored clippy and clippy_pedantic lints
#![allow(
    // clippy bug: https://github.com/rust-lang/rust-clippy/issues/5704
    clippy::unnested_or_patterns,
    // clippy bug: https://github.com/rust-lang/rust-clippy/issues/7768
    clippy::semicolon_if_nothing_returned,
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

use std::io::Write;
use std::net::TcpStream;

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
///
/// ```no_run
/// use lurk_lcsc::{PktMessage, Parser, PCap};
/// use std::net::TcpStream;
/// use std::sync::Arc;
///
/// let packet = PktMessage::server("Player1", "Welcome to the game!");
///
/// let mut buffer: Vec<u8> = Vec::new();
/// packet.write_to(&mut buffer).unwrap();
///
/// println!("{}", PCap::build(buffer));
/// ```
pub mod pcap;
/// Packet type definitions.
pub mod pkt_type;
/// The Protocol.
pub mod protocol;

#[cfg(feature = "tracing")]
pub use pcap::PCap;

/// Serialize a packet and write it directly to a [`TcpStream`].
pub fn send_to<'a>(
    stream: &TcpStream,
    packet: &(impl Parser<'a> + std::fmt::Display),
) -> Result<(), std::io::Error> {
    let mut buf = Vec::new();

    #[cfg(feature = "tracing")]
    tracing::info!("Sending packet: {}", packet);

    packet.write_to(&mut buf)?;

    #[cfg(feature = "tracing")]
    tracing::trace!("Packet:\n{}", PCap::build(buf.clone()));

    let mut writer = stream;
    writer.write_all(&buf)
}
