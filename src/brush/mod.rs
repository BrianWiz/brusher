pub mod brushlet;
mod node;

use crate::polygon::Polygon;
use brushlet::Brushlet;
use glam::DVec3;

#[derive(Debug)]
pub enum BrushError {
    BrushletAtIndexDoesNotExist(usize),
}

#[derive(Debug, Clone, Copy)]
pub enum BrushletOp {
    Knife(Knife),
}

#[derive(Debug, Clone)]
pub struct MeshData {
    pub polygons: Vec<Polygon>,
}

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

    pub fn select(&self, idx: usize) -> Result<&Brushlet, BrushError> {
        if idx >= self.brushlets.len() {
            return Err(BrushError::BrushletAtIndexDoesNotExist(idx));
        }
        Ok(&self.brushlets[idx])
    }

    pub fn add(&mut self, brushlet: Brushlet) {
        self.brushlets.push(brushlet);
    }

    pub fn update(&mut self, idx: usize, brushlet: Brushlet) -> Result<(), BrushError> {
        if idx >= self.brushlets.len() {
            return Err(BrushError::BrushletAtIndexDoesNotExist(idx));
        }
        self.brushlets[idx] = brushlet;

        Ok(())
    }

    pub fn remove(&mut self, idx: usize) -> Result<(), BrushError> {
        if idx >= self.brushlets.len() {
            return Err(BrushError::BrushletAtIndexDoesNotExist(idx));
        }
        self.brushlets.remove(idx);

        Ok(())
    }

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
            final_brushlet = final_brushlet.knife(*knife);
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
