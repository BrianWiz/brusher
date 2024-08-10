use crate::math::*;

use super::{SurfaceGroup, SurfaceOperation, SurfacePlane};

pub fn cuboid(relative_origin: Vector3, dimensions: CuboidDimensions, operation: SurfaceOperation) -> SurfaceGroup {
    let half_width = dimensions.width / 2.0;
    let half_height = dimensions.height / 2.0;
    let half_depth = dimensions.depth / 2.0;

    let mut planes = Vec::new();

    // front
    planes.push(SurfacePlane::new(Vector3::new(0.0, 0.0, 1.0), relative_origin.z + half_depth));

    // back
    planes.push(SurfacePlane::new(Vector3::new(0.0, 0.0, -1.0), -(relative_origin.z - half_depth)));

    // top
    planes.push(SurfacePlane::new(Vector3::new(0.0, 1.0, 0.0), relative_origin.y + half_height));

    // bottom
    planes.push(SurfacePlane::new(Vector3::new(0.0, -1.0, 0.0), -(relative_origin.y - half_height)));

    // right
    planes.push(SurfacePlane::new(Vector3::new(1.0, 0.0, 0.0), relative_origin.x + half_width));

    // left
    planes.push(SurfacePlane::new(Vector3::new(-1.0, 0.0, 0.0), -(relative_origin.x - half_width)));

    let mut group = SurfaceGroup::new(planes);
    group.operation = operation;
    group
}

pub fn cylinder(
    relative_origin: Vector3,
    dimensions: CylinderDimensions,
    slices: u32,
    operation: SurfaceOperation,
) -> SurfaceGroup {
    let half_height = dimensions.height / 2.0;
    let radius = dimensions.radius;

    let mut planes = Vec::new();

    // top cap
    planes.push(SurfacePlane::new(Vector3::new(0.0, 1.0, 0.0), relative_origin.y + half_height));

    // bottom cap
    planes.push(SurfacePlane::new(Vector3::new(0.0, -1.0, 0.0), -(relative_origin.y - half_height)));

    // side planes
    let angle_step = 2.0 * std::f64::consts::PI / slices as f64;
    for i in 0..slices {
        let angle = angle_step * i as f64;
        let normal = Vector3::new(angle.cos(), 0.0, angle.sin());
        let distance = relative_origin.x * normal.x + relative_origin.z * normal.z + radius;

        planes.push(SurfacePlane::new(normal, distance));
    }

    let mut group = SurfaceGroup::new(planes);
    group.operation = operation;
    group
}

