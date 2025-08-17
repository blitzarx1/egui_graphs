use egui::{Rect, Vec2};
use petgraph::EdgeType;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{DisplayEdge, DisplayNode, Graph};

/// An additional force to be applied after the base forces.
/// Implementors are zero-sized marker types with the behavior in `apply`.
pub trait ExtraForce: std::fmt::Debug + Default + Send + Sync + 'static {
    type Params: Clone + Default + std::fmt::Debug + Send + Sync + 'static;

    /// Apply the extra force: accumulate into `disp` (same convention as base helpers).
    fn apply<N, E, Ty, Ix, Dn, De>(
        params: &Self::Params,
        g: &Graph<N, E, Ty, Ix, Dn, De>,
        indices: &[petgraph::stable_graph::NodeIndex<Ix>],
        disp: &mut [Vec2],
        area: Rect,
        k: f32,
    ) where
        N: Clone,
        E: Clone,
        Ty: EdgeType,
        Ix: petgraph::csr::IndexType,
        Dn: DisplayNode<N, E, Ty, Ix>,
        De: DisplayEdge<N, E, Ty, Ix, Dn>;
}

/// A configured instance of an extra force (on/off + parameters).
#[derive(Serialize, Deserialize)]
#[serde(bound(
    serialize = "E::Params: Serialize",
    deserialize = "E::Params: Deserialize<'de>"
))]
pub struct Extra<E: ExtraForce, const ENABLED_DEFAULT: bool> {
    pub enabled: bool,
    pub params: E::Params,
}

impl<E: ExtraForce, const ENABLED_DEFAULT: bool> Extra<E, ENABLED_DEFAULT> {
    pub fn new(params: E::Params) -> Self {
        Self {
            enabled: true,
            params,
        }
    }
}

impl<E: ExtraForce, const ENABLED_DEFAULT: bool> Default for Extra<E, ENABLED_DEFAULT> {
    fn default() -> Self {
        Self {
            enabled: ENABLED_DEFAULT,
            params: E::Params::default(),
        }
    }
}

impl<E: ExtraForce, const ENABLED_DEFAULT: bool> Clone for Extra<E, ENABLED_DEFAULT> {
    fn clone(&self) -> Self {
        Self {
            enabled: self.enabled,
            params: self.params.clone(),
        }
    }
}

impl<E: ExtraForce, const ENABLED_DEFAULT: bool> std::fmt::Debug for Extra<E, ENABLED_DEFAULT> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Extra")
            .field("enabled", &self.enabled)
            .field("params", &self.params)
            .finish()
    }
}

/// Trait to apply a heterogeneous tuple of extras.
pub trait ExtrasTuple:
    Serialize + DeserializeOwned + Clone + Default + std::fmt::Debug + Send + Sync + 'static
{
    fn apply_all<N, EE, Ty, Ix, Dn, De>(
        &self,
        g: &Graph<N, EE, Ty, Ix, Dn, De>,
        indices: &[petgraph::stable_graph::NodeIndex<Ix>],
        disp: &mut [Vec2],
        area: Rect,
        k: f32,
    ) where
        N: Clone,
        EE: Clone,
        Ty: EdgeType,
        Ix: petgraph::csr::IndexType,
        Dn: DisplayNode<N, EE, Ty, Ix>,
        De: DisplayEdge<N, EE, Ty, Ix, Dn>;
}

impl ExtrasTuple for () {
    fn apply_all<N, EE, Ty, Ix, Dn, De>(
        &self,
        _g: &Graph<N, EE, Ty, Ix, Dn, De>,
        _indices: &[petgraph::stable_graph::NodeIndex<Ix>],
        _disp: &mut [Vec2],
        _area: Rect,
        _k: f32,
    ) where
        N: Clone,
        EE: Clone,
        Ty: EdgeType,
        Ix: petgraph::csr::IndexType,
        Dn: DisplayNode<N, EE, Ty, Ix>,
        De: DisplayEdge<N, EE, Ty, Ix, Dn>,
    {
    }
}

impl<Head, const B: bool, Tail> ExtrasTuple for (Extra<Head, B>, Tail)
where
    Head: ExtraForce,
    Head::Params: Serialize + DeserializeOwned,
    Tail: ExtrasTuple,
{
    fn apply_all<N, EE, Ty, Ix, Dn, De>(
        &self,
        g: &Graph<N, EE, Ty, Ix, Dn, De>,
        indices: &[petgraph::stable_graph::NodeIndex<Ix>],
        disp: &mut [Vec2],
        area: Rect,
        k: f32,
    ) where
        N: Clone,
        EE: Clone,
        Ty: EdgeType,
        Ix: petgraph::csr::IndexType,
        Dn: DisplayNode<N, EE, Ty, Ix>,
        De: DisplayEdge<N, EE, Ty, Ix, Dn>,
    {
        let (head, tail) = self;
        if head.enabled {
            Head::apply(&head.params, g, indices, disp, area, k);
        }
        tail.apply_all(g, indices, disp, area, k);
    }
}
