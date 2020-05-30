use image::GenericImageView;

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub fn from_image_path<P: AsRef<std::path::Path>>(device: &wgpu::Device, path: P) -> Result<(Self, wgpu::CommandBuffer), String> {
        let img = image::open(path.as_ref()).map_err(|e| e.to_string())?;
        let img_rgba = img.to_rgba();
        let dimensions = img.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth: 1, // represent the 2d texture by setting depth to 1
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: path.as_ref().to_str(),
            size,
            // Multiple textures can be stored in one Texture object
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            // SAMPLED => use in shaders,   COPY_DST => copy data to this texture (this is the copy destination)
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });

        let buffer = device.create_buffer_with_data(
            &img_rgba, 
            // COPY_SRC => copy data from this buffer (this is the copy source)
            wgpu::BufferUsage::COPY_SRC
        );

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("texture_buffer_copy_encoder"),
        });

        encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer: &buffer,
                offset: 0,
                bytes_per_row: 4 * dimensions.0, // RGBA = 4 bytes
                rows_per_image: dimensions.1,
            }, 
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                array_layer: 0,
                origin: wgpu::Origin3d::ZERO,
            }, 
            size,
        );

        // Command buffer has the copy instruction. Submit to the queue to execute.
        let command_buffer = encoder.finish();

        let view = texture.create_default_view();

        // Similar functionality to OpenGL textures, but 3d default
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: wgpu::CompareFunction::Always,
        });

        Ok((Self { texture, view, sampler }, command_buffer))
    }
}