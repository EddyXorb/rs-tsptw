use std::collections::HashMap;
use log::debug;

use super::beamsearch_solver::Node;
use rayon::prelude::*;

pub trait BeamsearchNode {
    fn fitness(&self) -> f64;
    fn level(&self) -> f64;
}

pub struct BeamsearchCollection<T>
where
    T: BeamsearchNode + Send + Sync,
{
    nodes: Vec<Node<T>>,
    sorted: bool,
}

impl<T> Default for BeamsearchCollection<T>
where
    T: BeamsearchNode + Send + Sync,
{
    fn default() -> Self {
        Self {
            nodes: Default::default(),
            sorted: true,
        }
    }
}

impl<T> BeamsearchCollection<T>
where
    T: BeamsearchNode + Send + Sync,
{
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Node<T>> {
        self.nodes.iter()
    }

    pub fn add(&mut self, node: Node<T>) {
        self.nodes.push(node);
        self.sorted = false;
    }

    fn inner_sort(to_sort: &mut Vec<Node<T>>) {
        to_sort.sort_by(|a, b| a.data().fitness().total_cmp(&b.data().fitness()));
    }

    pub fn sort(&mut self) {
        if self.sorted {
            return;
        }
        Self::inner_sort(&mut self.nodes);

        self.sorted = true;
    }

    pub fn keep_best(&mut self, target_size: usize) -> usize {
        if target_size >= self.len() {
            return 0;
        }

        self.sort();

        let deleted = self.len() - target_size;
        self.nodes.truncate(target_size);
        deleted
    }

    pub fn get_best(&self) -> Option<&Node<T>> {
        if self.sorted {
            return self.nodes.first();
        }
        self.nodes
            .iter()
            .min_by(|a, b| a.data().fitness().total_cmp(&b.data().fitness()))
    }

    pub fn remove_similars<S, SHash>(&mut self, is_similar: S, similarity_hash: SHash) -> usize
    where
        S: Fn(&Node<T>, &Node<T>) -> bool + Send + Sync,
        SHash: Fn(&Node<T>) -> u32,
    {
        let size_before = self.nodes.len();

        let old_nodes = std::mem::take(&mut self.nodes);
        let similarity_groups = Self::create_similarity_groups_from(old_nodes, similarity_hash);

        self.nodes = similarity_groups
            .into_par_iter()
            .map(|(_, group)| Self::remove_similars_for(group, &is_similar))
            .flatten()
            .collect();

        size_before - self.nodes.len()
    }

    fn remove_similars_for<S>(mut group: Vec<Node<T>>, is_similar: S) -> Vec<Node<T>>
    where
        S: Fn(&Node<T>, &Node<T>) -> bool,
    {
        Self::inner_sort(&mut group);

        let mut keep_mask = vec![true; group.len()];

        for outer_i in (0..(group.len())).rev() {
            if !keep_mask[outer_i] {
                continue;
            }
            for inner_i in (0..(outer_i)).rev() {
                if !keep_mask[inner_i] {
                    continue;
                }

                let o = &group[outer_i];
                let i = &group[inner_i];

                if is_similar(o, i) {
                    keep_mask[outer_i] = false;
                }
            }
        }

        for i in (0..group.len()).rev() {
            if !keep_mask[i] {
                group.swap_remove(i);
            }
        }
        group
    }

    fn create_similarity_groups_from<SHash>(
        nodes: Vec<Node<T>>,
        similarity_hash: SHash,
    ) -> HashMap<u32, Vec<Node<T>>>
    where
        T: BeamsearchNode + Send + Sync,
        SHash: Fn(&Node<T>) -> u32,
    {
        let mut similarity_groups = HashMap::<u32, Vec<Node<T>>>::new();
        for node in nodes {
            let hash = similarity_hash(&node);
            similarity_groups.entry(hash).or_default().push(node);
        }
        similarity_groups
    }
}

impl<T> IntoIterator for BeamsearchCollection<T>
where
    T: BeamsearchNode + Send + Sync,
{
    type Item = Node<T>;
    type IntoIter = std::vec::IntoIter<Node<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a BeamsearchCollection<T>
where
    T: BeamsearchNode + Send + Sync,
{
    type Item = &'a Node<T>;
    type IntoIter = std::slice::Iter<'a, Node<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.iter()
    }
}

impl<T> IntoParallelIterator for BeamsearchCollection<T>
where
    T: BeamsearchNode + Send + Sync,
{
    type Iter = rayon::vec::IntoIter<Node<T>>;
    type Item = Node<T>;

    fn into_par_iter(self) -> Self::Iter {
        self.nodes.into_par_iter()
    }
}

