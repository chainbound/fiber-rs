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

/// Module responsible for dispatching requests to the API backend.
/// This spawns a tokio task to handle requests and responses in the background.
pub mod dispatcher;
pub use dispatcher::{Dispatcher, SendType};

/// Module responsible for handling the public-facing API client.
pub mod client;
pub use client::{Client, ClientOptions};

#[allow(unreachable_pub)]
mod generated;
mod utils;
