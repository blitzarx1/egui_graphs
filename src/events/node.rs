use serde::{Deserialize, Serialize};

use super::event::Kind;

#[derive(Serialize, Deserialize, Debug)]
pub struct NodeMoved {
    pub kind: Kind,
    pub id: usize,
    pub diff: [f32; 2],
}
