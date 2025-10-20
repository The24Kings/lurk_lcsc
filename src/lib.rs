//! # Lurk LCSC Protocol
//!
//! A lightweight Rust implementation of the **Lurk** protocol created by S. Seth Long, Ph.D.
//! It provides the fundamental logic for building multiplayer game servers that communicate using the Lurk protocol.
//!
//! This crate provides types and utilities for working with Lurk protocol packets,
//! including parsing, error handling, and protocol definitions.
//!
//! ## Features
//! - Optional `tracing` support for structured logging and diagnostics.
//!
//! For more details about the protocol itself, see the [LURK Protocol Wiki](https://github.com/The24Kings/LurkProtocol/wiki).

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
pub use packet::{Packet, Parser};
pub use packet::{
    accept::PktAccept, change_room::PktChangeRoom, character::PktCharacter,
    connection::PktConnection, error::PktError, fight::PktFight, game::PktGame, leave::PktLeave,
    loot::PktLoot, message::PktMessage, pvp_fight::PktPVPFight, room::PktRoom, start::PktStart,
    version::PktVersion,
};
pub use pkt_type::PktType;
pub use protocol::Protocol;

/// Structures and utilities for character flags.
pub mod flags;
/// Error types for the Lurk protocol.
pub mod lurk_error;
/// Packet definitions and parsing logic.
pub mod packet;
#[cfg(feature = "tracing")]
/// Packet capture and tracing utilities.
pub mod pcap;
/// Packet type definitions.
pub mod pkt_type;
/// Protocol definitions.
pub mod protocol;

#[cfg(feature = "tracing")]
pub use pcap::PCap;
