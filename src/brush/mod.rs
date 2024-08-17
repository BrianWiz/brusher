pub mod brushlet;
mod node;
pub mod operations;

use crate::{broadphase::Raycast, polygon::Polygon};
use brushlet::Brushlet;
use operations::Knife;

#[derive(Debug, Clone)]
pub struct MeshData {
    pub polygons: Vec<Polygon>,
}

#[derive(Debug)]
pub enum BrushError {
    BrushletAtIndexDoesNotExist(usize),
}

#[derive(Debug, Clone)]
pub struct BrushSettings {
    pub name: String,
    pub knives: Vec<Knife>,
}

#[derive(Debug, Clone)]
pub struct BrushSelection<'a> {
    pub brush: &'a Brush,
    pub idx: usize,
    pub layer_idx: usize,
}

#[derive(Debug, Clone)]
pub struct BrushletSelection<'a> {
    pub brushlet: &'a Brushlet,
    pub idx: usize,
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

/// # Brush
///
/// A brush is a collection of brushlets that can be combined using boolean operations.
///
/// # Fields
/// * `brushlets` - The brushlets that make up the brush
/// * `knives` - The knives to use for cutting
#[derive(Debug, Clone)]
pub struct Brush {
    pub brushlets: Vec<Brushlet>,
    pub settings: BrushSettings,
}

impl Brush {
    pub fn new(name: &str) -> Self {
        Self {
            brushlets: Vec::new(),
            settings: BrushSettings {
                name: name.to_string(),
                knives: Vec::new(),
            },
        }
    }

    pub(crate) fn try_select(&self, raycast: &Raycast) -> bool {
        for brushlet in self.brushlets.iter() {
            if brushlet.try_select(&raycast) {
                return true;
            }
        }
        false
    }

    pub fn try_select_brushlet(&self, raycast: &Raycast) -> Option<BrushletSelection> {
        for (idx, brushlet) in self.brushlets.iter().enumerate() {
            if brushlet.try_select(&raycast) {
                return Some(BrushletSelection { brushlet, idx });
            }
        }
        None
    }

    /// Select a brushlet by index.
    pub fn select_brushlet(&self, idx: usize) -> Result<&Brushlet, BrushError> {
        if idx >= self.brushlets.len() {
            return Err(BrushError::BrushletAtIndexDoesNotExist(idx));
        }
        Ok(&self.brushlets[idx])
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
        for knife in &self.settings.knives {
            final_brushlet = knife.perform(&final_brushlet);
        }

        final_brushlet.to_mesh_data()
    }

    pub fn transform(&mut self, transform: glam::DAffine3) {
        for brushlet in &mut self.brushlets {
            brushlet.transform(transform);
        }
    }
}
