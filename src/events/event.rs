use serde::{de::DeserializeOwned, Deserialize, Serialize};

use super::node;

pub struct Event {
    kind: Kind,
    payload: Vec<u8>,
}

#[derive(Debug)]
pub enum Kind {
    NodeMoved,
}

impl Event {
    pub fn deserialize<'a, T: Serialize>(self) -> T {
        match self.kind {
            Kind::NodeMoved => {
                serde_json::from_slice::<node::NodeMoved>(self.payload.as_slice()).unwrap()
            }
        }
    }
}
