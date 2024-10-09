//! A compute shader that simulates Conway's Game of Life.
//!
//! Compute shaders use the GPU for computing arbitrary information, that may be independent of what
//! is rendered to the screen.

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::DVec2,
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_asset::{RenderAssetUsages, RenderAssets},
        render_graph::{self, RenderGraph, RenderLabel},
        render_resource::*,
        renderer::{RenderContext, RenderDevice},
        texture::GpuImage,
        Render, RenderApp, RenderSet,
    },
    window::WindowPlugin,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bytemuck::{Pod, Zeroable};
use std::{borrow::Cow, collections::HashMap};

const WORKGROUP_SIZE: u32 = 8;
const SIZE: (u32, u32) = (1920, 1080);

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    // uncomment for unthrottled FPS
                    // present_mode: bevy::window::PresentMode::AutoNoVsync,
                    ..default()
                }),
                ..default()
            }),
            GameOfLifeComputePlugin,
        ))
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let mut image = Image::new_fill(
        Extent3d {
            width: SIZE.0,
            height: SIZE.1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
    let image = images.add(image);

    let mut image2 = Image::new_fill(
        Extent3d {
            width: SIZE.0,
            height: SIZE.1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    image2.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
    let image2 = images.add(image2);

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(SIZE.0 as f32, SIZE.1 as f32)),
            ..default()
        },
        texture: image.clone(),
        ..default()
    });

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(SIZE.0 as f32, SIZE.1 as f32)),
            ..default()
        },
        texture: image.clone(),
        transform: Transform {
            translation: Vec3 {
                x: 1000.0,
                y: 1000.0,
                z: 0.0,
            },
            ..default()
        },
        ..default()
    });
    commands.spawn(Camera2dBundle::default());

    commands.spawn(GameOfLifeImage {
        texture: image,
        game_of_life_data: GameOfLifeUniform {
            view_scale: 10.0,
            view_pos: DVec2::new(0.0, 0.0),
            _padding: [0.0],
        },
    });

    commands.spawn(GameOfLifeImage {
        texture: image2,
        game_of_life_data: GameOfLifeUniform {
            view_scale: 10.0,
            view_pos: DVec2::new(1.0, 0.0),
            _padding: [0.0],
        },
    });
}

struct GameOfLifeComputePlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct GameOfLifeLabel;

impl Plugin for GameOfLifeComputePlugin {
    fn build(&self, app: &mut App) {
        // Extract the game of life image resource from the main world into the render world
        // for operation on by the compute shader and display on the sprite.
        app.add_plugins(ExtractComponentPlugin::<GameOfLifeImage>::default());
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .add_systems(
                Render,
                prepare_bind_group.in_set(RenderSet::PrepareBindGroups),
            )
            .insert_resource(RenderDataResource(HashMap::new()));

        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
        render_graph.add_node(GameOfLifeLabel, GameOfLifeNode::default());
    }
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct GameOfLifeUniform {
    view_scale: f64,
    view_pos: DVec2,
    _padding: [f64; 1],
}

#[derive(Clone, Debug)]
#[repr(C)]
struct GameOfLifeRenderData {
    texture_bind_group: BindGroup,
    init_pipeline: CachedComputePipelineId,
    update_pipeline: CachedComputePipelineId,
    state: GameOfLifeState,
}

#[derive(ExtractComponent, Clone, Deref, Component, AsBindGroup)]
struct GameOfLifeImage {
    #[storage_texture(0, image_format = Rgba8Unorm, access = ReadWrite)]
    texture: Handle<Image>,
    #[deref]
    game_of_life_data: GameOfLifeUniform,
}

#[derive(Resource)]
struct RenderDataResource(HashMap<AssetId<Image>, GameOfLifeRenderData>);

fn prepare_bind_group(
    gpu_images: Res<RenderAssets<GpuImage>>,
    game_of_life_images: Query<&GameOfLifeImage>,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    asset_server: Res<AssetServer>,
    mut render_data: ResMut<RenderDataResource>,
) {
    for image in game_of_life_images.iter() {
        let view = gpu_images.get(&image.texture).unwrap();

        let texture_bind_group_layout = create_texture_bind_group_layout(render_device.as_ref());

        // Create a uniform buffer
        let uniform_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("Game of Life Uniform Buffer"),
            contents: bytemuck::cast_slice(&[image.game_of_life_data]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        // Create the texture bind group
        let texture_bind_group = render_device.create_bind_group(
            None,
            &texture_bind_group_layout,
            &[
                BindGroupEntry {
                    binding: 0,
                    resource: view.texture_view.into_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &uniform_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        );

        let gaming = match &mut render_data.0.get_mut(&image.texture.id()) {
            Some(x) => {
                x.texture_bind_group = texture_bind_group;
                x.clone()
            }
            None => {
                let shader = asset_server.load("shaders/mandel.wgsl");
                let init_pipeline =
                    pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                        label: None,
                        layout: vec![texture_bind_group_layout.clone()],
                        push_constant_ranges: Vec::new(),
                        shader: shader.clone(),
                        shader_defs: vec![],
                        entry_point: Cow::from("init"),
                    });
                let update_pipeline =
                    pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                        label: None,
                        layout: vec![texture_bind_group_layout.clone()],
                        push_constant_ranges: Vec::new(),
                        shader: shader.clone(),
                        shader_defs: vec![],
                        entry_point: Cow::from("update"),
                    });
                GameOfLifeRenderData {
                    texture_bind_group,
                    init_pipeline,
                    update_pipeline,
                    state: GameOfLifeState::Loading,
                }
            }
        };

        render_data.0.insert(image.texture.id(), gaming);
    }
}

