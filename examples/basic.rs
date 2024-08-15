use bevy::prelude::*;
use bevy::render::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::texture::{
    ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor,
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use glam::DVec3;

use brusher::prelude::*;

enum MyMaterials {
    ProtoGrey = 0,
    ProtoGreen = 1,
}

impl From<MyMaterials> for usize {
    fn from(material: MyMaterials) -> usize {
        material as usize
    }
}

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");

    App::new()
        .add_plugins(DefaultPlugins)
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
    let material_proto_grey = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        ..default()
    });
    let material_proto_green = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle2.clone()),
        ..default()
    });

    // Create a brush that will combine two rooms
    let mut brush = Brush::new("Rooms");

    // Create a brushlet for the first room
    brush.add(Brushlet::from_cuboid(
        brusher::primitives::Cuboid {
            origin: DVec3::new(0.0, 0.0, 0.0),
            width: 8.0,
            height: 4.0,
            depth: 8.0,
            material_indices: CuboidMaterialIndices {
                front: MyMaterials::ProtoGrey.into(),
                back: MyMaterials::ProtoGreen.into(),
                left: MyMaterials::ProtoGreen.into(),
                right: MyMaterials::ProtoGrey.into(),
                top: MyMaterials::ProtoGrey.into(),
                bottom: MyMaterials::ProtoGrey.into(),
            },
        },
        BrushletSettings {
            name: "Room 1".to_string(),
            operation: BooleanOp::Subtract,
            // Cut the brushlet with a knife
            knives: vec![Knife {
                normal: DVec3::new(-1.0, -1.0, -1.0),
                distance_from_origin: 4.0,
                material_index: MyMaterials::ProtoGreen.into(),
            }],
            inverted: true,
        },
    ));

    // Create a brushlet for the second room
    brush.add(Brushlet::from_cuboid(
        brusher::primitives::Cuboid {
            origin: DVec3::new(4.0, 0.0, 4.0),
            width: 8.0,
            height: 4.0,
            depth: 8.0,
            material_indices: CuboidMaterialIndices {
                front: MyMaterials::ProtoGreen.into(),
                back: MyMaterials::ProtoGreen.into(),
                left: MyMaterials::ProtoGreen.into(),
                right: MyMaterials::ProtoGreen.into(),
                top: MyMaterials::ProtoGreen.into(),
                bottom: MyMaterials::ProtoGreen.into(),
            },
        },
        BrushletSettings {
            name: "Room 2".to_string(),
            operation: BooleanOp::Union,
            knives: vec![],
            inverted: false,
        },
    ));

    // Cut at the brush level with a knife to cut both rooms at once
    brush.settings.knives = vec![Knife {
        normal: DVec3::new(1.0, 1.0, 0.0),
        distance_from_origin: 4.0,
        material_index: MyMaterials::ProtoGrey.into(),
    }];

    let mesh_data = brush.to_mesh_data();
    let mut meshes_with_materials = csg_to_bevy_meshes(&mesh_data);

    let mut pillar_brush = Brush::new("Pillar");
    pillar_brush.add(create_beveled_pillar(DVec3::new(2.0, 0.0, 2.0)));

    // Spawn each mesh with the appropriate material
    let mesh_data = pillar_brush.to_mesh_data();
    meshes_with_materials.extend(csg_to_bevy_meshes(&mesh_data));
    for (mesh, material_index) in meshes_with_materials {
        let material = match material_index {
            0 => material_proto_grey.clone(),
            1 => material_proto_green.clone(),
            _ => material_proto_grey.clone(),
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
            name: "Stem".to_string(),
            operation: BooleanOp::Union,
            knives: vec![
                // Front-right edge
                Knife {
                    normal: DVec3::new(1.0, 0.0, 1.0).normalize(),
                    distance_from_origin: base_distance + (origin.x + origin.z) / sqrt2,
                    material_index: 1,
                },
                // Front-left edge
                Knife {
                    normal: DVec3::new(-1.0, 0.0, 1.0).normalize(),
                    distance_from_origin: base_distance + (-origin.x + origin.z) / sqrt2,
                    material_index: 1,
                },
                // Back-right edge
                Knife {
                    normal: DVec3::new(1.0, 0.0, -1.0).normalize(),
                    distance_from_origin: base_distance + (origin.x - origin.z) / sqrt2,
                    material_index: 1,
                },
                // Back-left edge
                Knife {
                    normal: DVec3::new(-1.0, 0.0, -1.0).normalize(),
                    distance_from_origin: base_distance + (-origin.x - origin.z) / sqrt2,
                    material_index: 1,
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
        meshes_with_materials.push((mesh, polygon.material_index));
    }

    meshes_with_materials
}
