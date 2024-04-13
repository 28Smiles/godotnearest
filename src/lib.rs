use godot::prelude::*;
use kdtree::KdTree;
use regex_automata::Anchored;
use regex_automata::meta::Regex;

struct GodotNearest;

#[gdextension]
unsafe impl ExtensionLibrary for GodotNearest {}

#[derive(GodotClass)]
#[class(base=Node2D)]
struct Nearest2D {
    #[export]
    groups: Array<GString>,

    groups_regex: Regex,
    tree: Vec<KdTree<f32, NodePath, [f32; 2]>>,

    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for Nearest2D {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            groups: Array::new(),
            groups_regex: Regex::new_many(AsRef::<[&str]>::as_ref(&[])).unwrap(),
            base,
            tree: Vec::new(),
        }
    }

    fn ready(&mut self) {
        let call = self.base().callable("child_entered_tree");
        self.base_mut().connect(
            StringName::from("child_entered_tree"),
            call,
        );
        let call = self.base().callable("child_exiting_tree");
        self.base_mut().connect(
            StringName::from("child_exiting_tree"),
            call,
        );
        self.index();
    }

    fn to_string(&self) -> GString {
        format!(
            "Nearest2D({{{}}})",
            self.tree.iter().zip(self.groups.iter_shared()).map(|(tree, group)| {
                format!(
                    "\"{}\": [{}]",
                    group.to_string(),
                    tree.iter_nearest(&[0.0, 0.0], &kdtree::distance::squared_euclidean).unwrap()
                        .map(|(_, path)| path.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }).collect::<Vec<_>>().join(", ")
        ).into()
    }

    fn set_property(&mut self, property: StringName, value: Variant) -> bool {
        if &property.to_string() == "groups" {
            self.groups.set_property(value.to());
            self.groups_regex = match Regex::new_many(
                &self.groups.iter_shared().map(|group| group.to_string()).collect::<Vec<_>>()
            ) {
                Ok(regex) => regex,
                Err(e) => {
                    godot_error!("Failed to compile regex: {:?}", e);
                    Regex::new_many(AsRef::<[&str]>::as_ref(&[])).unwrap()
                }
            };
            self.tree = (0..self.groups.len()).into_iter().map(|_| KdTree::new(2)).collect();
            self.index();
            true
        } else {
            false
        }
    }
}

#[godot_api]
impl Nearest2D {
    fn index(&mut self) {
        for child in self.base().get_children().iter_shared() {
            self.child_entered_tree(child);
        }
    }

    #[func]
    fn child_entered_tree(&mut self, child: Gd<Node>) {
        let child = match child.try_cast::<Node2D>() {
            Ok(child) => child,
            Err(_) => return,
        };
        // Add child to the tree
        let child_name = child.get_name().to_string();
        for m in self.groups_regex.find_iter(regex_automata::Input::new(&child_name).anchored(Anchored::Yes)) {
            let idx = m.pattern();
            let tree = self.tree.get_mut(idx.as_usize());
            let tree = match tree {
                Some(tree) => tree,
                None => {
                    godot_error!("Invalid group index: {}", idx.as_usize());
                    continue;
                }
            };
            let pos = child.get_global_position();
            tree.add([pos.x, pos.y], child.get_path()).unwrap_or(());
        }
    }

    #[func]
    fn child_exiting_tree(&mut self, child: Gd<Node>) {
        let child = match child.try_cast::<Node2D>() {
            Ok(child) => child,
            Err(_) => return,
        };
        // Remove child from the tree
        let child_name = child.get_name().to_string();
        for m in self.groups_regex.find_iter(&child_name) {
            let idx = m.pattern();
            let tree = self.tree.get_mut(idx.as_usize());
            let tree = match tree {
                Some(tree) => tree,
                None => {
                    godot_error!("Invalid group index: {}", idx.as_usize());
                    continue;
                }
            };
            let pos = child.get_global_position();
            tree.remove(&[pos.x, pos.y], &child.get_path()).unwrap_or(0);
        }
    }

    #[func]
    fn nearest(&self, pos: Vector2, group_idx: i64) -> Option<Gd<Node2D>> {
        let tree = self.tree.get(group_idx as usize);
        let tree = match tree {
            Some(tree) => tree,
            None => {
                godot_error!("Invalid group index: {}", group_idx);
                return None;
            }
        };
        match tree.iter_nearest(
            &[pos.x, pos.y],
            &kdtree::distance::squared_euclidean,
        ).map(|mut i| i.next()) {
            Ok(Some((_, path))) => {
                self.base().get_node(path.clone()).map(|node| node.cast())
            }
            _ => None,
        }
    }

    #[func]
    fn nearest_array(&self, pos: Vector2, group_idx: i64, length: i64) -> Array<Gd<Node2D>> {
        let tree = self.tree.get(group_idx as usize);
        let tree = match tree {
            Some(tree) => tree,
            None => {
                godot_error!("Invalid group index: {}", group_idx);
                return Array::new();
            }
        };
        let pos = [pos.x, pos.y];
        let iterator = match tree.iter_nearest(
            &pos,
            &kdtree::distance::squared_euclidean,
        ) {
            Ok(iter) => iter,
            _ => return Array::new(),
        };
        let length = if length < 0 {
            tree.size()
        } else {
            length as usize
        };
        let iterator = iterator.take(length);

        iterator.map(|(_, path)| {
            self.base().get_node(path.clone())
                .map(|node| node.cast())
        }).flatten().collect()
    }
}
