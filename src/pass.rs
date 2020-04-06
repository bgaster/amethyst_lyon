use amethyst::{
    core::{
        ecs::{
            DispatcherBuilder, Join, ReadStorage, SystemData, World,
        },
        math::{Vector2},
    },
    prelude::*,
    renderer::{
        bundle::{RenderOrder, RenderPlan, RenderPlugin, Target},
        pipeline::{PipelineDescBuilder, PipelinesBuilder},
        rendy::{
            command::{QueueId, RenderPassEncoder},
            factory::Factory,
            graph::{
                render::{PrepareResult, RenderGroup, RenderGroupDesc},
                GraphContext, NodeBuffer, NodeImage,
            },
            hal::{self, device::Device, pso, pso::ShaderStageFlags},
            mesh::{AsVertex},
            shader::{Shader, SpirvShader},
        },
        submodules::{DynamicIndexBuffer, DynamicVertexBuffer},
        types::Backend,
        util, ChangeDetection,
    },
    window::ScreenDimensions,
};

use amethyst_error::Error;
use derivative::Derivative;


use crate::utils::{Mesh, CustomArgs, PushConstant};

// Load SPIV shaders
// Note: Shaders are pre-built using build.rs and just load binaries.
lazy_static::lazy_static! {
    static ref VERTEX: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("../assets/gen/shaders/lyon.vert.spv"),
        ShaderStageFlags::VERTEX,
        "main",
    ).unwrap();

    static ref FRAGMENT: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("../assets/gen/shaders/lyon.frag.spv"),
        ShaderStageFlags::FRAGMENT,
        "main",
    ).unwrap();
}

#[derive(Clone, Debug, PartialEq, Derivative)]
#[derivative(Default(bound = ""))]
pub struct DrawLyonDesc;

impl DrawLyonDesc {
    /// Create instance of `DrawLyonDesc` render group
    pub fn new() -> Self {
        Default::default()
    }
}

impl<B: Backend> RenderGroupDesc<B, World> for DrawLyonDesc {
    fn build(
        self,
        _ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        _queue: QueueId,
        _world: &World,
        framebuffer_width: u32,
        framebuffer_height: u32,
        subpass: hal::pass::Subpass<'_, B>,
        _buffers: Vec<NodeBuffer>,
        _images: Vec<NodeImage>,
    ) -> Result<Box<dyn RenderGroup<B, World>>, failure::Error> {
        let vertex = DynamicVertexBuffer::new();
        let index = DynamicIndexBuffer::new();

        let (pipeline, pipeline_layout) = build_custom_pipeline(
            factory,
            subpass,
            framebuffer_width,
            framebuffer_height,
            Vec::new(),
        )?;

        Ok(Box::new(DrawCustom::<B> {
            pipeline,
            pipeline_layout,
            vertex,
            index,
            vertex_count: 0,
            index_count: 0,
            change: Default::default(),
            constant: PushConstant::default(),
            commands: Vec::new(),
        }))
    }
}

#[derive(Debug)]
struct DrawCmdOps {
	vertex_range: std::ops::Range<u32>,
	index_range: std::ops::Range<u32>,
	//scissor: hal::pso::Rect,
}

/// Draws triangles to the screen.
#[derive(Debug)]
pub struct DrawCustom<B: Backend> {
    pipeline: B::GraphicsPipeline,
    pipeline_layout: B::PipelineLayout,
    vertex: DynamicVertexBuffer<B, CustomArgs>,
    index: DynamicIndexBuffer<B, u16>,
    vertex_count: usize,
    index_count: usize,
    change: ChangeDetection,
    constant: PushConstant,
    commands: Vec<DrawCmdOps>,
}

