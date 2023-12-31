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

/// Client reader for client connection events.
#[derive(SystemParam)]
pub struct ClientConnectionReader<'w, E: EventPack>
{
    client : Res<'w, EventClientCore<E>>,
    events : Res<'w, ClientConnectionQueue<E>>,
}

impl<'w, E: EventPack> ClientConnectionReader<'w, E>
{
    /// Iterates the available connection reports.
    pub fn iter(&self) -> impl Iterator<Item = &ClientReport> + '_
    {
        self.events
            .iter()
            .map(|(counter, report)|
                {
                    self.client.try_clear_pending_connect(*counter);
                    report
                }
            )
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Client reader for server messages.
#[derive(SystemParam)]
pub struct ClientMessageReader<'w, E: EventPack, T: SimplenetEvent>
{
    events: Res<'w, ClientMessageQueue<E, T>>,
}

impl<'w, E: EventPack, T: SimplenetEvent> ClientMessageReader<'w, E, T>
{
    /// Iterates the available server messages.
    pub fn iter(&self) -> impl Iterator<Item = &T> + '_
    {
        self.events.iter()
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Client reader for server responses to client requests.
#[derive(SystemParam)]
pub struct ClientResponseReader<'w, E: EventPack, Req: SimplenetEvent, Resp: SimplenetEvent>
{
    events: Res<'w, ClientResponseQueue<E, Req, Resp>>,
}

impl<'w, E: EventPack, Req: SimplenetEvent, Resp: SimplenetEvent> ClientResponseReader<'w, E, Req, Resp>
{
    /// Iterates the available server responses.
    pub fn iter(&self) -> impl Iterator<Item = &ServerResponse<Resp>> + '_
    {
        self.events.iter()
    }
}

//-------------------------------------------------------------------------------------------------------------------
