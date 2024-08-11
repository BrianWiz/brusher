#[derive(Clone)]
pub struct CSG {
    pub polygons: Vec<Polygon>,
}

impl CSG {
    pub fn new() -> Self {
        Self {
            polygons: Vec::new(),
        }
    }

    pub fn cube(center: Option<[f64; 3]>, radius: Option<[f64; 3]>) -> CSG {
        let c = center.unwrap_or([0.0, 0.0, 0.0]);
        let r = radius.unwrap_or([1.0, 1.0, 1.0]);

        let mut polygons = Vec::new();
        let faces = [
            ([0, 4, 6, 2], [-1.0, 0.0, 0.0]),
            ([1, 3, 7, 5], [1.0, 0.0, 0.0]),
            ([0, 1, 5, 4], [0.0, -1.0, 0.0]),
            ([2, 6, 7, 3], [0.0, 1.0, 0.0]),
            ([0, 2, 3, 1], [0.0, 0.0, -1.0]),
            ([4, 5, 7, 6], [0.0, 0.0, 1.0]),
        ];

        for &(indices, normal) in &faces {
            let mut vertices = Vec::new();
            for &i in &indices {
                let pos = Vector::new(
                    c[0] + r[0] * (if i & 1 != 0 { 1.0 } else { -1.0 }),
                    c[1] + r[1] * (if i & 2 != 0 { 1.0 } else { -1.0 }),
                    c[2] + r[2] * (if i & 4 != 0 { 1.0 } else { -1.0 }),
                );
                vertices.push(Vertex::new(
                    pos,
                    Vector::new(normal[0], normal[1], normal[2]),
                ));
            }
            polygons.push(Polygon::new(vertices, 0)); // `0` can be used for the shared field
        }

        CSG::from_polygons(polygons)
    }

    pub fn sphere(
        center: Option<[f64; 3]>,
        radius: Option<f64>,
        slices: Option<usize>,
        stacks: Option<usize>,
    ) -> CSG {
        let c = center.unwrap_or([0.0, 0.0, 0.0]);
        let r = radius.unwrap_or(1.0);
        let slices = slices.unwrap_or(16);
        let stacks = stacks.unwrap_or(8);

        let mut polygons = Vec::new();

        for i in 0..slices {
            for j in 0..stacks {
                let mut vertices = Vec::new();
                for &(u, v) in &[
                    (i as f64 / slices as f64, j as f64 / stacks as f64),
                    ((i + 1) as f64 / slices as f64, j as f64 / stacks as f64),
                    (
                        (i + 1) as f64 / slices as f64,
                        (j + 1) as f64 / stacks as f64,
                    ),
                    (i as f64 / slices as f64, (j + 1) as f64 / stacks as f64),
                ] {
                    let theta = u * std::f64::consts::PI * 2.0;
                    let phi = v * std::f64::consts::PI;
                    let dir =
                        Vector::new(theta.cos() * phi.sin(), phi.cos(), theta.sin() * phi.sin());
                    vertices.push(Vertex::new(
                        Vector::new(c[0] + r * dir.x, c[1] + r * dir.y, c[2] + r * dir.z),
                        dir,
                    ));
                }
                polygons.push(Polygon::new(vertices, 0));
            }
        }

        CSG::from_polygons(polygons)
    }