impl<'a, T> IntoParallelIterator for &'a BeamsearchCollection<T>
where
    T: BeamsearchNode + Send + Sync,
{
    type Iter = rayon::slice::Iter<'a, Node<T>>;
    type Item = &'a Node<T>;

    fn into_par_iter(self) -> Self::Iter {
        self.nodes.par_iter()
    }
}

#[cfg(test)]
mod tests {

    use std::iter::zip;

    use super::super::mocks::TestNode;
    use super::*;
    use rand::{Rng, SeedableRng, rngs::StdRng};

    fn create_test_collection(size: usize) -> BeamsearchCollection<TestNode> {
        let mut rng = StdRng::seed_from_u64(42);

        let root = Node::new_root(TestNode {
            dummy_fitness: 0.0,
            dummy_level: 0.0,
        });
        let mut coll = BeamsearchCollection::<TestNode>::default();

        for _ in 0..size {
            coll.add(root.new_child(TestNode {
                dummy_fitness: rng.random_range(0.0..100.0),
                dummy_level: rng.random_range(0.0..100.0),
            }));
        }

        coll
    }

    #[test]
    fn test_into_iter() {
        let coll = create_test_collection(1);

        for _node in &coll {}
    }

    #[test]
    fn test_iter() {
        let coll = create_test_collection(1);

        for _node in coll.iter() {}
    }

    #[test]
    fn test_len() {
        for i in 0..4 {
            let coll = create_test_collection(i);

            assert_eq!(coll.len(), i)
        }
    }

    #[test]
    fn test_sort() {
        let mut coll = create_test_collection(10);

        coll.sort();

        let mut last_fitness = 0.0; // works because we generate only fitnesses between 0..100
        for node in &coll {
            assert!(node.data().fitness() >= last_fitness);
            last_fitness = node.data().fitness();
            debug!("{}", last_fitness)
        }
    }

    #[test]
    fn test_keep_best() {
        let mut coll = create_test_collection(10);

        let mut fitnesses: Vec<f64> = vec![];

        for node in &coll {
            fitnesses.push(node.data().fitness());
        }

        fitnesses.sort_by(|a, b| a.total_cmp(b));
        fitnesses.truncate(5);

        coll.keep_best(5);

        for (node, expected_fitness) in zip(&coll, fitnesses) {
            assert_eq!(node.data().fitness(), expected_fitness);
        }
    }

    #[test]
    fn test_get_best() {
        let coll = create_test_collection(10);

        let best = coll.get_best().unwrap();

        for node in &coll {
            assert!(best.data().fitness() <= node.data().fitness());
        }
    }

    #[test]
    fn test_remove_similars_removes_correct_number() {
        let mut coll = BeamsearchCollection::<TestNode>::default();

        let root = Node::new_root(TestNode {
            dummy_fitness: 0.0,
            dummy_level: 0.0,
        });

        for _ in 0..10 {
            coll.add(root.new_child(TestNode::default()));
        }

        assert_eq!(coll.len(), 10);

        let removed = coll.remove_similars(|a, b| a.data() == b.data(), |_| 0);

        assert_eq!(coll.len(), 1);
        assert_eq!(removed, 9);
    }

    #[test]
    fn test_remove_similars_removes_only_worse() {
        let mut coll_rising_fitness = BeamsearchCollection::<TestNode>::default();
        let mut coll_decreasing_fitness = BeamsearchCollection::<TestNode>::default();

        let root = Node::new_root(TestNode {
            dummy_fitness: 0.0,
            dummy_level: 0.0,
        });

        for i in 0..10 {
            let mut child = TestNode::default();
            child.dummy_fitness = i as f64;
            coll_rising_fitness.add(root.new_child(child));

            let mut child = TestNode::default();
            child.dummy_fitness = -i as f64;
            coll_decreasing_fitness.add(root.new_child(child));
        }

        assert_eq!(coll_rising_fitness.len(), 10);
        assert_eq!(coll_decreasing_fitness.len(), 10);

        //all are equal because all share the same level
        coll_rising_fitness.remove_similars(
            |a, b| a.data().level() == b.data().level(),
            |n| n.data().level() as u32,
        );
        coll_decreasing_fitness.remove_similars(
            |a, b| a.data().level() == b.data().level(),
            |n| n.data().level() as u32,
        );

        assert_eq!(coll_rising_fitness.len(), 1);
        assert_eq!(coll_decreasing_fitness.len(), 1);

        assert_eq!(coll_rising_fitness.nodes[0].data().fitness(), 0.0);
        assert_eq!(coll_decreasing_fitness.nodes[0].data().fitness(), -9.0);
    }
}
