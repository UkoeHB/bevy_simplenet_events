//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::app::App;
use bincode::Options;
use serde::{Serialize, Deserialize};
use serde_with::{Bytes, serde_as};

//standard shortcuts
use core::fmt::Debug;
use std::marker::PhantomData;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn setup_simplenet_event_framwork<E: EventPack>(app: &mut App)
{
    // only set up once
    if app.contains_resource::<EventRegistry<E>>() { return; }

    // add event registry
    // - this can only be done from within this crate
    app.init_resource::<EventRegistry<E>>();

    // prepare connection events
    #[cfg(feature = "server")]
    { self.add_event::<InnerBevyServerConnection<E>>(); }

    #[cfg(feature = "client")]
    { self.add_event::<InnerBevyClientConnection<E>>(); }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Internal event used to shuttle type-erased event data over the network.
#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InternalEvent
{
    pub(crate) id: u16,
    #[serde_as(as = "Bytes")]
    pub(crate) data: Vec<u8>
}

//-------------------------------------------------------------------------------------------------------------------

/// Wrapper trait that carries channel type information.
pub trait EventPack: Clone + Debug + 'static
{
    type ConnectMsg: Clone + Debug + Send + Sync + Serialize + for<'de> Deserialize<'de> + 'static;
}

impl<C, E: EventPack<ConnectMsg = C>> ChannelPack for E
{
    type ConnectMsg = C;
    type ClientMsg = InternalEvent;
    type ClientRequest = InternalEvent;
    type ServerMsg = InternalEvent;
    type ServerResponse = InternalEvent;
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

pub trait SimplenetEventAppExt
{
    /// Registers a message event.
    ///
    /// Server and client binaries must register events in the same order.
    ///
    /// If both the client and server can send messages of type `T`, then you only need to register the message type once
    /// even if the client and server are in the same app.
    fn register_message<E: EventPack, T: SimplenetEvent>(&mut self) -> &mut Self;

    /// Registers a request-response event.
    ///
    /// Server and client binaries must register events in the same order.
    ///
    /// If you only want to send acks for this request, then you may use `()` for the response type.
    fn register_request_response<E: EventPack, Req: SimplenetEvent, Resp: SimplenetEvent>(&mut self) -> &mut Self;
}

impl SimplenetEventAppExt for App
{
    fn register_simplenet_message<E: EventPack, T: SimplenetEvent>(&mut self) -> &mut Self
    {
        // setup
        setup_simplenet_event_framwork::<E>(self);

        // register type
        self.world
            .resource_mut::<EventRegistry<E>>()
            .register_message::<T>();

        // add event
        #[cfg(feature = "server")]
        { self.add_event::<InnerBevyMessageServer<E, T>>(); }

        #[cfg(feature = "client")]
        { self.add_event::<InnerBevyMessageClient<E, T>>(); }

        self
    }

    fn register_simplenet_request<E: EventPack, Req: SimplenetEvent, Resp: SimplenetEvent>(&mut self) -> &mut Self
    {
        // setup
        setup_simplenet_event_framwork::<E>(self);

        // register type
        self.world
            .resource_mut::<EventRegistry<E>>()
            .register_request_response::<Req, Resp>();

        // register event
        // - requests are read on the server
        // - responses are read on the client
        #[cfg(feature = "server")]
        { self.add_event::<InnerBevyRequest<E, Req, Resp>>(); }

        #[cfg(feature = "client")]
        { self.add_event::<InnerBevyResponse<E, Req, Resp>>(); }

        self
    }
}

//-------------------------------------------------------------------------------------------------------------------
