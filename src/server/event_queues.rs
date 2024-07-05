use std::marker::PhantomData;

use bevy_ecs::prelude::*;
use bevy_simplenet::{ClientId, RequestToken, ServerReport};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource)]
pub(crate) struct ServerConnectionQueue<E: EventPack>
{
    /// Includes event counter for use in synchronizing with [`EventServer::send`] the first time a
    /// `ServerReport::Connected` is iterated over.
    queue: Vec<(u32, ClientId, ServerReport<E::ConnectMsg>)>,
}

impl<E: EventPack> ServerConnectionQueue<E>
{
    pub(crate) fn clear(&mut self)
    {
        self.queue.clear();
    }

    pub(crate) fn send(&mut self, counter: u32, client_id: ClientId, report: ServerReport<E::ConnectMsg>)
    {
        self.queue.push((counter, client_id, report));
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &(u32, ClientId, ServerReport<E::ConnectMsg>)> + '_
    {
        self.queue.iter()
    }
}

impl<E: EventPack> Default for ServerConnectionQueue<E>
{
    fn default() -> Self
    {
        Self { queue: Vec::default() }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource)]
pub(crate) struct ServerMessageQueue<E: EventPack, T: SimplenetEvent>
{
    queue: Vec<Option<(ClientId, T)>>,
    phantom: PhantomData<E>,
}

impl<E: EventPack, T: SimplenetEvent> ServerMessageQueue<E, T>
{
    pub(crate) fn clear(&mut self)
    {
        self.queue.clear();
    }

    pub(crate) fn clear_session(&mut self, client_id: ClientId)
    {
        self.queue
            .iter_mut()
            .filter(|i| match i {
                Some(i) => i.0 == client_id,
                None => false,
            })
            .for_each(|i| {
                *i = None;
            });
    }

    pub(crate) fn send(&mut self, client_id: ClientId, message: T)
    {
        self.queue.push(Some((client_id, message)));
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &(ClientId, T)> + '_
    {
        self.queue.iter().filter_map(|i| i.as_ref())
    }
}

impl<E: EventPack, T: SimplenetEvent> Default for ServerMessageQueue<E, T>
{
    fn default() -> Self
    {
        Self { queue: Vec::default(), phantom: PhantomData::default() }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource)]
pub(crate) struct ServerRequestQueue<E: EventPack, Req: SimplenetEvent, Resp: SimplenetEvent>
{
    queue: Vec<Option<(RequestToken, Req)>>,
    phantom: PhantomData<(E, Resp)>,
}

impl<E: EventPack, Req: SimplenetEvent, Resp: SimplenetEvent> ServerRequestQueue<E, Req, Resp>
{
    pub(crate) fn clear(&mut self)
    {
        self.queue.clear();
    }

    pub(crate) fn clear_session(&mut self, client_id: ClientId)
    {
        self.queue
            .iter_mut()
            .filter(|i| match i {
                Some(i) => i.0.client_id() == client_id,
                None => false,
            })
            .for_each(|i| {
                *i = None;
            });
    }

    pub(crate) fn send(&mut self, request_token: RequestToken, request: Req)
    {
        self.queue.push(Some((request_token, request)));
    }

    pub(crate) fn drain(&mut self) -> impl Iterator<Item = (RequestToken, Req)> + '_
    {
        self.queue.drain(..).filter_map(|i| i)
    }
}

impl<E: EventPack, Req: SimplenetEvent, Resp: SimplenetEvent> Default for ServerRequestQueue<E, Req, Resp>
{
    fn default() -> Self
    {
        Self { queue: Vec::default(), phantom: PhantomData::default() }
    }
}

//-------------------------------------------------------------------------------------------------------------------
