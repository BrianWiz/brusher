pub mod brushlet;
mod node;
pub mod operations;

use crate::{polygon::Polygon, surface::Surface};
use brushlet::Brushlet;
use glam::DVec3;
use operations::Knife;

#[derive(Debug, Clone)]
pub struct MeshData {
    pub polygons: Vec<Polygon>,
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
