//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::app::App;
use bincode::Options;
use serde::{Serialize, Deserialize};
use serde_with::{Bytes, serde_as};

//standard shortcuts
use core::fmt::Debug;
use std::marker::PhantomData;

//-------------------------------------------------------------------------------------------------------------------

pub trait SimplenetServerEventAppExt
{
    /// Prepars a `bevy_simplenet` server for use in [`EventServer`].
    fn insert_simplenet_server<E: EventPack>(&mut self, server: Server<E>) -> &mut Self;
}

impl SimplenetServerEventAppExt for App
{
    fn insert_simplenet_server<E: EventPack>(&mut self, server: Server<E>) -> &mut Self
    {
        self.insert_resource(EventServerCore::new(server));
        self
    }
}

//-------------------------------------------------------------------------------------------------------------------
