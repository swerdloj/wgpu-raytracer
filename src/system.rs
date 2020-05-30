use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use wgpu::*;

use crate::quad::Quad;

pub struct SDL2 {
    sdl2_context: sdl2::Sdl,
    _video: sdl2::VideoSubsystem,
    window: sdl2::video::Window,
}

pub struct WGPU {
    render_surface: Surface,
    adapter: Adapter,
    pub device: Device,
    queue: Queue,
    sc_desc: SwapChainDescriptor,
    swap_chain: SwapChain,

    // TODO:
    // quad_render_pipeline
    // raytrace_compute_pipeline
}

pub struct System {
    pub sdl2: SDL2,
    pub wgpu: WGPU,

    quad_bind_group_layout: BindGroupLayout,
    quad_render_pipeline: RenderPipeline,
}

impl System {
    pub async fn new() -> Self {
        let sdl2 = Self::init_sdl2();
        let wgpu = Self::init_wgpu(&sdl2.window).await;


        // TODO: Quad program for rendering texture
        // let quad: Quad = todo!();

        // NOTE: bind_group_layouts MUST be SHARED. Creating the same one multiple times will cause errors.
        // Hence storing this in System.
        let quad_bind_group_layout = Quad::bind_group_layout(&wgpu.device);
        let quad_render_pipeline = Quad::create_render_pipeline(&wgpu.device, &quad_bind_group_layout, wgpu.sc_desc.format, None);


        // TODO: Compute program that writes to texture
        
        Self {
            sdl2,
            wgpu,
            quad_bind_group_layout,
            quad_render_pipeline,
        }
    }

    fn init_sdl2() -> SDL2 {
        let sdl2_context = sdl2::init().unwrap();
    
        let video = sdl2_context.video().unwrap();
    
        let window = video.window("Raytracing", 800, 600)
            .position_centered()
            .resizable()
            .build()
            .unwrap();

        SDL2 { sdl2_context, _video: video, window }
    }

    async fn init_wgpu(window: &sdl2::video::Window) -> WGPU {
        let window_size = window.size();

        let render_surface = Surface::create(window);

        let adapter = Adapter::request(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: Some(&render_surface),
            }, 
            BackendBit::PRIMARY,
        ).await
        .unwrap();

        let (device, queue) = adapter.request_device(&DeviceDescriptor {
            extensions: Extensions {
                anisotropic_filtering: false,
            },
            limits: Limits::default(),
        }).await;

        // For rendering outputs to screen
        let sc_desc = SwapChainDescriptor {
            usage: TextureUsage::OUTPUT_ATTACHMENT,
            format: TextureFormat::Bgra8UnormSrgb,
            width: window_size.0,
            height: window_size.1,
            present_mode: PresentMode::Fifo,
        };

        let swap_chain = device.create_swap_chain(&render_surface, &sc_desc);

        WGPU {
            render_surface,
            adapter,
            device,
            queue,
            sc_desc,
            swap_chain,
        }
    }

    fn render(&mut self, quad: &Quad) {
        let frame = self.wgpu.swap_chain.get_next_texture().unwrap();

        let mut encoder = self.wgpu.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("render_encoder"),
        });

        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            color_attachments: &[
                RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: LoadOp::Clear,
                    store_op: StoreOp::Store,
                    clear_color: Color {
                        r: 0.3, g: 0.1, b: 0.2, a: 1.0,
                    },
                },
            ],
            depth_stencil_attachment: None,
        });

        // Draw textured quad
        render_pass.set_pipeline(&self.quad_render_pipeline);
        render_pass.set_vertex_buffer(0, &quad.vertex_buffer, 0, 0);
        render_pass.set_index_buffer(&quad.index_buffer, 0, 0);
        render_pass.set_bind_group(0, &quad.bind_group, &[]);
        // 2 triangles => 6 indices
        render_pass.draw_indexed(0..6, 0, 0..1);

        drop(render_pass);

        self.wgpu.queue.submit(&[
            encoder.finish()
        ]);
    }

    // TODO: Move this to `Application` struct which handles application state as well
    pub fn run(&mut self, quad: Quad) {
        let mut event_pump = self.sdl2.sdl2_context.event_pump().unwrap();

        'run: loop {            
            for event in event_pump.poll_iter() {
                match event {
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. }
                    | Event::Quit { .. } => {
                        break 'run;
                    }

                    _ => {}
                }
            }

            self.render(&quad);
        }
    }

    pub fn create_texture_from_path<P: AsRef<std::path::Path>>(&self, path: P) -> crate::texture::Texture {
        let (texture, commands) = crate::texture::Texture::from_image_path(&self.wgpu.device, path).unwrap();
        self.wgpu.queue.submit(&[commands]);

        texture
    }

    pub fn create_quad_full_screen(&self, texture: crate::texture::Texture) -> Quad {
        Quad::create_full_screen(
            &self.wgpu.device, 
            &self.quad_bind_group_layout,
            texture,
        )
    }
}