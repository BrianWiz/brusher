use crate::{polygon::Polygon, surface::Surface};

#[derive(Debug, Clone)]
pub(crate) struct Node {
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

    pub fn clip_polygons(&self, polygons: Vec<Polygon>) -> Vec<Polygon> {
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

    pub fn clip_to(&mut self, bsp: &Node) {
        self.polygons = bsp.clip_polygons(self.polygons.clone());
        if let Some(ref mut front) = self.front {
            front.clip_to(bsp);
        }
        if let Some(ref mut back) = self.back {
            back.clip_to(bsp);
        }
    }

    pub fn all_polygons(&self) -> Vec<Polygon> {
        let mut polygons = self.polygons.clone();
        if let Some(ref front) = self.front {
            polygons.extend(front.all_polygons());
        }
        if let Some(ref back) = self.back {
            polygons.extend(back.all_polygons());
        }
        polygons
    }

    pub fn build(&mut self, mut polygons: Vec<Polygon>) {
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
