//! Description: 
//! 
//! Basic example showing usage for using Lyon with Amethyst.
//! 
//! Copyright © 2020 Benedict Gaster. All rights reserved.
//! 
//! 
use amethyst_lyon::{
    RenderLyon,
    utils::{Mesh, VertexType, ActiveMesh}
};

use amethyst::{
    input::{
        is_close_requested, is_key_down, InputBundle, InputEvent, StringBindings, Button,
    },
    core::{
        transform::TransformBundle, 
        math::{Vector2},
    },
    prelude::*,
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow}, 
        types::DefaultBackend, 
        RenderingBundle,
    },
    winit::VirtualKeyCode,
    ecs::prelude::{Entity, WorldExt},
    assets::{Loader},
    ui::{RenderUi, UiBundle, UiText, LineMode},
    ui::{Anchor, TtfFormat, UiTransform},
    utils::{application_root_dir},
};

extern crate lyon;
use lyon::math::{point, Point};
use lyon::path::Path;
use lyon::tessellation::*;

pub struct Text {
    pub text: Entity,
}

#[derive(Debug, Default)]
pub struct BasicUsageState {}

impl SimpleState for BasicUsageState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        let stroke_options = StrokeOptions::tolerance(0.02)
        .with_line_width(4.0)
        .with_line_join(LineJoin::Round)
        .with_line_cap(LineCap::Round);

        let mut geometry: VertexBuffers<VertexType, u16> = VertexBuffers::new();
        let mut tessellator_stroke = StrokeTessellator::new();

        let red = [0.7231, 0.0685, 0.0160, 1.0];
        let blue = [0.1170, 0.1170, 0.3813, 1.];
        let yellow = [0.8963, 0.4793, 0.0452, 1.];

        let mut builder = Path::builder();
        builder.move_to(point(59.7200,15.5400));
        builder.cubic_bezier_to(point(32.8000, 43.9700), point(86.0200, 72.9800), point(59.1000, 101.4100));
        builder.line_to(point(66.6300,101.4100));
        builder.cubic_bezier_to(point(93.5500, 72.9800), point(40.3300, 43.9700), point(67.2500, 15.5400));
        let path = builder.build();
        tessellator_stroke.tessellate_path(
            &path,
            &stroke_options,
            &mut BuffersBuilder::new(&mut geometry, |pos: Point, _: StrokeAttributes| {
                VertexType {
                    position: pos.to_array(),
                    colour: red,
                }
            }),
        ).unwrap();
        let mut builder = Path::builder();
        builder.move_to(point(74.7900,15.5400));
        builder.cubic_bezier_to(point(47.8700, 43.9700), point(101.0900, 72.9800), point(74.1700, 101.4100));
        let path = builder.build();
        tessellator_stroke.tessellate_path(
            &path,
            &stroke_options,
            &mut BuffersBuilder::new(&mut geometry, |pos: Point, _: StrokeAttributes| {
                VertexType {
                    position: pos.to_array(),
                    colour: yellow,
                }
            }),
        ).unwrap();
        let mut builder = Path::builder();
        builder.move_to(point(89.8600,15.5400));
        builder.line_to(point(82.3300,15.5400));
        builder.cubic_bezier_to(point(55.4100, 43.9700), point(108.6300, 72.9800), point(81.7100, 101.4100));
        builder.line_to(point(91.2400,101.4100));
        let path = builder.build();
        tessellator_stroke.tessellate_path(
            &path,
            &stroke_options,
            &mut BuffersBuilder::new(&mut geometry, |pos: Point, _: StrokeAttributes| {
                VertexType {
                    position: pos.to_array(),
                    colour: red,
                }
            }),
        ).unwrap();
        let mut builder = Path::builder();
        builder.move_to(point(97.3900,15.5400));
        builder.cubic_bezier_to(point(70.4700, 43.9700), point(123.6900, 72.9800), point(96.7700, 101.4100));
        let path = builder.build();
        tessellator_stroke.tessellate_path(
            &path,
            &stroke_options,
            &mut BuffersBuilder::new(&mut geometry, |pos: Point, _: StrokeAttributes| {
                VertexType {
                    position: pos.to_array(),
                    colour: blue,
                }
            }),
        ).unwrap();
        let mut builder = Path::builder();
        builder.move_to(point(28.6100,101.4100));
        builder.cubic_bezier_to(point(55.5300, 72.9800), point(2.3100, 43.9700), point(29.2300, 15.5400));
        builder.line_to(point(36.9400,15.5400));
        builder.cubic_bezier_to(point(10.0200, 43.9700), point(63.2400, 72.9800), point(36.3200, 101.4100));
        builder.line_to(point(44.0300,101.4100));
        builder.cubic_bezier_to(point(70.9500, 72.9800), point(17.7300, 43.9700), point(44.6500, 15.5400));
        builder.line_to(point(52.3600,15.6000));
        builder.cubic_bezier_to(point(25.4400, 44.0300), point(78.6600, 73.0400), point(51.7400, 101.4700));
        let path = builder.build();
        tessellator_stroke.tessellate_path(
            &path,
            &stroke_options,
            &mut BuffersBuilder::new(&mut geometry, |pos: Point, _: StrokeAttributes| {
                VertexType {
                    position: pos.to_array(),
                    colour: blue,
                }
            }),
        ).unwrap();
        let mut builder = Path::builder();
        builder.move_to(point(82.0100,58.4700));
        builder.line_to(point(86.9900,58.4700));
        let path = builder.build();
        tessellator_stroke.tessellate_path(
            &path,
            &stroke_options,
            &mut BuffersBuilder::new(&mut geometry, |pos: Point, _: StrokeAttributes| {
                VertexType {
                    position: pos.to_array(),
                    colour: red,
                }
            }),
        ).unwrap();

        let mesh = Mesh {
            vertices: geometry.vertices,
            indices: geometry.indices,
            scale: Vector2::new(4., 4.),
            ..Mesh::default()
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
        let StateData { world, .. } = data;

        match &event {
            StateEvent::Window(event) => {
                if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                    Trans::Quit
                } else {
                    Trans::None
                }
            }
            StateEvent::Input(input) => {      
                Trans::None
            }
            _ => Trans::None,
        }
    }
}

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
                        .with_clear([0.0, 0.0, 0.0, 1.0]),
                )
                .with_plugin(RenderFlat2D::default())
                .with_plugin(RenderUi::default())
                .with_plugin(RenderLyon::default()),     
        )?;

    let mut game = Application::new(assets_dir, BasicUsageState::default(), game_data)?;

    game.run();
    Ok(())
}