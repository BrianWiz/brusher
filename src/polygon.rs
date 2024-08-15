use glam::DVec3;

use super::surface::Surface;

#[derive(Debug, Clone)]
pub struct Polygon {
    pub vertices: Vec<Vertex>,
    pub material: usize,
    pub surface: Surface,
}

impl Polygon {
    pub fn new(vertices: Vec<Vertex>, shared: usize) -> Self {
        let plane = Surface::from_points(vertices[0].pos, vertices[1].pos, vertices[2].pos);
        Self {
            vertices,
            material: shared,
            surface: plane,
        }
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
}

#[derive(Debug, Clone)]
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
