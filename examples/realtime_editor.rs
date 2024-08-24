// NOTE: This doesnt really work yet...
// But I've decided to push it up so you can see where it's going.

use bevy::math::*;
use bevy::prelude::*;
use bevy::render::camera::CameraProjection;
use bevy::render::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::texture::{
    ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor,
};
use bevy::window::PrimaryWindow;
use bevy_egui::egui;
use bevy_egui::EguiContexts;
use bevy_egui::EguiPlugin;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use brusher::broadphase::Raycast;
use brusher::prelude::*;
use brusher::scene::BrusherScene;
use brusher::scene::Layer;
use mint::RowMatrix4;
use transform_gizmo_egui::prelude::*;
use transform_gizmo_egui::GizmoConfig;

// Helper enum to map materials to indices
enum MyMaterials {
    ProtoGrey = 0,
    ProtoGreen = 1,
}

impl From<MyMaterials> for usize {
    fn from(material: MyMaterials) -> usize {
        material as usize
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
struct TransformGizmo(Gizmo);

#[derive(Component)]
struct BrushMesh;

#[derive(Component)]
struct SelectedBrushlet {
    layer_idx: usize,
    brush_idx: usize,
    idx: usize,
}

#[derive(Resource, Default)]
struct ProtoMaterials {
    grey: Handle<StandardMaterial>,
    green: Handle<StandardMaterial>,
}

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(EguiPlugin)
        .add_systems(Startup, setup_system)
        .add_systems(
            Update,
            (
                refresh_brush_system,
                select_brush_system,
                gizmo_update_system,
            ),
        )
        .register_type::<DVec3>()
        .register_type::<DAffine3>()
        .register_type::<Brush>()
        .register_type::<Brushlet>()
        .register_type::<Knife>()
        .register_type::<brusher::prelude::Polygon>()
        .register_type::<Surface>()
        .register_type::<Vertex>()
        .init_resource::<ProtoMaterials>()
        .init_resource::<TransformGizmo>()
        .run();
}

fn setup_system(
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut proto_materials: ResMut<ProtoMaterials>,
    mut commands: Commands,
) {
    // Light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });

    // Camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
            ..default()
        },
        PanOrbitCamera::default(),
    ));

    // Load textures
    let sampler_desc = ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        ..Default::default()
    };

    let settings = move |s: &mut ImageLoaderSettings| {
        s.sampler = ImageSampler::Descriptor(sampler_desc.clone());
    };

    let texture_handle = asset_server.load_with_settings("proto.png", settings.clone());
    let texture_handle2 = asset_server.load_with_settings("proto2.png", settings);

    // Create materials
    proto_materials.grey = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        ..default()
    });
    proto_materials.green = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle2.clone()),
        ..default()
    });

    let time_now = std::time::Instant::now();

    // Create a brush that will combine two rooms
    let mut brush = Brush::new("Rooms");

    // Create a brushlet for the first room
    brush.brushlets.push(Brushlet::from_cuboid(
        brusher::primitives::Cuboid {
            origin: DVec3::new(0.0, 0.0, 0.0),
            width: 8.0,
            height: 4.0,
            depth: 8.0,
            material_indices: CuboidMaterialIndices::default(),
        },
        BrushletSettings {
            name: "Room 1".to_string(),
            operation: BooleanOp::Subtract,
            // Cut the brushlet with a knife
            knives: vec![Knife {
                normal: DVec3::new(-1.0, -1.0, -1.0),
                distance_from_origin: 4.0,
                material_index: MyMaterials::ProtoGrey.into(),
            }],
            inverted: true,
        },
    ));

    // Create a brushlet for the second room
    brush.brushlets.push(Brushlet::from_cuboid(
        brusher::primitives::Cuboid {
            origin: DVec3::new(4.0, 0.0, 4.0),
            width: 8.0,
            height: 4.0,
            depth: 8.0,
            material_indices: CuboidMaterialIndices::default(),
        },
        BrushletSettings {
            name: "Room 2".to_string(),
            operation: BooleanOp::Union,
            knives: vec![],
            inverted: false,
        },
    ));

    // Transform the brush
    brush.transform(DAffine3::from_translation(DVec3::new(-4.0, 0.0, -4.0)));

    // Cut at the brush level with a knife to cut both rooms at once
    brush.settings.knives = vec![Knife {
        normal: DVec3::new(1.0, 1.0, 0.0),
        distance_from_origin: 4.0,
        material_index: MyMaterials::ProtoGrey.into(),
    }];

    // Spawn the brush mesh
    spawn_brush_meshes(&mut commands, &mut meshes, &proto_materials, &brush);

    // Create and spawn the BrusherScene as a component
    commands.spawn(BrusherScene {
        layers: vec![Layer {
            name: "Test".to_string(),
            brushes: vec![brush],
            hidden: false,
        }],
    });

    println!("Time elapsed: {:?}", time_now.elapsed());
}

