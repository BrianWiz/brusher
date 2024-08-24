pub mod brushlet;
mod node;
pub mod operations;

use crate::{
    broadphase::{Raycast, RaycastResult},
    polygon::Polygon,
};

use brushlet::Brushlet;
use operations::Knife;

#[cfg(feature = "bevy")]
use bevy::{
    math::DAffine3,
    render::{
        mesh::{Indices, Mesh, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
};

#[cfg(not(feature = "bevy"))]
use glam::DAffine3;

pub type MaterialIndex = usize;

#[derive(Debug, Clone)]
pub struct MeshData {
    pub polygons: Vec<Polygon>,
}

impl MeshData {
    pub fn to_bevy_meshes(&self) -> Vec<(Mesh, MaterialIndex)> {
        let mut meshes_with_materials: Vec<(Mesh, MaterialIndex)> = vec![];

        for polygon in &self.polygons {
            let positions = polygon.positions_32();
            let normals = polygon.normals_32();
            let uvs = polygon.uvs();
            let indices = polygon.indices();
            let mut mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::default(),
            );
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
            mesh.insert_indices(Indices::U32(indices));
            meshes_with_materials.push((mesh, polygon.surface.material_idx));
        }

        meshes_with_materials
    }
}

#[derive(Debug)]
pub enum BrushError {
    BrushletAtIndexDoesNotExist(usize),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Reflect))]
pub struct BrushSettings {
    pub name: String,
    pub knives: Vec<Knife>,
}

#[derive(Debug)]
pub struct BrushSelection {
    pub idx: usize,
    pub layer_idx: usize,
    pub raycast_result: RaycastResult,
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
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Reflect))]
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
#[cfg_attr(
    feature = "bevy",
    derive(bevy::prelude::Component, bevy::prelude::Reflect)
)]
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

    pub fn try_select(&self, raycast: &Raycast) -> Option<RaycastResult> {
        let mut closest = None;
        let mut closest_distance = f64::INFINITY;
        for brushlet in self.brushlets.iter() {
            if let Some(result) = brushlet.try_select(&raycast) {
                if result.distance < closest_distance {
                    closest_distance = result.distance;
                    closest = Some(result);
                }
            }
        }
        closest
    }

    pub fn try_select_brushlet(&self, raycast: &Raycast) -> Option<usize> {
        let mut closest = None;
        let mut closest_distance = f64::INFINITY;
        for (idx, brushlet) in self.brushlets.iter().enumerate() {
            if let Some(result) = brushlet.try_select(&raycast) {
                if result.distance < closest_distance {
                    closest_distance = result.distance;
                    closest = Some(idx);
                }
            }
        }
        closest
    }

    pub fn get_brushlet_mut(&mut self, idx: usize) -> Option<&mut Brushlet> {
        self.brushlets.get_mut(idx)
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

    pub fn compute_transform(&self) -> DAffine3 {
        let mut transform = DAffine3::IDENTITY;
        for brushlet in &self.brushlets {
            transform = transform * brushlet.compute_transform();
        }
        transform
    }

    pub fn transform(&mut self, transform: DAffine3) {
        for brushlet in &mut self.brushlets {
            brushlet.transform(transform);
        }
    }
}
