//features
#![cfg_attr(docsrs, feature(doc_cfg))]

//documentation
#![doc = include_str!("../README.md")]
#![allow(unused_imports)]
use crate as bevy_simplenet_events;

//module tree
mod core;
mod event_registry;

#[cfg(feature = "client")]
#[cfg_attr(docsrs, doc(cfg(feature = "client")))]
mod client;

#[cfg(feature = "server")]
#[cfg_attr(docsrs, doc(cfg(feature = "server")))]
mod server;

//API exports
pub use crate::core::*;
pub use crate::event_registry::*;

#[cfg(feature = "client")]
pub use crate::client::*;
#[cfg(feature = "server")]
pub use crate::server::*;

pub use bevy_simplenet_events_derive::*;
