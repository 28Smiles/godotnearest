use godot::prelude::*;
use kdtree::KdTree;
use regex::Regex;

struct GodotNearest;

#[derive(GodotClass)]
#[class(base=Node2D)]
struct Nearest2D {
    #[export]
    groups: Array<GString>,
    regex_cache: Vec<Regex>,
    base: Base<Node2D>,
    tree: Vec<KdTree<f32, NodePath, [f32; 2]>>,
}

#[godot_api]
impl INode2D for Nearest2D {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            groups: Array::new(),
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
    }

    fn to_string(&self) -> GString {
        format!(
            "Nearest2D({{{}}})",
            self.tree.iter().zip(self.groups.iter_shared()).map(|(tree, group)| {
                format!(
                    "{}: {}",
                    group.to_string(),
                    tree.size()
                )
            }).collect::<Vec<_>>().join(", ")
        ).into()
    }
}

#[godot_api]
impl Nearest2D {
    #[func]
    fn child_entered_tree(&mut self, child: Gd<Node>) {
        let child = match child.try_cast::<Node2D>() {
            Ok(child) => child,
            Err(_) => return,
        };
        // Add child to the tree
        let pos = child.get_global_position();
        self.tree.get_mut(0).unwrap().add([pos.x, pos.y], child.get_path()).unwrap_or(());
        // Add pos update signal
    }

    #[func]
    fn child_exiting_tree(&mut self, child: Gd<Node>) {
        let child = match child.try_cast::<Node2D>() {
            Ok(child) => child,
            Err(_) => return,
        };
        // Remove child from the tree
        let pos = child.get_global_position();
        self.tree.get_mut(0).unwrap().remove(&[pos.x, pos.y], &child.get_path()).unwrap_or(0);
    }

    #[func]
    fn nearest(&self, pos: Vector2, group_idx: i64) -> Option<Gd<Node2D>> {
        match self.tree.get(group_idx as usize).unwrap().iter_nearest(
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
        let pos = [pos.x, pos.y];
        let iterator = match self.tree.get(group_idx as usize).unwrap().iter_nearest(
            &pos,
            &kdtree::distance::squared_euclidean,
        ) {
            Ok(iter) => iter,
            _ => return Array::new(),
        };
        let length = if length < 0 {
            self.tree.len()
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
