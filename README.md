# brusher
Experimental engine agnostic 3D CSG library for game development.

- [x] union
- [x] intersect
- [x] subtract
- [x] knife
- [ ] extrude
- [ ] bevel
    - technically already possible manually by using `knife` but just needs a helper function
- [x] construct `Brush` from `Vec<Polygons>`
- [x] construct `Brush` from `Vec<Surface>`
    - allows you to define a convex solid by defining its surfaces (planes)
- [ ] smooth normals
