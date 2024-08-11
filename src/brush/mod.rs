use crate::math::{CuboidDimensions, CylinderDimensions, Vector3};

pub mod primitives;

const EPSILON: f64 = 1e-6;

pub enum BoolOperation {
    Add,
    Subtract,
}

pub struct Triangle {
    pub vertices: [Vector3; 3],
    pub normal: Vector3,
}

pub struct Face {
    pub triangles: Vec<Triangle>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfacePlane {
    pub normal: Vector3,
    pub distance: f64,
}

impl SurfacePlane {
    pub fn new(normal: Vector3, distance: f64) -> Self {
        SurfacePlane { normal, distance }
    }

    pub fn threeway_intersection(
        p1: &SurfacePlane,
        p2: &SurfacePlane,
        p3: &SurfacePlane,
    ) -> Option<Vector3> {
        let n1 = &p1.normal;
        let n2 = &p2.normal;
        let n3 = &p3.normal;

        let denom = n1.dot(&n2.cross(n3));

        if denom.abs() < EPSILON {
            // The planes are parallel or nearly parallel
            return None;
        }

        let p =
            (n2.cross(n3) * p1.distance + n3.cross(n1) * p2.distance + n1.cross(n2) * p3.distance)
                / denom;

        Some(p)
    }
}

pub struct SurfaceGroup {
    pub planes: Vec<SurfacePlane>,
    pub operation: BoolOperation,
}

impl SurfaceGroup {
    pub fn new(planes: Vec<SurfacePlane>) -> Self {
        SurfaceGroup {
            planes,
            operation: BoolOperation::Add,
        }
    }

    pub fn new_add(planes: Vec<SurfacePlane>) -> Self {
        SurfaceGroup {
            planes,
            operation: BoolOperation::Add,
        }
    }

    pub fn new_subtract(planes: Vec<SurfacePlane>) -> Self {
        SurfaceGroup {
            planes,
            operation: BoolOperation::Subtract,
        }
    }

    pub fn knife(&mut self, plane: SurfacePlane) {
        self.planes.push(plane);
    }
}

pub struct MeshData {
    pub positions: Vec<Vector3>,
    pub normals: Vec<Vector3>,
    pub indices: Vec<u32>,
}

pub struct Brush {
    pub surface_groups: Vec<SurfaceGroup>,
    pub origin: Vector3,
}

impl Brush {
    pub fn new(origin: Vector3, surface_groups: Vec<SurfaceGroup>) -> Self {
        Brush {
            surface_groups,
            origin,
        }
    }

    pub fn concave_polygon(origin: Vector3) -> Self {
        Brush::new(
            origin,
            vec![primitives::concave_polygon(
                Vector3::ZERO,
                Vector3::new(1.0, 0.5, 1.0),
                BoolOperation::Add,
            )],
        )
    }

    pub fn cuboid(origin: Vector3, dimensions: CuboidDimensions) -> Self {
        Brush::new(
            origin,
            vec![primitives::cuboid(
                Vector3::ZERO,
                dimensions,
                BoolOperation::Add,
            )],
        )
    }

    pub fn cylinder(origin: Vector3, dimensions: CylinderDimensions, slices: u32) -> Self {
        Brush::new(
            origin,
            vec![primitives::cylinder(
                Vector3::ZERO,
                dimensions,
                slices,
                BoolOperation::Add,
            )],
        )
    }

    pub fn knife(&mut self, plane: SurfacePlane) -> &Self {
        for group in &mut self.surface_groups {
            group.knife(plane);
        }

        self
    }

