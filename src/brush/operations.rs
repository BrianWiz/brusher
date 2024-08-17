use glam::DVec3;

use super::brushlet::Brushlet;
use crate::surface::Surface;

/// A knife
///
/// A knife is a plane that is used to cut geometry.
///
/// # Fields
/// * `normal` - The normal of the plane
/// * `distance_from_origin` - The distance from the origin of the geometry
#[derive(Debug, Clone, Copy)]
pub struct Knife {
    pub normal: DVec3,
    pub distance_from_origin: f64,
    pub material_index: usize,
}

impl Knife {
    pub fn perform(&self, brushlet: &Brushlet) -> Brushlet {
        // Define a large value to ensure the cuboid encompasses the entire geometry
        const LARGE_VALUE: f64 = 1e5;

        // Create the primary cutting plane
        let cutting_plane = Surface::new(
            -self.normal,
            -self.distance_from_origin,
            self.material_index,
        );

        // Create two orthogonal vectors to the plane normal
        let mut u = if self.normal.x.abs() > self.normal.y.abs() {
            DVec3::new(0.0, 1.0, 0.0)
        } else {
            DVec3::new(1.0, 0.0, 0.0)
        };
        u = u.cross(self.normal).normalize();
        let v = self.normal.cross(u).normalize();

        // Create the six planes that define the cutting cuboid
        let planes = vec![
            cutting_plane,
            Surface::new(
                self.normal,
                self.distance_from_origin + LARGE_VALUE,
                self.material_index,
            ), // Back plane, far behind the cut
            Surface::new(u, LARGE_VALUE, self.material_index), // Large plane in one direction
            Surface::new(-u, LARGE_VALUE, self.material_index), // Large plane in the opposite direction
            Surface::new(v, LARGE_VALUE, self.material_index),  // Large plane in another direction
            Surface::new(-v, LARGE_VALUE, self.material_index), // Large plane in the opposite direction
        ];

        // Create the cutting cuboid from the defined planes
        let cutting_cuboid = Brushlet::from_surfaces(planes, brushlet.settings.clone());
        brushlet.subtract(&cutting_cuboid)
    }

    pub fn transform(&self, transform: glam::DAffine3) -> Self {
        let normal = transform.transform_vector3(self.normal).normalize();
        let distance_from_origin = self.distance_from_origin + normal.dot(transform.translation);
        Self {
            normal,
            distance_from_origin,
            material_index: self.material_index,
        }
    }
}
