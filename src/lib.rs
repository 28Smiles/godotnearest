use std::collections::BTreeSet;
use godot::prelude::*;
use kdtree::KdTree;

struct GodotNearest;

#[gdextension]
unsafe impl ExtensionLibrary for GodotNearest {}

#[derive(GodotClass)]
#[class(base=Node2D)]
struct Nearest2D {
    #[export]
    groups: Array<GString>,
    base: Base<Node2D>,
    needs_update: BTreeSet<String>,
    tree: KdTree<f32, NodePath, [f32; 2]>,
}

#[godot_api]
impl INode2D for Nearest2D {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            groups: Array::new(),
            base,
            needs_update: BTreeSet::new(),
            tree: KdTree::new(2),
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
            "Nearest2D({})",
            self.tree.iter_nearest(
                &[0.0, 0.0],
                &kdtree::distance::squared_euclidean,
            ).unwrap().map(|(_, path)| path.to_string()).collect::<Vec<_>>().join(", ")
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
        self.tree.add([pos.x, pos.y], child.get_path()).unwrap_or(());
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
        self.tree.remove(&[pos.x, pos.y], &child.get_path()).unwrap_or(0);
    }

    fn apply_updates(&mut self) {
        if !self.needs_update.is_empty() {
            for path in self.needs_update.iter() {
                let path = NodePath::from(path);
                if let Some(node) = self.base().get_node(path.clone()) {
                    if let Ok(node) = node.try_cast::<Node2D>() {
                        let pos = node.get_global_position();
                        self.tree.remove(&[pos.x, pos.y], &path).unwrap_or(0);
                        self.tree.add([pos.x, pos.y], path).unwrap_or(());
                    }
                }
            }
            self.needs_update.clear();
        }
    }

    #[func]
    fn child_update_pos(&mut self, child: Gd<Node2D>) {
        self.needs_update.insert(child.get_path().to_string());
    }

    #[func]
    fn nearest(&mut self, pos: Vector2, group_idx: i64) -> Option<Gd<Node2D>> {
        self.apply_updates();
        match self.tree.iter_nearest(
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
    fn nearest_array(&mut self, pos: Vector2, group_idx: i64) -> Array<Gd<Node2D>> {
        self.apply_updates();
        let pos = [pos.x, pos.y];
        match self.tree.iter_nearest(
            &pos,
            &kdtree::distance::squared_euclidean,
        ) {
            Ok(iter) => iter,
            _ => return Array::new(),
        }.map(|(_, path)| {
            self.base().get_node(path.clone())
                .map(|node| node.cast())
        }).flatten().collect()
    }
}
