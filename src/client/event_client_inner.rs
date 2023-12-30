//local shortcuts
use crate::*;

//third-party shortcuts
use bincode::Options;

//standard shortcuts
use core::fmt::Debug;
use std::container::VecDeque;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn is_newer_or_eq(a: u64, b: u64) -> bool
{
    a.wrapping_sub(b) < u64::MAX/2
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

struct PendingMessage
{
    counter: u64,
    data: Vec<u8>,
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

enum PendingResponseData
{
    Response(Vec<u8>),
    Ack,
    Reject,
    SendFailed,
    ResponseLost,
}

struct PendingResponse
{
    counter: u64,
    request_id: u64,
    response: PendingResponseData,
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

type InnerClientEvent = ClientEvent<InternalEvent, InternalEvent>;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct EventClientInner
{
    /// Internal event counter, used to synchronize with disconnect events.
    counter: u64,
    /// Un-consumed event counters. Used to detect when the user is failing to consume events.
    //todo: only in debug mode?
    //pending_events: BTreeSet<u64>,
    /// Most recent un-consumed connect report.
    /// - We cache this to block sending messages when there is a pending connect report. Note that sending is
    ///   blocked automatically by the simplenet Client during a disconnect before the Client has emitted a
    //    connection report (TODO: need upstream bug fix).
    pending_connect: Option<u64>,
    /// Most recent consumed disconnection report.
    /// - We cache this to discard messages older than the disconnection.
    recent_consumed_disconnect: Option<u64>,

    /// Maps client requests to request/response event ids.
    /// [ request id : (request event id, response event id) ]
    request_map: HashMap<u64, (u16, u16)>,

    /// Tracks pending connection events.
    /// { (connection event counter, report) }
    pending_connections: VecDeque<(u64, ClientReport)>,
    /// Tracks pending server messages.
    /// [ message event id : { message } ]
    pending_messages: HashMap<u16, VecDeque<PendingMessage>>,
    /// Tracks pending server responses, mapped to the client request's event id.
    /// - Note: Response event ids are mapped first since they can have duplicates.
    /// [ response event id : [ request event id : { response } ] ]
    pending_responses: HashMap<u16, HashMap<u16, VecDeque<PendingResponse>>>,
}

impl EventClientInner
{
    fn add_request(&mut self, request_id: u64, request_event_id: u16, response_event_id: u16)
    {
        request_map.insert(request_id, (request_event_id, response_event_id)).expect("request id duplicate");
    }

    fn update<E: EventPack>(&mut self, client: &Client<E>)
    {
        while let Some(event) = client.next()
        {
            self.counter += 1;
            let counter = self.counter;

            InnerClientEvent::Report(report) =>
            {
                if report == ClientReport::Connected
                {
                    self.pending_connect = Some(counter);
                }

                self.pending_connections.push_back((counter, report));
            }
            InnerClientEvent::Msg(message) =>
            {
                self.pending_messages.insert(message.id, PendingMessage{ counter, data: message.data });
            }
            InnerClientEvent::Response(response, request_id) =>
            {
                let (request_event_id, response_event_id) = request_map.remove(request_id).expect("request id missing");
                if response.id != response_event_id { panic!("received invalid request id"); }

                self.pending_responses
                    .entry(response_event_id)
                    .or_default()
                    .entry(request_event_id)
                    .or_default()
                    .push_back(PendingResponse{ counter, request_id, response: PendingResponseData::Response(response.data) });
            }
            InnerClientEvent::Ack(request_id) =>
            {
                let (request_event_id, response_event_id) = request_map.remove(request_id).expect("request id missing");
                self.pending_responses
                    .entry(response_event_id)
                    .or_default()
                    .entry(request_event_id)
                    .or_default()
                    .push_back(PendingResponse{ counter, request_id, response: PendingResponseData::Ack });
            }
            InnerClientEvent::Reject(request_id) =>
            {
                let (request_event_id, response_event_id) = request_map.remove(request_id).expect("request id missing");
                self.pending_responses
                    .entry(response_event_id)
                    .or_default()
                    .entry(request_event_id)
                    .or_default()
                    .push_back(PendingResponse{ counter, request_id, response: PendingResponseData::Reject });
            }
            InnerClientEvent::SendFailed(request_id) =>
            {
                let (request_event_id, response_event_id) = request_map.remove(request_id).expect("request id missing");
                self.pending_responses
                    .entry(response_event_id)
                    .or_default()
                    .entry(request_event_id)
                    .or_default()
                    .push_back(PendingResponse{ counter, request_id, response: PendingResponseData::SendFailed });
            }
            InnerClientEvent::ResponseLost(request_id)  =>
            {
                let (request_event_id, response_event_id) = request_map.remove(request_id).expect("request id missing");
                self.pending_responses
                    .entry(response_event_id)
                    .or_default()
                    .entry(request_event_id)
                    .or_default()
                    .push_back(PendingResponse{ counter, request_id, response: PendingResponseData::ResponseLost });
            }
        }
    }

    fn can_send(&self) -> bool
    {
        self.pending_connect.is_none()
    }

    fn next_connection(&mut self) -> Option<ClientReport>
    {
        let Some((counter, report)) = self.pending_connections.pop_front() else { return None; };

        if Some(counter) == self.pending_connect
        {
            self.pending_connect = None;
        }
        if report == ClientReport::Disconnected
        {
            self.recent_consumed_disconnect = Some(counter);
        }

        Some(report)
    }

    fn next_message<T: SimplenetEvent>(&mut self, event_type_id: u16) -> Option<T>
    {
        // access pending responses
        let pending_connect = self.pending_connect;
        let Some(mut pending) = self.pending_messages.get_mut(&event_type_id) else { return None; };

        // don't take the message if it's newer than a pending connect
        let Some(first) = pending.first() else { return None; };
        if let Some(pending_connect) = pending_connect
        {
            if is_newer_or_eq(first.counter, pending_connect) { return None; }
        }

        // discard the message if it's older than the last-consumed disconnect
        let message = pending.pop_front().unwrap();

        if let Some(recent_dc) = self.recent_consumed_disconnect
        {
            if is_newer_or_eq(recent_dc, message.counter)
            { tracing::warn!(message.counter, "discarding server message older than a recent disconnect"); return None; }

            // INVARIANT: there should be no messages between the last-consumed disconnect and a pending connect
            if let Some(pending_connect) = pending_connect
            {
                if is_newer_or_eq(message.counter, recent_dc) && is_newer_or_eq(pending_connect, message.counter)
                {
                    tracing::error!(recent_dc, pending_connect, message.counter,
                        "discarding server message unexpectedly positioned between a disconnect and connect message");
                    return None;
                }
            }
        }

        // deserialize the message
        let Ok(server_msg) = bincode::DefaultOptions::new().deserialize(&message.data[..])
        else
        {
            tracing::warn!("received server msg that failed to deserialize");
            return None;
        };

        Some(server_msg)
    }

    fn next_response<Resp>(&mut self, request_event_id: u16, response_event_id: u16) -> Option<ServerResponse<Resp>>
    where
        Resp: SimplenetEvent,
    {
        // access pending responses
        let pending_connect = self.pending_connect;
        let Some(mut pending) = self.pending_responses
            .get_mut(&response_event_id)
            .get_mut(&request_event_id)
        else { return None; };

        // don't take the response if it's newer than a pending connect
        let Some(first) = pending.first() else { return None; };
        if let Some(pending_connect) = pending_connect
        {
            if is_newer_or_eq(first.counter, pending_connect) { return None; }
        }

        // 'lose' the response if it's a response/ack older than the last-consumed disconnect
        let mut response = pending.pop_front().unwrap();

        if let Some(recent_dc) = self.recent_consumed_disconnect
        {
            if is_newer_or_eq(recent_dc, response.counter)
            {
                let lose_it = match &response
                {
                    ServerResponse::Response(_) |
                    ServerResponse::Ack         => true,
                    _ => false
                };
                if lose_it
                {
                    tracing::warn!(response.counter, "'losing' server response older than a recent disconnect");
                    response = ServerResponse::ResponseLost;
                }
            }
        }

        // deserialize the message
        match response.response
        {
            PendingResponseData::Response(data) =>
            {
                let Ok(response) = bincode::DefaultOptions::new().deserialize(&data[..])
                else
                {
                    tracing::warn!("received server response that failed to deserialize");
                    return None;
                };
                Some(ServerResponse::Response(response, response.request_id))
            }
            PendingResponseData::Ack          => Some(ServerResponse::Ack(response.request_id)),
            PendingResponseData::Reject       => Some(ServerResponse::Reject(response.request_id)),
            PendingResponseData::SendFailed   => Some(ServerResponse::SendFailed(response.request_id)),
            PendingResponseData::ResponseLost => Some(ServerResponse::ResponseLost(response.request_id)),
        }
    }
}

impl Default for EventClientInner
{
    fn default() -> Self
    {
        Self{
            counter: 0u64,
            pending_connect: None,
            recent_consumed_disconnect: None,
            request_map: HashMap::default(),
            pending_connections: VecDeque::default(),
            pending_messages: HashMap::default(),
            pending_responses: HashMap::default(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
