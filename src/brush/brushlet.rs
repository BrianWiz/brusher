use glam::{dvec3, DVec3};

use crate::{
    polygon::{Polygon, Vertex},
    primitives::Cuboid,
    surface::Surface,
};

use super::{node::Node, operations::Knife, BooleanOp, MeshData};

#[derive(Debug, Clone)]
pub struct BrushletSettings {
    pub name: String,
    pub operation: BooleanOp,
    pub knives: Vec<Knife>,
    pub inverted: bool,
}

/// A brushlet
///
/// A brushlet is a collection of polygons that can be combined using boolean operations.
/// They are meant to exist only as a child of a brush.
///
/// # Fields
/// * `polygons` - The polygons that make up the brushlet
/// * `operation` - The boolean operation to perform
/// * `knives` - The knives to use for cutting
/// * `inverted` - Whether the brushlet is inverted
#[derive(Debug, Clone)]
pub struct Brushlet {
    pub polygons: Vec<Polygon>,
    pub settings: BrushletSettings,
}

impl Brushlet {
    pub(crate) fn union(&self, other: &Brushlet) -> Self {
        let mut a = Node::new(self.polygons.clone());
        let mut b = Node::new(other.polygons.clone());
        a.clip_to(&b);
        b.clip_to(&a);
        b.invert();
        b.clip_to(&a);
        b.invert();
        let mut a = a;
        a.build(b.all_polygons());
        Brushlet {
            polygons: a.all_polygons(),
            settings: self.settings.clone(),
        }
    }

    pub(crate) fn subtract(&self, other: &Brushlet) -> Self {
        let mut a = Node::new(self.polygons.clone());
        let mut b = Node::new(other.polygons.clone());
        a.invert();
        a.clip_to(&b);
        b.clip_to(&a);
        b.invert();
        b.clip_to(&a);
        b.invert();
        a.build(b.all_polygons());
        a.invert();
        Brushlet {
            polygons: a.all_polygons(),
            settings: self.settings.clone(),
        }
    }

    pub(crate) fn intersect(&self, other: &Brushlet) -> Self {
        let mut a = Node::new(self.polygons.clone());
        let mut b = Node::new(other.polygons.clone());
        a.invert();
        b.clip_to(&a);
        b.invert();
        a.clip_to(&b);
        b.clip_to(&a);
        a.build(b.all_polygons());
        a.invert();
        Brushlet {
            polygons: a.all_polygons(),
            settings: self.settings.clone(),
        }
    }

    pub(crate) fn to_mesh_data(&self) -> MeshData {
        let mut final_brushlet = self.clone();

        for knife in &self.settings.knives {
            final_brushlet = knife.perform(&final_brushlet);
        }

        if self.settings.inverted {
            final_brushlet = final_brushlet.inverse();
        }

        MeshData {
            polygons: final_brushlet.polygons,
        }
    }

    pub(crate) fn inverse(&self) -> Self {
        let mut csg = Brushlet {
            polygons: self.polygons.clone(),
            settings: self.settings.clone(),
        };
        for polygon in &mut csg.polygons {
            polygon.flip();
        }
        csg
    }

    pub fn from_surfaces(surfaces: Vec<Surface>, settings: BrushletSettings) -> Self {
        let polygons = crate::util::generate_polygons_from_surfaces(&surfaces);
        Self { polygons, settings }
    }

    pub fn from_cuboid(cuboid: Cuboid, settings: BrushletSettings) -> Self {
        let half_width = cuboid.width * 0.5;
        let half_height = cuboid.height * 0.5;
        let half_depth = cuboid.depth * 0.5;

        let vertices = vec![
            // Define vertices without normals initially
            Vertex::new(
                cuboid.origin + dvec3(-half_width, -half_height, -half_depth),
                DVec3::ZERO,
            ),
            Vertex::new(
                cuboid.origin + dvec3(half_width, -half_height, -half_depth),
                DVec3::ZERO,
            ),
            Vertex::new(
                cuboid.origin + dvec3(half_width, half_height, -half_depth),
                DVec3::ZERO,
            ),
            Vertex::new(
                cuboid.origin + dvec3(-half_width, half_height, -half_depth),
                DVec3::ZERO,
            ),
            Vertex::new(
                cuboid.origin + dvec3(-half_width, -half_height, half_depth),
                DVec3::ZERO,
            ),
            Vertex::new(
                cuboid.origin + dvec3(half_width, -half_height, half_depth),
                DVec3::ZERO,
            ),
            Vertex::new(
                cuboid.origin + dvec3(half_width, half_height, half_depth),
                DVec3::ZERO,
            ),
            Vertex::new(
                cuboid.origin + dvec3(-half_width, half_height, half_depth),
                DVec3::ZERO,
            ),
        ];

        let polygons = vec![
            Polygon::new(
                vec![
                    Vertex::new(vertices[4].pos, DVec3::Z),
                    Vertex::new(vertices[5].pos, DVec3::Z),
                    Vertex::new(vertices[6].pos, DVec3::Z),
                    Vertex::new(vertices[7].pos, DVec3::Z),
                ],
                cuboid.material_indices.front,
            ),
            Polygon::new(
                vec![
                    Vertex::new(vertices[0].pos, -DVec3::Z),
                    Vertex::new(vertices[3].pos, -DVec3::Z),
                    Vertex::new(vertices[2].pos, -DVec3::Z),
                    Vertex::new(vertices[1].pos, -DVec3::Z),
                ],
                cuboid.material_indices.back,
            ),
            Polygon::new(
                vec![
                    Vertex::new(vertices[3].pos, DVec3::Y),
                    Vertex::new(vertices[7].pos, DVec3::Y),
                    Vertex::new(vertices[6].pos, DVec3::Y),
                    Vertex::new(vertices[2].pos, DVec3::Y),
                ],
                cuboid.material_indices.top,
            ),
            Polygon::new(
                vec![
                    Vertex::new(vertices[0].pos, -DVec3::Y),
                    Vertex::new(vertices[1].pos, -DVec3::Y),
                    Vertex::new(vertices[5].pos, -DVec3::Y),
                    Vertex::new(vertices[4].pos, -DVec3::Y),
                ],
                cuboid.material_indices.bottom,
            ),
            Polygon::new(
                vec![
                    Vertex::new(vertices[1].pos, DVec3::X),
                    Vertex::new(vertices[2].pos, DVec3::X),
                    Vertex::new(vertices[6].pos, DVec3::X),
                    Vertex::new(vertices[5].pos, DVec3::X),
                ],
                cuboid.material_indices.right,
            ),
            Polygon::new(
                vec![
                    Vertex::new(vertices[0].pos, -DVec3::X),
                    Vertex::new(vertices[4].pos, -DVec3::X),
                    Vertex::new(vertices[7].pos, -DVec3::X),
                    Vertex::new(vertices[3].pos, -DVec3::X),
                ],
                cuboid.material_indices.left,
            ),
        ];

        Brushlet { polygons, settings }
    }
}
