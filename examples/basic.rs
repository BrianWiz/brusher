use bevy::prelude::*;
use bevy::render::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

use brusher::math::*;
use brusher::brush::*;

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PanOrbitCameraPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    // Ground
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(5.0, 5.0)),
        material: materials.add(Color::srgb(0.3, 0.5, 0.3)),
        ..default()
    });

    // Light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
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

    // Create a cuboid, then knife it, creating a shape like this from the side view:
    //  _______
    // |       \
    // |        \
    // |         |
    // |_________|
    // let mut brush = Brush::cuboid(
    //     Vector3::new(0.0, 0.0, 0.0),
    //     CuboidDimensions {
    //         width: 1.0,
    //         height: 1.0,
    //         depth: 1.0,
    //     },
    // );

    let mut brush = Brush::cylinder(
        Vector3::new(0.0, 1.0, 1.0),
        CylinderDimensions {
            height: 1.0,
            radius: 0.5,
        },
        16,
    );
    
    brush.knife(SurfacePlane::new(
        Vector3::new(1.0, 1.0, 0.0).normalize(),
        0.5,
    ));

    let mesh = polygon_mesh_to_bevy_mesh(&brush.to_mesh_data());
    commands.spawn(PbrBundle {
        mesh: meshes.add(mesh),
        material: materials.add(Color::srgb(0.5, 0.5, 0.5)),
        transform: Transform::from_translation(Vec3::new(0.0, 0.5, 0.0)),
        ..default()
    });
}

fn polygon_mesh_to_bevy_mesh(polygon_mesh: &MeshData) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    // Extract normals
    let normals: Vec<[f32; 3]> = polygon_mesh
        .normals
        .iter()
        .map(|n| [n.x as f32, n.y as f32, n.z as f32])
        .collect();

    // Extract Vertices
    let positions: Vec<[f32; 3]> = polygon_mesh
        .positions
        .iter()
        .map(|p| [p.x as f32, p.y as f32, p.z as f32])
        .collect();

    // Set mesh attributes
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(polygon_mesh.indices.clone()));

    mesh
}
