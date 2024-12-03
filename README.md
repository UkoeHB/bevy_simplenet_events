## Bevy Simplenet Events

Provides an events-based API for handling a networked connection, built on top of [`bevy_simplenet`](https://github.com/UkoeHB/bevy_simplenet).


### Usage notes

- Client connection events, client/server message events, and server responses can be iterated with event readers in multiple systems. Client requests can be drained with [`ServerRequestSource`](bevy_simplenet_events::ServerRequestSource) in one system.
- An event 'channel' of a single type is FIFO, however different event channels will not be synchronized with each other. This crate is not well-suited for users who want global FIFO ordering for all client/server outputs (use [`bevy_simplenet`](https://github.com/UkoeHB/bevy_simplenet) directly instead).
- We assume the user's connection-event handlers are scheduled **after** [`RefreshSet`](bevy_simplenet_events::RefreshSet) in schedule `First` and **before** other event handlers.
- Events must be registered in the same order on the server and client.


### Synchronization guarantees

This crate's API is highly opinionated to facilitate precise handling of reconnects.

We update the client and server state every tick in [`RefreshSet`](bevy_simplenet_events::RefreshSet) in schedule `First`. All old events are cleared, and new events are inserted. If a user's connection-event handlers are scheduled before other event handlers as expected, then we guarantee the following:

**Clients**

- [`ClientMessageReader`](bevy_simplenet_events::ClientMessageReader) will only read server messages from the current connection session. Old messages (from before the last disconnect) are discarded.
- [`ClientResponseReader`](bevy_simplenet_events::ClientResponseReader) will only emit [`ServerResponse::Response`](bevy_simplenet_events::ServerResponse::Response) or [`ServerResponse::Ack`](bevy_simplenet_events::ServerResponse::Ack) for responses received in the current connection session. All other responses will fail with one of the response-fail variants (rejected/send failed/response lost). Note that we guarantee a response of some kind will be emitted for every client request sent.
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

Prepare message types and the channel tag that implements [`EventPack`](bevy_simplenet_events::EventPack).

```rust
#[derive(SimplenetEvent, Serialize, Deserialize)]
struct DemoMsg1(usize);

#[derive(SimplenetEvent, Serialize, Deserialize)]
struct DemoMsg2(usize);

#[derive(SimplenetEvent, Serialize, Deserialize)]
struct DemoRequest(usize);

#[derive(SimplenetEvent, Serialize, Deserialize)]
struct DemoResponse(usize);

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
struct DemoConnectMsg(String);

#[derive(Debug, Clone)]
struct DemoChannel;
impl EventPack for DemoChannel
{
    type ConnectMsg = DemoConnectMsg;
}
```

Prepare event setup function. This should be called on both the server and client apps.

```rust
fn event_setup(app: &mut App)
{
    app
        .register_simplenet_client_message::<DemoChannel, DemoMsg1>()
        .register_simplenet_client_message::<DemoChannel, DemoMsg2>()

        .register_simplenet_server_message::<DemoChannel, DemoMsg1>()
        .register_simplenet_server_message::<DemoChannel, DemoMsg2>()

        .register_simplenet_request_response::<DemoChannel, DemoRequest, DemoResponse>()
        ;
}
```

**Server**

Prepare server factory.

```rust
type DemoServerReport = bevy_simplenet::ServerReport<DemoConnectMsg>;

fn demo_server_factory() -> bevy_simplenet::ServerFactory<EventWrapper<DemoChannel>>
{
    bevy_simplenet::ServerFactory::<EventWrapper<DemoChannel>>::new("test")
}
```

Prepare server setup function (example).

```rust
fn setup_server(app: &mut App) -> url::Url
{
    let server = demo_server_factory().new_server(
            enfync::builtin::native::TokioHandle::adopt_or_default(),
            "127.0.0.1:0",
            bevy_simplenet::AcceptorConfig::Default,
            bevy_simplenet::Authenticator::None,
            bevy_simplenet::ServerConfig::default(),
        );
    let url = server.url();

    app.insert_simplenet_server(server);
    event_setup(app);

    url
}
```

**Client**

Prepare client factory.

```rust
fn demo_client_factory() -> bevy_simplenet::ClientFactory<EventWrapper<DemoChannel>>
{
    bevy_simplenet::ClientFactory::<EventWrapper<DemoChannel>>::new("test")
}
```

Prepare client setup function (example).

```rust
fn setup_client(app: &mut App, url: url::Url, client_id: SessionId, connect_msg: DemoConnectMsg)
{
    let client = demo_client_factory().new_client(
            enfync::builtin::Handle::adopt_or_default(),
            url,
            bevy_simplenet::AuthRequest::None{ client_id },
            bevy_simplenet::ClientConfig::default(),
            connect_msg
        );

    app.insert_simplenet_client(client);
    event_setup(app);
}
```


### Handling connections in the client

Client connection reports must be handled before all other client events each tick.

```rust
fn handle_client_connection_reports(reader: ClientConnectionReader<DemoChannel>)
{
    for connection in reader.iter()
    {
        match connection
        {
            bevy_simplenet::ClientReport::Connected         => todo!(),
            bevy_simplenet::ClientReport::Disconnected      => todo!(),
            bevy_simplenet::ClientReport::ClosedByServer(_) => todo!(),
            bevy_simplenet::ClientReport::ClosedBySelf      => todo!(),
            bevy_simplenet::ClientReport::IsDead(_)         => todo!(),
        }
    }
}
```


### Handling connections in the server

Server connection reports must be handled before all other server events each tick.

```rust
fn handle_server_connection_reports(reader: ServerConnectionReader<DemoChannel>)
{
    for (session_id, connection) in reader.iter()
    {
        match connection
        {
            bevy_simplenet::ServerReport::<DemoConnectMsg>::Connected(_, _) => todo!(),
            bevy_simplenet::ServerReport::<DemoConnectMsg>::Disconnected    => todo!(),
        }
    }
}
```


### Sending from the client

Any registered message type can be sent.

```rust
fn send_client_message(client: EventClient<DemoChannel>)
{
    client.send(DemoMsg1(42));
    client.send(DemoMsg2(24));
}
```


### Sending from the server

Any registered message type can be sent.

```rust
fn send_server_message(In(session_id): In<SessionId>, server: EventServer<DemoChannel>)
{
    server.send(session_id, DemoMsg1(42)).unwrap();
    server.send(session_id, DemoMsg2(24)).unwrap();
}
```


### Reading on the server

**Client messages**

```rust
fn read_client_messages(reader: ServerMessageReader<DemoChannel, DemoMsg1>)
{
    for (session_id, message) in reader.iter()
    {
        todo!()
    }
}
```

**Client requests**

Draining a request source consumes all requests, since we expect you to do something with the request token.

```rust
fn read_client_requests(source: ServerRequestSource<DemoChannel, DemoRequest1, DemoResponse1>)
{
    for (token, request) in source.drain()
    {
        todo!()
    }
}
```


### Reading on the client

**Server messages**

```rust
fn read_server_messages(reader: ClientMessageReader<DemoChannel, DemoMsg1>)
{
    for message in reader.iter()
    {
        todo!()
    }
}
```

**Server responses**

```rust
fn read_server_responses(reader: ClientResponseReader<DemoChannel, DemoRequest1, DemoResponse1>)
{
    for response in reader.iter()
    {
        match response
        {
            ServerResponse::Response(response, _) => todo!(),
            ServerResponse::Ack(_)                => todo!(),
            ServerResponse::Reject(_)             => todo!(),
            ServerResponse::SendFailed(_)         => todo!(),
            ServerResponse::ResponseLost(_)       => todo!(),
        }
    }
}
```



## Bevy compatability

| bevy   | bevy_simplenet_events |
|--------|-----------------------|
| 0.15   | v0.5                  |
| 0.14   | v0.4                  |
| 0.13   | v0.3                  |
| 0.12   | v0.1 - v0.2           |
