## Bevy Simplenet Events

Provides an events-based API for handling a networked connection, built on top of [`bevy_simplenet`](https://github.com/UkoeHB/bevy_simplenet).

This crate requires nightly rust.


### Usage notes

- Events must be registered in the same order on the server and client.
- Client connection events, client/server message events, and server responses can be iterated with event readers in multiple systems. Client requests can be drained with [`ServerRequestSource`] in one system.
- An event 'channel' of a single type is FIFO, however different event channels will not be synchronized with each other. This crate is not well-suited for users who want global FIFO ordering for all client/server outputs (use [`bevy_simplenet`](https://github.com/UkoeHB/bevy_simplenet) directly instead).
- We assume the user's connection-event handlers are scheduled **after** [`RefreshSet`](bevy_simplenet_events::RefreshSet) in schedule `First` and **before** other event handlers.


### Synchronization guarantees

This crate's API is highly opinionated to facilitate precise handling of reconnects.

We update the client and server state every tick in [`RefreshSet`](bevy_simplenet_events::RefreshSet) in schedule `First`. All old events are cleared, and new events are inserted. If a user's connection-event handlers are scheduled before other event handlers as expected, then we guarantee the following:

**Clients**

- [`ClientMessageReader`](bevy_simplenet_events::ClientMessageReader) will only read server messages from the current connection session. Old messages (from before the last disconnect) are discarded.
- [`ClientResponseReader`](bevy_simplenet_events::ClientResponseReader) will only emit [`ServerResponse::Response`](bevy_simplenet_events::Response) or [`ServerResponse::Ack`](bevy_simplenet_events::Ack) for responses received in the current connection session. All other responses will fail with one of the response-fail variants (rejected/send failed/response lost). Note that we guarantee a response of some kind will be emitted for every client request sent.
- Client messages/requests will silently fail to send or error-out if the most recent `ClientReport::Connected` has not been read by [`ClientConnectionReader`](bevy_simplenet_events::ClientConnectionReader) at least once (TODO: there is an upstream race condition), or if the client is not connected. Message statuses can be monitored with the `MessageSignal` returned from [`EventClient::send`](bevy_simplenet_events::EventClient::send), and request statuses can be monitored with the `RequestSignal` returned from [`EventClient::request`](bevy_simplenet_events::EventClient::request) or you can wait for a result to be emitted as an event. We include this guarantee to reduce the chance of clients sending messages based on stale client state while in the middle of handling connection events.

**Servers**

- [`ServerMessageReader`](bevy_simplenet_events::ServerMessageReader) and [`ServerRequestSource`](bevy_simplenet_events::ServerRequestSource) will only read client messages and requests from a client's current connection session. Old messages (from before the last disconnect) will be discarded.
- Server messages for a client will silently fail to send or error-out if the most recent `ServerReport::Connected` for that client has not been read by [`ServerConnectionReader`](bevy_simplenet_events::ServerConnectionReader) at least once (TODO: there is an upstream race condition), or if the client is not connected. We include this guarantee to reduce the chance of servers sending messages based on stale server state while in the middle of handling connection events. Note that responses from old connection sessions always fail to send to new sessions.


### Performance

This crate is less efficient than `bevy_simplenet`.
- Events are serialized and deserialized twice to enable ad-hoc event types.
- The client and server have additional indirection and copying to transmit messages from the internal client/server to the user.
- Events are exposed by reference rather than by value (except for client requests, which are drained by value on the server).


### Creating a channel

**Shared**


**Server**


**Client**


### Handling connections in the client


### Handling connections in the server


### Sending from the client


### Reading from the server


### Sending from the server


### Reading from the client