    pub fn to_mesh_data(&self) -> MeshData {
        let vertices = generate_vertices(self);
        let faces = triangulate_faces(self, &vertices);

        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut indices = Vec::new();

        for face in faces {
            for triangle in face.triangles {
                let start_index = positions.len() as u32;

                for vertex in &triangle.vertices {
                    positions.push(*vertex + self.origin);
                    normals.push(triangle.normal);
                }

                indices.extend_from_slice(&[start_index, start_index + 1, start_index + 2]);
            }
        }

        MeshData {
            positions,
            normals,
            indices,
        }
    }
}

fn generate_vertices(brush: &Brush) -> Vec<Vector3> {
    const EPSILON: f64 = 1e-6;

    let mut all_vertices = Vec::new();

    for group in &brush.surface_groups {
        let mut potential_vertices = Vec::new();
        let mut intersecting_planes = Vec::new();

        let planes = &group.planes;
        let plane_count = planes.len();

        for i in 0..plane_count {
            for j in (i + 1)..plane_count {
                for k in (j + 1)..plane_count {
                    if let Some(point) =
                        SurfacePlane::threeway_intersection(&planes[i], &planes[j], &planes[k])
                    {
                        // Check if the point is on all three intersecting planes
                        let on_intersection = [i, j, k].iter().all(|&idx| {
                            (planes[idx].normal.dot(&point) - planes[idx].distance).abs() < EPSILON
                        });

                        if on_intersection {
                            potential_vertices.push(point);
                            intersecting_planes.push((&planes[i], &planes[j], &planes[k]));
                        }
                    }
                }
            }
        }

        // Filter out invalid vertices
        let valid_vertices: Vec<Vector3> = potential_vertices
            .into_iter()
            .filter(|point| !is_outside_nearest_intersecting_planes(point, &intersecting_planes))
            .collect();

        println!("Total vertices found: {}", valid_vertices.len());
        for (i, v) in valid_vertices.iter().enumerate() {
            println!("Vertex {}: {:?}", i, v);
        }

        all_vertices.extend(valid_vertices);
    }

    all_vertices
}

fn is_outside_nearest_intersecting_planes(
    point: &Vector3,
    intersection_planes: &Vec<(&SurfacePlane, &SurfacePlane, &SurfacePlane)>,
) -> bool {
    let (a, b, c) = nearest_intersecting_planes(point, intersection_planes);

    let normal = (b.normal - a.normal).cross(&(c.normal - a.normal));
    let to_point = *point - a.normal * a.distance;
    let distance = to_point.dot(&normal);

    distance < 0.0
}

fn nearest_intersecting_planes(
    point: &Vector3,
    intersection_planes: &Vec<(&SurfacePlane, &SurfacePlane, &SurfacePlane)>,
) -> (SurfacePlane, SurfacePlane, SurfacePlane) {
    let mut min_distance = f64::INFINITY;
    let mut nearest_planes = (
        SurfacePlane::new(Vector3::ZERO, 0.0),
        SurfacePlane::new(Vector3::ZERO, 0.0),
        SurfacePlane::new(Vector3::ZERO, 0.0),
    );

    for (a, b, c) in intersection_planes {
        let distance =
            distance_to_plane(point, a) + distance_to_plane(point, b) + distance_to_plane(point, c);

        if distance < min_distance {
            min_distance = distance;
            nearest_planes = (**a, **b, **c);
        }
    }

    nearest_planes
}

fn distance_to_plane(point: &Vector3, plane: &SurfacePlane) -> f64 {
    (plane.normal.dot(point) - plane.distance).abs()
}

fn triangulate_faces(brush: &Brush, vertices: &[Vector3]) -> Vec<Face> {
    let mut faces = Vec::new();

    for group in &brush.surface_groups {
        for plane in &group.planes {
            let mut face_vertices = Vec::new();

            for (i, &vertex) in vertices.iter().enumerate() {
                if (plane.normal.dot(&vertex) - plane.distance).abs() < EPSILON {
                    face_vertices.push(i);
                }
            }

            if face_vertices.len() >= 3 {
                // Find the centroid of the face for sorting purposes
                let center = face_vertices
                    .iter()
                    .map(|&i| vertices[i])
                    .fold(Vector3::ZERO, |acc, p| acc + p)
                    / face_vertices.len() as f64;

                // Sort vertices around the center using a consistent plane projection
                face_vertices.sort_by(|&a, &b| {
                    let va = vertices[a] - center;
                    let vb = vertices[b] - center;

                    // Project onto the plane with the smallest component in the normal vector
                    let (angle_a, angle_b) = if plane.normal.z.abs() > plane.normal.x.abs()
                        && plane.normal.z.abs() > plane.normal.y.abs()
                    {
                        // Project onto the XY plane
                        (va.y.atan2(va.x), vb.y.atan2(vb.x))
                    } else if plane.normal.x.abs() > plane.normal.y.abs() {
                        // Project onto the YZ plane
                        (va.z.atan2(va.y), vb.z.atan2(vb.y))
                    } else {
                        // Project onto the XZ plane
                        (va.z.atan2(va.x), vb.z.atan2(vb.x))
                    };

                    angle_a
                        .partial_cmp(&angle_b)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                let mut triangles = Vec::new();

                // Ensure consistent winding order by checking cross product
                for i in 1..(face_vertices.len() - 1) {
                    let v0 = vertices[face_vertices[0]];
                    let v1 = vertices[face_vertices[i]];
                    let v2 = vertices[face_vertices[i + 1]];

                    // Check if the winding order is correct, if not, swap vertices
                    let cross = (v1 - v0).cross(&(v2 - v0));
                    if cross.dot(&plane.normal) < 0.0 {
                        triangles.push(Triangle {
                            vertices: [v0, v2, v1],
                            normal: plane.normal,
                        });
                    } else {
                        triangles.push(Triangle {
                            vertices: [v0, v1, v2],
                            normal: plane.normal,
                        });
                    }
                }

                faces.push(Face { triangles });
            }
        }
    }

    faces
}
