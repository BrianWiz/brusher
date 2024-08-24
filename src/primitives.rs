#[cfg(feature = "bevy")]
use bevy::math::DVec3;

#[cfg(not(feature = "bevy"))]
use glam::DVec3;

// A cuboid material indices
///
/// # Fields
/// * `top` - The material index for the top face
/// * `bottom` - The material index for the bottom face
/// * `front` - The material index for the front face
/// * `back` - The material index for the back face
/// * `left` - The material index for the left face
/// * `right` - The material index for the right face
#[derive(Debug, Clone, Default)]
pub struct CuboidMaterialIndices {
    pub top: usize,
    pub bottom: usize,
    pub front: usize,
    pub back: usize,
    pub left: usize,
    pub right: usize,
}

/// A cuboid
///
/// # Fields
/// * `origin` - The origin of the cuboid
/// * `width` - The width of the cuboid (x-axis)
/// * `height` - The height of the cuboid (y-axis)
/// * `depth` - The depth of the cuboid (z-axis)
/// * `material_indices` - The material indices for each face of the cuboid
#[derive(Debug, Clone)]
pub struct Cuboid {
    pub origin: DVec3,
    pub width: f64,
    pub height: f64,
    pub depth: f64,
    pub material_indices: CuboidMaterialIndices,
}
