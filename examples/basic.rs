use bevy::prelude::*;
use bevy::render::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

use brusher::brush::*;
use brusher::csg::CSG;
use brusher::math::*;

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

    let cube = CSG::cube(Some([0.0, 0.0, 0.0]), Some([0.5, 0.5, 0.5]));
    let cube2 = CSG::cube(Some([0.5, 0.5, 0.5]), Some([0.5, 0.5, 0.5]));

    // Perform a CSG operation (e.g., subtract cube2 from cube)
    let result = cube.subtract(&cube2);

    let mesh = csg_to_bevy_mesh(&result);

    commands.spawn(PbrBundle {
        mesh: meshes.add(mesh),
        material: materials.add(Color::srgb(0.5, 0.5, 0.5)),
        transform: Transform::from_translation(Vec3::new(0.0, 0.5, 0.0)),
        ..default()
    });

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

    // let mut brush = Brush::cylinder(
    //     Vector3::new(0.0, 1.0, 1.0),
    //     CylinderDimensions {
    //         height: 1.0,
    //         radius: 0.5,
    //     },
    //     16,
    // );

    // brush.knife(SurfacePlane::new(
    //     Vector3::new(1.0, 1.0, 0.0).normalize(),
    //     0.5,
    // ));

    // let mut brush = Brush::cylinder(
    //     Vector3::new(0.0, 0.0, 0.0),
    //     CylinderDimensions {
    //         height: 1.0,
    //         radius: 0.5,
    //     },
    //     16,
    // );

    // brush.surface_groups.push(primitives::cuboid(
    //     Vector3::ZERO,
    //     CuboidDimensions {
    //         width: 1.0,
    //         height: 0.5,
    //         depth: 1.0,
    //     },
    //     BoolOperation::Subtract,
    // ));

    // let mut brush = Brush::concave_polygon(Vector3::new(0.0, 0.0, 0.0));

    // let mesh = mesh_data_to_bevy_mesh(&brush.to_mesh_data());
    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(mesh),
    //     material: materials.add(Color::srgb(0.5, 0.5, 0.5)),
    //     transform: Transform::from_translation(Vec3::new(0.0, 0.5, 0.0)),
    //     ..default()
    // });
}

pub fn csg_to_bevy_mesh(csg: &CSG) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();
    let mut index_count = 0;

    for polygon in &csg.polygons {
        let start_index = index_count;
        for vertex in &polygon.vertices {
            positions.push([
                vertex.pos.x as f32,
                vertex.pos.y as f32,
                vertex.pos.z as f32,
            ]);
            normals.push([
                vertex.normal.x as f32,
                vertex.normal.y as f32,
                vertex.normal.z as f32,
            ]);
            index_count += 1;
        }

        // Triangulate the polygon if it has more than 3 vertices
        if polygon.vertices.len() > 3 {
            for i in 1..polygon.vertices.len() - 1 {
                indices.push(start_index as u32);
                indices.push((start_index + i) as u32);
                indices.push((start_index + i + 1) as u32);
            }
        } else {
            for i in 0..polygon.vertices.len() {
                indices.push((start_index + i) as u32);
            }
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}

fn mesh_data_to_bevy_mesh(polygon_mesh: &MeshData) -> Mesh {
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
