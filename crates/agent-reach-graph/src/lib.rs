//! Pure-Rust Epistemic Semantic Graph and Concept Bridge
//!
//! Provides a 100% pure-Rust in-memory graph store for concept connections,
//! multi-dimensional epistemic vector calculations, and language alignment
//! without C-FFI or external C compiler dependencies.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Epistemic vector representation across 4 foundational dimensions + 1 semantic affinity axis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpistemicVector {
    /// x-Axis: Semantic Affinity / Antonymy [-1.0 to +1.0]
    pub x_anlamsal: f32,
    /// y-Axis: Ontological dimension (Ease / Difficulty of connection)
    pub y_ontolojik: f32,
    /// z-Axis: Aesthetic dimension (Form / Elegance)
    pub z_estetik: f32,
    /// w-Axis: Epistemological dimension (Truth / Logic)
    pub w_epistemolojik: f32,
    /// v-Axis: Moral dimension (Ethics / Value)
    pub v_ahlaki: f32,
}

impl Default for EpistemicVector {
    fn default() -> Self {
        Self {
            x_anlamsal: 0.0,
            y_ontolojik: 0.0,
            z_estetik: 0.0,
            w_epistemolojik: 0.0,
            v_ahlaki: 0.0,
        }
    }
}

impl EpistemicVector {
    /// Calculate weighted 5-dimensional distance between two vectors.
    pub fn distance(&self, other: &Self) -> f32 {
        let dx = self.x_anlamsal - other.x_anlamsal;
        let dy = self.y_ontolojik - other.y_ontolojik;
        let dz = self.z_estetik - other.z_estetik;
        let dw = self.w_epistemolojik - other.w_epistemolojik;
        let dv = self.v_ahlaki - other.v_ahlaki;

        (dx * dx + dy * dy + dz * dz + dw * dw + dv * dv).sqrt()
    }
}

/// Node representing a concept or term in a specific language.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptNode {
    pub id: String,
    pub term: String,
    pub language: String,
    pub category: String,
}

/// Weighted edge connecting two concept nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptEdge {
    pub source_id: String,
    pub target_id: String,
    pub edge_type: String,
    pub vector: EpistemicVector,
    pub weight: f32,
    pub usage_count: u64,
    pub last_used_timestamp: i64,
}

/// Pure-Rust in-memory Epistemic Graph engine.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct EpistemicGraph {
    nodes: HashMap<String, ConceptNode>,
    edges: HashMap<(String, String), ConceptEdge>,
}

impl EpistemicGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a node to the graph.
    pub fn add_node(&mut self, id: &str, term: &str, language: &str, category: &str) {
        self.nodes.insert(
            id.to_string(),
            ConceptNode {
                id: id.to_string(),
                term: term.to_string(),
                language: language.to_string(),
                category: category.to_string(),
            },
        );
    }

    /// Add or update a weighted edge between two nodes.
    pub fn add_edge(
        &mut self,
        source_id: &str,
        target_id: &str,
        edge_type: &str,
        vector: EpistemicVector,
        weight: f32,
    ) {
        let key = (source_id.to_string(), target_id.to_string());
        let now = chrono::Utc::now().timestamp();
        self.edges.insert(
            key,
            ConceptEdge {
                source_id: source_id.to_string(),
                target_id: target_id.to_string(),
                edge_type: edge_type.to_string(),
                vector,
                weight: weight.clamp(0.0, 1.0),
                usage_count: 1,
                last_used_timestamp: now,
            },
        );
    }

    /// Reinforce edge weight upon successful search match (+0.05).
    pub fn reinforce_edge(&mut self, source_id: &str, target_id: &str) {
        let key = (source_id.to_string(), target_id.to_string());
        if let Some(edge) = self.edges.get_mut(&key) {
            edge.weight = (edge.weight + 0.05).min(1.0);
            edge.usage_count += 1;
            edge.last_used_timestamp = chrono::Utc::now().timestamp();
        }
    }

    /// Apply exponential decay over time ($w(t) = w_0 * e^{-\lambda * \Delta t}$).
    pub fn apply_decay(&mut self, lambda: f32, days_passed: f32) {
        for edge in self.edges.values_mut() {
            let decay_factor = (-lambda * days_passed).exp();
            edge.weight = (edge.weight * decay_factor).max(0.0);
        }
    }

    /// Retrieve connected target terms with weight above threshold.
    pub fn get_connected_terms(&self, source_id: &str, min_weight: f32) -> Vec<String> {
        let mut results = Vec::new();
        for ((src, tgt), edge) in &self.edges {
            if src == source_id && edge.weight >= min_weight {
                if let Some(node) = self.nodes.get(tgt) {
                    results.push(node.term.clone());
                }
            }
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epistemic_vector_distance() {
        let v1 = EpistemicVector {
            x_anlamsal: 1.0,
            y_ontolojik: 0.0,
            z_estetik: 0.0,
            w_epistemolojik: 0.0,
            v_ahlaki: 0.0,
        };
        let v2 = EpistemicVector {
            x_anlamsal: -1.0,
            y_ontolojik: 0.0,
            z_estetik: 0.0,
            w_epistemolojik: 0.0,
            v_ahlaki: 0.0,
        };
        assert_eq!(v1.distance(&v2), 2.0);
    }

    #[test]
    fn test_graph_reinforce_and_decay() {
        let mut graph = EpistemicGraph::new();
        graph.add_node("n1", "orm", "ENG", "tech");
        graph.add_node("n2", "database mapper", "ENG", "tech");
        graph.add_edge("n1", "n2", "SYNONYM", EpistemicVector::default(), 0.5);

        graph.reinforce_edge("n1", "n2");
        let terms = graph.get_connected_terms("n1", 0.5);
        assert_eq!(terms, vec!["database mapper"]);

        graph.apply_decay(0.1, 10.0);
        let terms_after_decay = graph.get_connected_terms("n1", 0.5);
        assert!(terms_after_decay.is_empty());
    }
}
