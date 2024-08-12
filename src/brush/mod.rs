use glam::DVec3;

pub mod types;

use types::*;

const EPSILON: f64 = 1e-5;

#[derive(Clone)]
pub struct Brush {
    pub polygons: Vec<Polygon>,
}

impl Brush {
    pub fn new() -> Self {
        Self {
            polygons: Vec::new(),
        }
    }

    pub fn cuboid(origin: DVec3, dimensions: DVec3) -> Self {
        let half_dims = dimensions * 0.5;
        let planes = vec![
            // Right
            Surface::new(DVec3::new(1.0, 0.0, 0.0), half_dims.x + origin.x),
            // Left
            Surface::new(DVec3::new(-1.0, 0.0, 0.0), half_dims.x - origin.x),
            // Top
            Surface::new(DVec3::new(0.0, 1.0, 0.0), half_dims.y + origin.y),
            // Bottom
            Surface::new(DVec3::new(0.0, -1.0, 0.0), half_dims.y - origin.y),
            // Front
            Surface::new(DVec3::new(0.0, 0.0, 1.0), half_dims.z + origin.z),
            // Back
            Surface::new(DVec3::new(0.0, 0.0, -1.0), half_dims.z - origin.z),
        ];

        Self::from_surfaces(planes)
    }

    /// Creates a CSG object from a list of polygons.
    pub fn from_polygons(polygons: Vec<Polygon>) -> Self {
        Self { polygons }
    }

    /// Creates a CSG object from a list of planes.
    pub fn from_surfaces(planes: Vec<Surface>) -> Self {
        let vertices = Self::generate_vertices(&planes);
        let polygons = Self::triangulate_polygons(&planes, &vertices);
        Self { polygons }
    }

    /// Converts the CSG object to a list of polygons.
    pub fn to_polygons(&self) -> Vec<Polygon> {
        self.polygons.clone()
    }

    /// Combines two CSG objects together.
    pub fn union(&self, csg: &Brush) -> Brush {
        let mut a = Node::new(self.clone().polygons);
        let mut b = Node::new(csg.clone().polygons);
        a.clip_to(&b);
        b.clip_to(&a);
        b.invert();
        b.clip_to(&a);
        b.invert();
        a.build(b.all_polygons());
        Brush::from_polygons(a.all_polygons())
    }

    /// Subtracts the passed in CSG from the current CSG.
    pub fn subtract(&self, csg: &Brush) -> Brush {
        let mut a = Node::new(self.clone().polygons);
        let mut b = Node::new(csg.clone().polygons);
        a.invert();
        a.clip_to(&b);
        b.clip_to(&a);
        b.invert();
        b.clip_to(&a);
        b.invert();
        a.build(b.all_polygons());
        a.invert();
        Brush::from_polygons(a.all_polygons())
    }

    /// Intersects this CSG with another CSG object.
    pub fn intersect(&self, csg: &Brush) -> Brush {
        let mut a = Node::new(self.clone().polygons);
        let mut b = Node::new(csg.clone().polygons);
        a.invert();
        b.clip_to(&a);
        b.invert();
        a.clip_to(&b);
        b.clip_to(&a);
        a.build(b.all_polygons());
        a.invert();
        Brush::from_polygons(a.all_polygons())
    }

    /// Cuts the CSG object with a plane, discarding anything behind the plane.
    pub fn knife(&mut self, plane: Plane) -> Self {
        const LARGE_VALUE: f64 = 1e6;

        // create a cuboid with it's front face being the passed in plane

        let mut u = if plane.normal.x.abs() > plane.normal.y.abs() {
            DVec3::new(0.0, 1.0, 0.0)
        } else {
            DVec3::new(1.0, 0.0, 0.0)
        };
        u = u.cross(plane.normal).normalize();
        let v = plane.normal.cross(u).normalize();

        let front_plane = Surface::new(-plane.normal, -plane.distance);
        let back_plane = Surface::new(plane.normal, plane.distance + LARGE_VALUE);
        let planes = vec![
            front_plane,                   // Cutting plane
            back_plane,                    // Far side plane
            Surface::new(u, LARGE_VALUE),  // Right plane
            Surface::new(-u, LARGE_VALUE), // Left plane
            Surface::new(v, LARGE_VALUE),  // Top plane
            Surface::new(-v, LARGE_VALUE), // Bottom plane
        ];

        let cutting_cuboid = Brush::from_surfaces(planes);
        self.subtract(&cutting_cuboid)
    }

    /// Returns the inverse of the CSG object.
    pub fn inverse(&self) -> Brush {
        let mut csg = self.clone();
        for p in &mut csg.polygons {
            p.flip();
        }
        csg
    }

