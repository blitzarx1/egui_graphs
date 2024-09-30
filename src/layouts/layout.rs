use crate::Graph;

pub trait Layout {
    fn next(&mut self, g: &mut Graph);
}
