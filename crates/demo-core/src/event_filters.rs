#![cfg(feature = "events")]

use crate::Event;

#[derive(Clone)]
pub struct EventFilters {
    pub pan: bool,
    pub zoom: bool,
    pub node_move: bool,
    pub node_drag_start: bool,
    pub node_drag_end: bool,
    pub node_hover_enter: bool,
    pub node_hover_leave: bool,
    pub node_select: bool,
    pub node_deselect: bool,
    pub node_click: bool,
    pub node_double_click: bool,
    pub edge_click: bool,
    pub edge_select: bool,
    pub edge_deselect: bool,
}

impl Default for EventFilters {
    fn default() -> Self {
        Self {
            pan: true,
            zoom: true,
            node_move: true,
            node_drag_start: true,
            node_drag_end: true,
            node_hover_enter: true,
            node_hover_leave: true,
            node_select: true,
            node_deselect: true,
            node_click: true,
            node_double_click: true,
            edge_click: true,
            edge_select: true,
            edge_deselect: true,
        }
    }
}

impl EventFilters {
    pub fn enabled_for(&self, e: &Event) -> bool {
        use Event::*;
        match e {
            Pan(_) => self.pan,
            Zoom(_) => self.zoom,
            NodeMove(_) => self.node_move,
            NodeDragStart(_) => self.node_drag_start,
            NodeDragEnd(_) => self.node_drag_end,
            NodeHoverEnter(_) => self.node_hover_enter,
            NodeHoverLeave(_) => self.node_hover_leave,
            NodeSelect(_) => self.node_select,
            NodeDeselect(_) => self.node_deselect,
            NodeClick(_) => self.node_click,
            NodeDoubleClick(_) => self.node_double_click,
            EdgeClick(_) => self.edge_click,
            EdgeSelect(_) => self.edge_select,
            EdgeDeselect(_) => self.edge_deselect,
        }
    }
    pub fn is_event_str_enabled(&self, ev: &str) -> Option<bool> {
        if ev.starts_with("Pan") {
            Some(self.pan)
        } else if ev.starts_with("Zoom") {
            Some(self.zoom)
        } else if ev.starts_with("NodeMove") {
            Some(self.node_move)
        } else if ev.starts_with("NodeDragStart") {
            Some(self.node_drag_start)
        } else if ev.starts_with("NodeDragEnd") {
            Some(self.node_drag_end)
        } else if ev.starts_with("NodeHoverEnter") {
            Some(self.node_hover_enter)
        } else if ev.starts_with("NodeHoverLeave") {
            Some(self.node_hover_leave)
        } else if ev.starts_with("NodeSelect") {
            Some(self.node_select)
        } else if ev.starts_with("NodeDeselect") {
            Some(self.node_deselect)
        } else if ev.starts_with("NodeClick") {
            Some(self.node_click)
        } else if ev.starts_with("NodeDoubleClick") {
            Some(self.node_double_click)
        } else if ev.starts_with("EdgeClick") {
            Some(self.edge_click)
        } else if ev.starts_with("EdgeSelect") {
            Some(self.edge_select)
        } else if ev.starts_with("EdgeDeselect") {
            Some(self.edge_deselect)
        } else {
            None
        }
    }
    pub fn purge_disabled(&self, events: &mut Vec<String>) {
        events.retain(|ev| self.is_event_str_enabled(ev.as_str()).unwrap_or(true));
    }
}
