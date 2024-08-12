use glam::DVec3;

#[derive(Clone, Copy)]
pub struct Plane {
    pub normal: DVec3,
    pub distance: f64,
    pub u: DVec3,
    pub v: DVec3,
}

impl Plane {
    pub const EPSILON: f64 = 1e-5;

    pub fn new(normal: DVec3, distance: f64) -> Self {
        Self {
            normal,
            distance,
            u: DVec3::ZERO,
            v: DVec3::ZERO,
        }
    }

    /// Creates a plane from three points.
    pub fn from_points(a: &DVec3, b: &DVec3, c: &DVec3) -> Self {
        let n = (*b - *a).cross(*c - *a).normalize();
        Self::new(n, n.dot(*a))
    }

    /// Flips the plane by reversing the normal and distance.
    pub fn flip(&mut self) {
        self.normal = -self.normal;
        self.distance = -self.distance;
    }

    /// Splits a polygon into coplanar, front, and back polygons.
    pub fn split_polygon(
        &self,
        polygon: &Polygon,
        coplanar_front: &mut Vec<Polygon>,
        coplanar_back: &mut Vec<Polygon>,
        front: &mut Vec<Polygon>,
        back: &mut Vec<Polygon>,
    ) {
        let coplanar = 0;
        let front_flag = 1;
        let back_flag = 2;
        let spanning = 3;

        let mut polygon_type = 0;
        let mut types = Vec::with_capacity(polygon.vertices.len());

        for v in &polygon.vertices {
            let t = self.normal.dot(v.pos) - self.distance;
            let type_ = if t < -Self::EPSILON {
                back_flag
            } else if t > Self::EPSILON {
                front_flag
            } else {
                coplanar
            };
            polygon_type |= type_;
            types.push(type_);
        }

        match polygon_type {
            0 => {
                if self.normal.dot(polygon.plane.normal) > 0.0 {
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

                    if ti != back_flag {
                        f.push(vi.clone());
                    }
                    if ti != front_flag {
                        b.push(if ti != back_flag {
                            vi.clone()
                        } else {
                            vi.clone()
                        });
                    }
                    if (ti | tj) == spanning {
                        let t = (self.distance - self.normal.dot(vi.pos))
                            / self.normal.dot(vj.pos - vi.pos);
                        let v = vi.lerp(vj, t);
                        f.push(v.clone());
                        b.push(v);
                    }
                }

                if f.len() >= 3 {
                    front.push(Polygon::new(f, polygon.shared));
                }
                if b.len() >= 3 {
                    back.push(Polygon::new(b, polygon.shared));
                }
            }
            _ => {}
        }
    }
}

/// A polygon in 3D space.
#[derive(Clone)]
pub struct Polygon {
    // Vertices of the polygon.
    pub vertices: Vec<Vertex>,

    // Index of the shared data.
    pub shared: i32,

    // Plane of the polygon.
    pub plane: Plane,
}

impl Polygon {
    pub fn new(vertices: Vec<Vertex>, shared: i32) -> Self {
        if vertices.len() < 3 {
            panic!("Polygon must have at least 3 vertices");
        }
        let plane = Plane::from_points(&vertices[0].pos, &vertices[1].pos, &vertices[2].pos);
        Self {
            vertices,
            shared,
            plane,
        }
    }

    /// Flips the polygon by reversing the order of its vertices and flipping their normals.
    pub fn flip(&mut self) {
        self.vertices.reverse();
        for v in &mut self.vertices {
            v.flip();
        }
        self.plane.flip();
    }
}

#[derive(Clone, Copy)]
pub struct Vertex {
    pub pos: DVec3,
    pub normal: DVec3,
}

impl Vertex {
    pub fn new(pos: DVec3, normal: DVec3) -> Self {
        Self { pos, normal }
    }

    /// Flips the vertex by reversing its normal.
    pub fn flip(&mut self) {
        self.normal = -self.normal;
    }

    /// Linearly interpolates between two vertices.
    pub fn lerp(&self, other: &Vertex, t: f64) -> Self {
        Self::new(
            self.pos.lerp(other.pos, t),
            self.normal.lerp(other.normal, t),
        )
    }
}

#[derive(Clone)]
pub struct Triangle {
    pub vertices: [DVec3; 3],
    pub normal: DVec3,
}
