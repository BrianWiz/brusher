use crate::math::{CuboidDimensions, CylinderDimensions, Vector3};

pub mod primitives;

const EPSILON: f64 = 1e-6;

pub enum SurfaceOperation {
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
    ) -> Vector3 {
        let n1 = &p1.normal;
        let n2 = &p2.normal;
        let n3 = &p3.normal;

        let n1n2 = n1.cross(&n2);
        let n2n3 = n2.cross(&n3);
        let n3n1 = n3.cross(&n1);

        let denom = n1.dot(&n2n3);

        let p1d = p1.distance;
        let p2d = p2.distance;
        let p3d = p3.distance;

        let p = n2n3 * p1d + n3n1 * p2d + n1n2 * p3d;

        (p / denom).into()
    }
}

pub struct SurfaceGroup {
    pub planes: Vec<SurfacePlane>,
    pub operation: SurfaceOperation,
}

impl SurfaceGroup {
    pub fn new(planes: Vec<SurfacePlane>) -> Self {
        SurfaceGroup {
            planes,
            operation: SurfaceOperation::Add,
        }
    }

    pub fn new_add(planes: Vec<SurfacePlane>) -> Self {
        SurfaceGroup {
            planes,
            operation: SurfaceOperation::Add,
        }
    }

    pub fn new_subtract(planes: Vec<SurfacePlane>) -> Self {
        SurfaceGroup {
            planes,
            operation: SurfaceOperation::Subtract,
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
        Brush { surface_groups, origin }
    }

    pub fn cuboid(origin: Vector3, dimensions: CuboidDimensions) -> Self {
        Brush::new(
            origin,
            vec![
                primitives::cuboid(Vector3::ZERO, dimensions, SurfaceOperation::Add),
            ])
    }

    pub fn cylinder(
        origin: Vector3,
        dimensions: CylinderDimensions,
        slices: u32,
    ) -> Self {
        Brush::new(
            origin,
            vec![
                primitives::cylinder(Vector3::ZERO, dimensions, slices, SurfaceOperation::Add),
            ])
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
    let mut vertices = Vec::new();

    for group in &brush.surface_groups {
        let planes = &group.planes;
        let plane_count = planes.len();

        for i in 0..plane_count {
            for j in (i + 1)..plane_count {
                for k in (j + 1)..plane_count {
                    let point = SurfacePlane::threeway_intersection(&planes[i], &planes[j], &planes[k]);

                    // ensure the point is inside all planes and is unique
                    if planes.iter().all(|p| p.normal.dot(&point) <= p.distance + EPSILON) {
                        if !vertices.iter().any(|v: &Vector3| (*v - point).magnitude() < EPSILON) {
                            vertices.push(point);
                        }
                    }
                }
            }
        }
    }

    // remove duplicate vertices
    vertices.dedup_by(|a, b| (*a - *b).magnitude() < EPSILON);

    vertices
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
