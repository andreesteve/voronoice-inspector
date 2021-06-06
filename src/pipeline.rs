use bevy::{
    prelude::*,
    asset::{Assets, HandleUntyped},
    reflect::TypeUuid,
    render::{
        render_graph::{
            RenderGraph, AssetRenderResourcesNode,
            base::{self, MainPass}
         },
        renderer::RenderResources,
        pipeline::{
            RenderPipeline, PipelineDescriptor
        },
        shader::{Shader, ShaderStage, ShaderStages},
    }
};

pub const FORWARD_PIPELINE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, 12148362314032771289);

pub(crate) fn build_forward_pipeline(shaders: &mut Assets<Shader>) -> PipelineDescriptor {
    PipelineDescriptor {
        ..PipelineDescriptor::default_config(ShaderStages {
            vertex: shaders.add(Shader::from_glsl(
                ShaderStage::Vertex,
                include_str!("forward.vert"),
            )),
            fragment: Some(shaders.add(Shader::from_glsl(
                ShaderStage::Fragment,
                include_str!("forward frag"),
            ))),
        })
    }
}

#[derive(Default)]
pub struct VertexColorPlugin;

impl Plugin for VertexColorPlugin {
    fn build(&self, app: &mut AppBuilder) {
        const COLOR_NODE: &str = "my_material_with_vertex_color_support";
        app.add_asset::<VertexColor>();

        let mut graph = app.world_mut().get_resource_mut::<RenderGraph>().unwrap();

        graph.add_system_node(
            COLOR_NODE,
            AssetRenderResourcesNode::<VertexColor>::new(true),
        );

        graph
            .add_node_edge(
                COLOR_NODE,
                base::node::MAIN_PASS,
            )
            .unwrap();

        let mut shaders = app.world_mut().get_resource_mut::<Assets<Shader>>().unwrap();
        let pipeline = build_forward_pipeline(&mut shaders);

        let mut pipelines = app.world_mut().get_resource_mut::<Assets<PipelineDescriptor>>().unwrap();
        pipelines.set_untracked(
            FORWARD_PIPELINE_HANDLE,
            pipeline,
        );

        app.world_mut().get_resource_mut::<Assets<VertexColor>>()
            .unwrap()
            .add(Default::default());
    }
}

#[derive(RenderResources, Default, TypeUuid)]
#[uuid = "0320b9b8-b3a3-4baa-8bfa-c94008177b17"]
pub struct VertexColor {}

#[derive(Bundle)]
pub struct ColorBundle {
    pub mesh: Handle<Mesh>,
    pub main_pass: MainPass,
    pub draw: Draw,
    pub visible: Visible,
    pub render_pipelines: RenderPipelines,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub color: VertexColor,
}

impl Default for ColorBundle {
    fn default() -> Self {
        Self {
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                FORWARD_PIPELINE_HANDLE.typed(),
            )]),
            mesh: Default::default(),
            visible: Default::default(),
            main_pass: Default::default(),
            draw: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
            color: Default::default(),
        }
    }
}