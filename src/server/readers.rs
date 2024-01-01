//local shortcuts
use crate::*;

//third-party shortcuts
use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use bevy_simplenet::ClientReport;

//standard shortcuts
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};

//-------------------------------------------------------------------------------------------------------------------

/// Server reader for client connection events.
#[derive(SystemParam)]
pub struct ServerConnectionReader<'w, E: EventPack>
{
    server : Res<'w, EventServerCore<E>>,
    events : Res<'w, ServerConnectionQueue<E>>,
}

impl<'w, E: EventPack> ServerConnectionReader<'w, E>
{
    /// Iterates the available connection reports.
    pub fn iter(&self) -> impl Iterator<Item = (SessionId, &ServerReport<E::ConnectMsg>)> + '_
    {
        self.events
            .iter()
            .map(|(counter, id, report)|
                {
                    self.server.try_clear_pending_connect(*id, *counter);
                    (*id, report)
                }
            )
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Server reader for client messages.
#[derive(SystemParam)]
pub struct ServerMessageReader<'w, E: EventPack, T: SimplenetEvent>
{
    events: Res<'w, ServerMessageQueue<E, T>>,
}

impl<'w, E: EventPack, T: SimplenetEvent> ServerMessageReader<'w, E, T>
{
    /// Iterates the available client messages.
    pub fn iter(&self) -> impl Iterator<Item = (SessionId, &T)> + '_
    {
        self.events
            .iter()
            .map(|(id, report)| (*id, report))
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Server source for client requests.
///
/// Requests can only be drained here, since we expect the user to do something with request tokens.
#[derive(SystemParam)]
pub struct ServerRequestSource<'w, E: EventPack, Req: SimplenetEvent, Resp: SimplenetEvent>
{
    events: ResMut<'w, ServerRequestQueue<E, Req, Resp>>,
}

impl<'w, E: EventPack, Req: SimplenetEvent, Resp: SimplenetEvent> ServerRequestSource<'w, E, Req, Resp>
{
    /// Drains all available client requests.
    pub fn drain(&mut self) -> impl Iterator<Item = (SessionId, Req, RequestToken)> + '_
    {
        self.events.drain()
    }
}

//-------------------------------------------------------------------------------------------------------------------
