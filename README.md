## Bevy Simplenet Events

Provides an events-based API for handling a networked connection, built on top of `bevy_simplenet`.


### Usage notes

- Events must be registered in the same order on the server and client.
- Events can be consumed in multiple systems with event readers.
- An event 'channel' of a single type is FIFO, however different event channels will not be synchronized with each other. This crate is not well-suited for users who want global FIFO ordering for all client/server outputs (use [`bevy_simplenet`](https://github.com/UkoeHB/bevy_simplenet) directly instead).


### Synchronization guarantees

This crate's API is highly opinionated to facilitate precise handling of reconnects.

If your on-disconnect or on-connect logic is spread across multiple systems, you must ensure synchronization guarantees are upheld (e.g. by chaining systems together and not running anything else in parallel), otherwise you will introduce race conditions that may cause very challenging bugs.

**Clients**

- New server messages/responses are unreadable until `ClientReport::Connected` has been read at least once.
- Old server messages are discarded if `ClientReport::Disconnected` has been read at least once.
- Old server responses of type `ServerResponse::Response` or `ServerResponse::Ack` are replaced with `ServerResponse::ResponseLost` if `ClientReport::Disconnected` has been read at least once. Other response variants are preserved.
- Client messages/requests will silently fail to send if the most recent `ClientReport::Connected` has not been read at least once or the client is not connected. Message statuses can be monitored with the `MessageSignal` returned from [`EventClient::send`], and request statuses can be monitored with `ResponseSignal` or you can wait for a result to be emitted as an event (request results are guaranteed to be emitted). (TODO: there is an upstream race condition)

**Servers**

- New client messages/requests are unreadable until `ServerReport::Connected` for that client has been read at least once.
- Old client messages/requests are discarded if `ServerReport::Disconnected` for that client has been read at least once.
- Server messages cannot be sent to a client until `ServerReport::Connected` for that client has been read at least once. (TODO: there is an upstream race condition)


### Performance

This crate is non-trivially less efficient than `bevy_simplenet`.
- Events are serialized and deserialized twice in order to enable ad-hoc event types.
- Both the client and the server use an internal mutex to enforce synchronization guarantees.
- Both the client and the server use several internal maps to buffer events while waiting for a reader to collect them.


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


