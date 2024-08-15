pub mod brushlet;
mod node;

use crate::{polygon::Polygon, surface::Surface};
use brushlet::Brushlet;
use glam::DVec3;

#[derive(Debug, Clone)]
pub struct MeshData {
    pub polygons: Vec<Polygon>,
}

impl MeshData {
    pub fn positions(&self) -> Vec<DVec3> {
        self.polygons
            .iter()
            .flat_map(|polygon| polygon.vertices.iter().map(|vertex| vertex.pos))
            .collect()
    }

    pub fn normals(&self) -> Vec<DVec3> {
        self.polygons
            .iter()
            .flat_map(|polygon| polygon.vertices.iter().map(|vertex| vertex.normal))
            .collect()
    }

    pub fn indices(&self) -> Vec<u32> {
        let mut indices = Vec::new();
        let mut offset = 0;

        for polygon in &self.polygons {
            let vertex_count = polygon.vertices.len() as u32;
            for i in 1..vertex_count - 1 {
                indices.push(offset);
                indices.push(offset + i);
                indices.push(offset + i + 1);
            }
            offset += vertex_count;
        }

        indices
    }
}

#[derive(Debug)]
pub enum BrushError {
    BrushletAtIndexDoesNotExist(usize),
}

/// A brushlet operation
///
/// A brushlet operation is a specific operation to perform on a brushlet.
///
/// # Fields
/// * `Knife` - A knife operation, slices the brushlet with a plane, disarding the part in front of the plane.
#[derive(Debug, Clone, Copy)]
pub enum BrushletOp {
    Knife(Knife),
}

/// A boolean operation to perform between two brushlets.
#[derive(Debug, Clone, Copy)]
pub enum BooleanOp {
    Union,
    Intersect,
    Subtract,
}

/// A brush
///
/// A brush is a collection of brushlets that can be combined using boolean operations.
///
/// # Fields
/// * `brushlets` - The brushlets that make up the brush
/// * `knives` - The knives to use for cutting
#[derive(Debug, Clone)]
pub struct Brush {
    brushlets: Vec<Brushlet>,
    pub knives: Vec<Knife>,
}

impl Brush {
    pub fn new() -> Self {
        Self {
            brushlets: Vec::new(),
            knives: Vec::new(),
        }
    }

    /// Select a brushlet by index.
    pub fn select(&self, idx: usize) -> Result<&Brushlet, BrushError> {
        if idx >= self.brushlets.len() {
            return Err(BrushError::BrushletAtIndexDoesNotExist(idx));
        }
        Ok(&self.brushlets[idx])
    }

    /// Add a brushlet. Returns the index of the added brushlet.
    pub fn add(&mut self, brushlet: Brushlet) -> usize {
        self.brushlets.push(brushlet);
        self.brushlets.len() - 1
    }

    /// Insert a brushlet at a specific index.
    pub fn update(&mut self, idx: usize, brushlet: Brushlet) -> Result<(), BrushError> {
        if idx >= self.brushlets.len() {
            return Err(BrushError::BrushletAtIndexDoesNotExist(idx));
        }
        self.brushlets[idx] = brushlet;

        Ok(())
    }

    /// Remove a brushlet by index.
    pub fn remove(&mut self, idx: usize) -> Result<(), BrushError> {
        if idx >= self.brushlets.len() {
            return Err(BrushError::BrushletAtIndexDoesNotExist(idx));
        }
        self.brushlets.remove(idx);

        Ok(())
    }

    /// Performs all operations on the brushlets and returns the
    /// resulting mesh data which can be used to render the geometry.
    pub fn to_mesh_data(&self) -> MeshData {
        if self.brushlets.is_empty() {
            return MeshData {
                polygons: Vec::new(),
            };
        }

        let mut final_brushlet = self.brushlets[0].clone();

        for other in self.brushlets.iter().skip(1) {
            final_brushlet = match other.settings.operation {
                BooleanOp::Union => final_brushlet.union(other),
                BooleanOp::Intersect => final_brushlet.intersect(other),
                BooleanOp::Subtract => final_brushlet.subtract(other),
            };
        }

        // do the final global knife operations
        for knife in &self.knives {
            final_brushlet = knife.perform(&final_brushlet);
        }

        final_brushlet.to_mesh_data()
    }
}

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
}

impl Knife {
    pub fn perform(&self, brushlet: &Brushlet) -> Brushlet {
        // Define a large value to ensure the cuboid encompasses the entire geometry
        const LARGE_VALUE: f64 = 1e5;

        // Create the primary cutting plane
        let cutting_plane = Surface::new(-self.normal, -self.distance_from_origin);

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
            Surface::new(self.normal, self.distance_from_origin + LARGE_VALUE), // Back plane, far behind the cut
            Surface::new(u, LARGE_VALUE), // Large plane in one direction
            Surface::new(-u, LARGE_VALUE), // Large plane in the opposite direction
            Surface::new(v, LARGE_VALUE), // Large plane in another direction
            Surface::new(-v, LARGE_VALUE), // Large plane in the opposite direction
        ];

        // Create the cutting cuboid from the defined planes
        let cutting_cuboid = Brushlet::from_surfaces(planes, brushlet.settings.clone());
        brushlet.subtract(&cutting_cuboid)
    }
}
