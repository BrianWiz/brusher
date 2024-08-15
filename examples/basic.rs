use bevy::color::palettes::css::WHITE;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::settings::{RenderCreation, WgpuFeatures, WgpuSettings};
use bevy::render::texture::{
    ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor,
};
use bevy::render::RenderPlugin;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

use brusher::brush::brushlet::{Brushlet, BrushletSettings};
use brusher::brush::{operations::Knife, BooleanOp, Brush, MeshData};
use brusher::primitives::CuboidMaterialIndices;
use glam::DVec3;

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");

    App::new()
        .add_plugins((
            DefaultPlugins.set(RenderPlugin {
                render_creation: RenderCreation::Automatic(WgpuSettings {
                    // WARN this is a native only feature. It will not work with webgl or webgpu
                    features: WgpuFeatures::POLYGON_MODE_LINE,
                    ..default()
                }),
                ..default()
            }),
            // You need to add this plugin to enable wireframe rendering
            WireframePlugin,
        ))
        // Wireframes can be configured with this resource. This can be changed at runtime.
        .insert_resource(WireframeConfig {
            // The global wireframe config enables drawing of wireframes on every mesh,
            // except those with `NoWireframe`. Meshes with `Wireframe` will always have a wireframe,
            // regardless of the global configuration.
            global: true,
            // Controls the default color of all wireframes. Used as the default color for global wireframes.
            // Can be changed per mesh using the `WireframeColor` component.
            default_color: WHITE.into(),
        })
        .add_plugins(PanOrbitCameraPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
    let material1 = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        ..default()
    });
    let material2 = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle2.clone()),
        ..default()
    });

    // Create a brush
    let mut brush = Brush::new();
    brush.knives = vec![Knife {
        normal: DVec3::new(1.0, 1.0, 0.0),
        distance_from_origin: 4.0,
    }];

    // Room 1
    brush.add(Brushlet::from_cuboid(
        brusher::primitives::Cuboid {
            origin: DVec3::new(0.0, 0.0, 0.0),
            width: 8.0,
            height: 4.0,
            depth: 8.0,
            material_indices: CuboidMaterialIndices {
                front: 0,
                back: 0,
                left: 0,
                right: 0,
                top: 0,
                bottom: 0,
            },
        },
        BrushletSettings {
            operation: BooleanOp::Subtract,
            knives: vec![Knife {
                normal: DVec3::new(-1.0, -1.0, -1.0),
                distance_from_origin: 4.0,
            }],
            inverted: true,
        },
    ));

    // Room 2
    brush.add(Brushlet::from_cuboid(
        brusher::primitives::Cuboid {
            origin: DVec3::new(4.0, 0.0, 4.0),
            width: 8.0,
            height: 4.0,
            depth: 8.0,
            material_indices: CuboidMaterialIndices {
                front: 1,
                back: 1,
                left: 1,
                right: 1,
                top: 1,
                bottom: 1,
            },
        },
        BrushletSettings {
            operation: BooleanOp::Union,
            knives: vec![],
            inverted: false,
        },
    ));

    let mesh_data = brush.to_mesh_data();
    let mut meshes_with_materials = csg_to_bevy_meshes(&mesh_data);

    let mut pillar_brush = Brush::new();
    pillar_brush.add(create_beveled_pillar(DVec3::new(2.0, 0.0, 2.0)));

    // Spawn each mesh with the appropriate material
    let mesh_data = pillar_brush.to_mesh_data();
    meshes_with_materials.extend(csg_to_bevy_meshes(&mesh_data));
    for (mesh, material_index) in meshes_with_materials {
        let material = match material_index {
            0 => material1.clone(),
            1 => material2.clone(),
            _ => material1.clone(), // default
        };

        commands.spawn(PbrBundle {
            mesh: meshes.add(mesh),
            material,
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..default()
        });
    }
}

fn create_beveled_pillar(origin: DVec3) -> Brushlet {
    let pillar_width = 1.0;
    let pillar_height = 4.0;
    let pillar_depth = 1.0;
    let bevel_size = 0.1;
    let sqrt2 = 2.0_f64.sqrt();
    let base_distance = (pillar_width / 2.0 + pillar_depth / 2.0 - bevel_size) / sqrt2;

    Brushlet::from_cuboid(
        brusher::primitives::Cuboid {
            origin,
            width: pillar_width,
            height: pillar_height,
            depth: pillar_depth,
            material_indices: CuboidMaterialIndices {
                front: 1,
                back: 1,
                left: 1,
                right: 1,
                top: 1,
                bottom: 1,
            },
        },
        BrushletSettings {
            operation: BooleanOp::Union,
            knives: vec![
                // Front-right edge
                Knife {
                    normal: DVec3::new(1.0, 0.0, 1.0).normalize(),
                    distance_from_origin: base_distance + (origin.x + origin.z) / sqrt2,
                },
                // Front-left edge
                Knife {
                    normal: DVec3::new(-1.0, 0.0, 1.0).normalize(),
                    distance_from_origin: base_distance + (-origin.x + origin.z) / sqrt2,
                },
                // Back-right edge
                Knife {
                    normal: DVec3::new(1.0, 0.0, -1.0).normalize(),
                    distance_from_origin: base_distance + (origin.x - origin.z) / sqrt2,
                },
                // Back-left edge
                Knife {
                    normal: DVec3::new(-1.0, 0.0, -1.0).normalize(),
                    distance_from_origin: base_distance + (-origin.x - origin.z) / sqrt2,
                },
            ],
            inverted: false,
        },
    )
}

pub fn csg_to_bevy_meshes(mesh_data: &MeshData) -> Vec<(Mesh, usize)> {
    let mut meshes_with_materials: Vec<(Mesh, usize)> = vec![];

    for polygon in &mesh_data.polygons {
        let positions = polygon.positions_32();
        let normals = polygon.normals_32();
        let uvs = polygon.uvs();
        let indices = polygon.indices();
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_indices(Indices::U32(indices));
        meshes_with_materials.push((mesh, polygon.material));
    }

    meshes_with_materials
}
