//local shortcuts
use crate::*;

//third-party shortcuts
use bevy_ecs::prelude::*;
use bevy_ecs::system::Command;
use bevy_kot_ecs::*;
use bevy_simplenet::ClientReport;
use bincode::Options;

//standard shortcuts
use std::fmt::Debug;
use std::collections::{HashMap, VecDeque};
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

struct PendingResponse
{
    request_id: u64,
    data: PendingResponseData,
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn clear_connection_queue<E: EventPack>(mut queue: ResMut<ClientConnectionQueue<E>>)
{
    queue.clear();
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn clear_message_queue<E: EventPack, T: SimplenetEvent>(mut queue: ResMut<ClientMessageQueue<E, T>>)
{
    queue.clear();
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn clear_response_queue<E: EventPack, Req: SimplenetEvent, Resp: SimplenetEvent>(
    mut queue: ResMut<ClientResponseQueue<E, Req, Resp>>
){
    queue.clear();
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn reset_response_queue<E: EventPack, Req: SimplenetEvent, Resp: SimplenetEvent>(
    mut queue: ResMut<ClientResponseQueue<E, Req, Resp>>
){
    queue.reset();
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn send_connection<E: EventPack>(In((counter, report)): In<(u32, ClientReport)>, mut queue: ResMut<ClientConnectionQueue<E>>)
{
    queue.send(counter, report);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn send_message<E: EventPack, T: SimplenetEvent>(In(data): In<Vec<u8>>, mut queue: ResMut<ClientMessageQueue<E, T>>)
{
    let Ok(message) = bincode::DefaultOptions::new().deserialize(&data[..])
    else
    {
        tracing::warn!("received server message that failed to deserialize");
        return;
    };

    queue.send(message);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn send_response<E: EventPack, Req: SimplenetEvent, Resp: SimplenetEvent>(
    In(response): In<PendingResponse>, mut queue: ResMut<ClientResponseQueue<E, Req, Resp>>
){
    let response = match response.data
    {
        PendingResponseData::Response(data) =>
        {
            let Ok(resp_ser) = bincode::DefaultOptions::new().deserialize(&data[..])
            else
            {
                tracing::warn!("received server response that failed to deserialize");
                return;
            };
            ServerResponse::<Resp>::Response(resp_ser, response.request_id)
        }
        PendingResponseData::Ack          => ServerResponse::Ack(response.request_id),
        PendingResponseData::Reject       => ServerResponse::Reject(response.request_id),
        PendingResponseData::SendFailed   => ServerResponse::SendFailed(response.request_id),
        PendingResponseData::ResponseLost => ServerResponse::ResponseLost(response.request_id),
    };

    queue.send(response);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Provides access to registered event queues.
#[derive(Resource)]
pub(crate) struct EventQueueConnectorClient<E: EventPack>
{
    /// Cached systems for clearing event queues.
    clear_message_queues: Vec<Callback<()>>,
    clear_response_queues: Vec<Callback<()>>,

    /// Cached systems for resetting stale responses.
    reset_response_queues: Vec<Callback<()>>,

    /// Cached systems for sending message events.
    /// [ message event id : callback ]
    send_messages: HashMap<u16, CallbackWith<(), Vec<u8>>>,
    /// Cached systems for sending response events.
    /// [ response event id : [ request event id : callback ] ]
    send_responses: HashMap<u16, HashMap<u16, CallbackWith<(), PendingResponse>>>,

    phantom: PhantomData<E>
}

impl<E: EventPack> EventQueueConnectorClient<E>
{
    pub(crate) fn register_message<T: SimplenetEvent>(&mut self, message_event_id: u16)
    {
        // add clear-message
        self.clear_message_queues.push(Callback::new(
            |world: &mut World| { syscall(world, (), clear_message_queue::<E, T>); }
        ));

        // add send-message
        self.send_messages.insert(
            message_event_id,
            CallbackWith::new(
                |world: &mut World, data: Vec<u8>|
                { syscall(world, data, send_message::<E, T>); }
            )
        ).unwrap();
    }

    pub(crate) fn register_response<Req: SimplenetEvent, Resp: SimplenetEvent>(
        &mut self,
        request_event_id: u16,
        response_event_id: u16
    ){
        // add clear-request
        self.clear_response_queues.push(Callback::new(
            |world: &mut World| { syscall(world, (), clear_response_queue::<E, Req, Resp>); }
        ));

        // add reset-response
        self.reset_response_queues.push(Callback::new(
            |world: &mut World| { syscall(world, (), reset_response_queue::<E, Req, Resp>); }
        ));

        // add send-request
        self.send_responses
            .entry(response_event_id)
            .or_default()
            .insert(
                request_event_id,
                CallbackWith::new(
                    |world: &mut World, response: PendingResponse|
                    { syscall(world, response, send_response::<E, Req, Resp>); }
                )
            ).unwrap();
    }

    pub(crate) fn clear_all(&self, world: &mut World)
    {
        // clear connection events
        syscall(world, (), clear_connection_queue::<E>);

        // clear messages
        for cb in self.clear_message_queues.iter()
        {
            cb.clone().apply(world);
        }

        // clear responses
        for cb in self.clear_response_queues.iter()
        {
            cb.clone().apply(world);
        }
    }

    pub(crate) fn handle_disconnect(&self, world: &mut World)
    {
        // clear messages
        for cb in self.clear_message_queues.iter()
        {
            cb.clone().apply(world);
        }

        // replace Response/Ack with ResponseLost
        for cb in self.reset_response_queues.iter()
        {
            cb.clone().apply(world);
        }
    }

    pub(crate) fn send_connection(&self, world: &mut World, counter: u32, report: ClientReport)
    {
        syscall(world, (counter, report), send_connection::<E>);
    }

    pub(crate) fn send_message(&self, world: &mut World, message_event_id: u16, data: Vec<u8>)
    {
        let Some(cb) = self.send_messages.get(&message_event_id)
        else { tracing::error!("tried to send message of unregistered message type"); return; };

        cb.call_with(data).apply(world);
    }

    pub(crate) fn send_response(&self,
        world             : &mut World,
        request_event_id  : u16,
        response_event_id : u16,
        request_id        : u64,
        data              : PendingResponseData,
    ){
        let Some(request_map) = self.send_responses.get(&response_event_id)
        else { tracing::error!("tried to send response of unregistered response type"); return; };

        let Some(cb) = request_map.get(&request_event_id)
        else { tracing::error!("tried to send response for unregistered request type"); return; };

        cb.call_with(PendingResponse{ request_id, data }).apply(world);
    }
}

impl<E: EventPack> Default for EventQueueConnectorClient<E>
{
    fn default() -> Self
    {
        Self{
            clear_message_queues: Vec::default(),
            clear_response_queues: Vec::default(),
            reset_response_queues: Vec::default(),
            send_messages: HashMap::default(),
            send_responses: HashMap::default(),
            phantom: PhantomData::default(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
