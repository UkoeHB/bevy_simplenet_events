//documentation
#![doc = include_str!("../README.md")]
#![allow(unused_imports)]
use crate as bevy_simplenet_events;

//module tree
mod core;
mod event_registry;

#[cfg(feature = "client")]
mod client;
//#[cfg(feature = "server")]
//mod server;

//API exports
pub use crate::core::*;
pub use crate::event_registry::*;

#[cfg(feature = "client")]
pub use crate::client::*;
//#[cfg(feature = "server")]
//pub use crate::server::*;
