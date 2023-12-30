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
pub(crate) struct InnerBevyClientConnection<E: EventPack>
{
    pub(crate) connection: ClientReport,
    pub(crate) phantom: PhantomData<E>,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Event)]
pub(crate) struct InnerBevyMessageClient<E: EventPack, T: SimplenetEvent>
{
    pub(crate) message: T,
    pub(crate) phantom: PhantomData<E>,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Event)]
pub(crate) struct InnerBevyResponse<E: EventPack, Req: SimplenetEvent, Resp: SimplenetEvent>
{
    pub(crate) response: Resp,
    pub(crate) phantom: PhantomData<(E, Req)>,
}

//-------------------------------------------------------------------------------------------------------------------
