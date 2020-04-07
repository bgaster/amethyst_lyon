 # amethyst-lyon

Amethyst-lyon provides integration for the [lyon crate](https://github.com/nical/lyon) within the [Amethyst](https://amethyst.rs) game engine.

Lyon is a path tessellation library written in rust for GPU-based 2D graphics rendering. This crate add vector graphics
to the Amethyst game engine.

## Integration

This crate provides an amethyst `RenderPlugin` (available since amethyst 0.12) which properly renders Meshes generated
by the `lyon` crate.

A minimal example is available at [examples/basic/main.rs](examples/basic/main.rs)

```
cargo run --example basic --features amethyst/metal
```

Lyon turns complex paths into triangle meshes that can be renderered by the GPU. Lyon is independent of a specifc renderer and corrdinate system, however, this crate is not. In particular, `amethyst-lyon` works in screen-space, which is additionally scaled to handle screen HiDPI.

`Lyon` translates (tesellates) paths into sets of 2D vertices and indexes, which are represented by as a ```Mesh``` in `amethyst-lyon`:

```rust
pub type IndexType = u16;

#[derive(Debug, Default)]
pub struct VertexType {
    pub position: [f32; 2],
    pub colour: [f32; 4],
}

/// Component for the triangles we wish to draw to the screen
#[derive(Debug, Default)]
pub struct Mesh {
    pub vertices: Vec<VertexType>,
    pub indices: Vec<IndexType>,  
}
```

A ```Mesh``` is a component type and thus can be associated with an entity. By default all mesh components are rendered, but this can be controlled with the use of ```ActiveMesh```:

```rust
pub struct ActiveMesh {
    pub entity: Option<Entity>,
}
```

If ```ActiveMesh``` is ```None```, its default state, then all meshes are rendered, and if it is ```Some(entity)```, then just the mesh associated with ```entity``` is rendered. This provides flexibility of generating lots of independent meshes, but with the additional cost of multiple draw calls, one for each meshes, compared with a single draw call for a single mesh. Provides the ability for debug meshes and so on.

## Usage 

This crate currently requires including the amethyst crate; this may introduce a full recompilation of amethyst due to differing features. If this is the case, you'll need to clone this git repository and and set the appropriate features. 

This create uses the amethyst `shader-compiler`, which relies on `shaderc` to compile its shaders at build  time. Finally, this crate exposes the same rendering features as amethyst, and will pass them along to amethyst.

Example Cargo.toml Usage:
```toml
amethyst-lyon = "not-yet"
```

`RenderPlugin` usage:
```rust
fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;
    let display_config_path = app_root.join("examples/config/display.ron");
    let assets_dir = app_root.join("examples/assets/");

    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_bundle(InputBundle::<StringBindings>::new())?
        .with_bundle(UiBundle::<StringBindings>::new())?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?
                        .with_clear([1.0, 1.0, 1.0, 1.0]),
                )
                .with_plugin(RenderFlat2D::default())
                .with_plugin(RenderUi::default())
                .with_plugin(RenderLyon::default()),     
        )?;

    let mut game = Application::new(assets_dir, BasicUsageState::default(), game_data)?;

    game.run();
    Ok(())
}
```

To create a mesh with a simple triangle using Lyon (for details of how Lyon works, see 
the [documentation](https://docs.rs/lyon/0.15.6/lyon/).):

```rust
// create a path builder and add a triangle
let mut builder = Path::builder();

builder.move_to(point(100., 350.));
builder.line_to(point(150. , 350.));
builder.line_to(point(155. , 250. ));
builder.line_to(point(100. , 350.));

let path = builder.build();

// allocate buffers and tessellate path
let mut geometry: VertexBuffers<VertexType, u16> = VertexBuffers::new();
let mut tessellator_fill = FillTessellator::new();
{
        tessellator_fill.tessellate_path(
        &path,
        &FillOptions::default(),
        &mut BuffersBuilder::new(&mut geometry, |pos: Point, _: FillAttributes| {
            VertexType {
                position: pos.to_array(),
                colour: [0., 1., 0., 1.],
            }
        }),
    ).unwrap();
}

// crate renderable mesh
let mesh = Mesh {
    vertices: geometry.vertices,
    indices: geometry.indices,
};

// add to world
world
    .create_entity()
    .with(mesh)
    .build());
```

## LICENSE

The `examples/assets/font/square.ttf` font is from [Amethyst crate](https://github.com/amethyst/amethyst).

Licensed under any of

    Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
    MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)
    Mozilla Public License 2.0

at your option.

Dual MIT/Apache2 is strictly more permissive.
