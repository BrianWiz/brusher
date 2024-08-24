use std::collections::HashMap;

#[cfg(feature = "bevy")]
use bevy::math::DVec3;

#[cfg(not(feature = "bevy"))]
use glam::DVec3;

use super::{
    polygon::{Polygon, Vertex},
    surface::Surface,
};

/// Generates vertices from a list of planes.
pub(crate) fn generate_polygons_from_surfaces(planes: &[Surface]) -> Vec<Polygon> {
    let plane_vertex_map = generate_vertices(planes);
    let mut polygons = Vec::new();

    for (surface, vertices) in plane_vertex_map {
        if vertices.len() < 3 {
            continue;
        }

        let mut polygon_vertices = vertices;

        // Sort vertices to ensure consistent winding order
        let center =
            polygon_vertices.iter().map(|v| v.pos).sum::<DVec3>() / polygon_vertices.len() as f64;
        polygon_vertices.sort_by(|a, b| {
            let va = a.pos - center;
            let vb = b.pos - center;
            surface
                .normal
                .cross(va)
                .dot(vb)
                .partial_cmp(&0.0)
                .unwrap()
                .reverse()
        });

        if polygon_vertices.len() >= 3 {
            polygons.push(Polygon {
                vertices: polygon_vertices,
                surface,
            });
        }
    }

    polygons
}

/// Generates vertices from a list of planes, grouped by plane.
pub(crate) fn generate_vertices(planes: &[Surface]) -> HashMap<Surface, Vec<Vertex>> {
    let plane_count = planes.len();
    let mut plane_vertex_map = HashMap::new();

    for i in 0..plane_count {
        for j in (i + 1)..plane_count {
            for k in (j + 1)..plane_count {
                if let Some(point) = threeway_intersection(&planes[i], &planes[j], &planes[k]) {
                    // Ensure the point is inside or on all planes
                    if planes
                        .iter()
                        .all(|p| p.normal.dot(point) <= p.distance_from_origin + Surface::EPSILON)
                    {
                        // Add the point to each of the three intersecting planes
                        for plane in [&planes[i], &planes[j], &planes[k]] {
                            let vertices = plane_vertex_map
                                .entry(plane.clone())
                                .or_insert_with(Vec::new);

                            // Ensure the point is unique for this plane
                            if !vertices.iter().any(|v: &Vertex| {
                                (v.pos - point).length_squared()
                                    < Surface::EPSILON * Surface::EPSILON
                            }) {
                                vertices.push(Vertex {
                                    pos: point,
                                    normal: plane.normal,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    plane_vertex_map
}

/// Finds the intersection point of three planes.
pub(crate) fn threeway_intersection(p1: &Surface, p2: &Surface, p3: &Surface) -> Option<DVec3> {
    let n1 = &p1.normal;
    let n2 = &p2.normal;
    let n3 = &p3.normal;

    let denom = n1.dot(n2.cross(*n3));

    if denom.abs() < Surface::EPSILON {
        return None;
    }

    let p = (n2.cross(*n3) * p1.distance_from_origin
        + n3.cross(*n1) * p2.distance_from_origin
        + n1.cross(*n2) * p3.distance_from_origin)
        / denom;

    Some(p)
}
