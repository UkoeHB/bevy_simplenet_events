//local shortcuts
use crate::*;

//third-party shortcuts
use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use bevy_simplenet::{CloseFrame, RequestToken, ClientId};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Event server used to send messages and responses to clients, and disconnect clients.
#[derive(SystemParam)]
pub struct EventServer<'w, E: EventPack>
{
    server   : Res<'w, EventServerCore<E>>,
    registry : Res<'w, EventRegistry<E>>,
}

impl<'w, E: EventPack> EventServer<'w, E>
{
    /// Sends a message to a client.
    ///
    /// This will fail if there is a pending `ServerReport::Connected` that hasn't been read by any systems.
    pub fn send<T: SimplenetEvent>(&self, client_id: ClientId, message: T)
    {
        self.server.send(&self.registry, client_id, message)
    }

    /// Responds to a client request.
    pub fn respond<Resp: SimplenetEvent>(&self, token: RequestToken, response: Resp)
    {
        self.server.respond(&self.registry, token, response)
    }

    /// Acknowledges a client request.
    pub fn ack(&self, token: RequestToken)
    {
        self.server.ack(token)
    }

    /// Rejects a client request.
    pub fn reject(&self, token: RequestToken)
    {
        self.server.reject(token)
    }

    /// Closes a client.
    ///
    /// All messages and requests submitted to the client after this is called will fail to send.
    pub fn disconnect_client(&self, client_id: ClientId, close_frame: Option<CloseFrame>)
    {
        self.server.disconnect_client(client_id, close_frame)
    }
}

//-------------------------------------------------------------------------------------------------------------------
