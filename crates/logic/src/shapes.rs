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

// TODO: convert to version 300 es
// lyon es version:
// https://github.com/Nilirad/bevy_prototype_lyon/blob/a6f65791b2eea1c6f10257a0f8c541a5c5a87914/src/render/shape.es.frag
// maybe take example of Lyon, they use a different shader if targetting wasm
// https://github.com/Nilirad/bevy_prototype_lyon/blob/79cdb49888bda1455cf1ed5fee6aa3d5a955385f/src/render/mod.rs
const VERTEX_SHADER: &str = r#"
#version 450
layout(location = 2) in vec2 Vertex_Uv;
layout(location = 0) in vec3 Vertex_Position;
layout(location = 2) out vec2 v_Uv;
layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};
layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};
void main() {
    gl_Position = ViewProj * Model * vec4(Vertex_Position, 1.0);
    v_Uv = Vertex_Uv;
}
"#;

const FRAGMENT_CIRCLE: &str = r#"
#version 450
layout(location = 0) out vec4 o_Target;
layout(location = 2) in vec2 v_Uv;


float draw_circle(vec2 coord, float radius) {
    float pct = length(coord - vec2(0.5));
    pct = 1.0 - pct;
    pct = smoothstep(0.5, 0.6, pct);
    return pct;
}
float draw_circle_hard(vec2 coord, float radius) {
    return step(length(coord), radius);
}
void main() {
    float circle = draw_circle_hard(v_Uv - vec2(0.5), 0.5);

    if (circle < 0.01) {
        discard;
    }
    vec3 color = vec3(circle * 0.6f, 0.0, 0.0);
    o_Target = vec4(color, circle);
}
"#;
