use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use bevy_ecs::prelude::*;
use bevy_simplenet::*;
use bincode::Options;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Event server resource that owns the internal `bevy_simplenet` server.
#[derive(Resource)]
pub(crate) struct EventServerCore<E: EventPack>
{
    /// Internal server.
    inner: Server<EventWrapper<E>>,

    /// Event counter.
    counter: u32,

    /// Tracks the most recent un-consumed connection messages for each client.
    /// A value > u32::MAX is equivalent to None.
    pending_connect: HashMap<ClientId, Arc<AtomicU64>>,
}

impl<E: EventPack> EventServerCore<E>
{
    /// Makes a new event server core.
    pub(crate) fn new(server: Server<EventWrapper<E>>) -> Self
    {
        Self {
            inner: server,
            counter: 0u32,
            pending_connect: HashMap::default(),
        }
    }

    /// Accesses the pending connect counter for a client.
    ///
    /// Returns `None` if the client is not connected.
    pub fn pending_connect(&self, client_id: ClientId) -> Option<u32>
    {
        let Some(entry) = self.pending_connect.get(&client_id) else {
            return None;
        };
        let counter = entry.load(Ordering::Relaxed);
        if counter > u32::MAX as u64 {
            return None;
        }
        Some(counter as u32)
    }

    /// Sets a new pending connect counter for a client.
    ///
    /// Does nothing if the client is not connected.
    pub fn set_pending_connect(&self, client_id: ClientId, new: Option<u32>)
    {
        let new_val = match new {
            Some(counter) => counter as u64,
            None => u64::MAX,
        };

        self.pending_connect
            .get(&client_id)
            .map(|c| c.store(new_val, Ordering::Relaxed));
    }

    /// Clears the pending connect counter for a client if the input counter equals it.
    ///
    /// Does nothing if the client is not connected.
    pub(crate) fn try_clear_pending_connect(&self, client_id: ClientId, counter: u32)
    {
        // RACE CONDITION SAFETY: This conditional races with setting the new value. We expect that new counters
        // will only be set once per tick in a system with full access to this struct, so this method can
        // only race with itself which is harmless.
        if Some(counter) == self.pending_connect(client_id) {
            self.set_pending_connect(client_id, None);
        }
    }

    /// Sends a message to a client.
    pub(crate) fn send<T: SimplenetEvent>(&self, registry: &EventRegistry<E>, client_id: ClientId, message: T)
    {
        if self.pending_connect(client_id).is_some() {
            tracing::warn!(client_id, "dropping message because there is a pending connect event");
            return;
        };

        let Some(message_event_id) = registry.get_message_id::<T>() else {
            tracing::error!("server message type is not registered");
            return;
        };

        let Ok(data) = bincode::DefaultOptions::new().serialize(&message) else {
            tracing::error!("failed serializing server message");
            return;
        };

        self.inner
            .send(client_id, InternalEvent { id: message_event_id, data })
    }

    /// Sends a response to a client.
    pub(crate) fn respond<Resp: SimplenetEvent>(
        &self,
        registry: &EventRegistry<E>,
        token: RequestToken,
        response: Resp,
    )
    {
        let client_id = token.client_id();
        if self.pending_connect(client_id).is_some() {
            tracing::warn!(client_id, "dropping response because there is a pending connect event");
            return;
        };

        let Some(response_event_id) = registry.get_response_id::<Resp>() else {
            tracing::error!("server response type is not registered");
            return;
        };

        let Ok(data) = bincode::DefaultOptions::new().serialize(&response) else {
            tracing::error!("failed serializing server response");
            return;
        };

        self.inner
            .respond(token, InternalEvent { id: response_event_id, data })
    }

    /// Sends an ack to a client.
    pub(crate) fn ack(&self, token: RequestToken)
    {
        let client_id = token.client_id();
        if self.pending_connect(client_id).is_some() {
            tracing::warn!(client_id, "dropping response because there is a pending connect event");
            return;
        };

        self.inner.ack(token)
    }

    /// Sends a request rejection to a client.
    pub(crate) fn reject(&self, token: RequestToken)
    {
        self.inner.reject(token);
    }

    /// Closes a client's connection.
    pub(crate) fn disconnect_client(&self, client_id: ClientId, close_frame: Option<CloseFrame>)
    {
        self.inner.disconnect_client(client_id, close_frame)
    }

    /// Extracts the next server event.
    pub(crate) fn next(&mut self) -> Option<(u32, ClientId, ServerEventFrom<EventWrapper<E>>)>
    {
        let Some((client_id, next)) = self.inner.next() else {
            return None;
        };
        self.counter += 1;

        match &next {
            ServerEventFrom::<EventWrapper<E>>::Report(ServerReport::<E::ConnectMsg>::Connected(..)) => {
                let _ = self
                    .pending_connect
                    .entry(client_id)
                    .or_insert_with(|| Arc::new(AtomicU64::new(u64::MAX)));
                self.set_pending_connect(client_id, Some(self.counter));
            }
            ServerEventFrom::<EventWrapper<E>>::Report(ServerReport::<E::ConnectMsg>::Disconnected) => {
                // cleanup
                // - we expect that readers **cannot** re-add this entry by accident, which would be a potential
                //   memory attack vector
                let _ = self.pending_connect.remove(&client_id);
            }
            _ => (),
        }

        Some((self.counter, client_id, next))
    }
}

//-------------------------------------------------------------------------------------------------------------------
