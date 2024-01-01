//local shortcuts
use crate::*;

//third-party shortcuts
use bevy_ecs::prelude::*;
use bevy_simplenet::ClientReport;
use bincode::Options;

//standard shortcuts
use std::fmt::Debug;
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource)]
pub(crate) struct ClientConnectionQueue<E: EventPack>
{
    /// Includes event counter for use in synchronizing with [`EventClient::send`] the first time a
    /// `ClientReport::Connected` is iterated over.
    queue: Vec<(u32, ClientReport)>,
    phantom: PhantomData<E>,
}

impl<E: EventPack> ClientConnectionQueue<E>
{
    pub(crate) fn clear(&mut self)
    {
        self.queue.clear();
    }

    pub(crate) fn send(&mut self, counter: u32, report: ClientReport)
    {
        self.queue.push((counter, report));
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &(u32, ClientReport)> + '_
    {
        self.queue.iter()
    }
}

impl<E: EventPack> Default for ClientConnectionQueue<E>
{
    fn default() -> Self { Self{ queue: Vec::default(), phantom: PhantomData::default() } }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource)]
pub(crate) struct ClientMessageQueue<E: EventPack, T: SimplenetEvent>
{
    queue: Vec<T>,
    phantom: PhantomData<E>,
}

impl<E: EventPack, T: SimplenetEvent> ClientMessageQueue<E, T>
{
    pub(crate) fn clear(&mut self)
    {
        self.queue.clear();
    }

    pub(crate) fn send(&mut self, message: T)
    {
        self.queue.push(message);
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &T> + '_
    {
        self.queue.iter()
    }
}

impl<E: EventPack, T: SimplenetEvent> Default for ClientMessageQueue<E, T>
{
    fn default() -> Self { Self{ queue: Vec::default(), phantom: PhantomData::default() } }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource)]
pub(crate) struct ClientResponseQueue<E: EventPack, Req: SimplenetEvent, Resp: SimplenetEvent>
{
    queue: Vec<ServerResponse<Resp>>,
    phantom: PhantomData<(E, Req)>,
}

impl<E: EventPack, Req: SimplenetEvent, Resp: SimplenetEvent> ClientResponseQueue<E, Req, Resp>
{
    pub(crate) fn clear(&mut self)
    {
        self.queue.clear();
    }

    pub(crate) fn reset(&mut self)
    {
        for response in self.queue.iter_mut()
        {
            let need_reset = match response
            {
                ServerResponse::Response(_, id) |
                ServerResponse::Ack(id)         => Some(*id),
                _                                => None,
            };
            tracing::warn!("'losing' server response older than a recent disconnect");
            if let Some(id) = need_reset { *response = ServerResponse::<Resp>::ResponseLost(id); }
        }
    }

    pub(crate) fn send(&mut self, response: ServerResponse<Resp>)
    {
        self.queue.push(response);
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &ServerResponse<Resp>> + '_
    {
        self.queue.iter()
    }
}

impl<E: EventPack, Req: SimplenetEvent, Resp: SimplenetEvent> Default for ClientResponseQueue<E, Req, Resp>
{
    fn default() -> Self { Self{ queue: Vec::default(), phantom: PhantomData::default() } }
}

//-------------------------------------------------------------------------------------------------------------------
