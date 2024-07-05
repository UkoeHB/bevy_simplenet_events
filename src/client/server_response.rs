use crate::*;

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

impl<T: SimplenetEvent> ServerResponse<T>
{
    /// Accesses the internal response if self is [`ServerResponse::Response`].
    pub fn response(&self) -> Option<&T>
    {
        match self {
            Self::Response(response, _) => Some(response),
            _ => None,
        }
    }

    /// Assesses the response's original request id.
    pub fn request_id(&self) -> u64
    {
        match self {
            Self::Response(_, request_id) => *request_id,
            Self::Ack(request_id) => *request_id,
            Self::Reject(request_id) => *request_id,
            Self::SendFailed(request_id) => *request_id,
            Self::ResponseLost(request_id) => *request_id,
        }
    }
}

impl<T: SimplenetEvent + Eq + PartialEq> Eq for ServerResponse<T> {}

impl<T: SimplenetEvent + Eq + PartialEq> PartialEq for ServerResponse<T>
{
    fn eq(&self, other: &Self) -> bool
    {
        match (self, other) {
            (Self::Response(l0, l1), Self::Response(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::Ack(l0), Self::Ack(r0)) => l0 == r0,
            (Self::Reject(l0), Self::Reject(r0)) => l0 == r0,
            (Self::SendFailed(l0), Self::SendFailed(r0)) => l0 == r0,
            (Self::ResponseLost(l0), Self::ResponseLost(r0)) => l0 == r0,
            _ => false,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
