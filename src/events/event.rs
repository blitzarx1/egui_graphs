use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PayloadPan {
    pub diff: [f32; 2],
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PyaloadZoom {
    pub diff: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Event {
    Pan(PayloadPan),
    Zoom(PyaloadZoom),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_contract_pan() {
        let event = Event::Pan(PayloadPan { diff: [1.0, 2.0] });
        let json = serde_json::to_string(&event).unwrap();
        assert_eq!(json, r#"{"Pan":{"diff":[1.0,2.0]}}"#);

        let event: Event = serde_json::from_str(&json).unwrap();
        assert_eq!(event, Event::Pan(PayloadPan { diff: [1.0, 2.0] }));
    }

    #[test]
    fn test_contract_zoom() {
        let event = Event::Zoom(PyaloadZoom { diff: 1.0 });
        let json = serde_json::to_string(&event).unwrap();
        assert_eq!(json, r#"{"Zoom":{"diff":1.0}}"#);

        let event: Event = serde_json::from_str(&json).unwrap();
        assert_eq!(event, Event::Zoom(PyaloadZoom { diff: 1.0 }));
    }
}
