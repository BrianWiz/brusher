# brusher
Experimental engine agnostic 3D CSG library for game development written in Rust. Started as a port of [csg.js](https://github.com/evanw/csg.js) to Rust.

## ultimate goal
My hope is that it can essentially provide an API that can be used to create an editor like Trenchbroom, GTKRadiant, Hammer, etc by providing easy to use public methods for creating and manipulating 3D "brushes" (solids) that can be used to create levels for games.

For things like curves, I'm considering adding [curvo by @mattatz](https://github.com/mattatz/curvo) as a dependency to provide a way to create curves like pipes & arches.

## features & todo

https://github.com/user-attachments/assets/c79d244f-47bc-4c98-81f9-dfb46ed5fb86

- [x] union
- [x] intersect
- [x] subtract
- [x] knife (WIP)
    - [ ] handle maintaining materials per surface
- [ ] serialization
- [ ] extrude
- [ ] bevel
    - technically already possible manually by using `knife` but just needs a helper function
- [x] construct `Brushlet` from `Vec<Polygons>`
- [x] construct `Brushlet` from `Vec<Surface>`
    - allows you to define a convex solid by defining its surfaces (planes)
- [ ] smooth normals with configurable angle tolerance
- [ ] editor API (WIP)

## example (Bevy)
`cargo run --example basic`

## usage
```rs
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

    // Create a brush that will combine two rooms
    let mut brush = Brush::new("Rooms");

    // Create a brushlet for the first room
    brush.add_brushlet(Brushlet::from_cuboid(
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
    brush.add_brushlet(Brushlet::from_cuboid(
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
```

### construct meshes from a brush
This example uses bevy, but you should be able to adapt it to any engine that supports meshes.
```rs

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

    // Create a brush (see above example)
    // ...

    // Define some materials
    let material_proto_grey = materials.add(Color::rgb(0.5, 0.5, 0.5).into());
    let material_proto_green = materials.add(Color::rgb(0.2, 0.8, 0.2).into());

    // Get the mesh data from the brush
    let mesh_data = brush.to_mesh_data()

    // Create a mesh for each face in the mesh data
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

    // Spawn the meshes and assign materials based on the material index
    for (mesh, material_index) in meshes_with_materials {
        let material = match material_index {
            MyMaterials::ProtoGrey => material_proto_grey.clone(),
            MyMaterials::ProtoGreen => material_proto_green.clone(),
            _ => material_proto_grey.clone(),
        };

        commands.spawn(PbrBundle {
            mesh: meshes.add(mesh),
            material,
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..default()
        });
    }
```

## special thanks
- Thank you to [csg.js by @evanw](https://github.com/evanw/csg.js) for the original csg.js library! Your work has done some serious heavy lifting for this project and I am grateful for it.
- Thank you to [shambler by @shfty](https://github.com/QodotPlugin/shambler) for the inspiration to start this project.
