//local shortcuts
use crate::*;

//third-party shortcuts

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Server response for a client request.
pub enum ServerResponse<T: SimplenetEvent>
{
    /// Response from the server.
    Response(T),
    /// Request is acknowledged. No response will be sent.
    Ack,
    /// Request is rejected. No response will be sent.
    Reject,
    /// Sending a request failed.
    SendFailed,
    /// The server received a request but the client failed to receive a response.
    ResponseLost,
}

//-------------------------------------------------------------------------------------------------------------------
