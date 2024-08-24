use std::{
    hash::{Hash, Hasher},
    ops::BitOr,
};

use super::polygon::Polygon;

#[cfg(feature = "bevy")]
use bevy::math::{DAffine3, DVec2, DVec3};

#[cfg(not(feature = "bevy"))]
use glam::{DAffine3, DVec2, DVec3};

/// The type of a polygon.
///
/// A polygon can be coplanar with a surface, in front of it, behind it, or spanning it.
///
/// # Variants
/// * `Coplanar` - The polygon is coplanar with the surface
/// * `Front` - The polygon is in front of the surface
/// * `Back` - The polygon is behind the surface
/// * `Spanning` - The polygon is spanning the surface
#[derive(Clone, Copy, PartialEq)]
enum PolygonType {
    Coplanar = 0,
    Front = 1,
    Back = 2,
    Spanning = 3,
}

impl BitOr for PolygonType {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        match (self as u8) | (rhs as u8) {
            0 => PolygonType::Coplanar,
            1 => PolygonType::Front,
            2 => PolygonType::Back,
            3 => PolygonType::Spanning,
            _ => unreachable!(),
        }
    }
}

/// A surface in 3D space.
///
/// A surface is defined by a normal vector and a distance from the origin.
///
/// # Fields
/// * `normal` - The normal vector of the surface
/// * `distance_from_origin` - The distance from the origin
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Reflect))]
pub struct Surface {
    pub normal: DVec3,
    pub distance_from_origin: f64,
    pub material_idx: usize,
}

impl Hash for Surface {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.quantized_normal().hash(state);
        self.quantized_distance().hash(state);
    }
}

impl PartialEq for Surface {
    fn eq(&self, other: &Self) -> bool {
        self.quantized_normal() == other.quantized_normal()
            && self.quantized_distance() == other.quantized_distance()
    }
}

impl Eq for Surface {}

impl Surface {
    pub const EPSILON: f64 = 1e-5;
    const QUANTIZATION_FACTOR: f64 = 1_000_000.0;

    fn quantize(value: f64) -> i64 {
        (value * Self::QUANTIZATION_FACTOR).round() as i64
    }

    fn quantized_normal(&self) -> (i64, i64, i64) {
        (
            Self::quantize(self.normal.x),
            Self::quantize(self.normal.y),
            Self::quantize(self.normal.z),
        )
    }

    fn quantized_distance(&self) -> i64 {
        Self::quantize(self.distance_from_origin)
    }

    pub fn new(normal: DVec3, distance_from_origin: f64, material_idx: usize) -> Self {
        Self {
            normal,
            distance_from_origin,
            material_idx,
        }
    }

    pub fn from_points(a: DVec3, b: DVec3, c: DVec3, material_index: usize) -> Self {
        let normal = (b - a).cross(c - a).normalize();
        Self::new(normal, normal.dot(a), material_index)
    }

    pub fn flip(&mut self) {
        self.normal = -self.normal;
        self.distance_from_origin = -self.distance_from_origin;
    }

    pub fn split_polygon(
        &self,
        polygon: &Polygon,
    ) -> (Vec<Polygon>, Vec<Polygon>, Vec<Polygon>, Vec<Polygon>) {
        let mut coplanar_front = Vec::new();
        let mut coplanar_back = Vec::new();
        let mut front = Vec::new();
        let mut back = Vec::new();

        let mut polygon_type = PolygonType::Coplanar;
        let mut types = Vec::with_capacity(polygon.vertices.len());

        for vertex in &polygon.vertices {
            let t = self.normal.dot(vertex.pos) - self.distance_from_origin;
            let typ = if t < -Self::EPSILON {
                PolygonType::Back
            } else if t > Self::EPSILON {
                PolygonType::Front
            } else {
                PolygonType::Coplanar
            };
            polygon_type = polygon_type | typ;
            types.push(typ);
        }

        match polygon_type {
            PolygonType::Coplanar => {
                if self.normal.dot(polygon.surface.normal) > 0.0 {
                    coplanar_front.push(polygon.clone());
                } else {
                    coplanar_back.push(polygon.clone());
                }
            }
            PolygonType::Front => front.push(polygon.clone()),
            PolygonType::Back => back.push(polygon.clone()),
            PolygonType::Spanning => {
                let mut f = Vec::new();
                let mut b = Vec::new();
                for i in 0..polygon.vertices.len() {
                    let j = (i + 1) % polygon.vertices.len();
                    let ti = types[i];
                    let tj = types[j];
                    let vi = &polygon.vertices[i];
                    let vj = &polygon.vertices[j];
                    if ti != PolygonType::Back {
                        f.push(vi.clone());
                    }
                    if ti != PolygonType::Front {
                        b.push(if ti != PolygonType::Back {
                            vi.clone()
                        } else {
                            vi.clone()
                        });
                    }
                    if (ti as u8 | tj as u8) == PolygonType::Spanning as u8 {
                        let t = (self.distance_from_origin - self.normal.dot(vi.pos))
                            / self.normal.dot(vj.pos - vi.pos);
                        let v = vi.interpolate(vj, t);
                        f.push(v.clone());
                        b.push(v);
                    }
                }
                if f.len() >= 3 {
                    front.push(Polygon::new(f, polygon.surface.material_idx));
                }
                if b.len() >= 3 {
                    back.push(Polygon::new(b, polygon.surface.material_idx));
                }
            }
        }

        (coplanar_front, coplanar_back, front, back)
    }

    /// Computes UV coordinates for a point on the plane.
    pub fn compute_uv(&self, point: DVec3) -> DVec2 {
        let (u_axis, v_axis) = Self::compute_uv_axes(&self.normal);
        let projected = point - self.normal * self.distance_from_origin;
        DVec2::new(projected.dot(u_axis), projected.dot(v_axis))
    }

    /// Computes UV axes for the plane.
    fn compute_uv_axes(normal: &DVec3) -> (DVec3, DVec3) {
        let up = if normal.x.abs() < 0.9 {
            DVec3::X
        } else {
            DVec3::Y
        };
        let u_axis = up.cross(*normal).normalize();
        let v_axis = normal.cross(u_axis);
        (u_axis, v_axis)
    }

    pub fn transform(&self, transform: DAffine3) -> Self {
        let normal = transform.transform_vector3(self.normal);
        let distance_from_origin = self.distance_from_origin + normal.dot(transform.translation);
        Self::new(normal, distance_from_origin, self.material_idx)
    }
}
