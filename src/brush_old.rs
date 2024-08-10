use truck_meshalgo::prelude::{MeshableShape, MeshedShape};
use truck_modeling::{
    builder, BSplineSurface, BoundedCurve, ClosedSweep, Curve, Edge, EdgeID, EuclideanSpace,
    ExtrudedCurve, Face, FaceID, InnerSpace, Line, Mapped, MultiSweep, ParametricSurface3D, Plane,
    Point3, Rad, Shell, Solid, Surface, Sweep, Tolerance, Vector3, Vertex, Wire,
};

pub use truck_meshalgo::prelude::PolygonMesh;

pub mod brush;

pub struct Brush {
    solid: Solid,
}

pub struct Dimensions {
    pub width: f64,
    pub height: f64,
    pub depth: f64,
}

impl Dimensions {
    pub fn new(width: f64, height: f64, depth: f64) -> Self {
        Dimensions {
            width,
            height,
            depth,
        }
    }
}

pub struct Point {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Point { x, y, z }
    }
}

impl Into<Point3> for Point {
    fn into(self) -> Point3 {
        Point3::new(self.x, self.y, self.z)
    }
}

pub struct Normal {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Normal {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Normal { x, y, z }
    }

    pub fn normalize(&self) -> Normal {
        let len = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        Normal {
            x: self.x / len,
            y: self.y / len,
            z: self.z / len,
        }
    }
}

impl Into<Vector3> for Normal {
    fn into(self) -> Vector3 {
        Vector3::new(self.x, self.y, self.z)
    }
}

impl Brush {
    pub fn some_solid2(origin: Point, size: f64) -> Self {
        let origin: Point3 = origin.into();

        //  v0 _________ v1
        //    |          \
        //    |           \ v2
        //    |            |
        //    |            |
        // v4 |____________| v3

        let v0 = builder::vertex(Point3::new(0.0, 0.0, 1.0));
        let v1 = builder::vertex(Point3::new(0.5, 0.0, 1.0));
        let v2 = builder::vertex(Point3::new(1.0, 0.0, 0.5));
        let v3 = builder::vertex(Point3::new(1.0, 0.0, 0.0));
        let v4 = builder::vertex(Point3::new(0.0, 0.0, 0.0));

        let base_wire: Wire = vec![
            builder::line(&v0, &v1),
            builder::line(&v1, &v2),
            builder::line(&v2, &v3),
            builder::line(&v3, &v4),
            builder::line(&v4, &v0),
        ]
        .into();

        // extrude up
        let mut shell = builder::tsweep(&base_wire, Vector3::unit_y() * size);
        let wires = shell.extract_boundaries();

        // at this point we have a shell with no top or bottom

        // cap the top
        shell.push(
            builder::try_attach_plane(&[wires[0].clone()])
                .unwrap()
                .inverse(),
        );

        // cap the bottom
        shell.push(
            builder::try_attach_plane(&[wires[1].clone()])
                .unwrap()
                .inverse(),
        );

        Brush {
            solid: Solid::new(vec![shell]),
        }
    }

    pub fn cuboid(origin: Point, dimensions: Dimensions) -> Self {
        let v = builder::vertex(origin.into());
        let e = builder::tsweep(&v, Vector3::unit_x() * dimensions.width);
        let f = builder::tsweep(&e, Vector3::unit_y() * dimensions.height);
        let solid = builder::tsweep(&f, Vector3::unit_z() * dimensions.depth);
        Brush { solid }
    }

    pub fn to_polygon_mesh(&self) -> PolygonMesh {
        self.solid.triangulation(0.05).to_polygon()
    }
}

pub fn cuboid() -> Solid {
    let v = builder::vertex(Point3::new(0.0, 0.0, 0.0));
    let e = builder::tsweep(&v, Vector3::unit_x() * 1.0);
    let f = builder::tsweep(&e, Vector3::unit_y() * 1.0);
    builder::tsweep(&f, Vector3::unit_z() * 1.0)
}
