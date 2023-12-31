//local shortcuts
use crate::*;

//third-party shortcuts

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Server response for a client request.
pub enum ServerResponse<T: SimplenetEvent>
{
    /// Response from the server.
    Response(T, u64),
    /// Request is acknowledged. No response will be sent.
    Ack(u64),
    /// Request is rejected. No response will be sent.
    Reject(u64),
    /// Sending a request failed.
    SendFailed(u64),
    /// The server received a request but the client failed to receive a response.
    ResponseLost(u64),
}

//-------------------------------------------------------------------------------------------------------------------
