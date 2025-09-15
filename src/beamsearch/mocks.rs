use super::beamsearch_collection::BeamsearchNode;

pub struct TestNode {
    pub dummy_fitness: f64,
    pub dummy_level: f64,
}

impl Default for TestNode {
    fn default() -> Self {
        Self {
            dummy_fitness: 0.0,
            dummy_level: 0.0,
        }
    }
}

impl BeamsearchNode for TestNode {
    fn fitness(&self) -> f64 {
        self.dummy_fitness
    }

    fn level(&self) -> f64 {
        self.dummy_level
    }
}
