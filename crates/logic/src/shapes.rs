use bevy::{
    prelude::*,
    render::{
        pipeline::PipelineDescriptor,
        shader::{ShaderStage, ShaderStages},
    },
};

pub struct ShapeMeshes {
    pub circle: Handle<Mesh>,
    pub pipeline_circle: Handle<PipelineDescriptor>,
}

pub struct ShapesPlugin;

impl Plugin for ShapesPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(init_shapes.system());
    }
}

pub fn init_shapes(
    mut commands: Commands,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: shaders.add(Shader::from_glsl(
            ShaderStage::Vertex,
            include_str!("shaders/shape.es.vert"),
        )),
        fragment: Some(shaders.add(Shader::from_glsl(
            ShaderStage::Fragment,
            include_str!("shaders/circle.es.frag"),
        ))),
    }));
    let m = meshes.add(Mesh::from(shape::Quad {
        size: Vec2::new(2f32, 2f32),
        flip: false,
    }));
    commands.insert_resource(ShapeMeshes {
        circle: m,
        pipeline_circle: pipeline_handle,
    })
}
