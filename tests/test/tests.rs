//local shortcuts
use bevy_simplenet_events::*;

//third-party shortcuts
use bevy_app::*;
use bevy_ecs::prelude::*;
use bevy_kot_ecs::*;
use bevy_simplenet::{MessageStatus, RequestToken, SessionId};
use enfync::AdoptOrDefault;
use serde::{Serialize, Deserialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(SimplenetEvent, Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct DemoMsg1(usize);

#[derive(SimplenetEvent, Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct DemoMsg2(usize);

#[derive(SimplenetEvent, Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct DemoRequest1(usize);

#[derive(SimplenetEvent, Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct DemoRequest2(usize);

#[derive(SimplenetEvent, Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
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

fn get_server_requests<Req: SimplenetEvent + Eq + PartialEq, Resp: SimplenetEvent + Eq + PartialEq>(
    In(client_id) : In<SessionId>,
    mut reader    : ServerRequestSource<DemoChannel, Req, Resp>
) -> Vec<(RequestToken, Req)>
{
    reader.drain()
        .filter(
            |(token, _)|
            {
                token.client_id() == client_id
            }
        )
        .collect()
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn check_client_received_response<Req: SimplenetEvent + Eq + PartialEq, Resp: SimplenetEvent + Eq + PartialEq>(
    In(response) : In<ServerResponse<Resp>>,
    reader       : ClientResponseReader<DemoChannel, Req, Resp>
) -> bool
{
    for server_response in reader.iter()
    {
        if *server_response == response { return true; }
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
    client.send(msg);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn try_send_client_message<T: SimplenetEvent>(In(msg): In<T>, client: EventClient<DemoChannel>) -> bool
{
    let signal = client.send(msg);
    signal.status() != MessageStatus::Failed
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn send_client_request<Req: SimplenetEvent>(In(request): In<Req>, client: EventClient<DemoChannel>)
{
    client.request(request).unwrap();
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn send_server_message<T: SimplenetEvent>(In((client_id, msg)): In<(SessionId, T)>, server: EventServer<DemoChannel>)
{
    server.send(client_id, msg);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn send_server_response<Req: SimplenetEvent, Resp: SimplenetEvent>(
    In((token, response)) : In<(RequestToken, Resp)>,
    server                : EventServer<DemoChannel>
){
    server.respond(token, response);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn send_server_ack<Req: SimplenetEvent, Resp: SimplenetEvent>(
    In(token) : In<RequestToken>,
    server    : EventServer<DemoChannel>
){
    server.ack(token);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn send_server_reject<Req: SimplenetEvent, Resp: SimplenetEvent>(
    In(token) : In<RequestToken>,
    server    : EventServer<DemoChannel>
){
    server.reject(token);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn disconnect_client_on_server(In(session_id): In<SessionId>, server: EventServer<DemoChannel>)
{
    server.close_session(session_id, None);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn disconnect_client_on_client(client: EventClient<DemoChannel>)
{
    client.close();
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
    let mut client_app1 = App::new();
    let mut client_app2 = App::new();

    let url = setup_server(&mut server_app);
    let client_id1 = 0u128;
    let client_id2 = 1u128;
    setup_client(&mut client_app1, url.clone(), client_id1, DemoConnectMsg(String::default()));
    setup_client(&mut client_app2, url, client_id2, DemoConnectMsg(String::default()));

    setup_event_app(&mut server_app);
    setup_event_app(&mut client_app1);
    setup_event_app(&mut client_app2);

    server_app.update();
    client_app1.update();
    client_app2.update();

    std::thread::sleep(std::time::Duration::from_millis(50));

    server_app.update();
    client_app1.update();
    client_app2.update();

    //note: must read connection events before sending is allowed
    assert_eq!(syscall(&mut server_app.world, (), num_connection_events_server), 2);
    assert_eq!(syscall(&mut client_app1.world, (), num_connection_events_client), 1);
    assert_eq!(syscall(&mut client_app2.world, (), num_connection_events_client), 1);

    syscall(&mut client_app1.world, DemoMsg1(10), send_client_message::<DemoMsg1>);
    syscall(&mut client_app1.world, DemoMsg2(20), send_client_message::<DemoMsg2>);

    std::thread::sleep(std::time::Duration::from_millis(50));

    server_app.update();
    client_app1.update();
    client_app2.update();

    assert_eq!(syscall(&mut server_app.world, (), num_message_events_server::<DemoMsg1>), 1);
    assert_eq!(syscall(&mut server_app.world, (), num_message_events_server::<DemoMsg2>), 1);

    assert!(syscall(&mut server_app.world, (client_id1, DemoMsg1(10)), check_server_received_message::<DemoMsg1>));
    assert!(syscall(&mut server_app.world, (client_id1, DemoMsg2(20)), check_server_received_message::<DemoMsg2>));
}

//-------------------------------------------------------------------------------------------------------------------

// client: multi-system reader
//server message
//client receives in multiple systems
#[test]
fn client_multisystem_reader()
{
    let mut server_app = App::new();
    let mut client_app1 = App::new();
    let mut client_app2 = App::new();

    let url = setup_server(&mut server_app);
    let client_id1 = 0u128;
    let client_id2 = 1u128;
    setup_client(&mut client_app1, url.clone(), client_id1, DemoConnectMsg(String::default()));
    setup_client(&mut client_app2, url, client_id2, DemoConnectMsg(String::default()));

    setup_event_app(&mut server_app);
    setup_event_app(&mut client_app1);
    setup_event_app(&mut client_app2);

    server_app.update();
    client_app1.update();
    client_app2.update();

    std::thread::sleep(std::time::Duration::from_millis(75));

    server_app.update();
    client_app1.update();
    client_app2.update();

    //note: must read connection events before sending is allowed
    assert_eq!(syscall(&mut server_app.world, (), num_connection_events_server), 2);
    assert_eq!(syscall(&mut client_app1.world, (), num_connection_events_client), 1);
    assert_eq!(syscall(&mut client_app2.world, (), num_connection_events_client), 1);

    syscall(&mut server_app.world, (client_id1, DemoMsg1(10)), send_server_message::<DemoMsg1>);
    syscall(&mut server_app.world, (client_id1, DemoMsg2(20)), send_server_message::<DemoMsg2>);

    std::thread::sleep(std::time::Duration::from_millis(50));

    server_app.update();
    client_app1.update();
    client_app2.update();

    assert_eq!(syscall(&mut client_app1.world, (), num_message_events_client::<DemoMsg1>), 1);
    assert_eq!(syscall(&mut client_app1.world, (), num_message_events_client::<DemoMsg2>), 1);
    assert_eq!(syscall(&mut client_app2.world, (), num_message_events_client::<DemoMsg1>), 0);
    assert_eq!(syscall(&mut client_app2.world, (), num_message_events_client::<DemoMsg2>), 0);

    assert!(syscall(&mut client_app1.world, DemoMsg1(10), check_client_received_message::<DemoMsg1>));
    assert!(syscall(&mut client_app1.world, DemoMsg2(20), check_client_received_message::<DemoMsg2>));
}


//-------------------------------------------------------------------------------------------------------------------

// client request w/ response
//client request
//server receives and responds
//client receives
#[test]
fn client_request()
{
    // prepare tracing
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

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

    syscall(&mut client_app.world, DemoRequest1(1), send_client_request::<DemoRequest1>);

    std::thread::sleep(std::time::Duration::from_millis(50));

    server_app.update();
    client_app.update();

    let mut reqs = syscall(&mut server_app.world, client_id, get_server_requests::<DemoRequest1, DemoResponse1>);
    let (token, req) = reqs.pop().unwrap();
    let request_id = token.request_id();
    assert_eq!(req, DemoRequest1(1));

    syscall(&mut server_app.world, (token, DemoResponse1(2)), send_server_response::<DemoRequest1, DemoResponse1>);

    std::thread::sleep(std::time::Duration::from_millis(50));

    server_app.update();
    client_app.update();

    assert!(syscall(
            &mut client_app.world,
            ServerResponse::Response(DemoResponse1(2), request_id),
            check_client_received_response::<DemoRequest1, DemoResponse1>
        ));
    assert!(syscall(
            &mut client_app.world,
            ServerResponse::Response(DemoResponse1(2), request_id),
            check_client_received_response::<DemoRequest1, DemoResponse1>
        ));
}

//-------------------------------------------------------------------------------------------------------------------

// client request w/ acked/rejected (using () for the response type)
//client request
//server receives and acks/rejects
//client receives
#[test]
fn client_request_acked_rejected()
{
    // prepare tracing
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

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

    syscall(&mut client_app.world, DemoRequest2(2), send_client_request::<DemoRequest2>);
    syscall(&mut client_app.world, DemoRequest2(22), send_client_request::<DemoRequest2>);

    std::thread::sleep(std::time::Duration::from_millis(50));

    server_app.update();
    client_app.update();

    let mut requests = syscall(&mut server_app.world, client_id, get_server_requests::<DemoRequest2, ()>);
    let (token1, req1) = requests.pop().unwrap();
    let (token2, req2) = requests.pop().unwrap();
    let request_id1 = token1.request_id();
    let request_id2 = token2.request_id();
    let mut reqs = [req1, req2]; reqs.sort();
    assert_eq!(reqs, [DemoRequest2(2), DemoRequest2(22)]);

    syscall(&mut server_app.world, token1, send_server_ack::<DemoRequest2, ()>);
    syscall(&mut server_app.world, token2, send_server_reject::<DemoRequest2, ()>);

    std::thread::sleep(std::time::Duration::from_millis(50));

    server_app.update();
    client_app.update();

    assert!(syscall(
            &mut client_app.world,
            ServerResponse::Ack(request_id1),
            check_client_received_response::<DemoRequest2, ()>
        ));
    assert!(syscall(
            &mut client_app.world,
            ServerResponse::Reject(request_id2),
            check_client_received_response::<DemoRequest2, ()>
        ));
}

//-------------------------------------------------------------------------------------------------------------------

// client and server in same app
#[test]
fn client_server_shared_app()
{
    let mut shared_app = App::new();

    let url = setup_server(&mut shared_app);
    let client_id = 0u128;
    setup_client(&mut shared_app, url, client_id, DemoConnectMsg(String::default()));

    setup_event_app(&mut shared_app);

    shared_app.update();

    std::thread::sleep(std::time::Duration::from_millis(50));

    shared_app.update();

    assert_eq!(syscall(&mut shared_app.world, (), num_connection_events_server), 1);
    assert_eq!(syscall(&mut shared_app.world, (), num_connection_events_client), 1);

    assert!(syscall(&mut shared_app.world, client_id, check_client_connected_on_server));
    assert!(syscall(&mut shared_app.world, (), check_client_connected_on_client));

    syscall(&mut shared_app.world, (client_id, DemoMsg1(1)), send_server_message::<DemoMsg1>);
    syscall(&mut shared_app.world, DemoMsg1(11), send_client_message::<DemoMsg1>);

    std::thread::sleep(std::time::Duration::from_millis(50));

    shared_app.update();

    assert!(syscall(&mut shared_app.world, (client_id, DemoMsg1(11)), check_server_received_message::<DemoMsg1>));
    assert!(syscall(&mut shared_app.world, DemoMsg1(1), check_client_received_message::<DemoMsg1>));
}

//-------------------------------------------------------------------------------------------------------------------

// client: old server message dropped after disconnect
#[test]
fn client_drops_old_server_msg()
{
    // prepare tracing
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

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

    syscall(&mut server_app.world, (client_id, DemoMsg1(1)), send_server_message::<DemoMsg1>);
    std::thread::sleep(std::time::Duration::from_millis(50));
    syscall(&mut server_app.world, client_id, disconnect_client_on_server);

    std::thread::sleep(std::time::Duration::from_millis(200));

    server_app.update();
    client_app.update();

    assert_eq!(syscall(&mut client_app.world, (), num_connection_events_client), 2);
    assert_eq!(syscall(&mut client_app.world, (), num_message_events_client::<DemoMsg1>), 0);
}

//-------------------------------------------------------------------------------------------------------------------

// client: old server response of type 'response' or 'ack' replaced with 'response lost' after disconnect
#[test]
fn client_loses_old_server_response()
{
    // prepare tracing
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

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

    syscall(&mut client_app.world, DemoRequest1(1), send_client_request::<DemoRequest1>);
    syscall(&mut client_app.world, DemoRequest1(2), send_client_request::<DemoRequest1>);

    std::thread::sleep(std::time::Duration::from_millis(50));

    server_app.update();

    let mut requests = syscall(&mut server_app.world, client_id, get_server_requests::<DemoRequest1, DemoResponse1>);
    let (token1, _req1) = requests.pop().unwrap();
    let (token2, _req2) = requests.pop().unwrap();
    let request_id1 = token1.request_id();
    let request_id2 = token2.request_id();

    syscall(&mut server_app.world, (token1, DemoResponse1(1)), send_server_response::<DemoRequest1, DemoResponse1>);
    syscall(&mut server_app.world, token2, send_server_ack::<DemoRequest1, DemoResponse1>);
    std::thread::sleep(std::time::Duration::from_millis(50));
    syscall(&mut server_app.world, client_id, disconnect_client_on_server);

    std::thread::sleep(std::time::Duration::from_millis(200));

    server_app.update();
    client_app.update();

    assert_eq!(syscall(&mut client_app.world, (), num_connection_events_client), 2);
    assert_eq!(syscall(&mut client_app.world, (), num_response_events_client::<DemoRequest1, DemoResponse1>), 2);

    assert!(syscall(
            &mut client_app.world,
            ServerResponse::ResponseLost(request_id1),
            check_client_received_response::<DemoRequest1, DemoResponse1>
        ));
    assert!(syscall(
            &mut client_app.world,
            ServerResponse::ResponseLost(request_id2),
            check_client_received_response::<DemoRequest1, DemoResponse1>
        ));
}

//-------------------------------------------------------------------------------------------------------------------

// client: message send blocked by connect event
#[test]
fn client_send_blocked_until_read_connect()
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

    assert!(!syscall(&mut client_app.world, DemoMsg1(1), try_send_client_message::<DemoMsg1>));

    assert_eq!(syscall(&mut server_app.world, (), num_connection_events_server), 1);
    assert_eq!(syscall(&mut client_app.world, (), num_connection_events_client), 1);

    assert!(syscall(&mut client_app.world, DemoMsg1(10), try_send_client_message::<DemoMsg1>));

    std::thread::sleep(std::time::Duration::from_millis(50));

    server_app.update();
    client_app.update();

    assert_eq!(syscall(&mut server_app.world, (), num_message_events_server::<DemoMsg1>), 1);

    assert!(syscall(&mut server_app.world, (client_id, DemoMsg1(10)), check_server_received_message::<DemoMsg1>));
}

//-------------------------------------------------------------------------------------------------------------------

// server: old client message/request dropped after disconnect
#[test]
fn server_drops_old_client_msg()
{
    // prepare tracing
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

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

    syscall(&mut client_app.world, DemoMsg1(1), send_client_message::<DemoMsg1>);
    syscall(&mut client_app.world, DemoRequest1(11), send_client_request::<DemoRequest1>);
    std::thread::sleep(std::time::Duration::from_millis(50));
    syscall(&mut client_app.world, (), disconnect_client_on_client);

    std::thread::sleep(std::time::Duration::from_millis(200));

    server_app.update();
    client_app.update();

    assert_eq!(syscall(&mut server_app.world, (), num_connection_events_server), 1);
    assert_eq!(syscall(&mut server_app.world, (), num_message_events_server::<DemoMsg1>), 0);
    assert_eq!(syscall(&mut server_app.world, (), num_request_events_server::<DemoRequest1, DemoResponse1>), 0);
}

//-------------------------------------------------------------------------------------------------------------------

// server: message send blocked by connect event
#[test]
fn server_send_blocked_until_read_connect()
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

    syscall(&mut server_app.world, (client_id, DemoMsg1(1)), send_server_message::<DemoMsg1>);

    assert_eq!(syscall(&mut server_app.world, (), num_connection_events_server), 1);
    assert_eq!(syscall(&mut client_app.world, (), num_connection_events_client), 1);

    syscall(&mut server_app.world, (client_id, DemoMsg1(10)), send_server_message::<DemoMsg1>);

    std::thread::sleep(std::time::Duration::from_millis(50));

    server_app.update();
    client_app.update();

    assert_eq!(syscall(&mut client_app.world, (), num_message_events_client::<DemoMsg1>), 1);

    assert!(syscall(&mut client_app.world, DemoMsg1(10), check_client_received_message::<DemoMsg1>));
}

//-------------------------------------------------------------------------------------------------------------------
