use bevy::prelude::*;
use bevy::render::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::texture::{
    ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor,
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

use brusher::brush::types::{Plane, Surface, SurfaceType};
use brusher::brush::Brush;
use glam::{DVec3, Vec2 as GlamVec2};

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

    // CSG operation
    let cube = cuboid(DVec3::new(0.0, 0.0, 0.0), DVec3::new(1.0, 1.0, 1.0));
    let cube2 = cuboid(DVec3::new(0.5, 0.5, 0.5), DVec3::new(1.0, 1.0, 1.0));
    let final_solid = cube.subtract(&cube2).knife(Plane {
        normal: DVec3::new(1.0, 1.0, 1.0),
        distance: 0.5,
    });

    let meshes_with_materials = csg_to_bevy_meshes(&final_solid);

    // Spawn each mesh with the appropriate material
    for (mesh, material_index) in meshes_with_materials {
        let material = match material_index {
            0 => material1.clone(),
            1 => material2.clone(),
            _ => material1.clone(), // default
        };

        commands.spawn(PbrBundle {
            mesh: meshes.add(mesh),
            material,
            transform: Transform::from_translation(Vec3::new(0.0, 0.5, 0.0)),
            ..default()
        });
    }
}

pub fn cuboid(origin: DVec3, dimensions: DVec3) -> Brush {
    let half_dims = dimensions * 0.5;
    let planes = vec![
        // Right
        Surface {
            normal: DVec3::new(1.0, 0.0, 0.0),
            distance: half_dims.x + origin.x,
            texture_offset: GlamVec2::new(0.0, 0.0),
            texture_scale: GlamVec2::new(1.0, 1.0),
            texture_rotation: 0.0,
            material: Some(0),
            surface_type: SurfaceType::Concrete(0),
        },
        // Left
        Surface {
            normal: DVec3::new(-1.0, 0.0, 0.0),
            distance: half_dims.x - origin.x,
            texture_offset: GlamVec2::new(0.0, 0.0),
            texture_scale: GlamVec2::new(1.0, 1.0),
            texture_rotation: 0.0,
            material: Some(1),
            surface_type: SurfaceType::Concrete(0),
        },
        // Top
        Surface {
            normal: DVec3::new(0.0, 1.0, 0.0),
            distance: half_dims.y + origin.y,
            texture_offset: GlamVec2::new(0.0, 0.0),
            texture_scale: GlamVec2::new(1.0, 1.0),
            texture_rotation: 0.0,
            material: Some(0),
            surface_type: SurfaceType::Concrete(0),
        },
        // Bottom
        Surface {
            normal: DVec3::new(0.0, -1.0, 0.0),
            distance: half_dims.y - origin.y,
            texture_offset: GlamVec2::new(0.0, 0.0),
            texture_scale: GlamVec2::new(1.0, 1.0),
            texture_rotation: 0.0,
            material: Some(1),
            surface_type: SurfaceType::Concrete(0),
        },
        // Front
        Surface {
            normal: DVec3::new(0.0, 0.0, 1.0),
            distance: half_dims.z + origin.z,
            texture_offset: GlamVec2::new(0.0, 0.0),
            texture_scale: GlamVec2::new(1.0, 1.0),
            texture_rotation: 0.0,
            material: Some(0),
            surface_type: SurfaceType::Concrete(0),
        },
        // Back
        Surface {
            normal: DVec3::new(0.0, 0.0, -1.0),
            distance: half_dims.z - origin.z,
            texture_offset: GlamVec2::new(0.0, 0.0),
            texture_scale: GlamVec2::new(1.0, 1.0),
            texture_rotation: 0.0,
            material: Some(1),
            surface_type: SurfaceType::Concrete(0),
        },
    ];

    Brush::from_surfaces(planes)
}

pub fn csg_to_bevy_meshes(csg: &Brush) -> Vec<(Mesh, usize)> {
    let mut meshes_with_materials: Vec<(Mesh, usize)> = vec![];

    for polygon in &csg.polygons {
        let mut positions = vec![];
        let mut normals = vec![];
        let mut uvs = vec![];
        let mut indices = vec![];
        let mut index_count = 0;

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

            let uv = polygon.surface.compute_texture_coordinates(vertex.pos);
            uvs.push([uv.x, uv.y]);

            index_count += 1;
        }

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
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_indices(Indices::U32(indices));

        meshes_with_materials.push((mesh, polygon.surface.material.unwrap_or(0)));
    }

    meshes_with_materials
}
