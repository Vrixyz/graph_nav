use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        pipeline::PipelineDescriptor,
        render_graph::{base, AssetRenderResourcesNode, RenderGraph},
        renderer::RenderResources,
        shader::{ShaderStage, ShaderStages},
    },
};

#[derive(RenderResources, Default, TypeUuid)]
#[uuid = "1e08866c-0b8a-437e-8bce-37733b25127e"]
pub struct ColorMaterial {
    pub color: Color,
}

pub struct ShapeMeshes {
    pub quad2x2: Handle<Mesh>,
    pub pipeline_circle: Handle<PipelineDescriptor>,
    pub pipeline_triangle: Handle<PipelineDescriptor>,
    pub mat_white: Handle<ColorMaterial>,
    pub mat_orange: Handle<ColorMaterial>,
    pub mat_fuchsia: Handle<ColorMaterial>,
    pub mat_green: Handle<ColorMaterial>,
    pub mat_gray: Handle<ColorMaterial>,
}

pub struct ShapesPlugin;

impl Plugin for ShapesPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(init_shapes.system())
            .add_asset::<ColorMaterial>();
    }
}

pub fn init_shapes(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut render_graph: ResMut<RenderGraph>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Watch for changes
    asset_server.watch_for_changes().unwrap();
    // Add an AssetRenderResourcesNode to our Render Graph. This will bind MyMaterial resources to
    // our shader
    render_graph.add_system_node(
        "color_material",
        AssetRenderResourcesNode::<ColorMaterial>::new(true),
    );
    render_graph
        .add_node_edge("color_material", base::node::MAIN_PASS)
        .unwrap();
    #[cfg(target_arch = "wasm32")]
    let vert = shaders.add(Shader::from_glsl(
        ShaderStage::Vertex,
        include_str!("../assets/shaders/shape.es.vert"),
    ));
    #[cfg(not(target_arch = "wasm32"))]
    let vert = asset_server.load::<Shader, _>("../../logic/assets/shaders/shape.vert");

    #[cfg(target_arch = "wasm32")]
    let circle_frag = shaders.add(Shader::from_glsl(
        ShaderStage::Fragment,
        include_str!("../assets/shaders/circle.es.frag"),
    ));
    #[cfg(not(target_arch = "wasm32"))]
    let circle_frag = asset_server.load::<Shader, _>("../../logic/assets/shaders/circle.frag");

    #[cfg(target_arch = "wasm32")]
    let triangle_frag = shaders.add(Shader::from_glsl(
        ShaderStage::Fragment,
        include_str!("../assets/shaders/circle.es.frag"),
    ));
    #[cfg(not(target_arch = "wasm32"))]
    let triangle_frag = asset_server.load::<Shader, _>("../../logic/assets/shaders/triangle.frag");

    let pipeline_circle_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: vert.clone(),
        fragment: Some(circle_frag),
    }));

    let pipeline_triangle_handle =
        pipelines.add(PipelineDescriptor::default_config(ShaderStages {
            vertex: vert,
            fragment: Some(triangle_frag),
        }));
    let m = meshes.add(Mesh::from(shape::Quad {
        size: Vec2::new(2f32, 2f32),
        flip: false,
    }));
    commands.insert_resource(ShapeMeshes {
        quad2x2: m,
        pipeline_circle: pipeline_circle_handle,
        pipeline_triangle: pipeline_triangle_handle,
        mat_white: materials.add(ColorMaterial {
            color: Color::WHITE,
        }),
        mat_green: materials.add(ColorMaterial {
            color: Color::GREEN,
        }),
        mat_orange: materials.add(ColorMaterial {
            color: Color::ORANGE_RED,
        }),
        mat_fuchsia: materials.add(ColorMaterial {
            color: Color::FUCHSIA,
        }),
        mat_gray: materials.add(ColorMaterial { color: Color::GRAY }),
    })
}
