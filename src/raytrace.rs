use wgpu::*;

use crate::quad::Quad;
use crate::texture;

#[repr(C)]
#[derive(Copy, Clone)]
// TODO: Why isn't padding needed here?
struct Uniforms {
    dimensions: cgmath::Vector2<f32>,
    sample_number: u32,
    // _padding1: [u32; 3],
    samples_per_pixel: u32,
    // _padding2: [u32; 3],
    max_ray_bounces: u32,
}
unsafe impl bytemuck::Pod for Uniforms {}
unsafe impl bytemuck::Zeroable for Uniforms {}


pub struct RayTracer {
    texture_bind_group: BindGroup,
    texture_bind_group_layout: BindGroupLayout,

    uniforms: Uniforms,
    uniform_buffer: Buffer,
    uniform_bind_group: BindGroup,

    pipeline: RenderPipeline,
}

impl RayTracer {
    const FORMAT: TextureFormat = TextureFormat::Rgba32Float;

    pub fn sample_count(&self) -> u32 {
        self.uniforms.sample_number
    }

    fn reset_samples(&mut self) {
        self.uniforms.sample_number = 1;
    }

    pub fn resize(&mut self, device: &Device, width: u32, height: u32) {
        // Reset samples to reset frame blending
        self.reset_samples();

        // Set the uniform dimensions
        self.uniforms.dimensions = (width as f32, height as f32).into();

        // Create a new texture to fit the new size
        self.texture_bind_group = Self::create_texture_bind_group(device, &self.texture_bind_group_layout, width, height);
    }

    pub fn render_to_frame(&mut self, device: &Device, queue: &Queue, frame: &TextureView) {
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("ray_trace_encoder"),
        });

        let staging_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(&[self.uniforms]), 
            BufferUsage::COPY_SRC
        );

        encoder.copy_buffer_to_buffer(
                    &staging_buffer, 0, 
            &self.uniform_buffer, 0, 
                size_of!(Uniforms) as _,
        );

        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            color_attachments: &[
                RenderPassColorAttachmentDescriptor {
                    attachment: &frame,
                    resolve_target: None,
                    load_op: LoadOp::Clear,
                    store_op: StoreOp::Store,
                    clear_color: Color {
                        r: 0.1, g: 0.05, b: 0.1, a: 1.0,
                    },
                },
            ],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
        render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
        render_pass.draw(0..6, 0..1);

        drop(render_pass);

        queue.submit(&[encoder.finish()]);

        self.uniforms.sample_number += 1;
    }

    fn create_texture_bind_group(device: &Device, layout: &BindGroupLayout, width: u32, height: u32) -> BindGroup {
        let size = Extent3d {
            width,
            height,
            depth: 1, //2d
        };

        let texture = device.create_texture(&TextureDescriptor {
            label: Some("ray_trace_texture"),
            size,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: Self::FORMAT,
            usage: TextureUsage::SAMPLED | TextureUsage::STORAGE,
        });

        let texture_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &layout,
            bindings: &[
                Binding {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture.create_default_view()),
                },
            ],
            label: Some("ray_trace_texture_bind_group"),
        });

        texture_bind_group
    }

    pub fn new(device: &Device, texture_size: (u32, u32)) -> Self {
        let vert_src = include_str!("../shaders/raytrace/rt.vert");
        let vert_spirv = glsl_to_spirv::compile(vert_src, glsl_to_spirv::ShaderType::Vertex).unwrap();
        let vert_data = read_spirv(vert_spirv).unwrap();

        let frag_src = include_str!("../shaders/raytrace/rt.frag");
        let frag_spirv = glsl_to_spirv::compile(frag_src, glsl_to_spirv::ShaderType::Fragment).unwrap();
        let frag_data = read_spirv(frag_spirv).unwrap();

        let vert_module = device.create_shader_module(&vert_data);
        let frag_module = device.create_shader_module(&frag_data);

        let texture_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            bindings: &[
                // Storage texture
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::StorageTexture {
                        dimension: TextureViewDimension::D2,
                        component_type: TextureComponentType::Uint,
                        format: Self::FORMAT,
                        readonly: false,
                    },
                },
            ],
            label: Some("ray_trace_texture_bind_group_layout"),
        });

        let texture_bind_group = Self::create_texture_bind_group(device, &texture_bind_group_layout, texture_size.0, texture_size.1);

        // let texture_view = texture.create_default_view();

        let uniforms = Uniforms {
            dimensions: (texture_size.0 as f32, texture_size.1 as f32).into(),
            sample_number: 1,
            // _padding1: [0u32; 3],
            samples_per_pixel: 2,
            // _padding2: [0u32; 3],
            max_ray_bounces: 12,
        };

        let uniform_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(&[uniforms]), 
            BufferUsage::UNIFORM | BufferUsage::COPY_DST,
        );

        let uniform_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            bindings: &[
                // Uniform buffer
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::UniformBuffer {
                        dynamic: false,
                    },
                },
            ],
            label: Some("ray_trace_uniform_bind_group_layout"),
        });

        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            bindings: &[
                Binding {
                    binding: 0,
                    resource: BindingResource::Buffer {
                        buffer: &uniform_buffer,
                        range: 0..size_of!(ref uniforms) as _,
                    },
                },
            ],
            label: Some("ray_Trace_uniform_bind_group"),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &[
                &texture_bind_group_layout,
                &uniform_bind_group_layout,
            ],
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            layout: &pipeline_layout,
            vertex_stage: ProgrammableStageDescriptor {
                module: &vert_module,
                entry_point: "main",
            },
            fragment_stage: Some(ProgrammableStageDescriptor {
                module: &frag_module,
                entry_point: "main",
            }),
            rasterization_state: Some(RasterizationStateDescriptor {
                front_face: FrontFace::Ccw,
                cull_mode: CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: PrimitiveTopology::TriangleList,
            color_states: &[
                ColorStateDescriptor {
                    format: TextureFormat::Bgra8UnormSrgb,
                    alpha_blend: BlendDescriptor::REPLACE,
                    color_blend: BlendDescriptor::REPLACE,
                    write_mask: ColorWrite::ALL,
                },
            ],
            depth_stencil_state: None,
            vertex_state: VertexStateDescriptor {
                index_format: IndexFormat::Uint16,
                vertex_buffers: &[],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        Self {
            texture_bind_group, 
            texture_bind_group_layout,

            uniforms,
            uniform_buffer,
            uniform_bind_group,

            pipeline: render_pipeline,
        }
    }
}