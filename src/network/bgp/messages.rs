use crate::network::bgp::{BGPOrigin, RouteEntry};
use ipnet::IpNet;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BGPMessage {
    Open(OpenMessage),
    Update(UpdateMessage),
    Notification(NotificationMessage),
    Keepalive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenMessage {
    pub version: u8,
    pub my_asn: u32,
    pub hold_time: u16,
    pub bgp_identifier: IpAddr,
    pub optional_parameters: Vec<OptionalParameter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionalParameter {
    pub parameter_type: u8,
    pub parameter_length: u8,
    pub parameter_value: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMessage {
    pub withdrawn_routes: Vec<IpNet>,
    pub path_attributes: Vec<PathAttribute>,
    pub network_layer_reachability_info: Vec<IpNet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathAttribute {
    pub flags: u8,
    pub type_code: u8,
    pub length: u16,
    pub value: AttributeValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttributeValue {
    Origin(BGPOrigin),
    AsPath(Vec<u32>),
    NextHop(IpAddr),
    MultiExitDisc(u32),
    LocalPref(u32),
    Communities(Vec<u32>),
    Unknown(Vec<u8>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationMessage {
    pub error_code: u8,
    pub error_subcode: u8,
    pub data: Vec<u8>,
}

impl BGPMessage {
    pub fn new_open(asn: u32, hold_time: u16, router_id: IpAddr) -> Self {
        BGPMessage::Open(OpenMessage {
            version: 4,
            my_asn: asn,
            hold_time,
            bgp_identifier: router_id,
            optional_parameters: vec![],
        })
    }

    pub fn new_keepalive() -> Self {
        BGPMessage::Keepalive
    }

    pub fn new_update(routes: Vec<RouteEntry>) -> Self {
        let mut nlri = Vec::new();
        let mut path_attributes = Vec::new();

        for route in routes {
            nlri.push(route.network);

            // Add ORIGIN attribute
            path_attributes.push(PathAttribute {
                flags: 0x40,  // Well-known mandatory
                type_code: 1, // ORIGIN
                length: 1,
                value: AttributeValue::Origin(route.origin),
            });

            // Add AS_PATH attribute
            path_attributes.push(PathAttribute {
                flags: 0x40,  // Well-known mandatory
                type_code: 2, // AS_PATH
                length: (route.as_path.len() * 4) as u16,
                value: AttributeValue::AsPath(route.as_path),
            });

            // Add NEXT_HOP attribute
            path_attributes.push(PathAttribute {
                flags: 0x40,  // Well-known mandatory
                type_code: 3, // NEXT_HOP
                length: 4,
                value: AttributeValue::NextHop(route.next_hop),
            });

            // Add LOCAL_PREF attribute (if present)
            if route.local_pref != 100 {
                path_attributes.push(PathAttribute {
                    flags: 0x40,  // Well-known discretionary
                    type_code: 5, // LOCAL_PREF
                    length: 4,
                    value: AttributeValue::LocalPref(route.local_pref),
                });
            }

            // Add MED attribute (if present)
            if route.med != 0 {
                path_attributes.push(PathAttribute {
                    flags: 0x80,  // Optional non-transitive
                    type_code: 4, // MULTI_EXIT_DISC
                    length: 4,
                    value: AttributeValue::MultiExitDisc(route.med),
                });
            }
        }

        BGPMessage::Update(UpdateMessage {
            withdrawn_routes: vec![],
            path_attributes,
            network_layer_reachability_info: nlri,
        })
    }

    pub fn new_notification(error_code: u8, error_subcode: u8, data: Vec<u8>) -> Self {
        BGPMessage::Notification(NotificationMessage {
            error_code,
            error_subcode,
            data,
        })
    }

    pub fn serialize(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(data)
    }
}

// BGP Error Codes
pub const BGP_ERROR_MESSAGE_HEADER: u8 = 1;
pub const BGP_ERROR_OPEN_MESSAGE: u8 = 2;
pub const BGP_ERROR_UPDATE_MESSAGE: u8 = 3;
pub const BGP_ERROR_HOLD_TIMER_EXPIRED: u8 = 4;
pub const BGP_ERROR_FSM: u8 = 5;
pub const BGP_ERROR_CEASE: u8 = 6;

// BGP Attribute Types
pub const BGP_ATTR_ORIGIN: u8 = 1;
pub const BGP_ATTR_AS_PATH: u8 = 2;
pub const BGP_ATTR_NEXT_HOP: u8 = 3;
pub const BGP_ATTR_MULTI_EXIT_DISC: u8 = 4;
pub const BGP_ATTR_LOCAL_PREF: u8 = 5;
pub const BGP_ATTR_COMMUNITIES: u8 = 8;
