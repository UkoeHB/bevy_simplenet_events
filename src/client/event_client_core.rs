//local shortcuts
use crate::*;

//third-party shortcuts
use bevy_ecs::prelude::*;
use bevy_kot_utils::*;
use bevy_simplenet::*;
use bincode::Options;

//standard shortcuts
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

//-------------------------------------------------------------------------------------------------------------------

/// Event client resource that owns the internal `bevy_simplenet` client.
#[derive(Resource)]
pub(crate) struct EventClientCore<E: EventPack>
{
    /// Internal client.
    inner: Client<EventWrapper<E>>,

    /// Event counter.
    counter: u32,

    /// Tracks the most recent un-consumed connection message.
    /// A value > u32::MAX is equivalent to None.
    pending_connect: Arc<AtomicU64>,

    /// Maps client requests to request/response event ids.
    /// [ request id : (request event id, response event id) ]
    request_sender: Sender<(u64, (u16, u16))>,
    request_receiver: Receiver<(u64, (u16, u16))>,
    request_map: HashMap<u64, (u16, u16)>,
}

impl<E: EventPack> EventClientCore<E>
{
    /// Makes a new event client core.
    pub(crate) fn new(client: Client<EventWrapper<E>>) -> Self
    {
        let (request_sender, request_receiver) = new_channel();
        Self{
            inner           : client,
            counter         : 0u32,
            pending_connect : Arc::new(AtomicU64::new(u64::MAX)),
            request_sender,
            request_receiver,
            request_map     : HashMap::default(),
        }
    }

    /// Accesses the pending connect counter.
    pub(crate) fn pending_connect(&self) -> Option<u32>
    {
        let counter = self.pending_connect.load(Ordering::Relaxed);
        if counter > u32::MAX as u64 { return None; }
        Some(counter as u32)
    }

    /// Sets a new pending connect counter.
    pub(crate) fn set_pending_connect(&self, new: Option<u32>)
    {
        match new
        {
            Some(counter) => self.pending_connect.store(counter as u64, Ordering::Relaxed),
            None          => self.pending_connect.store(u64::MAX, Ordering::Relaxed),
        }
    }

    /// Clears the pending connect counter if the input counter equals it.
    ///
    /// Does nothing if the client is not connected.
    pub(crate) fn try_clear_pending_connect(&self, counter: u32)
    {
        // RACE CONDITION SAFETY: This conditional races with setting the new value. We expect that new counters will
        // only be set once per tick in a system with full access to this struct, so this method can only race with
        // itself which is harmless.
        if Some(counter) == self.pending_connect()
        {
            self.set_pending_connect(None);
        }
    }

    /// Sends a message to the server.
    pub(crate) fn send<T: SimplenetEvent>(&self, registry: &EventRegistry<E>, message: T) -> Result<MessageSignal, ()>
    {
        if self.pending_connect().is_some()
        { tracing::warn!("dropping client message because there is a pending connect event"); return Err(()); };

        let Some(message_event_id) = registry.get_message_id::<T>()
        else { tracing::error!("client message type is not registered"); return Err(()); };

        let Ok(data) = bincode::DefaultOptions::new().serialize(&message)
        else { tracing::error!("failed serializing client message"); return Err(()); };

        self.inner.send(InternalEvent{ id: message_event_id, data })
    }

    /// Sends a request to the server.
    pub(crate) fn request<Req: SimplenetEvent>(&self, registry: &EventRegistry<E>, request: Req) -> Result<RequestSignal, ()>
    {
        if self.pending_connect().is_some()
        { tracing::warn!("dropping client request because there is a pending connect event"); return Err(()); };

        let Some(request_event_id) = registry.get_request_id::<Req>()
        else { tracing::error!("client request type is not registered"); return Err(()); };
        let Some(response_event_id) = registry.get_response_id_from_request::<Req>()
        else { tracing::error!("no response type registered for the given client request type"); return Err(()); };

        let Ok(data) = bincode::DefaultOptions::new().serialize(&request)
        else { tracing::error!("failed serializing client request"); return Err(()); };

        let result = self.inner.request(InternalEvent{ id: request_event_id, data });

        if let Ok(signal) = &result
        {
            // use channel since we are immutable
            if self.request_sender.send((signal.id(), (request_event_id, response_event_id))).is_err()
            {
                tracing::error!("request tracker channel is broken");
            }
        }

        result
    }

    /// Removes a request from the request tracker.
    pub(crate) fn remove_request(&mut self, request_id: u64) -> Option<(u16, u16)>
    {
        // drain pending request-tracker entries now that we are mutable
        while let Some((request_id, event_ids)) = self.request_receiver.try_recv()
        {
            self.request_map.insert(request_id, event_ids);
        }

        self.request_map.remove(&request_id)
    }

    /// Closes the client.
    pub(crate) fn close(&self)
    {
        self.inner.close();
    }

    /// Extracts the next client event.
    pub(crate) fn next(&mut self) -> Option<(u32, ClientEventFrom<EventWrapper<E>>)>
    {
        let Some(next) = self.inner.next() else { return None; };
        self.counter += 1;

        match &next
        {
            ClientEventFrom::<EventWrapper<E>>::Report(ClientReport::Connected) =>
            {
                self.set_pending_connect(Some(self.counter));
            }
            _ => ()
        }

        Some((self.counter, next))
    }
}

//-------------------------------------------------------------------------------------------------------------------
