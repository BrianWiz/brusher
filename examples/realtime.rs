use bevy::prelude::*;
use bevy::render::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::texture::{
    ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor,
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use glam::{DAffine3, DVec3};

use brusher::prelude::*;

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

#[derive(Component)]
struct BrushComponent {
    brush: Brush,
}

#[derive(Component)]
struct BrushMesh;

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
        .add_systems(Startup, setup_system)
        .add_systems(Update, animate_brush_system)
        .init_resource::<ProtoMaterials>()
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
                material_index: MyMaterials::ProtoGreen.into(),
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

    // Spawn the brush entity
    commands.spawn(BrushComponent { brush });

    println!("Time elapsed: {:?}", time_now.elapsed());
}

fn animate_brush_system(
    time: Res<Time>,
    mut commands: Commands,
    mut brush_components: Query<&mut BrushComponent>,
    mut mesh_components: Query<Entity, With<BrushMesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
    proto_materials: Res<ProtoMaterials>,
) {
    // delete the old meshes
    for entity in mesh_components.iter_mut() {
        commands.entity(entity).despawn_recursive();
    }

    for brush_component in brush_components.iter_mut() {
        // animate the knives on a sine wave
        let time = time.elapsed_seconds_f64();

        let mut brush = brush_component.brush.clone();

        // animate the brush knives
        for knife in &mut brush.settings.knives {
            knife.distance_from_origin = 4.0 + (time * 6.0 * std::f64::consts::PI).sin();
        }

        // animate the first brushlet's origin
        brush.brushlets[0] = brush.brushlets[0].transform(DAffine3::from_translation(DVec3::new(
            0.0,
            (time * 0.1 * std::f64::consts::PI).sin(),
            0.0,
        )));

        for brushlet in &mut brush.brushlets {
            // animate the knives
            for knife in &mut brushlet.settings.knives {
                knife.distance_from_origin = 4.0 + (time * 2.0 * std::f64::consts::PI).sin();
            }
        }

        // spawn the new meshes
        spawn_brush_meshes(&mut commands, &mut meshes, &proto_materials, &brush);
    }
}

fn spawn_brush_meshes(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    proto_materials: &ProtoMaterials,
    brush: &Brush,
) {
    let mesh_data = brush.to_mesh_data();
    let meshes_with_materials = csg_to_bevy_meshes(&mesh_data);
    for (mesh, material_index) in meshes_with_materials {
        let material = match material_index {
            0 => proto_materials.grey.clone(),
            1 => proto_materials.green.clone(),
            _ => proto_materials.grey.clone(),
        };

        commands.spawn((
            PbrBundle {
                mesh: meshes.add(mesh),
                material,
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                ..default()
            },
            BrushMesh,
        ));
    }
}

fn csg_to_bevy_meshes(mesh_data: &MeshData) -> Vec<(Mesh, usize)> {
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
        meshes_with_materials.push((mesh, polygon.surface.material_index));
    }

    meshes_with_materials
}
