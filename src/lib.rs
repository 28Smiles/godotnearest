use godot::prelude::*;

mod nearest;
mod nearest2d;
mod nearest3d;

#[cfg(feature = "double-precision")]
pub type FloatType = f64;
#[cfg(not(feature = "double-precision"))]
pub type FloatType = f32;

struct GodotNearest;

#[gdextension]
unsafe impl ExtensionLibrary for GodotNearest {}
