pub mod broadphase;
pub mod brush;
pub mod polygon;
pub mod primitives;
pub mod scene;
pub mod surface;
mod util;

pub mod prelude {
    pub use crate::brush::{
        brushlet::{Brushlet, BrushletSettings},
        operations::Knife,
        BooleanOp, Brush, BrushError, BrushSettings, BrushletOp, MeshData,
    };
    pub use crate::polygon::*;
    pub use crate::primitives::*;
    pub use crate::surface::*;

    #[cfg(not(feature = "bevy"))]
    pub use glam::{DAffine3, DVec2, DVec3};
}
