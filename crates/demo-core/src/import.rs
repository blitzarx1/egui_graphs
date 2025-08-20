use egui_graphs::Graph;
use petgraph::{stable_graph::DefaultIx, Directed, Undirected};

/// Abstraction for importing a graph from text.
pub trait GraphImport {
    /// Import a graph from the given text, returning a typed graph
    /// that reflects directedness from the source.
    fn import(&self, text: &str) -> Result<ImportResult, String>;
}

/// Default importer: accepts two minimal JSON forms:
/// 1) Edges-only array of pairs: [[a,b],[c,d],...]
/// 2) Object: {"nodes":[id...], "edges":[[a,b], ...]}
pub struct JsonMinimalImport;

impl GraphImport for JsonMinimalImport {
    fn import(&self, text: &str) -> Result<ImportResult, String> {
        import_json_minimal(text)
    }
}

#[derive(serde::Deserialize)]
struct JsonGraphMinimal {
    #[serde(default)]
    nodes: Vec<i64>,
    #[serde(default)]
    edges: Vec<(i64, i64)>,
    #[serde(default)]
    directed: Option<bool>,
}

/// Public entry point used by the demo app.
pub fn import_graph_from_str(text: &str) -> Result<ImportResult, String> {
    JsonMinimalImport.import(text)
}

/// Result of an import operation.
#[derive(Debug, Clone)]
pub enum ImportedGraph {
    Directed(Graph<(), (), Directed, DefaultIx>),
    Undirected(Graph<(), (), Undirected, DefaultIx>),
}

#[derive(Debug, Clone)]
pub struct ImportResult {
    pub g: ImportedGraph,
}

fn import_json_minimal(text: &str) -> Result<ImportResult, String> {
    // Parse JSON value first to support either array or object forms.
    let v: serde_json::Value =
        serde_json::from_str(text).map_err(|e| format!("invalid json: {e}"))?;

    let parsed: JsonGraphMinimal = if v.is_array() {
        let edges_arr = v.as_array().unwrap();
        let mut edges: Vec<(i64, i64)> = Vec::with_capacity(edges_arr.len());
        let mut nodes_set: std::collections::BTreeSet<i64> = std::collections::BTreeSet::new();
        for e in edges_arr {
            let pair = e
                .as_array()
                .ok_or_else(|| "edge must be [a,b]".to_string())?;
            if pair.len() != 2 {
                return Err("edge must have 2 items".into());
            }
            let a = pair[0]
                .as_i64()
                .ok_or_else(|| "edge endpoints must be integers".to_string())?;
            let b = pair[1]
                .as_i64()
                .ok_or_else(|| "edge endpoints must be integers".to_string())?;
            edges.push((a, b));
            nodes_set.insert(a);
            nodes_set.insert(b);
        }
        JsonGraphMinimal {
            nodes: nodes_set.into_iter().collect(),
            edges,
            directed: Some(true),
        }
    } else {
        serde_json::from_value(v).map_err(|e| format!("schema error: {e}"))?
    };

    // Build a new empty graph and add nodes/edges
    let directed = parsed.directed.unwrap_or(true);
    if directed {
        // Build Directed graph
        use std::collections::HashMap;
        let sg: petgraph::stable_graph::StableGraph<(), (), Directed, DefaultIx> =
            petgraph::stable_graph::StableGraph::default();
        let mut g: Graph<(), (), Directed, DefaultIx> = Graph::from(&sg);
        let mut id_to_idx: HashMap<i64, petgraph::stable_graph::NodeIndex<DefaultIx>> =
            HashMap::new();
        for id in parsed.nodes.iter() {
            let idx = g.add_node(());
            id_to_idx.insert(*id, idx);
        }
        for (a, b) in parsed.edges.into_iter() {
            let ai = match id_to_idx.get(&a) {
                Some(i) => *i,
                None => {
                    let i = g.add_node(());
                    id_to_idx.insert(a, i);
                    i
                }
            };
            let bi = match id_to_idx.get(&b) {
                Some(i) => *i,
                None => {
                    let i = g.add_node(());
                    id_to_idx.insert(b, i);
                    i
                }
            };
            if ai != bi {
                let _ = g.add_edge(ai, bi, ());
            }
        }
        Ok(ImportResult {
            g: ImportedGraph::Directed(g),
        })
    } else {
        // Build Undirected graph (no deduplication; allow parallel edges)
        use std::collections::HashMap;
        let sg: petgraph::stable_graph::StableGraph<(), (), Undirected, DefaultIx> =
            petgraph::stable_graph::StableGraph::default();
        let mut g: Graph<(), (), Undirected, DefaultIx> = Graph::from(&sg);
        let mut id_to_idx: HashMap<i64, petgraph::stable_graph::NodeIndex<DefaultIx>> =
            HashMap::new();
        for id in parsed.nodes.iter() {
            let idx = g.add_node(());
            id_to_idx.insert(*id, idx);
        }
        for (a, b) in parsed.edges.into_iter() {
            let ai = match id_to_idx.get(&a) {
                Some(i) => *i,
                None => {
                    let i = g.add_node(());
                    id_to_idx.insert(a, i);
                    i
                }
            };
            let bi = match id_to_idx.get(&b) {
                Some(i) => *i,
                None => {
                    let i = g.add_node(());
                    id_to_idx.insert(b, i);
                    i
                }
            };
            if ai != bi {
                let _ = g.add_edge(ai, bi, ());
            }
        }
        Ok(ImportResult {
            g: ImportedGraph::Undirected(g),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn import_edges_array_valid() {
        let s = "[[0,1],[1,2],[2,0]]";
        let r = import_graph_from_str(s).expect("should import");
        match r.g {
            ImportedGraph::Directed(g) => {
                assert_eq!(g.node_count(), 3);
                assert_eq!(g.edge_count(), 3);
            }
            _ => panic!("expected directed graph"),
        }
    }

    #[test]
    fn import_object_valid() {
        let s = r#"{"nodes":[10,11],"edges":[[10,11]]}"#;
        let r = import_graph_from_str(s).expect("should import");
        match r.g {
            ImportedGraph::Directed(g) => {
                assert_eq!(g.node_count(), 2);
                assert_eq!(g.edge_count(), 1);
            }
            _ => panic!("expected directed graph"),
        }
    }

    #[test]
    fn import_invalid_json() {
        let err = import_graph_from_str("not-json").unwrap_err();
        assert!(err.contains("invalid json"));
    }

    #[test]
    fn import_non_int_endpoint() {
        let err = import_graph_from_str("[[\"a\",1]]").unwrap_err();
        assert!(err.contains("integers"));
    }

    #[test]
    fn import_object_undirected() {
        let s = r#"{"nodes":[0,1,2],"edges":[[0,1],[1,2],[2,0]],"directed":false}"#;
        let r = import_graph_from_str(s).expect("should import");
        match r.g {
            ImportedGraph::Undirected(g) => {
                assert_eq!(g.node_count(), 3);
                // Three undirected edges
                assert_eq!(g.edge_count(), 3);
            }
            _ => panic!("expected undirected graph"),
        }
    }

    #[test]
    fn import_undirected_deduplicates_pairs() {
        let s = r#"{"edges":[[0,1],[1,0],[0,1],[2,2]],"directed":false}"#;
        let r = import_graph_from_str(s).expect("should import");
        match r.g {
            ImportedGraph::Undirected(g) => {
                // Duplicates are preserved and self-loops are ignored
                assert_eq!(g.node_count(), 3);
                assert_eq!(g.edge_count(), 3);
            }
            _ => panic!("expected undirected graph"),
        }
    }
}
