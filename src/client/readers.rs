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

/// Client reader for client connection events.
#[derive(SystemParam)]
pub struct ClientConnectionReader<'w, 's, E: EventPack>
{
    client : Res<'w, EventClientCore<E>>,
    reader : Local<'s, ManualEventReader<InnerBevyClientConnection<E>>>,
    events : ResMut<'w, Events<InnerBevyMessageClient<E, T>>>,
}

impl<'w, 's, E: EventPack, T: SimplenetEvent> ClientMessageReader<'w, 's, E, T>
{
    /// Gets the next available message event.
    ///
    /// Each call to this method synchronizes with calls to [`ClientConnectionReader::next`].
    pub fn next(&mut self) -> Option<&T>
    {
        // insert an event if we have no events
        if self.reader.len(&self.events) == 0
        {
            if let Some(message) = self.client.next_connection::<T>()
            {
                self.events.send(InnerBevyMessageClient{ message, phantom: PhantomData::default() });
            }
        }

        // get the next available message
        self.reader.read(&self.events).next()
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Client reader for server-sent messages.
#[derive(SystemParam)]
pub struct ClientMessageReader<'w, 's, E: EventPack, T: SimplenetEvent>
{
    client   : Res<'w, EventClientCore<E>>,
    registry : Res<'w, EventRegistry>,
    reader   : Local<'s, ManualEventReader<InnerBevyMessageClient<E, T>>>,
    events   : ResMut<'w, Events<InnerBevyMessageClient<E, T>>>,
}

impl<'w, 's, E: EventPack, T: SimplenetEvent> ClientMessageReader<'w, 's, E, T>
{
    /// Gets the next available message event.
    ///
    /// Each call to this method synchronizes with calls to [`ClientConnectionReader::next`].
    pub fn next(&mut self) -> Option<&T>
    {
        // insert an event if we have no events
        if self.reader.len(&self.events) == 0
        {
            if let Some(message) = self.client.next_message::<T>(&self.registry)
            {
                self.events.send(InnerBevyMessageClient{ message, phantom: PhantomData::default() });
            }
        }

        // get the next available message
        self.reader.read(&self.events).next()
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Client reader for server-sent responses to client requests.
#[derive(SystemParam)]
pub struct ClientResponseReader<'w, 's, E: EventPack, Req: SimplenetEvent, Resp: SimplenetEvent>
{
    client   : Res<'w, EventClientCore<E>>,
    registry : Res<'w, EventRegistry>,
    reader   : Local<'s, ManualEventReader<InnerBevyResponse<E, Req, Resp>>>,
    events   : ResMut<'w, Events<InnerBevyResponse<E, Req, Resp>>>,
}

impl<'w, 's, E: EventPack, Req: SimplenetEvent, Resp: SimplenetEvent> ClientResponseReader<'w, 's, E, Req, Resp>
{
    /// Gets the next available response event.
    ///
    /// Each call to this method synchronizes with calls to [`ClientConnectionReader::next`].
    pub fn next(&mut self) -> Option<&Resp>
    {
        // insert an event if we have no events
        if self.reader.len(&self.events) == 0
        {
            if let Some(response) = self.client.next_response::<Req, Resp>(&self.registry)
            {
                self.events.send(InnerBevyResponse{ response, phantom: PhantomData::default() });
            }
        }

        // get the next available message
        self.reader.read(&self.events).next()
    }
}

//-------------------------------------------------------------------------------------------------------------------
