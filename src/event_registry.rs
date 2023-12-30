//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::ecs::event::Event;
use bincode::Options;
use serde::{Serialize, Deserialize};
use serde_with::{Bytes, serde_as};

//standard shortcuts
use core::fmt::Debug;

//-------------------------------------------------------------------------------------------------------------------

/// Used to register simplenet event types that can be sent over the network.
///
/// We parameterize on `E` so the registry does not cause interference between multiple clients/servers in the same app.
/// We also use the event registry as a proxy for checking if simplenet event framework initialization has occurred yet.
#[derive(Resource, Debug, Default)]
pub(crate) struct EventRegistry<E: EventPack>
{
    id_counter           : u16,
    message_map          : HashMap<TypeId, u16>,
    request_map          : HashMap<TypeId, u16>,
    response_map         : HashMap<TypeId, u16>,
    request_response_map : HashMap<TypeId, TypeId>,
}

impl<E: EventPack> EventRegistry<E>
{
    pub(crate) fn register_message<E: SimplenetEvent>(&mut self)
    {
        self.id_counter += 1;
        let id = self.id_counter;

        let type_id = std::any::TypeId::of::<E>();
        let _ = self.message_map.insert(type_id, id);  //allow reentry in case of client/server having same message type
    }

    pub(crate) fn register_request_response<Req: SimplenetEvent, Resp: SimplenetEvent>(&mut self)
    {
        self.id_counter += 1;
        let req_id = self.id_counter;
        self.id_counter += 1;
        let resp_id = self.id_counter;

        let req_type_id = std::any::TypeId::of::<Req>();
        let resp_type_id = std::any::TypeId::of::<Req>();

        self.request_map.insert(req_type_id, req_id).expect("simplenet requests may only be registered once");
        let _ = self.response_map.insert(resp_type_id, resp_id);  //allow reentry
        self.request_response_map.insert(req_type_id, resp_type_id).unwrap();
    }

    pub(crate) fn get_message_id<E>(&self) -> Option<u16>
    {
        self.message_map.get(&std::any::TypeId::of::<E>()).map(|i| *i)
    }

    pub(crate) fn get_request_id<Req>(&self) -> Option<u16>
    {
        self.request_map.get(&std::any::TypeId::of::<Req>()).map(|i| *i)
    }

    pub(crate) fn get_response_id<Req>(&self) -> Option<u16>
    {
        self.response_map.get(&std::any::TypeId::of::<Req>()).map(|i| *i)
    }

    pub(crate) fn get_response_id_from_request<Req>(&self) -> Option<u16>
    {
        self.request_response_map
            .get(&std::any::TypeId::of::<Req>())
            .map(|t|
                self.response_map
                    .get(t)
                    .map(|i| *i)
            )
    }
}

//-------------------------------------------------------------------------------------------------------------------
