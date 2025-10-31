use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use egui::{Pos2, Rect};
use egui_graphs::{to_graph, DefaultEdgeShape, DefaultNodeShape, Graph};
use petgraph::stable_graph::StableGraph;
use std::hint::black_box;
use std::time::Duration;

use egui_graphs::ForceAlgorithm;
use egui_graphs::FruchtermanReingold;
use egui_graphs::FruchtermanReingoldState;

fn make_graph(
    num_nodes: usize,
    num_edges: usize,
) -> Graph<
    (),
    (),
    petgraph::Directed,
    petgraph::stable_graph::DefaultIx,
    DefaultNodeShape,
    DefaultEdgeShape,
> {
    let mut g: StableGraph<(), ()> = StableGraph::default();
    for _ in 0..num_nodes {
        g.add_node(());
    }
    // add a simple chain for determinism
    for i in 1..num_nodes {
        g.add_edge(
            petgraph::prelude::NodeIndex::new(i - 1),
            petgraph::prelude::NodeIndex::new(i),
            (),
        );
    }
    // sprinkle some extra edges up to num_edges
    let mut extra = num_edges.saturating_sub(num_nodes.saturating_sub(1));
    let mut i = 0usize;
    while extra > 0 && num_nodes >= 2 {
        let a = i % num_nodes;
        let b = (i * 37 + 11) % num_nodes;
        if a != b {
            g.add_edge(
                petgraph::prelude::NodeIndex::new(a),
                petgraph::prelude::NodeIndex::new(b),
                (),
            );
            extra -= 1;
        }
        i += 1;
    }

    let mut graph = to_graph(&g);
    // spread nodes along a line to start
    let idxs: Vec<_> = graph.g().node_indices().collect();
    for (i, idx) in idxs.into_iter().enumerate() {
        graph
            .g_mut()
            .node_weight_mut(idx)
            .unwrap()
            .set_location(Pos2::new(i as f32 * 5.0, 0.0));
    }
    graph
}

fn bench_fr_step(c: &mut Criterion) {
    let view = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1200.0, 800.0));
    let mut group = c.benchmark_group("fr_steps_adaptive");
    group.sample_size(10);
    group.measurement_time(Duration::from_millis(600));
    group.warm_up_time(Duration::from_millis(200));

    group.bench_function("n500_m1000_steps100", |b| {
        b.iter_batched(
            || {
                let g = make_graph(500, 1000);
                let alg = FruchtermanReingold::from_state(FruchtermanReingoldState::default());
                (g, alg)
            },
            |(mut g, mut alg)| {
                for _ in 0..100 {
                    alg.step(&mut g, view);
                }
                black_box(g);
                black_box(alg);
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("n10000_m20000_steps1", |b| {
        b.iter_batched(
            || {
                let g = make_graph(10000, 20000);
                let alg = FruchtermanReingold::from_state(FruchtermanReingoldState::default());
                (g, alg)
            },
            |(mut g, mut alg)| {
                alg.step(&mut g, view);
                black_box(g);
                black_box(alg);
            },
            BatchSize::SmallInput,
        );
    });
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().configure_from_args();
    targets = bench_fr_step
}
criterion_main!(benches);
