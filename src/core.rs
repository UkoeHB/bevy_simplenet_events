use std::fmt::Debug;
use std::marker::PhantomData;

use bevy_app::App;
use bevy_ecs::prelude::*;
use bevy_simplenet::ChannelPack;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, Bytes};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn setup_simplenet_event_framwork<E: EventPack>(app: &mut App)
{
    // only set up once
    if app.world().contains_resource::<EventRegistry<E>>() {
        return;
    }

    // add event registry
    // - this can only be done from within this crate
    app.init_resource::<EventRegistry<E>>();

    // prepare internals
    #[cfg(feature = "server")]
    {
        app.init_resource::<EventQueueConnectorServer<E>>();
        app.init_resource::<ServerConnectionQueue<E>>();
    }

    #[cfg(feature = "client")]
    {
        app.init_resource::<EventQueueConnectorClient<E>>();
        app.init_resource::<ClientConnectionQueue<E>>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Internal event used to shuttle type-erased event data over the network.
#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InternalEvent
{
    pub(crate) id: u16,
    #[serde_as(as = "Bytes")]
    pub(crate) data: Vec<u8>,
}

//-------------------------------------------------------------------------------------------------------------------

/// Wrapper trait that carries channel type information.
pub trait EventPack: Clone + Debug + Send + Sync + 'static
{
    type ConnectMsg: Clone + Debug + Send + Sync + Serialize + for<'de> Deserialize<'de> + 'static;
}

/// Wrapper struct for carrying type information when constructing clients and servers.
#[derive(Clone, Debug)]
pub struct EventWrapper<E: EventPack>(PhantomData<E>);

impl<C, E: EventPack<ConnectMsg = C>> ChannelPack for EventWrapper<E>
where
    C: Clone + Debug + Send + Sync + Serialize + for<'de> Deserialize<'de> + 'static,
{
    type ConnectMsg = C;
    type ClientMsg = InternalEvent;
    type ClientRequest = InternalEvent;
    type ServerMsg = InternalEvent;
    type ServerResponse = InternalEvent;
}

impl<E: EventPack> Default for EventWrapper<E>
{
    fn default() -> Self
    {
        Self(PhantomData::default())
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Tags event types that can be sent by servers and clients.
///
/// Example:
/**
#[derive(SimplenetEvent, Serialize, Deserialize)]
struct MyEvent(usize);
*/
pub trait SimplenetEvent: Send + Sync + Serialize + for<'de> Deserialize<'de> + 'static {}

impl SimplenetEvent for () {}

//-------------------------------------------------------------------------------------------------------------------

/// Contains refresh systems for the event framework backend.
#[derive(SystemSet, Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
pub struct RefreshSet;

//-------------------------------------------------------------------------------------------------------------------

pub trait SimplenetEventAppExt
{
    /// Registers a client-sent message event.
    ///
    /// Server and client binaries must register events in the same order.
    fn register_simplenet_client_message<E: EventPack, T: SimplenetEvent>(&mut self) -> &mut Self;

    /// Registers a server-sent message event.
    ///
    /// Server and client binaries must register events in the same order.
    fn register_simplenet_server_message<E: EventPack, T: SimplenetEvent>(&mut self) -> &mut Self;

    /// Registers a request-response event.
    ///
    /// Server and client binaries must register events in the same order.
    ///
    /// If you only want to send acks for this request, then you may use `()` for the response type.
    fn register_simplenet_request_response<E: EventPack, Req: SimplenetEvent, Resp: SimplenetEvent>(
        &mut self,
    ) -> &mut Self;
}

impl SimplenetEventAppExt for App
{
    fn register_simplenet_client_message<E: EventPack, T: SimplenetEvent>(&mut self) -> &mut Self
    {
        // setup
        setup_simplenet_event_framwork::<E>(self);

        #[cfg(feature = "client")]
        {
            // register type
            let message_event_id = self
                .world_mut()
                .resource_mut::<EventRegistry<E>>()
                .register_message::<T>();

            // register event
            self.world_mut()
                .resource_mut::<EventQueueConnectorClient<E>>()
                .register_message::<T>(message_event_id);
            self.init_resource::<ClientMessageQueue<E, T>>();
        }

        self
    }

    fn register_simplenet_server_message<E: EventPack, T: SimplenetEvent>(&mut self) -> &mut Self
    {
        // setup
        setup_simplenet_event_framwork::<E>(self);

        #[cfg(feature = "server")]
        {
            // register type
            let message_event_id = self
                .world_mut()
                .resource_mut::<EventRegistry<E>>()
                .register_message::<T>();

            // register event
            self.world_mut()
                .resource_mut::<EventQueueConnectorServer<E>>()
                .register_message::<T>(message_event_id);
            self.init_resource::<ServerMessageQueue<E, T>>();
        }

        self
    }

    fn register_simplenet_request_response<E: EventPack, Req: SimplenetEvent, Resp: SimplenetEvent>(
        &mut self,
    ) -> &mut Self
    {
        // setup
        setup_simplenet_event_framwork::<E>(self);

        // register type
        let (request_event_id, response_event_id) = self
            .world_mut()
            .resource_mut::<EventRegistry<E>>()
            .register_request_response::<Req, Resp>();

        // register event
        // - requests are read on the server
        // - responses are read on the client
        #[cfg(feature = "server")]
        {
            self.world_mut()
                .resource_mut::<EventQueueConnectorServer<E>>()
                .register_request::<Req, Resp>(request_event_id, response_event_id);
            self.init_resource::<ServerRequestQueue<E, Req, Resp>>();
        }

        #[cfg(feature = "client")]
        {
            self.world_mut()
                .resource_mut::<EventQueueConnectorClient<E>>()
                .register_response::<Req, Resp>(request_event_id, response_event_id);
            self.init_resource::<ClientResponseQueue<E, Req, Resp>>();
        }

        self
    }
}

//-------------------------------------------------------------------------------------------------------------------
