use godot::prelude::*;
use crate::FloatType;
use crate::nearest::Nearest;

#[derive(GodotClass)]
#[class(base=Node3D)]
struct Nearest3D {
    #[export]
    groups: Array<GString>,
    nearest: Nearest<3, FloatType>,
    base: Base<Node3D>,
}

#[godot_api]
impl INode2D for Nearest3D {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            groups: Array::new(),
            nearest: Nearest::new(),
            base,
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
        format!("Nearest2D({})", self.nearest).into()
    }

    fn set_property(&mut self, property: StringName, value: Variant) -> bool {
        if &property.to_string() == "groups" {
            self.groups.set_property(value.to());
            let groups = self.groups.iter_shared()
                .map(|group| group.to_string())
                .collect::<Vec<_>>();
            let ref_groups = groups.iter()
                .map(|group| group.as_str())
                .collect::<Vec<_>>();
            self.nearest = Nearest::new_from(ref_groups);
            self.index();
            true
        } else {
            false
        }
    }
}

#[godot_api]
impl Nearest3D {
    fn index(&mut self) {
        for child in self.base().get_children().iter_shared() {
            self.child_entered_tree(child);
        }
    }


    #[func]
    fn child_entered_tree(&mut self, child: Gd<Node>) {
        let child = match child.try_cast::<Node3D>() {
            Ok(child) => child,
            Err(_) => return,
        };
        // Add child to the tree
        let child_name = child.get_name().to_string();
        let pos = child.get_global_position();
        let pos = [pos.x, pos.y, pos.z];
        let path = child.get_path();
        self.nearest.add(
            &child_name,
            pos,
            path,
        );
    }

    #[func]
    fn child_exiting_tree(&mut self, child: Gd<Node>) {
        let child = match child.try_cast::<Node3D>() {
            Ok(child) => child,
            Err(_) => return,
        };
        // Remove child from the tree
        let child_name = child.get_name().to_string();
        let pos = child.get_global_position();
        let pos = [pos.x, pos.y, pos.z];
        let path = child.get_path();
        self.nearest.remove(
            &child_name,
            &pos,
            &path,
        );
    }

    #[func]
    fn nearest(&self, pos: Vector3, group_idx: i64) -> Option<Gd<Node3D>> {
        let pos = [pos.x, pos.y, pos.z];
        let node = match self.nearest.nearest(&pos, group_idx as usize) {
            Some(iter) => iter
                .map(|(_, path)| self.base().get_node(path.clone()))
                .flatten()
                .map(|node| node.cast())
                .next(),
            None => None,
        };

        node
    }

    #[func]
    fn nearest_array(&self, pos: Vector3, group_idx: i64, length: i64) -> Array<Gd<Node3D>> {
        let pos = [pos.x, pos.y, pos.z];
        let length = length as usize;
        let array = match self.nearest.nearest(&pos, group_idx as usize) {
            Some(iter) => iter
                .take(length)
                .map(|(_, path)| self.base().get_node(path.clone()))
                .flatten()
                .map(|node| node.cast())
                .collect::<Array<_>>(),
            None => Array::new(),
        };

        array
    }
}