    pub fn cylinder(
        start: Option<[f64; 3]>,
        end: Option<[f64; 3]>,
        radius: Option<f64>,
        slices: Option<usize>,
    ) -> CSG {
        let s = start.unwrap_or([0.0, -1.0, 0.0]);
        let e = end.unwrap_or([0.0, 1.0, 0.0]);
        let r = radius.unwrap_or(1.0);
        let slices = slices.unwrap_or(16);

        let ray = Vector::new(e[0] - s[0], e[1] - s[1], e[2] - s[2]);
        let axis_z = ray.unit();
        let is_y = axis_z.y.abs() > 0.5;
        let axis_x = Vector::new(is_y as i32 as f64, !is_y as i32 as f64, 0.0)
            .cross(&axis_z)
            .unit();
        let axis_y = axis_x.cross(&axis_z).unit();
        let start_vertex = Vertex::new(Vector::new(s[0], s[1], s[2]), axis_z.negated());
        let end_vertex = Vertex::new(Vector::new(e[0], e[1], e[2]), axis_z.clone());

        let mut polygons = Vec::new();
        for i in 0..slices {
            let t0 = i as f64 / slices as f64;
            let t1 = (i + 1) as f64 / slices as f64;
            let mut vertices = vec![
                start_vertex.clone(),
                point(0.0, t0, -1.0, &axis_x, &axis_y, &axis_z, &s, &ray, r),
                point(0.0, t1, -1.0, &axis_x, &axis_y, &axis_z, &s, &ray, r),
            ];
            polygons.push(Polygon::new(vertices, 0));

            vertices = vec![
                point(0.0, t1, 0.0, &axis_x, &axis_y, &axis_z, &s, &ray, r),
                point(0.0, t0, 0.0, &axis_x, &axis_y, &axis_z, &s, &ray, r),
                point(1.0, t0, 0.0, &axis_x, &axis_y, &axis_z, &s, &ray, r),
                point(1.0, t1, 0.0, &axis_x, &axis_y, &axis_z, &s, &ray, r),
            ];
            polygons.push(Polygon::new(vertices, 0));

            vertices = vec![
                end_vertex.clone(),
                point(1.0, t1, 1.0, &axis_x, &axis_y, &axis_z, &s, &ray, r),
                point(1.0, t0, 1.0, &axis_x, &axis_y, &axis_z, &s, &ray, r),
            ];
            polygons.push(Polygon::new(vertices, 0));
        }

        CSG::from_polygons(polygons)
    }

    pub fn from_polygons(polygons: Vec<Polygon>) -> Self {
        Self { polygons }
    }

    pub fn clone(&self) -> Self {
        Self {
            polygons: self.polygons.iter().map(|p| p.clone()).collect(),
        }
    }

    pub fn to_polygons(&self) -> Vec<Polygon> {
        self.polygons.clone()
    }

    pub fn union(&self, csg: &CSG) -> CSG {
        let mut a = Node::new(self.clone().polygons);
        let mut b = Node::new(csg.clone().polygons);
        a.clip_to(&b);
        b.clip_to(&a);
        b.invert();
        b.clip_to(&a);
        b.invert();
        a.build(b.all_polygons());
        CSG::from_polygons(a.all_polygons())
    }

    pub fn subtract(&self, csg: &CSG) -> CSG {
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
        CSG::from_polygons(a.all_polygons())
    }

    pub fn intersect(&self, csg: &CSG) -> CSG {
        let mut a = Node::new(self.clone().polygons);
        let mut b = Node::new(csg.clone().polygons);
        a.invert();
        b.clip_to(&a);
        b.invert();
        a.clip_to(&b);
        b.clip_to(&a);
        a.build(b.all_polygons());
        a.invert();
        CSG::from_polygons(a.all_polygons())
    }

    pub fn inverse(&self) -> CSG {
        let mut csg = self.clone();
        for p in &mut csg.polygons {
            p.flip();
        }
        csg
    }
}

#[derive(Clone)]
pub struct Vector {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn negated(&self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }

    pub fn plus(&self, a: &Vector) -> Self {
        Self::new(self.x + a.x, self.y + a.y, self.z + a.z)
    }

    pub fn minus(&self, a: &Vector) -> Self {
        Self::new(self.x - a.x, self.y - a.y, self.z - a.z)
    }

    pub fn times(&self, a: f64) -> Self {
        Self::new(self.x * a, self.y * a, self.z * a)
    }

    pub fn divided_by(&self, a: f64) -> Self {
        Self::new(self.x / a, self.y / a, self.z / a)
    }

