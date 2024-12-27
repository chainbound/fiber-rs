#![doc = "A Rust client to interact with the Fiber Network. 
Please see the repository [README](https://github.com/chainbound/fiber-rs/blob/main/README.md) for more information."]
#![warn(
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    clippy::missing_const_for_fn,
    rustdoc::all
)]
#![deny(unused_must_use, rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

/// Module responsible for handling the public-facing API client.
mod client;
pub use client::{Client, ClientOptions};

/// Module responsible for dispatching requests to the API backend.
/// This spawns a tokio task to handle requests and responses in the background.
///
/// This module is not intended to be used directly by the end-user and as such
/// is not exposed in the library API.
mod dispatcher;
use dispatcher::{Dispatcher, SendType};

/// Auto-generated gRPC API bindings.
#[allow(unreachable_pub)]
mod generated;

/// Utilities for the library.
mod utils;