fn refresh_brush_system(
    brusher_scene_query: Query<&BrusherScene>,
    proto_materials: Res<ProtoMaterials>,
    mut commands: Commands,
    mut mesh_components: Query<Entity, With<BrushMesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let brusher_scene = if let Ok(scene) = brusher_scene_query.get_single() {
        scene
    } else {
        return; // No BrusherScene found
    };

    // delete the old meshes
    for entity in mesh_components.iter_mut() {
        commands.entity(entity).despawn_recursive();
    }

    // spawn the new meshes
    for layer in brusher_scene.layers.iter() {
        for brush in layer.brushes.iter() {
            spawn_brush_meshes(&mut commands, &mut meshes, &proto_materials, brush);
        }
    }
}

fn spawn_brush_meshes(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    proto_materials: &ProtoMaterials,
    brush: &Brush,
) {
    let meshes_with_materials = brush.to_mesh_data().to_bevy_meshes();
    for mesh_material_map in meshes_with_materials {
        let material = match mesh_material_map.1 {
            0 => proto_materials.grey.clone(),
            1 => proto_materials.green.clone(),
            _ => proto_materials.grey.clone(),
        };

        commands.spawn((
            PbrBundle {
                mesh: meshes.add(mesh_material_map.0),
                material,
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                ..default()
            },
            BrushMesh,
        ));
    }
}

