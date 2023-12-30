//module tree

// client connection
//client connects
//server receives connection in multiple systems
//client receives connection in multiple systems

// server: multi-system reader
//client message
//server receives in multiple systems

// client: multi-system reader
//server message
//client receives in multiple systems

// client request w/ response
//client request
//server receives and responds
//client receives

// client request w/ acked/rejected (using () for the response type)
//client request
//server receives and acks/rejects
//client receives

// client: new server message blocked by connect event
//server sends message, disconnects client, waits for reconnect, sends new message
//client receives message 1, does not receive message 2, receives disconnect, does not receive message 2, receives connect,
//  receives message 2

// client: old server message dropped after disconnect consumed
//server sends message, disconnect client, waits for reconnect
//client receives disconnect, receives nothing, receives connect, receives nothing

// client: old server response of type 'response' or 'acl' replaced with 'response lost' after disconnect consumed
//client sends request, server sends response, disconnect client, waits for reconnect
//client receives disconnect, receives response lost

// client: message send blocked by connect event
//server sends message, disconnect client, waits for reconnect
//client receives message 1, fails to send new message, receives disconnect, can't send new message, receives connect,
//  can send

// server: new client message blocked by connect event
//client sends message, server disconnects, waits for reconnect, sends new message
//server receives message 1, does not receive message 2, receives disconnect, does not receive message 2, receives connect,
//  receives message 2

// server: old client message dropped after disconnect consumed
//client sends message, server disconnects, waits for reconnect
//server receives disconnect, receives nothing, receives connect, receives nothing

// server: old client request dropped after disconnect consumed
//client sends request, server disconnects, waits for reconnect
//server receives disconnect, receives nothing, receives connect, receives nothing

// server: message send blocked by connect event
//client sends message, server disconnects, waits for reconnect
//server receives message 1, fails to send new message, receives disconnect, can't send new message, receives connect,
//  can send
