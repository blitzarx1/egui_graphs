use egui::Pos2;
use petgraph::stable_graph::{IndexType, NodeIndex};
use rand::Rng;

use crate::Graph;

use super::Layout;

const DEFAULT_SPAWN_SIZE: f32 = 250.;

pub struct Random {}

impl Layout for Random {
    fn next(&mut self, g: &mut Graph) {
        todo!()
    }

    fn next_for_node<Ix: IndexType>(&mut self, g: &mut Graph, idx: NodeIndex<Ix>) {
        todo!()
    }
}

fn random_location(size: f32) -> Pos2 {
    let mut rng = rand::thread_rng();
    Pos2::new(rng.gen_range(0. ..size), rng.gen_range(0. ..size))
}
