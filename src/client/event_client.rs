//local shortcuts
use crate::*;

//third-party shortcuts
use bincode::Options;

//standard shortcuts
use core::fmt::Debug;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

//-------------------------------------------------------------------------------------------------------------------

/// Event client used to send messages and requests, and close the client.
#[derive(SystemParam)]
pub struct EventClient<'w, 's, E: EventPack>
{
    client   : Res<'w, EventClientCore<E>>,
    registry : Res<'w, EventRegistry<E>>,
}

impl<E: EventPack> EventClient<E>
{
    /// Sends a message to the server.
    pub fn send(&self, message: E::ClientMsg) -> Result<MessageSignal, ()>
    {
        self.client.send(&self.registry, message)
    }

    /// Sends a request to the server.
    pub fn request<Req: SimplenetEvent>(&self, request: Req) -> Result<RequestSignal, ()>
    {
        self.client.request(&self.registry, request)
    }

    /// Closes the client.
    ///
    /// All messages and requests submitted after this is called will fail to send.
    pub fn close(&self) -> Result<MessageSignal, ()>
    {
        self.client.close()
    }
}

//-------------------------------------------------------------------------------------------------------------------
