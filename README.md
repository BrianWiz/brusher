# brusher
Experimental engine agnostic 3D CSG library for game development written in Rust. Started as a port of [csg.js](https://github.com/evanw/csg.js) to Rust.

## ultimate goal
My hope is that it can essentially provide an API that can be used to create an editor like Trenchbroom, GTKRadiant, Hammer, etc by providing easy to use public methods for creating and manipulating 3D "brushes" (solids) that can be used to create levels for games.

For things like curves, I'm considering adding [curvo by @mattatz](https://github.com/mattatz/curvo) as a dependency to provide a way to create curves like pipes & arches.

## features & todo

![image](https://github.com/user-attachments/assets/e893433f-f732-4a21-be0d-e5bbe624a115)
^ a cuboid subtracting another cuboid, then have a corner sliced off.

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
    // Create a brush
    let mut brush = Brush::new();
    brush.knives = vec![brusher::brush::Knife {
        normal: DVec3::new(1.0, 1.0, 0.0),
        distance_from_origin: 4.0,
    }];

    // Room 1
    brush.add(Brushlet::cuboid(brusher::brush::Cuboid {
        origin: DVec3::new(0.0, 0.0, 0.0),
        width: 8.0,
        height: 4.0,
        depth: 8.0,
        material: 0,
        operation: BrushletBooleanOp::Subtract,
        knives: vec![brusher::brush::Knife {
            normal: DVec3::new(-1.0, -1.0, -1.0),
            distance_from_origin: 4.0,
        }],
        inverted: true,
    }));

    // Room 2
    brush.add(Brushlet::cuboid(brusher::brush::Cuboid {
        origin: DVec3::new(4.0, 0.0, 4.0),
        width: 8.0,
        height: 4.0,
        depth: 8.0,
        material: 1,
        operation: BrushletBooleanOp::Union,
        knives: vec![],
        inverted: false,
    }));

    let mesh_data = brush.to_mesh_data();
```

## special thanks
- Thank you to [csg.js by @evanw](https://github.com/evanw/csg.js) for the original csg.js library! Your work has done some serious heavy lifting for this project and I am grateful for it.
- Thank you to [shambler by @shfty](https://github.com/QodotPlugin/shambler) for the inspiration to start this project.
