use crate::{
    broadphase::Raycast,
    brush::{Brush, BrushSelection},
};

pub struct Layer {
    pub name: String,
    pub brushes: Vec<Brush>,
    pub hidden: bool,
}

pub struct Scene {
    pub layers: Vec<Layer>,
}

impl Scene {
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    pub fn select_brush(&mut self, layer_idx: usize, idx: usize) -> Option<BrushSelection> {
        let layer = self.layers.get_mut(layer_idx)?;
        let brush = layer.brushes.get_mut(idx)?;
        Some(BrushSelection {
            brush,
            idx,
            layer_idx,
        })
    }

    pub fn try_select_brush(&mut self, raycast: &Raycast) -> Option<BrushSelection> {
        for (layer_idx, layer) in self.layers.iter_mut().enumerate() {
            if layer.hidden {
                continue;
            }
            for (idx, brush) in layer.brushes.iter().enumerate() {
                if brush.try_select(raycast) {
                    return Some(BrushSelection {
                        brush,
                        idx,
                        layer_idx,
                    });
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use glam::DVec3;

    use crate::{
        broadphase::Raycast,
        brush::BooleanOp,
        prelude::{Brushlet, BrushletSettings, Cuboid, CuboidMaterialIndices},
    };

    use super::{Brush, Layer, Scene};

    #[test]
    fn test_try_select_brush_hit() {
        let mut scene = Scene::new();
        let mut layer = Layer {
            name: "Test".to_string(),
            brushes: vec![Brush::new("Test")],
            hidden: false,
        };

        let mut brush0 = Brush::new("Brush 0");
        brush0.brushlets.push(Brushlet::from_cuboid(
            Cuboid {
                origin: DVec3::new(0.0, 0.0, 0.0),
                width: 8.0,
                height: 4.0,
                depth: 8.0,
                material_indices: CuboidMaterialIndices::default(),
            },
            BrushletSettings {
                name: "Room 1".to_string(),
                operation: BooleanOp::Subtract,
                inverted: false,
                knives: vec![],
            },
        ));

        layer.brushes.push(brush0);
        scene.layers.push(layer);

        let raycast = Raycast::new(DVec3::new(0.0, 0.0, -2.0), DVec3::new(0.0, 0.0, 1.0));
        let selection = scene.try_select_brush(&raycast).unwrap();
        assert_eq!(selection.brush.settings.name, "Brush 0");
    }

    #[test]
    fn test_try_select_brush_miss() {
        let mut scene = Scene::new();
        let mut layer = Layer {
            name: "Test".to_string(),
            brushes: vec![Brush::new("Test")],
            hidden: false,
        };

        let mut brush0 = Brush::new("Brush 0");
        brush0.brushlets.push(Brushlet::from_cuboid(
            Cuboid {
                origin: DVec3::new(0.0, 0.0, 0.0),
                width: 8.0,
                height: 4.0,
                depth: 8.0,
                material_indices: CuboidMaterialIndices::default(),
            },
            BrushletSettings {
                name: "Room 1".to_string(),
                operation: BooleanOp::Subtract,
                inverted: false,
                knives: vec![],
            },
        ));

        layer.brushes.push(brush0);
        scene.layers.push(layer);

        let raycast = Raycast::new(DVec3::new(10.0, 10.0, 10.0), DVec3::new(0.0, 0.0, 1.0));
        let selection = scene.try_select_brush(&raycast);
        assert!(selection.is_none());
    }
}