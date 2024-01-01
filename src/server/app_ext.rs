//local shortcuts
use crate::*;

//third-party shortcuts
use bevy_app::{App, First};
use bevy_ecs::prelude::*;
use bevy_simplenet::{Server, ServerEvent, ServerReport};
use bincode::Options;
use serde::{Serialize, Deserialize};
use serde_with::{Bytes, serde_as};

//standard shortcuts
use std::fmt::Debug;
use std::marker::PhantomData;

//-------------------------------------------------------------------------------------------------------------------

//note: need to validate event ids against event registry to avoid memory use attack

//-------------------------------------------------------------------------------------------------------------------

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

type InnerServerEvent<C> = ServerEvent<C, InternalEvent, InternalEvent>;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn drain_server<E: EventPack>(world: &mut World)
{
    let mut server = world.remove_resource::<EventServerCore<E>>().unwrap();
    let queues = world.remove_resource::<EventQueueConnectorServer<E>>().unwrap();
    let registry = world.remove_resource::<EventRegistry<E>>().unwrap();

    // clear existing events
    queues.clear_all(world);

    // drain events
    while let Some((counter, session_id, event)) = server.next()
    {
        match event
        {
            InnerServerEvent::Report(report) =>
            {
                match &report
                {
                    ServerReport::<E::ConnectMsg>::Disconnected => queues.handle_disconnect(world, session_id),
                    _                                           => (),
                }

                queues.send_connection(world, counter, session_id, report);
            }
            InnerServerEvent::Msg(message) =>
            {
                if !registry.has_message_id(message.id)
                { tracing::trace!(request.id, "ignoring message with unknown event id"); continue; }

                queues.send_message(world, session_id, message.id, message.data);
            }
            InnerServerEvent::Request(request, request_id) =>
            {
                let Some(response_event_id) = registry.get_response_id_from_request_id(request.id)
                else { tracing::trace!(request.id, "ignoring request with unknown event id"); continue; };

                queues.send_request(world, session_id, request.id, response_event_id, request_id, request.data);
            }
        }
    }

    world.insert_resource(server);
    world.insert_resource(queues);
    world.insert_resource(registry);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

pub trait SimplenetServerEventAppExt
{
    /// Inserts a `bevy_simplenet` server for use in the events API.
    fn insert_simplenet_server<E: EventPack>(&mut self, server: Server<EventWrapper<E>>) -> &mut Self;
}

impl SimplenetServerEventAppExt for App
{
    fn insert_simplenet_server<E: EventPack>(&mut self, server: Server<EventWrapper<E>>) -> &mut Self
    {
        if self.world.contains_resource::<EventServerCore<E>>()
        {
            panic!("event server was already inserted");
        }

        self.insert_resource(EventServerCore::new(server));

        self.add_systems(First, drain_server::<E>.in_set(RefreshSet));

        self
    }
}

//-------------------------------------------------------------------------------------------------------------------
