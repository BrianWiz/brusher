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
