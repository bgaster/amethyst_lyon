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
    core::{transform::TransformBundle},
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
pub struct BasicUsageState {
    mesh1: Option<Entity>,
    mesh2: Option<Entity>,
}

impl SimpleState for BasicUsageState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        // Add some text describing controls
        let font = world.read_resource::<Loader>().load(
            "font/square.ttf",
            TtfFormat,
            (),
            &world.read_resource(),
        );

        let text_transform = UiTransform::new(
            "tf".to_string(), Anchor::TopMiddle, Anchor::TopLeft,
            -150., -50., 1., 400., 150.,
        );
    
        let mut text = UiText::new(
            font.clone(),
            "Press 1 to display red shapes\nPress 2 to display green shapes\nPress 0 to display all shapes".to_string(),
            [0., 0., 0., 1.],
            25.,
        );

        text.line_mode = LineMode::Wrap;
        text.align = Anchor::TopLeft;

        world
            .create_entity()
            .with(text_transform)
            .with(text)
            .build();

        
        // Tesserlate a few shapes and add to the world. The first two are attached to the same
        // mesh, while the 2nd one is added to another one.

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
        self.mesh1 = Some(world
            .create_entity()
            .with(mesh)
            .build());

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
        self.mesh2 = Some(world
            .create_entity()
            .with(mesh)
            .build());
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
            // 1,2 use just mesh1 and mesh2, respectively. 0 uses both.
            StateEvent::Input(input) => {
                // Key1 implies mesh1 is the ActiveMesh and only that one is displayed
                if let InputEvent::ButtonPressed(Button::Key(VirtualKeyCode::Key1)) = input {
                    let mut active_mesh = world.write_resource::<ActiveMesh>();
                    *active_mesh = ActiveMesh { entity: self.mesh1 };
                }

                // Key1 implies mesh2 is the ActiveMesh and only that one is displyed
                if let InputEvent::ButtonPressed(Button::Key(VirtualKeyCode::Key2)) = input {
                    let mut active_mesh = world.write_resource::<ActiveMesh>();
                    *active_mesh = ActiveMesh { entity: self.mesh2 };
                }

                // Key0 implies there is no ActiveMesh and thus all meshs are displayed
                if let InputEvent::ButtonPressed(Button::Key(VirtualKeyCode::Key0)) = input {
                    let mut active_mesh = world.write_resource::<ActiveMesh>();
                    *active_mesh = ActiveMesh { entity: None };
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