//local shortcuts
use bevy_simplenet_events::*;

//third-party shortcuts
use bevy_app::*;
use bevy_ecs::prelude::*;
use bevy_kot_ecs::*;
use bevy_simplenet::SessionId;
use enfync::AdoptOrDefault;
use serde::{Serialize, Deserialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(SimplenetEvent, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
struct DemoMsg1(usize);

#[derive(SimplenetEvent, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
struct DemoMsg2(usize);

#[derive(SimplenetEvent, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
struct DemoRequest1(usize);

#[derive(SimplenetEvent, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
struct DemoRequest2(usize);

#[derive(SimplenetEvent, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
struct DemoResponse1(usize);

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
struct DemoConnectMsg(String);

#[derive(Debug, Clone)]
struct DemoChannel;
impl EventPack for DemoChannel
{
    type ConnectMsg = DemoConnectMsg;
}

type DemoServerReport = bevy_simplenet::ServerReport<DemoConnectMsg>;

fn demo_server_factory() -> bevy_simplenet::ServerFactory<EventWrapper<DemoChannel>>
{
    bevy_simplenet::ServerFactory::<EventWrapper<DemoChannel>>::new("test")
}

fn demo_client_factory() -> bevy_simplenet::ClientFactory<EventWrapper<DemoChannel>>
{
    bevy_simplenet::ClientFactory::<EventWrapper<DemoChannel>>::new("test")
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn setup_server(app: &mut App) -> url::Url
{
    tracing::info!("launching server...");

    let websocket_server = demo_server_factory().new_server(
            enfync::builtin::native::TokioHandle::default(),
            "127.0.0.1:0",
            bevy_simplenet::AcceptorConfig::Default,
            bevy_simplenet::Authenticator::None,
            bevy_simplenet::ServerConfig::default(),
        );
    let url = websocket_server.url();

    app.insert_simplenet_server(websocket_server);

    url
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn setup_client(app: &mut App, url: url::Url, client_id: SessionId, connect_msg: DemoConnectMsg)
{
    tracing::info!(client_id, "launching client...");

    let websocket_client = demo_client_factory().new_client(
            enfync::builtin::Handle::adopt_or_default(),
            url,
            bevy_simplenet::AuthRequest::None{ client_id },
            bevy_simplenet::ClientConfig::default(),
            connect_msg
        );

    app.insert_simplenet_client(websocket_client);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn setup_event_app(app: &mut App)
{
    app
        .register_simplenet_client_message::<DemoChannel, DemoMsg1>()
        .register_simplenet_client_message::<DemoChannel, DemoMsg2>()

        .register_simplenet_server_message::<DemoChannel, DemoMsg1>()
        .register_simplenet_server_message::<DemoChannel, DemoMsg2>()

        .register_simplenet_request_response::<DemoChannel, DemoRequest1, DemoResponse1>()
        .register_simplenet_request_response::<DemoChannel, DemoRequest2, ()>()

        ;
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn num_connection_events_server(reader: ServerConnectionReader<DemoChannel>) -> usize
{
    reader.iter().count()
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn num_connection_events_client(reader: ClientConnectionReader<DemoChannel>) -> usize
{
    reader.iter().count()
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn num_message_events_server<T: SimplenetEvent>(reader: ServerMessageReader<DemoChannel, T>) -> usize
{
    reader.iter().count()
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn num_message_events_client<T: SimplenetEvent>(reader: ClientMessageReader<DemoChannel, T>) -> usize
{
    reader.iter().count()
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

//note that this consumes the requests
fn num_request_events_server<Req: SimplenetEvent, Resp: SimplenetEvent>(
    mut source: ServerRequestSource<DemoChannel, Req, Resp>
) -> usize
{
    source.drain().count()
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn num_response_events_client<Req: SimplenetEvent, Resp: SimplenetEvent>(
    reader: ClientResponseReader<DemoChannel, Req, Resp>
) -> usize
{
    reader.iter().count()
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn check_server_received_message<T: SimplenetEvent + Eq + PartialEq>(
    In((client_id, msg)) : In<(SessionId, T)>,
    reader               : ServerMessageReader<DemoChannel, T>
) -> bool
{
    for (session_id, client_msg) in reader.iter()
    {
        if client_id != session_id { continue; }
        if msg == *client_msg { return true; }
    }

    false
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn check_client_received_message<T: SimplenetEvent + Eq + PartialEq>(
    In(msg) : In<T>,
    reader  : ClientMessageReader<DemoChannel, T>
) -> bool
{
    for client_msg in reader.iter()
    {
        if msg == *client_msg { return true; }
    }

    false
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn check_client_connected_on_server(In(client_id): In<SessionId>, reader: ServerConnectionReader<DemoChannel>) -> bool
{
    for (session_id, connection) in reader.iter()
    {
        match connection
        {
            DemoServerReport::Connected(..) =>
            {
                if session_id == client_id
                {
                    return true;
                }
            }
            _ => ()
        }
    }

    false
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn check_client_connected_on_client(reader: ClientConnectionReader<DemoChannel>) -> bool
{
    for connection in reader.iter()
    {
        match connection
        {
            bevy_simplenet::ClientReport::Connected => return true,
            _ => ()
        }
    }

    return false
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn send_client_message<T: SimplenetEvent>(In(msg): In<T>, client: EventClient<DemoChannel>)
{
    client.send(msg).unwrap();
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn send_server_message<T: SimplenetEvent>(In((client_id, msg)): In<(SessionId, T)>, server: EventServer<DemoChannel>)
{
    server.send(client_id, msg).unwrap();
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

// client connection
//client connects
//server receives connection in multiple systems
//client receives connection in multiple systems
#[test]
fn client_connection()
{
    let mut server_app = App::new();
    let mut client_app = App::new();

    let url = setup_server(&mut server_app);
    let client_id = 0u128;
    setup_client(&mut client_app, url, client_id, DemoConnectMsg(String::default()));

    setup_event_app(&mut server_app);
    setup_event_app(&mut client_app);

    server_app.update();
    client_app.update();

    std::thread::sleep(std::time::Duration::from_millis(50));

    server_app.update();
    client_app.update();

    assert_eq!(syscall(&mut server_app.world, (), num_connection_events_server), 1);
    assert_eq!(syscall(&mut client_app.world, (), num_connection_events_client), 1);

    assert!(syscall(&mut server_app.world, client_id, check_client_connected_on_server));
    assert!(syscall(&mut client_app.world, (), check_client_connected_on_client));
}

//-------------------------------------------------------------------------------------------------------------------

// server: multi-system reader
//client message
//server receives in multiple systems
#[test]
fn server_multisystem_reader()
{
    let mut server_app = App::new();
    let mut client_app = App::new();

    let url = setup_server(&mut server_app);
    let client_id = 0u128;
    setup_client(&mut client_app, url, client_id, DemoConnectMsg(String::default()));

    setup_event_app(&mut server_app);
    setup_event_app(&mut client_app);

    server_app.update();
    client_app.update();

    std::thread::sleep(std::time::Duration::from_millis(50));

    server_app.update();
    client_app.update();

    //note: must read connection events before sending is allowed
    assert_eq!(syscall(&mut server_app.world, (), num_connection_events_server), 1);
    assert_eq!(syscall(&mut client_app.world, (), num_connection_events_client), 1);

    syscall(&mut client_app.world, DemoMsg1(10), send_client_message::<DemoMsg1>);
    syscall(&mut client_app.world, DemoMsg2(20), send_client_message::<DemoMsg2>);

    std::thread::sleep(std::time::Duration::from_millis(50));

    server_app.update();
    client_app.update();

    assert_eq!(syscall(&mut server_app.world, (), num_message_events_server::<DemoMsg1>), 1);
    assert_eq!(syscall(&mut server_app.world, (), num_message_events_server::<DemoMsg2>), 1);

    assert!(syscall(&mut server_app.world, (client_id, DemoMsg1(10)), check_server_received_message::<DemoMsg1>));
    assert!(syscall(&mut server_app.world, (client_id, DemoMsg2(20)), check_server_received_message::<DemoMsg2>));
}

//-------------------------------------------------------------------------------------------------------------------

// client: multi-system reader
//server message
//client receives in multiple systems

//-------------------------------------------------------------------------------------------------------------------

// client request w/ response
//client request
//server receives and responds
//client receives

//-------------------------------------------------------------------------------------------------------------------

// client request w/ acked/rejected (using () for the response type)
//client request
//server receives and acks/rejects
//client receives

//-------------------------------------------------------------------------------------------------------------------

// client and server in same app

//-------------------------------------------------------------------------------------------------------------------

// client: new server message blocked by connect event
//server sends message, disconnects client, waits for reconnect, sends new message
//client receives message 1, does not receive message 2, receives disconnect, does not receive message 2, receives connect,
//  receives message 2

//-------------------------------------------------------------------------------------------------------------------

// client: old server message dropped after disconnect consumed
//server sends message, disconnect client, waits for reconnect
//client receives disconnect, receives nothing, receives connect, receives nothing

//-------------------------------------------------------------------------------------------------------------------

// client: old server response of type 'response' or 'acl' replaced with 'response lost' after disconnect consumed
//client sends request, server sends response, disconnect client, waits for reconnect
//client receives disconnect, receives response lost

//-------------------------------------------------------------------------------------------------------------------

// client: message send blocked by connect event
//server sends message, disconnect client, waits for reconnect
//client receives message 1, fails to send new message, receives disconnect, can't send new message, receives connect,
//  can send

//-------------------------------------------------------------------------------------------------------------------

// server: new client message blocked by connect event
//client sends message, server disconnects, waits for reconnect, sends new message
//server receives message 1, does not receive message 2, receives disconnect, does not receive message 2, receives connect,
//  receives message 2

//-------------------------------------------------------------------------------------------------------------------

// server: old client message dropped after disconnect consumed
//client sends message, server disconnects, waits for reconnect
//server receives disconnect, receives nothing, receives connect, receives nothing

//-------------------------------------------------------------------------------------------------------------------

// server: old client request dropped after disconnect consumed
//client sends request, server disconnects, waits for reconnect
//server receives disconnect, receives nothing, receives connect, receives nothing

//-------------------------------------------------------------------------------------------------------------------

// server: message send blocked by connect event
//client sends message, server disconnects, waits for reconnect
//server receives message 1, fails to send new message, receives disconnect, can't send new message, receives connect,
//  can send

//-------------------------------------------------------------------------------------------------------------------
