use std::hash::{Hash, Hasher};

use glam::{dvec3, DVec2, DVec3};
mod util;

const EPSILON: f64 = 1e-5;

#[derive(Debug)]
pub enum BrushError {
    BrushletAtIndexDoesNotExist(usize),
}

#[derive(Debug, Clone, Copy)]
pub enum BrushletBooleanOp {
    Union,
    Intersect,
    Subtract,
}

#[derive(Debug, Clone, Copy)]
pub enum BrushletOp {
    Knife(Knife),
}

#[derive(Debug, Clone)]
pub struct MeshData {
    pub polygons: Vec<Polygon>,
}

#[derive(Debug, Clone)]
pub struct Brush {
    brushlets: Vec<Brushlet>,
    pub knives: Vec<Knife>,
}

impl Brush {
    pub fn new() -> Self {
        Self {
            brushlets: Vec::new(),
            knives: Vec::new(),
        }
    }

    pub fn select(&self, idx: usize) -> Result<&Brushlet, BrushError> {
        if idx >= self.brushlets.len() {
            return Err(BrushError::BrushletAtIndexDoesNotExist(idx));
        }
        Ok(&self.brushlets[idx])
    }

    pub fn add(&mut self, brushlet: Brushlet) {
        self.brushlets.push(brushlet);
    }

    pub fn update(&mut self, idx: usize, brushlet: Brushlet) -> Result<(), BrushError> {
        if idx >= self.brushlets.len() {
            return Err(BrushError::BrushletAtIndexDoesNotExist(idx));
        }
        self.brushlets[idx] = brushlet;

        Ok(())
    }

    pub fn remove(&mut self, idx: usize) -> Result<(), BrushError> {
        if idx >= self.brushlets.len() {
            return Err(BrushError::BrushletAtIndexDoesNotExist(idx));
        }
        self.brushlets.remove(idx);

        Ok(())
    }

