use crate::Graph;

struct Hierarchical<'a, N: Clone, E: Clone> {
    g: &'a mut Graph<N, E>,
}

impl<'a, N: Clone, E: Clone> Hierarchical<'a, N, E> {
    pub fn new(g: &'a mut Graph<N, E>) -> Hierarchical<'a, N, E> {
        Self { g }
    }

    pub fn apply(&mut self) {
        todo!()
    }
}
