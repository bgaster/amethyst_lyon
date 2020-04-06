//! Custom Render Pass example

mod pass;
mod utils;

use crate::pass::{RenderLyon};
use crate::utils::{Mesh, VertexType};

use amethyst::{
    input::{
        is_close_requested, is_key_down, InputBundle, InputEvent, ScrollDirection, StringBindings,
    },
    prelude::*,
    renderer::{
        camera::Projection,
        plugins::{RenderFlat2D, RenderToWindow}, 
        types::DefaultBackend, 
        RenderingBundle,
        Camera, ActiveCamera
    },
    ecs::{Entity},
    core::{Transform},
    utils::application_root_dir,
    winit::VirtualKeyCode,
    window::ScreenDimensions,
};

extern crate lyon;
use lyon::math::{point, Point};
use lyon::path::Path;
use lyon::path::builder::*;
use lyon::tessellation::*;

use log::info;

#[derive(Debug, Default)]
pub struct CustomShaderState;

impl SimpleState for CustomShaderState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        // Hollow triangle
        let mut builder = Path::builder();

        builder.move_to(point(400., 200.));
        builder.line_to(point(600. , 200.));
        builder.line_to(point(600. , 400. ));
        builder.line_to(point(400. , 200.));

        let path = builder.build();
        let mut geometry: VertexBuffers<VertexType, u16> = VertexBuffers::new();
        let mut tessellator_stroke = StrokeTessellator::new();
        {
            let stroke_options = StrokeOptions::tolerance(0.02)
            .with_line_width(6.0)
            .with_line_join(LineJoin::Round)
            .with_line_cap(LineCap::Round);

            tessellator_stroke.tessellate_path(
                &path,
                &stroke_options,
                &mut BuffersBuilder::new(&mut geometry, |pos: Point, _: StrokeAttributes| {
                    VertexType {
                        position: pos.to_array(),
                        colour: [1., 0., 0., 1.],
                    }
                }),
            ).unwrap();
        }

        // Add a box to the current mesh
        let mut builder = Path::builder();
        builder.move_to(point(136.5, 214.5));
        builder.line_to(point(50.7, 214.5));
        builder.cubic_bezier_to(point(42.7, 214.5), point(36.2, 208.1), point(36.2, 200.1));
        builder.line_to(point(36.2, 100.7));
        builder.cubic_bezier_to(point(36.2, 92.7), point(42.7, 86.3), point(50.7, 86.3));
        builder.line_to(point(136.5, 86.3));
        builder.cubic_bezier_to(point(144.5, 86.3), point(151.0, 92.7), point(151.0, 100.7));
        builder.line_to(point(151.0, 200.1));
        builder.cubic_bezier_to(point(151.0, 208.1), point(144.5, 214.5), point(136.5, 214.5));
        builder.close();
        let path = builder.build();
        {
            let stroke_options = StrokeOptions::tolerance(0.02)
            .with_line_width(4.0)
            .with_line_join(LineJoin::Round)
            .with_line_cap(LineCap::Round);

            tessellator_stroke.tessellate_path(
                &path,
                &stroke_options,
                &mut BuffersBuilder::new(&mut geometry, |pos: Point, _: StrokeAttributes| {
                    VertexType {
                        position: pos.to_array(),
                        colour: [1., 0., 0., 1.],
                    }
                }),
            ).unwrap();
        }

        let mesh = Mesh {
            vertices: geometry.vertices,
            indices: geometry.indices,
        };
        world
            .create_entity()
            .with(mesh)
            .build();

        //-------------------------------------
        // solid triangle

        let mut builder = Path::builder();

        builder.move_to(point(100., 350.));
        builder.line_to(point(150. , 350.));
        builder.line_to(point(155. , 250. ));
        builder.line_to(point(100. , 350.));
     
        let path = builder.build();

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

        let mesh = Mesh {
            vertices: geometry.vertices,
            indices: geometry.indices,
        };
        world
            .create_entity()
            .with(mesh)
            .build();        
    }

    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        match &event {
            StateEvent::Window(event) => {
                if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                    Trans::Quit
                } else {
                    Trans::None
                }
            }
            // Using the Mouse Wheel to control the scale
            StateEvent::Input(input) => {
                if let InputEvent::MouseWheelMoved(dir) = input {
                    // let mut scale = data.world.write_resource::<CustomUniformArgs>();
                    // match dir {
                    //     ScrollDirection::ScrollUp => (*scale).scale *= 1.1,
                    //     ScrollDirection::ScrollDown => (*scale).scale /= 1.1,
                    //     _ => {}
                    // }
                }
                Trans::None
            }
            _ => Trans::None,
        }
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;
    let display_config_path = app_root.join("config/display.ron");
    let assets_dir = app_root.join("examples/assets/");

    let game_data = GameDataBuilder::default()
        .with_bundle(InputBundle::<StringBindings>::new())?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?
                        .with_clear([1.0, 1.0, 1.0, 1.0]),
                )
                .with_plugin(RenderFlat2D::default())
                // Add our custom render plugin to the rendering bundle.
                .with_plugin(RenderLyon::default()),
        )?;

    let mut game = Application::new(assets_dir, CustomShaderState::default(), game_data)?;

    game.run();
    Ok(())
}
