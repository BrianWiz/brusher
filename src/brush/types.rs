use glam::{DVec2, DVec3, Vec2};

pub trait TPlane {
    const EPSILON: f64 = 1e-5;

    fn normal(&self) -> &DVec3;
    fn distance(&self) -> &f64;
    fn normal_mut(&mut self) -> &mut DVec3;
    fn distance_mut(&mut self) -> &mut f64;

    /// Flips the plane by reversing the normal and distance.
    fn flip(&mut self) {
        *self.normal_mut() = -*self.normal();
        *self.distance_mut() = -*self.distance();
    }

    /// Splits a polygon into coplanar, front, and back polygons.
    fn split_polygon(
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
            let t = self.normal().dot(v.pos) as f64 - self.distance();
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
                if self.normal().dot(polygon.surface.normal) > 0.0 {
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
                        let t = (self.distance() - self.normal().dot(vi.pos))
                            / self.normal().dot(vj.pos - vi.pos);
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

#[derive(Clone, Copy)]
pub struct Plane {
    pub normal: DVec3,
    pub distance: f64,
}

impl TPlane for Plane {
    fn normal(&self) -> &DVec3 {
        &self.normal
    }

    fn distance(&self) -> &f64 {
        &self.distance
    }

    fn normal_mut(&mut self) -> &mut DVec3 {
        &mut self.normal
    }

    fn distance_mut(&mut self) -> &mut f64 {
        &mut self.distance
    }
}

#[derive(Clone, Copy)]
pub struct Surface {
    pub normal: DVec3,
    pub distance: f64,
    pub u_axis: DVec3,
    pub v_axis: DVec3,
}

impl Surface {
    pub fn new(normal: DVec3, distance: f64) -> Self {
        let (u_axis, v_axis) = Self::compute_uv_axes(&normal);
        Self {
            normal: normal.normalize(),
            distance,
            u_axis,
            v_axis,
        }
    }

    /// Computes UV coordinates for a point on the plane.
    pub fn compute_uv(&self, point: DVec3) -> DVec2 {
        let projected = point - self.normal * self.distance;
        DVec2::new(projected.dot(self.u_axis), projected.dot(self.v_axis))
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

    /// Creates a plane from three points.
    pub fn from_points(a: &DVec3, b: &DVec3, c: &DVec3) -> Surface {
        let n = (*b - *a).cross(*c - *a).normalize();
        Self::new(n, n.dot(*a) as f64)
    }
}

impl TPlane for Surface {
    fn normal(&self) -> &DVec3 {
        &self.normal
    }

    fn distance(&self) -> &f64 {
        &self.distance
    }

    fn normal_mut(&mut self) -> &mut DVec3 {
        &mut self.normal
    }

    fn distance_mut(&mut self) -> &mut f64 {
        &mut self.distance
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
    pub surface: Surface,
}

impl Polygon {
    pub fn new(vertices: Vec<Vertex>, shared: i32) -> Self {
        if vertices.len() < 3 {
            panic!("Polygon must have at least 3 vertices");
        }
        let surface = Surface::from_points(&vertices[0].pos, &vertices[1].pos, &vertices[2].pos);
        Self {
            vertices,
            shared,
            surface,
        }
    }

    /// Flips the polygon by reversing the order of its vertices and flipping their normals.
    pub fn flip(&mut self) {
        self.vertices.reverse();
        for v in &mut self.vertices {
            v.flip();
        }
        self.surface.flip();
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
