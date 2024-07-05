use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use bevy_simplenet::{MessageSignal, RequestSignal};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Event client used to send messages and requests, and close the client.
#[derive(SystemParam)]
pub struct EventClient<'w, E: EventPack>
{
    client: Res<'w, EventClientCore<E>>,
    registry: Res<'w, EventRegistry<E>>,
}

impl<'w, E: EventPack> EventClient<'w, E>
{
    /// Sends a message to the server.
    ///
    /// This will fail if there is a pending `ClientReport::Connected` that hasn't been read by any systems.
    pub fn send<T: SimplenetEvent>(&self, message: T) -> MessageSignal
    {
        self.client.send(&self.registry, message)
    }

    /// Sends a request to the server.
    ///
    /// This will fail if there is a pending `ClientReport::Connected` that hasn't been read by any systems.
    pub fn request<Req: SimplenetEvent>(&self, request: Req) -> Result<RequestSignal, ()>
    {
        self.client.request(&self.registry, request)
    }

    /// Closes the client.
    ///
    /// All messages and requests submitted after this is called will fail to send.
    pub fn close(&self)
    {
        self.client.close();
    }
}

//-------------------------------------------------------------------------------------------------------------------
