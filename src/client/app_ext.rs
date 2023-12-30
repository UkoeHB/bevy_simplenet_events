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

pub trait SimplenetClientEventAppExt
{
    /// Inserts a `bevy_simplenet` client.
    ///
    /// This must be called on your app in order to use [`EventClient`].
    fn insert_client<E: EventPack>(&mut self, client: Client<E>) -> &mut Self;
}

impl SimplenetClientEventAppExt for App
{
    fn insert_client<E: EventPack>(&mut self, client: Client<E>) -> &mut Self
    {
        self.insert_resource(EventClientCore::new(client));
        self
    }
}

//-------------------------------------------------------------------------------------------------------------------
