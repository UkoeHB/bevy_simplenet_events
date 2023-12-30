//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::app::App;
use bincode::Options;
use serde::{Serialize, Deserialize};
use serde_with::{Bytes, serde_as};

//standard shortcuts
use core::fmt::Debug;
use std::marker::PhantomData;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Event)]
pub(crate) struct InnerBevyServerConnection<E: EventPack>
{
    pub(crate) connection: ServerReport<E::ConnectMsg>,
    pub(crate) phantom: PhantomData<E>,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Event)]
pub(crate) struct InnerBevyMessageServer<E: EventPack, T: SimplenetEvent>
{
    pub(crate) message: T,
    pub(crate) phantom: PhantomData<E>,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Event)]
pub(crate) struct InnerBevyRequest<E: EventPack, Req: SimplenetEvent, Resp: SimplenetEvent>
{
    pub(crate) request: Req,
    pub(crate) phantom: PhantomData<(E, Resp)>,
}

//-------------------------------------------------------------------------------------------------------------------