    pub fn to_mesh_data(&self) -> MeshData {
        if self.brushlets.is_empty() {
            return MeshData {
                polygons: Vec::new(),
            };
        }

        let mut final_brushlet = self.brushlets[0].clone();

        for other in self.brushlets.iter().skip(1) {
            final_brushlet = match other.operation {
                BrushletBooleanOp::Union => final_brushlet.union(other),
                BrushletBooleanOp::Intersect => final_brushlet.intersect(other),
                BrushletBooleanOp::Subtract => final_brushlet.subtract(other),
            };
        }

        // do the final global knife operations
        for knife in &self.knives {
            final_brushlet = final_brushlet.knife(*knife);
        }

        final_brushlet.to_mesh_data()
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

#[derive(Debug, Clone, Copy)]
pub struct Knife {
    pub normal: DVec3,
    pub distance_from_origin: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct Surface {
    normal: DVec3,
    distance_from_origin: f64,
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

    pub fn new(normal: DVec3, w: f64) -> Self {
        Self {
            normal,
            distance_from_origin: w,
        }
    }

    pub fn from_points(a: DVec3, b: DVec3, c: DVec3) -> Self {
        let normal = (b - a).cross(c - a).normalize();
        Self::new(normal, normal.dot(a))
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

        let mut polygon_type = 0;
        let mut types = Vec::with_capacity(polygon.vertices.len());

        for vertex in &polygon.vertices {
            let t = self.normal.dot(vertex.pos) - self.distance_from_origin;
            let typ = if t < -EPSILON {
                2
            } else if t > EPSILON {
                1
            } else {
                0
            };
            polygon_type |= typ;
            types.push(typ);
        }

        match polygon_type {
            0 => {
                if self.normal.dot(polygon.surface.normal) > 0.0 {
                    coplanar_front.push(polygon.clone());
                } else {
                    coplanar_back.push(polygon.clone());
                }
            }
            1 => front.push(polygon.clone()),
            2 => back.push(polygon.clone()),
            3 => {
                let mut f = Vec::new();
                let mut b = Vec::new();
                for i in 0..polygon.vertices.len() {
                    let j = (i + 1) % polygon.vertices.len();
                    let ti = types[i];
                    let tj = types[j];
                    let vi = &polygon.vertices[i];
                    let vj = &polygon.vertices[j];
                    if ti != 2 {
                        f.push(vi.clone());
                    }
                    if ti != 1 {
                        b.push(if ti != 2 { vi.clone() } else { vi.clone() });
                    }
                    if (ti | tj) == 3 {
                        let t = (self.distance_from_origin - self.normal.dot(vi.pos))
                            / self.normal.dot(vj.pos - vi.pos);
                        let v = vi.interpolate(vj, t);
                        f.push(v.clone());
                        b.push(v);
                    }
                }
                if f.len() >= 3 {
                    front.push(Polygon::new(f, polygon.material));
                }
                if b.len() >= 3 {
                    back.push(Polygon::new(b, polygon.material));
                }
            }
            _ => {}
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
}

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
struct Node {
    plane: Option<Surface>,
    front: Option<Box<Node>>,
    back: Option<Box<Node>>,
    polygons: Vec<Polygon>,
}

impl Node {
    fn new(polygons: Vec<Polygon>) -> Self {
        let mut node = Self {
            plane: None,
            front: None,
            back: None,
            polygons: Vec::new(),
        };
        node.build(polygons);
        node
    }

    fn invert(&mut self) {
        for polygon in &mut self.polygons {
            polygon.flip();
        }
        if let Some(ref mut plane) = self.plane {
            plane.flip();
        }
        if let Some(ref mut front) = self.front {
            front.invert();
        }
        if let Some(ref mut back) = self.back {
            back.invert();
        }
        std::mem::swap(&mut self.front, &mut self.back);
    }

    fn clip_polygons(&self, polygons: Vec<Polygon>) -> Vec<Polygon> {
        if self.plane.is_none() {
            return polygons;
        }

        let mut front = Vec::new();
        let mut back = Vec::new();

        for polygon in polygons {
            let (mut cp_front, mut cp_back, mut f, mut b) =
                self.plane.as_ref().unwrap().split_polygon(&polygon);
            front.append(&mut cp_front);
            front.append(&mut f);
            back.append(&mut cp_back);
            back.append(&mut b);
        }

        if let Some(ref f) = self.front {
            front = f.clip_polygons(front);
        }

        if let Some(ref b) = self.back {
            back = b.clip_polygons(back);
        } else {
            back.clear();
        }

        front.extend(back);
        front
    }

    fn clip_to(&mut self, bsp: &Node) {
        self.polygons = bsp.clip_polygons(self.polygons.clone());
        if let Some(ref mut front) = self.front {
            front.clip_to(bsp);
        }
        if let Some(ref mut back) = self.back {
            back.clip_to(bsp);
        }
    }

    fn all_polygons(&self) -> Vec<Polygon> {
        let mut polygons = self.polygons.clone();
        if let Some(ref front) = self.front {
            polygons.extend(front.all_polygons());
        }
        if let Some(ref back) = self.back {
            polygons.extend(back.all_polygons());
        }
        polygons
    }

    fn build(&mut self, mut polygons: Vec<Polygon>) {
        if polygons.is_empty() {
            return;
        }
        if self.plane.is_none() {
            self.plane = Some(polygons[0].surface.clone());
        }
        let plane = self.plane.as_ref().unwrap();
        let mut front = Vec::new();
        let mut back = Vec::new();

        for polygon in polygons.drain(..) {
            let (mut cp_front, mut cp_back, mut f, mut b) = plane.split_polygon(&polygon);
            self.polygons.append(&mut cp_front);
            self.polygons.append(&mut cp_back);
            front.append(&mut f);
            back.append(&mut b);
        }

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

/// A cuboid brushlet
///
/// # Fields
/// * `origin` - The origin of the cuboid
/// * `width` - The width of the cuboid (x-axis)
/// * `height` - The height of the cuboid (y-axis)
/// * `depth` - The depth of the cuboid (z-axis)
/// * `material` - The material index
/// * `operation` - The boolean operation to perform
#[derive(Debug, Clone)]
pub struct Cuboid {
    pub origin: DVec3,
    pub width: f64,
    pub height: f64,
    pub depth: f64,
    pub material: usize,
    pub operation: BrushletBooleanOp,
    pub knives: Vec<Knife>,
    pub inverted: bool,
}

#[derive(Debug, Clone)]
pub struct Brushlet {
    pub polygons: Vec<Polygon>,
    pub operation: BrushletBooleanOp,
    pub knives: Vec<Knife>,
    pub inverted: bool,
}

impl Brushlet {
    fn from_surfaces(
        surfaces: Vec<Surface>,
        operation: BrushletBooleanOp,
        knives: Vec<Knife>,
        inverted: bool,
    ) -> Self {
        let polygons = util::generate_polygons_from_surfaces(&surfaces);
        Self {
            polygons,
            operation,
            knives,
            inverted,
        }
    }

    fn union(&self, other: &Brushlet) -> Self {
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
            operation: self.operation,
            knives: self.knives.clone(),
            inverted: self.inverted,
        }
    }

    fn subtract(&self, other: &Brushlet) -> Self {
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
            operation: self.operation,
            knives: self.knives.clone(),
            inverted: self.inverted,
        }
    }

    fn intersect(&self, other: &Brushlet) -> Self {
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
            operation: self.operation,
            knives: self.knives.clone(),
            inverted: self.inverted,
        }
    }

    fn to_mesh_data(&self) -> MeshData {
        let mut final_brushlet = self.clone();

        for knife in &self.knives {
            final_brushlet = final_brushlet.knife(*knife);
        }

        if self.inverted {
            final_brushlet = final_brushlet.inverse();
        }

        MeshData {
            polygons: final_brushlet.polygons,
        }
    }

    /// Cuts the Brushlet with a plane, discarding anything in front of the plane.
    fn knife(&self, plane: Knife) -> Self {
        // Define a large value to ensure the cuboid encompasses the entire geometry
        const LARGE_VALUE: f64 = 1e5;

        // Create the primary cutting plane
        let cutting_plane = Surface::new(-plane.normal, -plane.distance_from_origin);

        // Create two orthogonal vectors to the plane normal
        let mut u = if plane.normal.x.abs() > plane.normal.y.abs() {
            DVec3::new(0.0, 1.0, 0.0)
        } else {
            DVec3::new(1.0, 0.0, 0.0)
        };
        u = u.cross(plane.normal).normalize();
        let v = plane.normal.cross(u).normalize();

        // Create the six planes that define the cutting cuboid
        let planes = vec![
            cutting_plane,
            Surface::new(plane.normal, plane.distance_from_origin + LARGE_VALUE), // Back plane, far behind the cut
            Surface::new(u, LARGE_VALUE), // Large plane in one direction
            Surface::new(-u, LARGE_VALUE), // Large plane in the opposite direction
            Surface::new(v, LARGE_VALUE), // Large plane in another direction
            Surface::new(-v, LARGE_VALUE), // Large plane in the opposite direction
        ];

        // Create the cutting cuboid from the defined planes
        let cutting_cuboid =
            Brushlet::from_surfaces(planes, self.operation, self.knives.clone(), self.inverted);

        // Intersect the original geometry with the inverted cutting cuboid
        self.subtract(&cutting_cuboid)
    }

    fn inverse(&self) -> Self {
        let mut csg = Brushlet {
            polygons: self.polygons.clone(),
            operation: self.operation,
            knives: self.knives.clone(),
            inverted: self.inverted,
        };
        for polygon in &mut csg.polygons {
            polygon.flip();
        }
        csg
    }

    pub fn cuboid(settings: Cuboid) -> Self {
        let half_width = settings.width * 0.5;
        let half_height = settings.height * 0.5;
        let half_depth = settings.depth * 0.5;

        let vertices = vec![
            // Define vertices without normals initially
            Vertex::new(
                settings.origin + dvec3(-half_width, -half_height, -half_depth),
                DVec3::ZERO,
            ),
            Vertex::new(
                settings.origin + dvec3(half_width, -half_height, -half_depth),
                DVec3::ZERO,
            ),
            Vertex::new(
                settings.origin + dvec3(half_width, half_height, -half_depth),
                DVec3::ZERO,
            ),
            Vertex::new(
                settings.origin + dvec3(-half_width, half_height, -half_depth),
                DVec3::ZERO,
            ),
            Vertex::new(
                settings.origin + dvec3(-half_width, -half_height, half_depth),
                DVec3::ZERO,
            ),
            Vertex::new(
                settings.origin + dvec3(half_width, -half_height, half_depth),
                DVec3::ZERO,
            ),
            Vertex::new(
                settings.origin + dvec3(half_width, half_height, half_depth),
                DVec3::ZERO,
            ),
            Vertex::new(
                settings.origin + dvec3(-half_width, half_height, half_depth),
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
                settings.material,
            ),
            Polygon::new(
                vec![
                    Vertex::new(vertices[0].pos, -DVec3::Z),
                    Vertex::new(vertices[3].pos, -DVec3::Z),
                    Vertex::new(vertices[2].pos, -DVec3::Z),
                    Vertex::new(vertices[1].pos, -DVec3::Z),
                ],
                settings.material,
            ),
            Polygon::new(
                vec![
                    Vertex::new(vertices[3].pos, DVec3::Y),
                    Vertex::new(vertices[7].pos, DVec3::Y),
                    Vertex::new(vertices[6].pos, DVec3::Y),
                    Vertex::new(vertices[2].pos, DVec3::Y),
                ],
                settings.material,
            ),
            Polygon::new(
                vec![
                    Vertex::new(vertices[0].pos, -DVec3::Y),
                    Vertex::new(vertices[1].pos, -DVec3::Y),
                    Vertex::new(vertices[5].pos, -DVec3::Y),
                    Vertex::new(vertices[4].pos, -DVec3::Y),
                ],
                settings.material,
            ),
            Polygon::new(
                vec![
                    Vertex::new(vertices[1].pos, DVec3::X),
                    Vertex::new(vertices[2].pos, DVec3::X),
                    Vertex::new(vertices[6].pos, DVec3::X),
                    Vertex::new(vertices[5].pos, DVec3::X),
                ],
                settings.material,
            ),
            Polygon::new(
                vec![
                    Vertex::new(vertices[0].pos, -DVec3::X),
                    Vertex::new(vertices[4].pos, -DVec3::X),
                    Vertex::new(vertices[7].pos, -DVec3::X),
                    Vertex::new(vertices[3].pos, -DVec3::X),
                ],
                settings.material,
            ),
        ];

        Brushlet {
            polygons,
            operation: settings.operation,
            knives: settings.knives,
            inverted: settings.inverted,
        }
    }
}