    /// Generates a list of vertices from a list of planes.
    fn generate_vertices(planes: &[Surface]) -> Vec<DVec3> {
        let mut vertices = Vec::new();

        let plane_count = planes.len();

        for i in 0..plane_count {
            for j in (i + 1)..plane_count {
                for k in (j + 1)..plane_count {
                    if let Some(point) =
                        Self::threeway_intersection(&planes[i], &planes[j], &planes[k])
                    {
                        // ensure the point is inside all planes and is unique
                        if planes
                            .iter()
                            .all(|p| p.normal.dot(point) <= p.distance + EPSILON)
                        {
                            if !vertices
                                .iter()
                                .any(|v: &DVec3| (*v - point).length() < EPSILON)
                            {
                                vertices.push(point);
                            }
                        }
                    }
                }
            }
        }

        // remove duplicate vertices
        vertices.dedup_by(|a, b| (*a - *b).length() < EPSILON);

        vertices
    }

    /// Finds the intersection point of three planes.
    fn threeway_intersection(p1: &Surface, p2: &Surface, p3: &Surface) -> Option<DVec3> {
        let n1 = &p1.normal;
        let n2 = &p2.normal;
        let n3 = &p3.normal;

        let denom = n1.dot(n2.cross(*n3));

        if denom.abs() < EPSILON {
            return None;
        }

        let p = (n2.cross(*n3) * p1.distance
            + n3.cross(*n1) * p2.distance
            + n1.cross(*n2) * p3.distance)
            / denom;

        Some(p)
    }

