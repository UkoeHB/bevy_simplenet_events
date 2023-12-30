//local shortcuts
use crate::*;

//third-party shortcuts
use bincode::Options;

//standard shortcuts
use core::fmt::Debug;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

//-------------------------------------------------------------------------------------------------------------------

/// Event client resource that owns the internal `bevy_simplenet` client.
#[derive(Resource)]
pub(crate) struct EventClientCore<E: EventPack>
{
    /// Internal client.
    client: Client<E>,

    /// Internal state.
    /// - Protected by a mutex so event clients can be accessed by multiple bevy system parameters.
    inner: Arc<Mutex<EventClientInner>>,
}

impl<E: EventPack> EventClientCore<E>
{
    /// Makes a new event client core.
    pub(crate) fn new(client: Client<E>) -> Self
    {
        Self{
            client,
            inner: Arc::new(Mutex::new(EventClientInner::default())),
        }
    }

    /// Sends a message to the server.
    pub(crate) fn send<T: SimplenetEvent>(&self, registry: &EventRegistry<E>, message: T) -> Result<MessageSignal, ()>
    {
        let Ok(inner) = self.inner.lock()
        else { tracing::error!("event client inner mutex is broken"); return Err(()); };

        if !inner.can_send()
        else { tracing::warn!("dropping client message because there is a pending connect message"); return Err(()); };

        let Some(message_event_id) = registry.get_message_id::<T>()
        else { tracing::error!("client message type is not registered"); return Err(()); };

        let Ok(ser_msg) = bincode::DefaultOptions::new().serialize(&message)
        else { tracing::error!("failed serializing client message"); return Err(()); };

        self.client.send(InternalEvent{ id: message_event_id, data: ser_msg })
    }

    /// Sends a request to the server.
    pub(crate) fn request<Req: SimplenetEvent>(&self, registry: &EventRegistry<E>, request: Req) -> Result<RequestSignal, ()>
    {
        let Ok(inner) = self.inner.lock()
        else { tracing::error!("event client inner mutex is broken"); return Err(()); };

        if !inner.can_send()
        else { tracing::warn!("dropping client message because there is a pending connect message"); return Err(()); };

        let Some(request_event_id) = registry.get_request_id::<Req>()
        else { tracing::error!("client request type is not registered"); return Err(()); };

        let Ok(ser_msg) = bincode::DefaultOptions::new().serialize(&request)
        else { tracing::error!("failed serializing client message"); return Err(()); };

        self.client.request(InternalEvent{ id: request_event_id, data: ser_msg })
    }

    /// Closes the client.
    pub(crate) fn close(&self)
    {
        self.client.close();
    }

    /// Extracts the next connection event.
    ///
    /// If this returns a `ClientReport::Disconnected` then events blocked by it will be unblocked.
    pub(crate) fn next_connection(&self) -> Option<ClientReport>
    {
        let Ok(mut inner) = self.inner.lock()
        else { tracing::error!("event client inner mutex is broken"); return None; };

        inner.update(&self.client);
        inner.next_connection()
    }

    /// Extracts the next message of type `T`.
    pub(crate) fn next_message<T: SimplenetEvent>(&self, registry: &EventRegistry<E>) -> Option<T>
    {
        let Some(message_event_id) = registry.get_message_id::<T>()
        else { tracing::error!("requested simplenet message for unregistered type"); return None; };

        let Ok(mut inner) = self.inner.lock()
        else { tracing::error!("event client inner mutex is broken"); return None; };

        inner.update(&self.client);
        inner.next_message(message_event_id)
    }

    /// Extracts the next response of type `Resp` targeted at request of type `Req`.
    pub(crate) fn next_response<Req, Resp>(&self, registry: &EventRegistry<E>) -> Option<ServerResponse<Resp>>
    where
        Req: SimplenetEvent,
        Resp: SimplenetEvent,
    {
        let Some(request_event_id) = registry.get_request_id::<Req>()
        else { tracing::error!("requested simplenet response for unregistered request"); return None; };
        let Some(response_event_id) = registry.get_response_id::<Resp>()
        else { tracing::error!("requested simplenet response for unregistered response"); return None; };

        let Ok(mut inner) = self.inner.lock()
        else { tracing::error!("event client inner mutex is broken"); return None; };

        inner.update(&self.client);
        inner.next_response::<Resp>(request_event_id, response_event_id)
    }
}

//-------------------------------------------------------------------------------------------------------------------
