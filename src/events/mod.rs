mod event;

pub use event::{
    Event, PayloadEdgeClick, PayloadEdgeDeselect, PayloadEdgeSelect, PayloadNodeClick,
    PayloadNodeDeselect, PayloadNodeDoubleClick, PayloadNodeDragEnd, PayloadNodeDragStart,
    PayloadNodeHoverEnter, PayloadNodeHoverLeave, PayloadNodeMove, PayloadNodeSelect, PayloadPan,
    PayloadZoom,
};

/// A simple, object-safe sink for graph interaction events.
///
/// This trait is wasm-friendly and thread-agnostic. Implementations can forward
/// events to channels, buffers, logs, or JS callbacks.
pub trait EventSink {
    fn send(&self, e: Event);
}

// Convenience implementations

/// Forward events to any Fn(Event) callback (including closures).
impl<F> EventSink for F
where
    F: Fn(Event),
{
    fn send(&self, e: Event) {
        (self)(e);
    }
}

/// Forward events into an Rc<RefCell<Vec<Event>>> buffer (useful for wasm UIs).
#[cfg(feature = "events")]
impl EventSink for std::rc::Rc<std::cell::RefCell<Vec<Event>>> {
    fn send(&self, e: Event) {
        if let Ok(mut v) = self.try_borrow_mut() {
            v.push(e);
        }
    }
}

/// Forward events into a crossbeam channel Sender (native-friendly).
#[cfg(feature = "events")]
impl EventSink for crossbeam::channel::Sender<Event> {
    fn send(&self, e: Event) {
        let _ = crossbeam::channel::Sender::send(self, e);
    }
}

/// Allow passing an immutable reference to a crossbeam Sender as a sink.
#[cfg(feature = "events")]
impl<'a> EventSink for &'a crossbeam::channel::Sender<Event> {
    fn send(&self, e: Event) {
        let _ = crossbeam::channel::Sender::send(*self, e);
    }
}
