use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PayloadPan {
    pub diff: [f32; 2],
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PayloadZoom {
    pub diff: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PayloadNodeMove {
    pub id: usize,
    pub diff: [f32; 2],
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PayloadNodeDragStart {
    pub id: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PayloadNodeDragEnd {
    pub id: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PayloadNodeSelect {
    pub id: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PayloadNodeDeselect {
    pub id: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PayloadNodeClick {
    pub id: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PayloadNodeDoubleClick {
    pub id: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PayloadEdgeClick {
    pub id: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PayloadEdgeSelect {
    pub id: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PayloadEdgeDeselect {
    pub id: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Event {
    Pan(PayloadPan),
    Zoom(PayloadZoom),
    NodeMove(PayloadNodeMove),
    NodeDragStart(PayloadNodeDragStart),
    NodeDragEnd(PayloadNodeDragEnd),
    NodeSelect(PayloadNodeSelect),
    NodeDeselect(PayloadNodeDeselect),
    NodeClick(PayloadNodeClick),
    NodeDoubleClick(PayloadNodeDoubleClick),
    EdgeClick(PayloadEdgeClick),
    EdgeSelect(PayloadEdgeSelect),
    EdgeDeselect(PayloadEdgeDeselect),
}
