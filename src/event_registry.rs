//local shortcuts
use crate::*;

//third-party shortcuts
use bevy_ecs::prelude::*;

//standard shortcuts
use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;

//-------------------------------------------------------------------------------------------------------------------

/// Used to register simplenet event types that can be sent over the network.
///
/// We parameterize on `E` so the registry does not cause interference between multiple clients/servers in the same app.
/// We also use the event registry as a proxy for checking if simplenet event framework initialization has occurred yet.
#[derive(Resource, Debug)]
pub(crate) struct EventRegistry<E: EventPack>
{
    id_counter           : u16,
    message_map          : HashMap<TypeId, u16>,
    request_map          : HashMap<TypeId, u16>,
    response_map         : HashMap<TypeId, u16>,
    request_response_map : HashMap<TypeId, TypeId>,
    phantom              : PhantomData<E>
}

impl<E: EventPack> EventRegistry<E>
{
    pub(crate) fn register_message<T: SimplenetEvent>(&mut self) -> u16
    {
        // allow re-entry in case of client/server having same message type
        let type_id = std::any::TypeId::of::<T>();
        if let Some(id) = self.message_map.get(&type_id) { return *id; }

        // make new entry
        self.id_counter += 1;
        let id = self.id_counter;

        self.message_map.insert(type_id, id).unwrap();

        id
    }

    pub(crate) fn register_request_response<Req: SimplenetEvent, Resp: SimplenetEvent>(&mut self) -> (u16, u16)
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

        (req_id, resp_id)
    }

    pub(crate) fn get_message_id<T>(&self) -> Option<u16>
    {
        self.message_map.get(&std::any::TypeId::of::<T>()).map(|i| *i)
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
            .flatten()
    }
}

impl<E: EventPack> Default for EventRegistry<E>
{
    fn default() -> Self
    {
        Self{
            id_counter           : 0u16,
            message_map          : HashMap::default(),
            request_map          : HashMap::default(),
            response_map         : HashMap::default(),
            request_response_map : HashMap::default(),
            phantom              : PhantomData::default(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
