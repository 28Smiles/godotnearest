use std::fmt::{Debug, Display};
use godot::prelude::*;
use kdtree::KdTree;
use regex_automata::meta::Regex;
use num_traits::{Float, One, Zero};

pub struct Nearest<const DIMS: usize, A: PartialEq> {
    groups_regex: Regex,
    tree: Vec<KdTree<A, NodePath, [A; DIMS]>>,
}

impl<const DIMS: usize, A: PartialEq + Float + Zero + One> Nearest<DIMS, A> {
    pub fn new() -> Self {
        Self {
            groups_regex: Regex::new_many(AsRef::<[&str]>::as_ref(&[])).unwrap(),
            tree: Vec::new(),
        }
    }

    pub fn new_from(mut groups: Vec<&str>) -> Self {
        let groups_regex = match Regex::new_many(&groups) {
            Ok(groups_regex) => groups_regex,
            Err(e) => {
                godot_error!("Nearest Lib-Regex Error: {:?}", e);
                if let Some(idx) = e.pattern() {
                    groups[idx] = "[a&&b]"; // Match nothing
                    return Self::new_from(groups)
                } else {
                    return Self::new()
                }
            },
        };

        Self {
            groups_regex,
            tree: (0..groups.len()).into_iter().map(|_| KdTree::new(DIMS)).collect(),
        }
    }

    pub fn add(&mut self, name: &str, pos: [A; DIMS], node_path: NodePath) {
        for idx in self.groups_regex.find_iter(name).map(|m| m.pattern().as_usize()) {
            if let Some(tree) = self.tree.get_mut(idx) {
                tree.add(pos.clone(), node_path.clone()).unwrap_or(());
            }
        }
    }

    pub fn remove(&mut self, name: &str, pos: &[A; DIMS], node_path: &NodePath) {
        for idx in self.groups_regex.find_iter(name).map(|m| m.pattern().as_usize()) {
            if let Some(tree) = self.tree.get_mut(idx) {
                tree.remove(pos, node_path).unwrap_or(0);
            }
        }
    }

    pub fn nearest<'a>(&'a self, pos: &'a [A; DIMS], group_idx: usize) -> Option<impl Iterator<Item=(A, &NodePath)> + 'a > {
        self.tree.get(group_idx)
            .map(|tree| tree.iter_nearest(pos, &kdtree::distance::squared_euclidean).unwrap())
    }
}

impl<const DIMS: usize, A: PartialEq + Debug> Display for Nearest<DIMS, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Nearest({:?}, {:?})",
            self.groups_regex,
            self.tree,
        )
    }
}