impl<B: Backend> RenderGroup<B, World> for DrawCustom<B> {
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        world: &World,
    ) -> PrepareResult {
        let (mesh,) = <(ReadStorage<'_, Mesh>,)>::fetch(world);
        
        let screen_dimensions = world.read_resource::<ScreenDimensions>();
        let (width, height) = {
            (screen_dimensions.width(), screen_dimensions.height())
        };

        // setup scaling from screen space to homgenous coords (including HIDPI scaling)
        let hidpi = screen_dimensions.hidpi_factor() as f32;
        self.constant
			.set_scale(Vector2::new(hidpi * (2.0 / width), hidpi * (2.0 / height)));
        self.constant.set_translation(Vector2::new(-1.0, -1.0));

        //Update vertex count and see if it has changed
        let old_vertex_count = self.vertex_count;
        let old_index_count = self.index_count;

        self.vertex_count = mesh.join().fold(0, |sum, mesh| sum + mesh.vertices.len());
        self.index_count = mesh.join().fold(0, |sum, mesh| sum + mesh.indices.len());
        let changed = old_vertex_count != self.vertex_count || old_index_count != self.index_count;

        let mut index_range = std::ops::Range::<u32> { start: 0, end: 0 };

        let mut vertices = Vec::with_capacity(self.vertex_count as usize);
		let mut indices = Vec::with_capacity(self.index_count as usize);

        for m in mesh.join() {
            index_range.start = index_range.end;
            index_range.end += m.indices.len() as u32;
            
            self.commands.push(DrawCmdOps {
                vertex_range: std::ops::Range {
                    start: vertices.len() as u32,
					end: (vertices.len() + m.vertices.len()) as u32,
                },
                index_range: index_range.clone(),
            });

            vertices.extend(m.get_args().iter().map(|v| (*v)).collect::<Vec<CustomArgs>>());
			indices.extend(m.indices.iter().map(|v| (*v).into()).collect::<Vec<u16>>());
        }

        self.vertex.write(factory, index, vertices.len() as u64, &[vertices.iter()]);
        self.index.write(factory, index, indices.len() as u64, &[indices.iter()]);

        // Return with we can reuse the draw buffers using the utility struct ChangeDetection
        self.change.prepare_result(index, changed)
    }

    fn draw_inline(
        &mut self,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        _world: &World,
    ) {
        // Don't worry about drawing if there are no vertices. Like before the state adds them to the screen.
        if self.vertex_count == 0 {
            return;
        }

        // Bind the pipeline to the the encoder
        let layout = &self.pipeline_layout;
        encoder.bind_graphics_pipeline(&self.pipeline);

        // Bind the vertex buffer to the encoder
        self.vertex.bind(index, 0, 0, &mut encoder);
		self.index.bind(index, 0, &mut encoder);

        for draw in &self.commands {
            // Draw the vertices
            unsafe {
                encoder.push_constants(
                    layout,
                    pso::ShaderStageFlags::VERTEX,
                    0,
                    hal::memory::cast_slice::<f32, u32>(self.constant.raw()),
                );

                encoder.draw_indexed(
					draw.index_range.clone(),
					draw.vertex_range.start as i32,
					std::ops::Range { start: 0, end: 1 },
				);
            }
        }

        self.commands.clear();
    }

    fn dispose(self: Box<Self>, factory: &mut Factory<B>, _world: &World) {
        unsafe {
            factory.device().destroy_graphics_pipeline(self.pipeline);
            factory
                .device()
                .destroy_pipeline_layout(self.pipeline_layout);
        }
    }
}

fn build_custom_pipeline<B: Backend>(
    factory: &Factory<B>,
    subpass: hal::pass::Subpass<'_, B>,
    framebuffer_width: u32,
    framebuffer_height: u32,
    layouts: Vec<&B::DescriptorSetLayout>,
) -> Result<(B::GraphicsPipeline, B::PipelineLayout), failure::Error> {

    let pipeline_layout = unsafe {
		factory
			.device()
			.create_pipeline_layout(layouts, &[(pso::ShaderStageFlags::VERTEX, 0..16)])
	}?;

    // Load the shaders
    let shader_vertex = unsafe { VERTEX.module(factory).unwrap() };
    let shader_fragment = unsafe { FRAGMENT.module(factory).unwrap() };

    // Build the pipeline
    let pipes = PipelinesBuilder::new()
        .with_pipeline(
            PipelineDescBuilder::new()
                // This Pipeline uses our custom vertex description and does not use instancing
                .with_vertex_desc(&[(CustomArgs::vertex(), pso::VertexInputRate::Vertex)])
                .with_input_assembler(pso::InputAssemblerDesc::new(hal::Primitive::TriangleList))
                // Add the shaders
                .with_shaders(util::simple_shader_set(
                    &shader_vertex,
                    Some(&shader_fragment),
                ))
                .with_layout(&pipeline_layout)
                .with_subpass(subpass)
                .with_framebuffer_size(framebuffer_width, framebuffer_height)
                .with_baked_states(hal::pso::BakedStates {
					viewport: Some(hal::pso::Viewport {
						rect: hal::pso::Rect {
							x: 0,
							y: 0,
							w: framebuffer_width as i16,
							h: framebuffer_height as i16,
						},
						depth: 0.0..1.0,
					}),
					scissor: None,
					..Default::default()
				})
                // We are using alpha blending
                .with_blend_targets(vec![pso::ColorBlendDesc {
                    mask: pso::ColorMask::ALL,
                    blend: Some(pso::BlendState::ALPHA),
                }]),
        )
        .build(factory, None);

    // Destoy the shaders once loaded
    unsafe {
        factory.destroy_shader_module(shader_vertex);
        factory.destroy_shader_module(shader_fragment);
    }

    // Handle the Errors
    match pipes {
        Err(e) => {
            unsafe {
                factory.device().destroy_pipeline_layout(pipeline_layout);
            }
            Err(e)
        }
        Ok(mut pipes) => Ok((pipes.remove(0), pipeline_layout)),
    }
}

/// A [RenderPlugin] for our custom plugin
#[derive(Default, Debug)]
pub struct RenderLyon {}

impl<B: Backend> RenderPlugin<B> for RenderLyon {
    fn on_build<'a, 'b>(
        &mut self,
        world: &mut World,
        _builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        // Add the required components to the world ECS
        world.register::<Mesh>();
        Ok(())
    }

    fn on_plan(
        &mut self,
        plan: &mut RenderPlan<B>,
        _factory: &mut Factory<B>,
        _world: &World,
    ) -> Result<(), Error> {
        plan.extend_target(Target::Main, |ctx| {
            // Add our Description
            ctx.add(RenderOrder::Transparent, DrawLyonDesc::new().builder())?;
            Ok(())
        });
        Ok(())
    }
}