    pub fn dot(&self, a: &Vector) -> f64 {
        self.x * a.x + self.y * a.y + self.z * a.z
    }

    pub fn length(&self) -> f64 {
        self.dot(self).sqrt()
    }

    pub fn unit(&self) -> Self {
        self.divided_by(self.length())
    }

    pub fn cross(&self, a: &Vector) -> Self {
        Self::new(
            self.y * a.z - self.z * a.y,
            self.z * a.x - self.x * a.z,
            self.x * a.y - self.y * a.x,
        )
    }
}

#[derive(Clone)]
pub struct Vertex {
    pub pos: Vector,
    pub normal: Vector,
}

impl Vertex {
    pub fn new(pos: Vector, normal: Vector) -> Self {
        Self { pos, normal }
    }

    pub fn clone(&self) -> Self {
        Self::new(self.pos.clone(), self.normal.clone())
    }

    pub fn flip(&mut self) {
        self.normal = self.normal.negated();
    }

    pub fn interpolate(&self, other: &Vertex, t: f64) -> Self {
        Self::new(
            self.pos.plus(&other.pos.minus(&self.pos).times(t)),
            self.normal.plus(&other.normal.minus(&self.normal).times(t)),
        )
    }
}

#[derive(Clone)]
pub struct Plane {
    normal: Vector,
    w: f64,
}

impl Plane {
    pub const EPSILON: f64 = 1e-5;

    pub fn from_points(a: &Vector, b: &Vector, c: &Vector) -> Self {
        let n = b.minus(a).cross(&c.minus(a)).unit();
        Self::new(n.clone(), n.dot(a))
    }

    pub fn new(normal: Vector, w: f64) -> Self {
        Self { normal, w }
    }

    pub fn clone(&self) -> Self {
        Self::new(self.normal.clone(), self.w)
    }

    pub fn flip(&mut self) {
        self.normal = self.normal.negated();
        self.w = -self.w;
    }

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
            let t = self.normal.dot(&v.pos) - self.w;
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
                if self.normal.dot(&polygon.plane.normal) > 0.0 {
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
                        let t = (self.w - self.normal.dot(&vi.pos))
                            / self.normal.dot(&vj.pos.minus(&vi.pos));
                        let v = vi.interpolate(vj, t);
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

#[derive(Clone)]
pub struct Polygon {
    pub vertices: Vec<Vertex>,
    pub shared: i32,
    pub plane: Plane,
}

impl Polygon {
    pub fn new(vertices: Vec<Vertex>, shared: i32) -> Self {
        let plane = Plane::from_points(&vertices[0].pos, &vertices[1].pos, &vertices[2].pos);
        Self {
            vertices,
            shared,
            plane,
        }
    }

    pub fn clone(&self) -> Self {
        Self {
            vertices: self.vertices.iter().map(|v| v.clone()).collect(),
            shared: self.shared,
            plane: self.plane.clone(),
        }
    }

    pub fn flip(&mut self) {
        self.vertices.reverse();
        for v in &mut self.vertices {
            v.flip();
        }
        self.plane.flip();
    }
}

#[derive(Clone)]
pub struct Node {
    plane: Option<Plane>,
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
            self.plane = Some(polygons[0].plane.clone());
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

fn point(
    stack: f64,
    slice: f64,
    normal_blend: f64,
    axis_x: &Vector,
    axis_y: &Vector,
    axis_z: &Vector,
    s: &[f64; 3],
    ray: &Vector,
    r: f64,
) -> Vertex {
    let angle = slice * std::f64::consts::PI * 2.0;
    let out = axis_x.times(angle.cos()).plus(&axis_y.times(angle.sin()));
    let pos = Vector::new(
        s[0] + ray.x * stack + out.x * r,
        s[1] + ray.y * stack + out.y * r,
        s[2] + ray.z * stack + out.z * r,
    );
    let normal = out
        .times(1.0 - normal_blend.abs())
        .plus(&axis_z.times(normal_blend));
    Vertex::new(pos, normal)
}