fn select_brush_system(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut selected_brushlet_query: Query<Entity, With<SelectedBrushlet>>,
    mut commands: Commands,
    mut brusher_scene_query: Query<&mut BrusherScene>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        let mut brusher_scene = if let Ok(scene) = brusher_scene_query.get_single_mut() {
            scene
        } else {
            return;
        };

        for (camera, global_transform) in camera.iter() {
            if let Ok(window) = window.get_single() {
                let cursor_position = window.cursor_position().unwrap_or(Vec2::new(0.0, 0.0));

                if let Some(ray) =
                    Camera::viewport_to_world(camera, global_transform, cursor_position)
                {
                    let raycast = Raycast {
                        origin: ray.origin.as_dvec3(),
                        direction: ray.direction.as_dvec3(),
                    };

                    // Clear the selection
                    for entity in selected_brushlet_query.iter_mut() {
                        commands.entity(entity).despawn_recursive();
                    }

                    // Check if a brush was selected
                    if let Some(selection) = brusher_scene.try_select_brush(&raycast) {
                        if let Some(brush) =
                            brusher_scene.get_brush_mut(selection.layer_idx, selection.idx)
                        {
                            if let Some(brushlet_idx) = brush.try_select_brushlet(&raycast) {
                                if let Some(brushlet) = brush.get_brushlet_mut(brushlet_idx) {
                                    // Spawn the selected brushlet
                                    commands.spawn((
                                        SelectedBrushlet {
                                            layer_idx: selection.layer_idx,
                                            brush_idx: selection.idx,
                                            idx: brushlet_idx,
                                        },
                                        to_bevy_wireframe_mesh(
                                            &mut meshes,
                                            &mut materials,
                                            brushlet,
                                        ),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
struct Edge([Vec3; 2]);

impl Edge {
    const EPSILON: f32 = 1e-5;

    fn new(a: Vec3, b: Vec3) -> Self {
        if vec3_less_than(a, b) {
            Edge([a, b])
        } else {
            Edge([b, a])
        }
    }

    fn approx_eq(&self, other: &Self) -> bool {
        self.0[0].abs_diff_eq(other.0[0], Self::EPSILON)
            && self.0[1].abs_diff_eq(other.0[1], Self::EPSILON)
    }
}

fn vec3_less_than(a: Vec3, b: Vec3) -> bool {
    if a.x != b.x {
        return a.x < b.x;
    }
    if a.y != b.y {
        return a.y < b.y;
    }
    a.z < b.z
}

fn to_bevy_wireframe_mesh(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    brushlet: &Brushlet,
) -> PbrBundle {
    let mesh_data = brushlet.to_mesh_data();

    let mut mesh = Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::default());
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut edges = Vec::new();

    for polygon in &mesh_data.polygons {
        let positions = polygon.positions_32();
        for i in 0..positions.len() {
            let j = (i + 1) % positions.len();
            let edge = Edge::new(Vec3::from(positions[i]), Vec3::from(positions[j]));

            if !edges.iter().any(|e: &Edge| e.approx_eq(&edge)) {
                edges.push(edge);
                let index = vertices.len() as u32;
                vertices.push(edge.0[0].to_array());
                vertices.push(edge.0[1].to_array());
                indices.push(index);
                indices.push(index + 1);
            }
        }
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_indices(Indices::U32(indices));

    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        unlit: true,
        ..Default::default()
    });

    PbrBundle {
        mesh: meshes.add(mesh),
        material,
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ..default()
    }
}

fn gizmo_update_system(
    selected_brushlet_query: Query<&SelectedBrushlet>,
    mut gizmo: ResMut<TransformGizmo>,
    mut brusher_scene_query: Query<&mut BrusherScene>,
    mut contexts: EguiContexts,
    camera_query: Query<(&Camera, &Projection, &Transform, &GlobalTransform), With<Camera3d>>,
) {
    let (camera, projection, camera_transform, camera_global_transform) = camera_query.single();

    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(egui::Color32::TRANSPARENT))
        .show(contexts.ctx_mut(), |ui| {
            ui.label("Gizmo Controls:");

            if let Ok(selected_brushlet) = selected_brushlet_query.get_single() {
                if let Ok(mut brusher_scene) = brusher_scene_query.get_single_mut() {
                    if let Some(brush) = brusher_scene
                        .get_brush_mut(selected_brushlet.layer_idx, selected_brushlet.brush_idx)
                    {
                        if let Some(mut brushlet) = brush.get_brushlet_mut(selected_brushlet.idx) {
                            let (scale, rotation, translation) =
                                brushlet.compute_transform().to_scale_rotation_translation();

                            let target = transform_gizmo_egui::math::Transform {
                                scale: mint::Vector3 {
                                    x: scale.x,
                                    y: scale.y,
                                    z: scale.z,
                                },
                                rotation: mint::Quaternion {
                                    v: mint::Vector3 {
                                        x: rotation.x,
                                        y: rotation.y,
                                        z: rotation.z,
                                    },
                                    s: rotation.w,
                                },
                                translation: mint::Vector3 {
                                    x: translation.x,
                                    y: translation.y,
                                    z: translation.z,
                                },
                            };

                            let viewport = ui.clip_rect();

                            let projection_matrix = match projection {
                                Projection::Perspective(persp) => {
                                    transform_gizmo_egui::math::DMat4::perspective_rh_gl(
                                        persp.fov as f64,
                                        persp.aspect_ratio as f64,
                                        persp.near as f64,
                                        persp.far as f64,
                                    )
                                }
                                Projection::Orthographic(ortho) => {
                                    transform_gizmo_egui::math::DMat4::orthographic_rh_gl(
                                        ortho.area.max.x as f64,
                                        ortho.area.min.x as f64,
                                        ortho.area.max.y as f64,
                                        ortho.area.min.y as f64,
                                        ortho.near as f64,
                                        ortho.far as f64,
                                    )
                                }
                            };

                            let view_matrix =
                                transform_gizmo_egui::math::DMat4::from_cols_array_2d(
                                    &camera_global_transform
                                        .compute_matrix()
                                        .as_dmat4()
                                        .to_cols_array_2d(),
                                )
                                .inverse();

                            gizmo.0.update_config(GizmoConfig {
                                viewport,
                                modes: GizmoMode::all(),
                                orientation: GizmoOrientation::Global,
                                projection_matrix: projection_matrix.into(),
                                view_matrix: view_matrix.into(),
                                ..Default::default()
                            });

                            if let Some((_, new_transforms)) = gizmo.0.interact(ui, &[target]) {
                                if let Some(new_transform) = new_transforms.first() {
                                    let new_transform = DAffine3::from_scale_rotation_translation(
                                        DVec3::new(
                                            new_transform.scale.x as f64,
                                            new_transform.scale.y as f64,
                                            new_transform.scale.z as f64,
                                        ),
                                        DQuat::from_xyzw(
                                            new_transform.rotation.v.x as f64,
                                            new_transform.rotation.v.y as f64,
                                            new_transform.rotation.v.z as f64,
                                            new_transform.rotation.s as f64,
                                        ),
                                        DVec3::new(
                                            new_transform.translation.x as f64,
                                            new_transform.translation.y as f64,
                                            new_transform.translation.z as f64,
                                        ),
                                    );
                                    *brushlet = brushlet.transform(new_transform);
                                }
                            }

                            ui.label(format!("Brushlet Position: {:?}", translation));
                        }
                    }
                }
            } else {
                ui.label("No brushlet selected");
            }
        });

    contexts.ctx_mut().request_repaint();
}
