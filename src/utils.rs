//! Description: 
//! 
//! Basic structures used for the Lyon render plugin.
//! 
//! Copyright © 2020 Benedict Gaster. All rights reserved.
//! 
use amethyst::{
    core::ecs::{Component, DenseVecStorage, Entity},
    core::{
		math::{Vector2, Vector4},
	},
    renderer::{
        rendy::{
            hal::{format::Format},
            mesh::{AsVertex, VertexFormat},
        },
    },
};

use glsl_layout::*;

/// Vertex Arguments to pass into shader.
/// VertexData in shader:
/// layout(location = 0) out VertexData {
///    vec2 pos;
///    vec4 color;
/// } vertex;
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, AsStd140)]
#[repr(C, align(4))]
pub struct CustomArgs {
    /// vec2 pos;
    pub pos: vec2,
    /// vec4 color;
    pub color: vec4,
}

/// Required to send data into the shader.
/// These names must match the shader.
impl AsVertex for CustomArgs {
    fn vertex() -> VertexFormat {
        VertexFormat::new((
            // vec2 pos;
            (Format::Rg32Sfloat, "pos"),
            // vec4 color;
            (Format::Rgba32Sfloat, "color"),
        ))
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct PushConstant {
	inner: Vector4<f32>,
}

impl PushConstant {
	pub fn new(scale_x: f32, scale_y: f32, trans_x: f32, trans_y: f32) -> Self {
		Self {
			inner: Vector4::new(scale_x, scale_y, trans_x, trans_y),
		}
	}

	pub fn raw(&self) -> &[f32] { &self.inner.data }

	pub fn scale(&self) -> Vector2<f32> { Vector2::new(self.inner.x, self.inner.y) }

	pub fn translation(&self) -> Vector2<f32> { Vector2::new(self.inner.z, self.inner.w) }

	pub fn set_scale(&mut self, scale: Vector2<f32>) {
		self.inner.x = scale.x;
		self.inner.y = scale.y;
	}

	pub fn set_translation(&mut self, translation: Vector2<f32>) {
		self.inner.z = translation.x;
		self.inner.w = translation.y;
    }
}

impl Default for PushConstant {
	fn default() -> Self {
		Self {
			inner: Vector4::new(1.0, 1.0, 0.0, 0.0),
		}
	}
}

pub type IndexType = u16;

/// Vertex information
#[derive(Debug, Default)]
pub struct VertexType {
    /// 2D position of vertex
    pub position: [f32; 2],
    /// Colour of vertex
    pub colour: [f32; 4],
}

/// Component for the triangles to be drawn to the screen
#[derive(Debug, Default)]
pub struct Mesh {
    /// list of vertices contained within mesh
    pub vertices: Vec<VertexType>,
    /// indices for vertices of each triangle in mesh
    pub indices: Vec<IndexType>,  
}

impl Component for Mesh {
    type Storage = DenseVecStorage<Self>;
}

impl Mesh {
    /// Returns an vector of vertices expected by GLSL vert shader
    pub fn get_args(&self) -> Vec<CustomArgs> {
        let mut vec = Vec::new();
        vec.extend((0..self.vertices.len()).map(|i| {
            //let vt: VertexType = self.vertices[i].clone();
            CustomArgs {
            pos: self.vertices[i].position.into(),
            color: self.vertices[i].colour.into(),
        }}));
        vec
    }
}

/// Active mesh resource, used by the renderer to choose 
/// which Lyon mesh to render. If no active mesh is found, 
/// then all Lyon meshes are rendered.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ActiveMesh {
    /// Entity of mesh to be rendered
    pub entity: Option<Entity>,
}

impl Component for ActiveMesh {
    type Storage = DenseVecStorage<Self>;
}