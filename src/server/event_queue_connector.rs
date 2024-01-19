//local shortcuts
use crate::*;

//third-party shortcuts
use bevy_ecs::prelude::*;
use bevy_ecs::system::Command;
use bevy_kot_ecs::*;
use bevy_simplenet::{RequestToken, ServerReport, SessionId};
use bincode::Options;

//standard shortcuts
use std::collections::HashMap;
use std::marker::PhantomData;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn clear_connection_queue<E: EventPack>(mut queue: ResMut<ServerConnectionQueue<E>>)
{
    queue.clear();
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn clear_message_queue<E: EventPack, T: SimplenetEvent>(
    In(session_id) : In<Option<SessionId>>,
    mut queue      : ResMut<ServerMessageQueue<E, T>>
){
    match session_id
    {
        Some(session_id) => queue.clear_session(session_id),
        None             => queue.clear(),
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn clear_request_queue<E: EventPack, Req: SimplenetEvent, Resp: SimplenetEvent>(
    In(session_id) : In<Option<SessionId>>,
    mut queue      : ResMut<ServerRequestQueue<E, Req, Resp>>
){
    match session_id
    {
        Some(session_id) => queue.clear_session(session_id),
        None             => queue.clear(),
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn send_connection<E: EventPack>(
    In((counter, session_id, report)) : In<(u32, SessionId, ServerReport<E::ConnectMsg>)>,
    mut queue                         : ResMut<ServerConnectionQueue<E>>
){
    queue.send(counter, session_id, report);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn send_message<E: EventPack, T: SimplenetEvent>(
    In((session_id, data)) : In<(SessionId, Vec<u8>)>,
    mut queue              : ResMut<ServerMessageQueue<E, T>>
){
    let Ok(message) = bincode::DefaultOptions::new().deserialize(&data[..])
    else
    {
        tracing::warn!("received client message that failed to deserialize");
        return;
    };

    queue.send(session_id, message);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn send_request<E: EventPack, Req: SimplenetEvent, Resp: SimplenetEvent>(
    In((request_token, data)) : In<(RequestToken, Vec<u8>)>,
    mut queue                 : ResMut<ServerRequestQueue<E, Req, Resp>>
){
    let Ok(request) = bincode::DefaultOptions::new().deserialize(&data[..])
    else
    {
        tracing::warn!("received client request that failed to deserialize");
        return;
    };

    queue.send(request_token, request);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Provides access to registered event queues.
#[derive(Resource)]
pub(crate) struct EventQueueConnectorServer<E: EventPack>
{
    /// Cached systems for clearing event queues.
    clear_message_queues: Vec<CallbackWith<(), Option<SessionId>>>,
    clear_request_queues: Vec<CallbackWith<(), Option<SessionId>>>,

    /// Cached systems for sending message events.
    /// [ message event id : callback ]
    send_messages: HashMap<u16, CallbackWith<(), (SessionId, Vec<u8>)>>,
    /// Cached systems for sending response events.
    /// [ response event id : [ request event id : callback ] ]
    send_requests: HashMap<u16, HashMap<u16, CallbackWith<(), (RequestToken, Vec<u8>)>>>,

    phantom: PhantomData<E>,
}

impl<E: EventPack> EventQueueConnectorServer<E>
{
    pub(crate) fn register_message<T: SimplenetEvent>(&mut self, message_event_id: u16)
    {
        // add clear-message
        self.clear_message_queues.push(CallbackWith::new(
            |world: &mut World, target: Option<SessionId>| { syscall(world, target, clear_message_queue::<E, T>); }
        ));

        // add send-message
        if self.send_messages.insert(
                message_event_id,
                CallbackWith::new(
                    |world: &mut World, package: (SessionId, Vec<u8>)|
                    { syscall(world, package, send_message::<E, T>); }
                )
            ).is_some()
        { panic!("message was already registered"); }
    }

    pub(crate) fn register_request<Req: SimplenetEvent, Resp: SimplenetEvent>(
        &mut self,
        request_event_id: u16,
        response_event_id: u16
    ){
        // add clear-request
        self.clear_request_queues.push(CallbackWith::new(
            |world: &mut World, target: Option<SessionId>| { syscall(world, target, clear_request_queue::<E, Req, Resp>); }
        ));

        // add send-request
        if self.send_requests
            .entry(response_event_id)
            .or_default()
            .insert(
                request_event_id,
                CallbackWith::new(
                    |world: &mut World, package: (RequestToken, Vec<u8>)|
                    { syscall(world, package, send_request::<E, Req, Resp>); }
                )
            ).is_some()
        { panic!("request/response was already registered"); }
    }

    pub(crate) fn clear_all(&self, world: &mut World)
    {
        // clear connection events
        syscall(world, (), clear_connection_queue::<E>);

        // clear messages
        for cb in self.clear_message_queues.iter()
        {
            cb.call_with(None).apply(world);
        }

        // clear requests
        for cb in self.clear_request_queues.iter()
        {
            cb.call_with(None).apply(world);
        }
    }

    pub(crate) fn handle_disconnect(&self, world: &mut World, session_id: SessionId)
    {
        tracing::trace!(session_id, "clearing server queues on disconnect");

        // clear messages for this client
        for cb in self.clear_message_queues.iter()
        {
            cb.call_with(Some(session_id)).apply(world);
        }

        // clear requests for this client
        for cb in self.clear_request_queues.iter()
        {
            cb.call_with(Some(session_id)).apply(world);
        }
    }

    pub(crate) fn send_connection(
        &self,
        world      : &mut World,
        counter    : u32,
        session_id : SessionId,
        report     : ServerReport<E::ConnectMsg>
    ){
        syscall(world, (counter, session_id, report), send_connection::<E>);
    }

    pub(crate) fn send_message(&self, world: &mut World, session_id: SessionId, message_event_id: u16, data: Vec<u8>)
    {
        let Some(cb) = self.send_messages.get(&message_event_id)
        else { tracing::error!("tried to send message of unregistered message type"); return; };

        cb.call_with((session_id, data)).apply(world);
    }

    pub(crate) fn send_request(&self,
        world             : &mut World,
        request_event_id  : u16,
        response_event_id : u16,
        request_token     : RequestToken,
        data              : Vec<u8>,
    ){
        let Some(request_map) = self.send_requests.get(&response_event_id)
        else { tracing::error!("tried to send request of unregistered response type"); return; };

        let Some(cb) = request_map.get(&request_event_id)
        else { tracing::error!("tried to send request for unregistered request type"); return; };

        cb.call_with((request_token, data)).apply(world);
    }
}

impl<E: EventPack> Default for EventQueueConnectorServer<E>
{
    fn default() -> Self
    {
        Self{
            clear_message_queues: Vec::default(),
            clear_request_queues: Vec::default(),
            send_messages: HashMap::default(),
            send_requests: HashMap::default(),
            phantom: PhantomData::default(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
