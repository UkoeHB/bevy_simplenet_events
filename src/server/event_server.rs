//local shortcuts
use crate::*;

//third-party shortcuts
use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use bevy_simplenet::{CloseFrame, MessageSignal, RequestSignal};
use bincode::Options;

//standard shortcuts
use std::fmt::Debug;
use std::sync::atomic::{AtomicBool, Ordering};

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
    pub fn send<T: SimplenetEvent>(&self, session_id: SessionId, message: T) -> Result<(), ()>
    {
        self.server.send(&self.registry, session_id, message)
    }

    /// Responds to a client request.
    pub fn respond<Resp: SimplenetEvent>(&self, token: RequestToken, response: Resp) -> Result<(), ()>
    {
        self.server.respond(&self.registry, token, response)
    }

    /// Acknowledges a client request.
    pub fn ack(&self, token: RequestToken) -> Result<(), ()>
    {
        self.server.ack(&self.registry, token)
    }

    /// Rejects a client request.
    pub fn reject(&self, token: RequestToken) -> Result<(), ()>
    {
        self.server.reject(&self.registry, token)
    }

    /// Closes a client.
    ///
    /// All messages and requests submitted to the client after this is called will fail to send.
    pub fn close_session(&self, session_id: SessionId, close_frame: Option<CloseFrame>)
    {
        self.server.close_session(session_id, close_frame)
    }
}

//-------------------------------------------------------------------------------------------------------------------
