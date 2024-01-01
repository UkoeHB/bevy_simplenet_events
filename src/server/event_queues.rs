//local shortcuts
use crate::*;

//third-party shortcuts
use bevy_ecs::prelude::*;
use bevy_simplenet::{RequestToken, ServerReport};
use bincode::Options;

//standard shortcuts
use std::fmt::Debug;
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource)]
pub(crate) struct ServerConnectionQueue<E: EventPack>
{
    /// Includes event counter for use in synchronizing with [`EventServer::send`] the first time a
    /// `ServerReport::Connected` is iterated over.
    queue: Vec<(u32, SessionId, ServerReport<E::ConnectMsg>)>,
}

impl<E: EventPack> ServerConnectionQueue<E>
{
    pub(crate) fn clear(&mut self)
    {
        self.queue.clear();
    }

    pub(crate) fn send(&mut self, counter: u32, session_id: SessionId, report: ServerReport<E::ConnectMsg>)
    {
        self.queue.push((counter, session_id, report));
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &(u32, SessionId, ServerReport<E::ConnectMsg>)> + '_
    {
        self.queue.iter()
    }
}

impl<E: EventPack> Default for ServerConnectionQueue<E>
{
    fn default() -> Self { Self{ queue: Vec::default() } }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource)]
pub(crate) struct ServerMessageQueue<E: EventPack, T: SimplenetEvent>
{
    queue: Vec<(SessionId, T)>,
    phantom: PhantomData<E>,
}

impl<E: EventPack, T: SimplenetEvent> ServerMessageQueue<E, T>
{
    pub(crate) fn clear(&mut self)
    {
        self.queue.clear();
    }

    pub(crate) fn send(&mut self, session_id: SessionId, message: T)
    {
        self.queue.push((session_id, message));
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &(SessionId, T)> + '_
    {
        self.queue.iter()
    }
}

impl<E: EventPack, T: SimplenetEvent> Default for ServerMessageQueue<E, T>
{
    fn default() -> Self { Self{ queue: Vec::default(), phantom: PhantomData::default() } }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource)]
pub(crate) struct ServerRequestQueue<E: EventPack, Req: SimplenetEvent, Resp: SimplenetEvent>
{
    queue: Vec<(SessionId, Req, RequestToken)>,
    phantom: PhantomData<(E, Resp)>,
}

impl<E: EventPack, Req: SimplenetEvent, Resp: SimplenetEvent> ServerRequestQueue<E, Req, Resp>
{
    pub(crate) fn clear(&mut self)
    {
        self.queue.clear();
    }

    pub(crate) fn send(&mut self, session_id: SessionId, request: Req, request_token: RequestToken)
    {
        self.queue.push((session_id, request, request_token));
    }

    pub(crate) fn drain(&mut self) -> impl Iterator<Item = (SessionId, Req, RequestToken)> + '_
    {
        self.queue.drain(..)
    }
}

impl<E: EventPack, Req: SimplenetEvent, Resp: SimplenetEvent> Default for ServerRequestQueue<E, Req, Resp>
{
    fn default() -> Self { Self{ queue: Vec::default(), phantom: PhantomData::default() } }
}

//-------------------------------------------------------------------------------------------------------------------
