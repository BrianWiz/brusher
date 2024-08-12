# brusher
Experimental engine agnostic 3D CSG library for game development. Started as a port of [csg.js](https://github.com/evanw/csg.js) to Rust.

## ultimate goal
My hope is that it can essentially provide an API that can be used to create an editor like Trenchbroom, GTKRadiant, Hammer, etc by providing easy to use public methods for creating and manipulating 3D "brushes" (solids) that can be used to create levels for games.

For things like curves, I'm considering adding [curvo by @mattatz](https://github.com/mattatz/curvo) as a dependency to provide a way to create curves like pipes & arches.

## features

![image](https://github.com/user-attachments/assets/e893433f-f732-4a21-be0d-e5bbe624a115)

- [x] union
    - [ ] handle maintaining materials per surface
- [x] intersect
    - [ ] handle maintaining materials per surface 
- [x] subtract
    - [ ] handle maintaining materials per surface
- [x] knife
- [ ] extrude
- [ ] bevel
    - technically already possible manually by using `knife` but just needs a helper function
- [x] construct `Brush` from `Vec<Polygons>`
- [x] construct `Brush` from `Vec<Surface>`
    - allows you to define a convex solid by defining its surfaces (planes)
- [ ] smooth normals with configurable angle tolerance

## example (Bevy)
`cargo run --example basic`

## usage
```rs
// subtract a cube from another cube and then chop a corner off
let cube = Brush::cuboid(DVec3::new(0.0, 0.0, 0.0), DVec3::new(1.0, 1.0, 1.0));
let cube2 = Brush::cuboid(DVec3::new(0.5, 0.5, 0.5), DVec3::new(1.0, 1.0, 1.0));
let final_solid = cube
                    // subtract cube2 from cube, leaving an indent
                    .subtract(&cube2)
                    // slice off a corner
                    .knife(Plane {
                        normal: DVec3::new(1.0, 1.0, 1.0), // top right corner
                        distance: 0.5, // distance from the origin
                    });
```

## special thanks
- Thank you to [csg.js by @evanw](https://github.com/evanw/csg.js) for the original csg.js library! Your work has done some serious heavy lifting for this project and I am grateful for it.
- Thank you to [shambler by @shfty](https://github.com/QodotPlugin/shambler) for the inspiration to start this project.
