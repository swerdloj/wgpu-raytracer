use wgpu::*;

pub struct RayTracer {
    texture: Texture,
    bind_group: BindGroup,
    pipeline: ComputePipeline,

    size: (u32, u32),
}

impl RayTracer {
    pub fn dispatch_compute(&self, device: &Device, queue: &Queue) {
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("compute_encoder"),
        });

        let mut compute_pass = encoder.begin_compute_pass();

        compute_pass.set_pipeline(&self.pipeline);
        compute_pass.set_bind_group(0, &self.bind_group, &[]);
        compute_pass.dispatch(self.size.0, self.size.1, 1);

        drop(compute_pass);
        queue.submit(&[encoder.finish()]);
    }

    // fn create_compute_pipeline(device: &Device, texture_size: (u32, u32)) -> Self {
    pub fn new(device: &Device, texture_size: (u32, u32)) -> Self {
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
            usage: TextureUsage::STORAGE,
        });

        let texture_view = texture.create_default_view();

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            bindings: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::COMPUTE,
                    ty: BindingType::StorageTexture {
                        dimension: TextureViewDimension::D2,
                        component_type: TextureComponentType::Uint,
                        format: TextureFormat::Rgba8UnormSrgb,
                        readonly: false,
                    },
                }
            ],
            label: Some("compute_bind_group_layout"),
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &[
                Binding {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture_view),
                },
            ],
            label: Some("compute_bind_group"),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &[
                &bind_group_layout,
            ],
        });

        let compute_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            layout: &pipeline_layout,
            compute_stage: ProgrammableStageDescriptor {
                module: &compute_module,
                entry_point: "main",
            },
        });

        Self {
            texture, 
            bind_group, 
            pipeline: compute_pipeline,
            size: texture_size,
        }
    }
}