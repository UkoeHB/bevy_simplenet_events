//local shortcuts
use crate::*;

//third-party shortcuts
use bevy_app::{App, First};
use bevy_ecs::prelude::*;
use bevy_simplenet::{Client, ClientEvent, ClientReport};
use bincode::Options;
use serde::{Serialize, Deserialize};
use serde_with::{Bytes, serde_as};

//standard shortcuts
use std::fmt::Debug;
use std::marker::PhantomData;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

type InnerClientEvent = ClientEvent<InternalEvent, InternalEvent>;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn drain_client<E: EventPack>(world: &mut World)
{
    let mut client = world.remove_resource::<EventClientCore<E>>().unwrap();
    let queues = world.remove_resource::<EventQueueConnectorClient<E>>().unwrap();

    // clear existing events
    queues.clear_all(world);

    // drain events
    let mut pending_connect: Option<u32> = None;
    while let Some((counter, event)) = client.next()
    {
        match event
        {
            InnerClientEvent::Report(report) =>
            {
                match &report
                {
                    ClientReport::Disconnected => queues.handle_disconnect(world),
                    _                          => ()
                }

                queues.send_connection(world, counter, report);
            }
            InnerClientEvent::Msg(message) =>
            {
                queues.send_message(world, message.id, message.data);
            }
            InnerClientEvent::Response(response, request_id) =>
            {
                let (request_event_id, response_event_id) = client.remove_request(request_id).expect("request id missing");
                if response.id != response_event_id { panic!("received invalid request id"); }

                queues.send_response(world,
                    request_event_id,
                    response_event_id,
                    request_id,
                    PendingResponseData::Response(response.data)
                );
            }
            InnerClientEvent::Ack(request_id) =>
            {
                let (request_event_id, response_event_id) = client.remove_request(request_id).expect("request id missing");
                queues.send_response(world, request_event_id, response_event_id, request_id, PendingResponseData::Ack);
            }
            InnerClientEvent::Reject(request_id) =>
            {
                let (request_event_id, response_event_id) = client.remove_request(request_id).expect("request id missing");
                queues.send_response(world, request_event_id, response_event_id, request_id, PendingResponseData::Reject);
            }
            InnerClientEvent::SendFailed(request_id) =>
            {
                let (request_event_id, response_event_id) = client.remove_request(request_id).expect("request id missing");
                queues.send_response(world, request_event_id, response_event_id, request_id, PendingResponseData::SendFailed);
            }
            InnerClientEvent::ResponseLost(request_id)  =>
            {
                let (request_event_id, response_event_id) = client.remove_request(request_id).expect("request id missing");
                queues.send_response(world, request_event_id, response_event_id, request_id, PendingResponseData::ResponseLost);
            }
        }
    }

    world.insert_resource(client);
    world.insert_resource(queues);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

pub enum PendingResponseData
{
    Response(Vec<u8>),
    Ack,
    Reject,
    SendFailed,
    ResponseLost,
}

//-------------------------------------------------------------------------------------------------------------------

pub trait SimplenetClientEventAppExt
{
    /// Inserts a `bevy_simplenet` client for use in the events API.
    fn insert_simplenet_client<E: EventPack>(&mut self, client: Client<EventWrapper<E>>) -> &mut Self;
}

impl SimplenetClientEventAppExt for App
{
    fn insert_simplenet_client<E: EventPack>(&mut self, client: Client<EventWrapper<E>>) -> &mut Self
    {
        if self.world.contains_resource::<EventClientCore<E>>()
        {
            panic!("event client was already inserted");
        }

        self.insert_resource(EventClientCore::new(client));

        self.add_systems(First, drain_client::<E>.in_set(RefreshSet));

        self
    }
}

//-------------------------------------------------------------------------------------------------------------------