    /// Triangulates a set of vertices based on a set of planes, creating a list of polygons.
    fn triangulate_polygons(planes: &[Surface], vertices: &[DVec3]) -> Vec<Polygon> {
        let mut polygons = Vec::new();

        for plane in planes {
            let mut polygon_vertices: Vec<Vertex> = vertices
                .iter()
                .filter(|&&v| (plane.normal.dot(v) - plane.distance).abs() < EPSILON)
                .map(|&v| Vertex {
                    pos: v,
                    normal: plane.normal,
                })
                .collect();

            if polygon_vertices.len() >= 3 {
                // Find the centroid of the polygon for sorting purposes
                let center = polygon_vertices.iter().map(|v| v.pos).sum::<DVec3>()
                    / polygon_vertices.len() as f64;

                // Sort vertices around the center of the polygon
                polygon_vertices.sort_by(|a, b| {
                    let va = a.pos - center;
                    let vb = b.pos - center;

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

                // Triangulate the polygon
                for i in 1..(polygon_vertices.len() - 1) {
                    let v0 = polygon_vertices[0].pos;
                    let v1 = polygon_vertices[i].pos;
                    let v2 = polygon_vertices[i + 1].pos;

                    // Check if the winding order is correct, if not, swap vertices
                    let cross = (v1 - v0).cross(v2 - v0);
                    let triangle_vertices = if cross.dot(plane.normal) < 0.0 {
                        vec![
                            polygon_vertices[0],
                            polygon_vertices[i + 1],
                            polygon_vertices[i],
                        ]
                    } else {
                        vec![
                            polygon_vertices[0],
                            polygon_vertices[i],
                            polygon_vertices[i + 1],
                        ]
                    };

                    polygons.push(Polygon {
                        vertices: triangle_vertices,
                        surface: plane.clone(),
                        shared: 0,
                    });
                }
            }
        }

        polygons
    }
}

#[derive(Clone)]
pub struct Node {
    plane: Option<Surface>,
    front: Option<Box<Node>>,
    back: Option<Box<Node>>,
    polygons: Vec<Polygon>,
}

impl Node {
    pub fn new(polygons: Vec<Polygon>) -> Self {
        let mut node = Self {
            plane: None,
            front: None,
            back: None,
            polygons: Vec::new(),
        };
        node.build(polygons);
        node
    }

    pub fn invert(&mut self) {
        for p in &mut self.polygons {
            p.flip();
        }
        if let Some(plane) = &mut self.plane {
            plane.flip();
        }
        if let Some(front) = &mut self.front {
            front.invert();
        }
        if let Some(back) = &mut self.back {
            back.invert();
        }
        std::mem::swap(&mut self.front, &mut self.back);
    }

    pub fn clip_polygons(&self, polygons: Vec<Polygon>) -> Vec<Polygon> {
        if self.plane.is_none() {
            return polygons;
        }
        let mut front = Vec::new();
        let mut back = Vec::new();
        for p in polygons {
            self.plane.as_ref().unwrap().split_polygon(
                &p,
                &mut Vec::new(),
                &mut Vec::new(),
                &mut front,
                &mut back,
            );
        }
        if let Some(f) = &self.front {
            front = f.clip_polygons(front);
        }
        if let Some(b) = &self.back {
            back = b.clip_polygons(back);
        } else {
            back = Vec::new();
        }
        front.extend(back);
        front
    }

    pub fn clip_to(&mut self, bsp: &Node) {
        self.polygons = bsp.clip_polygons(self.polygons.clone());
        if let Some(front) = &mut self.front {
            front.clip_to(bsp);
        }
        if let Some(back) = &mut self.back {
            back.clip_to(bsp);
        }
    }

    pub fn all_polygons(&self) -> Vec<Polygon> {
        let mut polygons = self.polygons.clone();
        if let Some(front) = &self.front {
            polygons.extend(front.all_polygons());
        }
        if let Some(back) = &self.back {
            polygons.extend(back.all_polygons());
        }
        polygons
    }

    pub fn build(&mut self, polygons: Vec<Polygon>) {
        if polygons.is_empty() {
            return;
        }

        if self.plane.is_none() {
            self.plane = Some(polygons[0].surface.clone());
        }

        let mut front = Vec::new();
        let mut back = Vec::new();
        let mut coplanar_front = Vec::new();
        let mut coplanar_back = Vec::new();

        for p in polygons {
            self.plane.as_ref().unwrap().split_polygon(
                &p,
                &mut coplanar_front,
                &mut coplanar_back,
                &mut front,
                &mut back,
            );
        }

        self.polygons.extend(coplanar_front);
        self.polygons.extend(coplanar_back);

        if !front.is_empty() {
            if self.front.is_none() {
                self.front = Some(Box::new(Node::new(Vec::new())));
            }
            self.front.as_mut().unwrap().build(front);
        }

        if !back.is_empty() {
            if self.back.is_none() {
                self.back = Some(Box::new(Node::new(Vec::new())));
            }
            self.back.as_mut().unwrap().build(back);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_cube() {
        let origin = DVec3::new(0.0, 0.0, 0.0);
        let dimensions = DVec3::new(1.0, 1.0, 1.0);
        let cube = Brush::cuboid(origin, dimensions);

        // Therte should be 8 vertices in a cube
        let expected_vertices = vec![
            DVec3::new(-0.5, -0.5, -0.5),
            DVec3::new(-0.5, -0.5, 0.5),
            DVec3::new(-0.5, 0.5, -0.5),
            DVec3::new(-0.5, 0.5, 0.5),
            DVec3::new(0.5, -0.5, -0.5),
            DVec3::new(0.5, -0.5, 0.5),
            DVec3::new(0.5, 0.5, -0.5),
            DVec3::new(0.5, 0.5, 0.5),
        ];

        let mut result_vertices: Vec<DVec3> = cube
            .polygons
            .iter()
            .flat_map(|polygon| polygon.vertices.iter().map(|v| v.pos))
            .collect();

        // Remove duplicates and sort for consistent comparison
        result_vertices.sort_by(|a, b| {
            a.x.partial_cmp(&b.x)
                .unwrap()
                .then(a.y.partial_cmp(&b.y).unwrap())
                .then(a.z.partial_cmp(&b.z).unwrap())
        });
        result_vertices.dedup();

        // Check if all expected vertices are present in the result
        for expected_vertex in &expected_vertices {
            assert!(
                result_vertices.iter().any(|&v| v == *expected_vertex),
                "Expected vertex {:?} not found in result",
                expected_vertex
            );
        }

        // Check if the number of vertices matches
        assert_eq!(
            result_vertices.len(),
            expected_vertices.len(),
            "Number of vertices doesn't match. Expected {}, got {}",
            expected_vertices.len(),
            result_vertices.len()
        );

        // Check if the number of polygons is correct (12 triangles for a cube)
        assert_eq!(
            cube.polygons.len(),
            12,
            "Number of polygons doesn't match. Expected 12, got {}",
            cube.polygons.len()
        );
    }

    #[test]
    fn test_cube_subtraction() {
        let origin = DVec3::new(0.0, 0.0, 0.0);
        let dimensions = DVec3::new(1.0, 1.0, 1.0);
        let cube = Brush::cuboid(origin, dimensions);

        let origin2 = DVec3::new(0.5, 0.5, 0.5);
        let dimensions2 = DVec3::new(1.0, 1.0, 1.0);
        let cube2 = Brush::cuboid(origin2, dimensions2);

        let result = cube.subtract(&cube2);

        // there should be 14 vertices in the result
        let expected_vertices = vec![
            DVec3::new(-0.5, -0.5, -0.5),
            DVec3::new(-0.5, -0.5, 0.5),
            DVec3::new(-0.5, 0.5, -0.5),
            DVec3::new(-0.5, 0.5, 0.5),
            DVec3::new(0.5, -0.5, -0.5),
            DVec3::new(0.5, -0.5, 0.5),
            DVec3::new(0.5, 0.5, -0.5),
            DVec3::new(0.0, 0.0, 0.5),
            DVec3::new(0.0, 0.5, 0.0),
            DVec3::new(0.0, 0.5, 0.5),
            DVec3::new(0.5, 0.0, 0.0),
            DVec3::new(0.5, 0.0, 0.5),
            DVec3::new(0.5, 0.5, 0.0),
        ];

        // Extract vertices from the resulting CSG
        let mut result_vertices: Vec<DVec3> = result
            .polygons
            .iter()
            .flat_map(|polygon| polygon.vertices.iter().map(|v| v.pos))
            .collect();

        // Remove duplicates and sort for consistent comparison
        result_vertices.sort_by(|a, b| {
            a.x.partial_cmp(&b.x)
                .unwrap()
                .then(a.y.partial_cmp(&b.y).unwrap())
                .then(a.z.partial_cmp(&b.z).unwrap())
        });

        // Check if all expected vertices are present in the result
        for expected_vertex in &expected_vertices {
            assert!(
                result_vertices.iter().any(|&v| v == *expected_vertex),
                "Expected vertex {:?} not found in result",
                expected_vertex
            );
        }
    }
}
