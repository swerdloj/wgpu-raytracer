use wgpu::*;

use crate::quad::Quad;
use crate::texture;

#[repr(C)]
#[derive(Copy, Clone)]
struct Uniforms {
    sample_number: u32,
    // _padding1: [u32; 3],
    samples_per_pixel: u32,
    // _padding2: [u32; 3],
    max_ray_bounces: u32,
}
unsafe impl bytemuck::Pod for Uniforms {}
unsafe impl bytemuck::Zeroable for Uniforms {}


pub struct RayTracer {
    pub quad_with_texture: Quad,
    texture_bind_group: BindGroup,

    uniforms: Uniforms,
    uniform_buffer: Buffer,
    uniform_bind_group: BindGroup,

    pipeline: ComputePipeline,

    size: (u32, u32),
}

impl RayTracer {
    pub fn dispatch_compute(&mut self, device: &Device, queue: &Queue) {
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("compute_encoder"),
        });

        let mut compute_pass = encoder.begin_compute_pass();

        compute_pass.set_pipeline(&self.pipeline);
        compute_pass.set_bind_group(0, &self.texture_bind_group, &[]);
        compute_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
        compute_pass.dispatch(self.size.0, self.size.1, 1);

        drop(compute_pass);

        // Update sample number after the compute
        self.uniforms.sample_number += 1;

        let staging_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(&[self.uniforms]), 
            BufferUsage::COPY_SRC
        );

        encoder.copy_buffer_to_buffer(
                 &staging_buffer, 0, 
            &self.uniform_buffer, 0, 
              size_of!(Uniforms) as _,
        );

        queue.submit(&[encoder.finish()]);
    }

    // fn create_compute_pipeline(device: &Device, texture_size: (u32, u32)) -> Self {
    pub fn new(device: &Device, quad_bind_group_layout: &BindGroupLayout, texture_size: (u32, u32)) -> Self {
        let compute_src = include_str!("../shaders/raytrace/test.comp");
        let compute_spirv = glsl_to_spirv::compile(compute_src, glsl_to_spirv::ShaderType::Compute).unwrap();
        let compute_data = read_spirv(compute_spirv).unwrap();

        let compute_module = device.create_shader_module(&compute_data);

        let size = Extent3d {
            width: texture_size.0,
            height: texture_size.1,
            depth: 1, //2d
        };

        let texture = device.create_texture(&TextureDescriptor {
            label: Some("compute_texture"),
            size,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsage::SAMPLED | TextureUsage::STORAGE,
        });

        let texture_view = texture.create_default_view();

        let texture_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            bindings: &[
                // Storage texture
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::COMPUTE,
                    ty: BindingType::StorageTexture {
                        dimension: TextureViewDimension::D2,
                        component_type: TextureComponentType::Uint,
                        format: TextureFormat::Rgba8UnormSrgb,
                        readonly: false,
                    },
                },
            ],
            label: Some("compute_texture_bind_group_layout"),
        });

        let texture_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            bindings: &[
                Binding {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture_view),
                },
            ],
            label: Some("compute_texture_bind_group"),
        });

        let uniforms = Uniforms {
            sample_number: 0,
            // _padding1: [0u32; 3],
            samples_per_pixel: 10,
            // _padding2: [0u32; 3],
            max_ray_bounces: 5,
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
                    visibility: ShaderStage::COMPUTE,
                    ty: BindingType::UniformBuffer {
                        dynamic: false,
                    },
                },
            ],
            label: Some("compute_uniform_bind_group_layout"),
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
            label: Some("compute_uniform_bind_group"),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &[
                &texture_bind_group_layout,
                &uniform_bind_group_layout,
            ],
        });

        let compute_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            layout: &pipeline_layout,
            compute_stage: ProgrammableStageDescriptor {
                module: &compute_module,
                entry_point: "main",
            },
        });

        let quad_texture = texture::Texture::from_wgpu_texture(device, texture);
        let quad = Quad::new(device, quad_bind_group_layout, quad_texture, Some(texture_size));

        Self {
            quad_with_texture: quad, 
            texture_bind_group, 

            uniforms,
            uniform_buffer,
            uniform_bind_group,

            pipeline: compute_pipeline,
            size: texture_size,
        }
    }
}