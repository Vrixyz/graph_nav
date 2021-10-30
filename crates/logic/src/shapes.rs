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

#[derive(RenderResources, Default, TypeUuid)]
#[uuid = "1e08866c-0b8a-437e-8bce-37733b25127f"]
pub struct CircleGaugeMaterial {
    pub color: Color,
    pub ratio: f32,
}

pub struct ShapeMeshes {
    pub quad2x2: Handle<Mesh>,
    pub pipeline_circle: Handle<PipelineDescriptor>,
    pub pipeline_triangle: Handle<PipelineDescriptor>,
    pub pipeline_circle_gauge: Handle<PipelineDescriptor>,
    pub mat_white: Handle<ColorMaterial>,
    pub mat_orange: Handle<ColorMaterial>,
    pub mat_fuchsia: Handle<ColorMaterial>,
    pub mat_green: Handle<ColorMaterial>,
    pub mat_gray: Handle<ColorMaterial>,
    pub mat_circle_gauge: Handle<CircleGaugeMaterial>,
}

pub struct ShapesPlugin;

impl Plugin for ShapesPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(init_shapes.system())
            .add_asset::<ColorMaterial>()
            .add_asset::<CircleGaugeMaterial>();
    }
}

pub fn init_shapes(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut render_graph: ResMut<RenderGraph>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials_color: ResMut<Assets<ColorMaterial>>,
    mut materials_circle_gauge: ResMut<Assets<CircleGaugeMaterial>>,
) {
    // Watch for changes
    asset_server.watch_for_changes().unwrap();
    // Add an AssetRenderResourcesNode to our Render Graph. This will bind MyMaterial resources to
    // our shader
    render_graph.add_system_node(
        "color_material",
        AssetRenderResourcesNode::<ColorMaterial>::new(true),
    );
    render_graph.add_system_node(
        "circle_gauge_material",
        AssetRenderResourcesNode::<CircleGaugeMaterial>::new(true),
    );
    render_graph
        .add_node_edge("circle_gauge_material", base::node::MAIN_PASS)
        .unwrap();

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
        include_str!("../assets/shaders/triangle.es.frag"),
    ));
    #[cfg(not(target_arch = "wasm32"))]
    let triangle_frag = asset_server.load::<Shader, _>("../../logic/assets/shaders/triangle.frag");

    let pipeline_circle_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: vert.clone(),
        fragment: Some(circle_frag),
    }));

    let pipeline_triangle_handle =
        pipelines.add(PipelineDescriptor::default_config(ShaderStages {
            vertex: vert.clone(),
            fragment: Some(triangle_frag),
        }));
    #[cfg(not(target_arch = "wasm32"))]
    let circle_gauge_frag =
        asset_server.load::<Shader, _>("../../logic/assets/shaders/circle_gauge.frag");
    #[cfg(target_arch = "wasm32")]
    let circle_gauge_frag = shaders.add(Shader::from_glsl(
        ShaderStage::Fragment,
        include_str!("../assets/shaders/circle_gauge.es.frag"),
    ));
    // For dynamic, copy ../../logic/assets into web/public and use that code
    // let circle_gauge_frag = asset_server.load::<Shader, _>("shaders/circle_gauge.es.frag");

    let pipeline_circle_gauge_handle =
        pipelines.add(PipelineDescriptor::default_config(ShaderStages {
            vertex: vert.clone(),
            fragment: Some(circle_gauge_frag),
        }));
    let m = meshes.add(Mesh::from(shape::Quad {
        size: Vec2::new(2f32, 2f32),
        flip: false,
    }));
    commands.insert_resource(ShapeMeshes {
        quad2x2: m,
        pipeline_circle: pipeline_circle_handle,
        pipeline_triangle: pipeline_triangle_handle,
        mat_white: materials_color.add(ColorMaterial {
            color: Color::WHITE,
        }),
        mat_green: materials_color.add(ColorMaterial {
            color: Color::GREEN,
        }),
        mat_orange: materials_color.add(ColorMaterial {
            color: Color::ORANGE_RED,
        }),
        mat_fuchsia: materials_color.add(ColorMaterial {
            color: Color::FUCHSIA,
        }),
        mat_gray: materials_color.add(ColorMaterial { color: Color::GRAY }),
        pipeline_circle_gauge: pipeline_circle_gauge_handle,
        mat_circle_gauge: materials_circle_gauge.add(CircleGaugeMaterial {
            ratio: 0.5f32,
            color: Color::BEIGE,
        }),
    })
}
