use super::surface::Surface;

#[cfg(feature = "bevy")]
use bevy::math::{DAffine3, DVec3};

#[cfg(not(feature = "bevy"))]
use glam::{DAffine3, DVec3};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Reflect))]
pub struct Polygon {
    pub vertices: Vec<Vertex>,
    pub surface: Surface,
}

impl Polygon {
    pub fn new(vertices: Vec<Vertex>, material_index: usize) -> Self {
        let mut surface = Surface::from_points(
            vertices[0].pos,
            vertices[1].pos,
            vertices[2].pos,
            material_index,
        );

        surface.material_idx = material_index;

        Self { vertices, surface }
    }

    pub fn flip(&mut self) {
        for vertex in &mut self.vertices {
            vertex.flip();
        }
        self.vertices.reverse();
        self.surface.flip();
    }

    pub fn indices(&self) -> Vec<u32> {
        let mut indices = Vec::new();
        let vertex_count = self.vertices.len() as u32;

        for i in 1..vertex_count - 1 {
            indices.push(0);
            indices.push(i);
            indices.push(i + 1);
        }

        indices
    }

    pub fn normals(&self) -> Vec<DVec3> {
        self.vertices.iter().map(|vertex| vertex.normal).collect()
    }

    pub fn normals_32(&self) -> Vec<[f32; 3]> {
        self.vertices
            .iter()
            .map(|vertex| vertex.normal)
            .map(|normal| [normal.x as f32, normal.y as f32, normal.z as f32])
            .collect()
    }

    pub fn positions(&self) -> Vec<DVec3> {
        self.vertices.iter().map(|vertex| vertex.pos).collect()
    }

    pub fn positions_32(&self) -> Vec<[f32; 3]> {
        self.vertices
            .iter()
            .map(|vertex| vertex.pos)
            .map(|pos| [pos.x as f32, pos.y as f32, pos.z as f32])
            .collect()
    }

    pub fn uvs(&self) -> Vec<[f32; 2]> {
        self.vertices
            .iter()
            .map(|vertex| self.surface.compute_uv(vertex.pos))
            .map(|uv| [uv.x as f32, uv.y as f32])
            .collect()
    }

    pub fn compute_transform(&self) -> DAffine3 {
        let normal = self.surface.normal;
        let tangent = if normal.x.abs() > 0.9 {
            DVec3::new(0.0, 1.0, 0.0)
        } else {
            DVec3::new(1.0, 0.0, 0.0)
        };
        let bitangent = normal.cross(tangent).normalize();
        let tangent = bitangent.cross(normal).normalize();

        DAffine3::from_cols(tangent, bitangent, normal, self.vertices[0].pos)
    }

    pub fn transform(&self, transform: DAffine3) -> Self {
        let vertices = self
            .vertices
            .iter()
            .map(|vertex| {
                let pos = transform.transform_point3(vertex.pos);
                let normal = transform.transform_vector3(vertex.normal);
                Vertex::new(pos, normal)
            })
            .collect();

        Self {
            vertices,
            surface: self.surface.transform(transform),
        }
    }

    pub fn contains_point(&self, point: DVec3) -> bool {
        let normal = self.surface.normal;
        let d = -normal.dot(self.vertices[0].pos);
        let distance = normal.dot(point) + d;
        distance.abs() < 0.0001
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Reflect))]
pub struct Vertex {
    pub pos: DVec3,
    pub normal: DVec3,
}

impl Vertex {
    pub fn new(pos: DVec3, normal: DVec3) -> Self {
        Self { pos, normal }
    }

    pub fn interpolate(&self, other: &Self, t: f64) -> Self {
        Self {
            pos: self.pos.lerp(other.pos, t),
            normal: self.normal.lerp(other.normal, t).normalize(),
        }
    }

    pub fn flip(&mut self) {
        self.normal = -self.normal;
    }
}
