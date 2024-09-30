use egui::Pos2;
use rand::Rng;

use crate::Graph;

use super::Layout;

pub struct Random {}

impl Layout for Random {
    fn next(&mut self, g: &mut Graph) {}
}

fn random_location(size: f32) -> Pos2 {
    let mut rng = rand::thread_rng();
    Pos2::new(rng.gen_range(0. ..size), rng.gen_range(0. ..size))
}