fn create_texture_bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
    let entries = &[
        BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::StorageTexture {
                access: StorageTextureAccess::ReadWrite,
                format: TextureFormat::Rgba8Unorm, // or whatever format you need
                view_dimension: TextureViewDimension::D2,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
    ];

    render_device.create_bind_group_layout("GameOfLife Texture Bind Group Layout", entries)
}

#[derive(Default, Clone, Debug)]
enum GameOfLifeState {
    #[default]
    Loading,
    Init,
    Update,
}

#[derive(Default)]
struct GameOfLifeNode {
    query: HashMap<AssetId<Image>, GameOfLifeRenderData>,
}

impl render_graph::Node for GameOfLifeNode {
    fn update(&mut self, world: &mut World) {
        self.query = world.resource::<RenderDataResource>().0.clone();
        for (_k, v) in self.query.iter_mut() {
            let pipeline_cache = world.resource::<PipelineCache>();
            match v.state.clone() {
                GameOfLifeState::Loading => {
                    if let CachedPipelineState::Ok(_) =
                        pipeline_cache.get_compute_pipeline_state(v.init_pipeline)
                    {
                        v.state = GameOfLifeState::Init;
                    }
                }
                GameOfLifeState::Init => {
                    if let CachedPipelineState::Ok(_) =
                        pipeline_cache.get_compute_pipeline_state(v.update_pipeline)
                    {
                        v.state = GameOfLifeState::Update;
                    }
                }
                GameOfLifeState::Update => {}
            }
        }
        // if the corresponding pipeline has loaded, transition to the next stage
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        for (k, bind) in self.query.iter() {
            let mut pass = render_context
                .command_encoder()
                .begin_compute_pass(&ComputePassDescriptor::default());

            pass.set_bind_group(0, &bind.texture_bind_group, &[]);
            println!("{:#?} {:#?}", bind.texture_bind_group, k);
            // select the pipeline based on the current state
            match bind.state.clone() {
                GameOfLifeState::Loading => {}
                GameOfLifeState::Init => {
                    let init_pipeline = pipeline_cache
                        .get_compute_pipeline(bind.init_pipeline)
                        .unwrap();
                    pass.set_pipeline(init_pipeline);
                    pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);
                }
                GameOfLifeState::Update => {
                    let update_pipeline = pipeline_cache
                        .get_compute_pipeline(bind.update_pipeline)
                        .unwrap();
                    pass.set_pipeline(update_pipeline);
                    pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);
                }
            }
        }
        Ok(())
    }
}
