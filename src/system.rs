use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::{Scancode, Keycode, KeyboardState};

use wgpu::*;

use crate::timing::Timer;
use crate::quad::{Quad, QuadBuilder};
use crate::raytrace::RayTracer;
use crate::application::ApplicationState;

pub enum Message {
    /// Application should exit
    Quit,
    /// Accumulator should be reset (when camera is moved). Event is consumed.
    RestartRender,
    /// Event should not be passed forwrd
    ConsumeEvent,
    /// No action to be taken
    Nothing,
}

// I plan on using a similar API for various components. For example, a UI, a console, the camera, and so on
pub trait Runnable {
    /// Called *once* as soon as program main loop begins
    fn init(&mut self, sdl2: &SDL2);
    /// Called for *every* event in a frame.
    fn update(&mut self, sdl2: &SDL2, raytracer: &mut RayTracer, event: &Event) -> Message;
    /// Called once per frame
    fn fixed_update(&mut self, sdl2: &SDL2, keys: &KeyboardState, raytracer: &mut RayTracer);
}

pub struct SDL2 {
    sdl2_context: sdl2::Sdl,
    _video: sdl2::VideoSubsystem,
    window: sdl2::video::Window,
}

impl SDL2 {
    pub fn set_relative_mouse_mode(&self, on: bool) {
        self.sdl2_context.mouse().set_relative_mouse_mode(on);
    }

    // TODO: Allow user to choose between borderless or normal
    pub fn toggle_full_screen(&mut self) -> (u32, u32) {
        match self.window.fullscreen_state() {
            // Enable borderless
            sdl2::video::FullscreenType::Off => {
                self.window.set_fullscreen(sdl2::video::FullscreenType::Desktop).unwrap();
            }

            // Disable fullscreen
            sdl2::video::FullscreenType::Desktop
            | sdl2::video::FullscreenType::True => {
                self.window.set_fullscreen(sdl2::video::FullscreenType::Off).unwrap();
            }
        }

        self.window.size()
    }
}

pub struct WGPU {
    render_surface: Surface,
    adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
    sc_desc: SwapChainDescriptor,
    pub swap_chain: SwapChain,
}

pub struct System {
    pub sdl2: SDL2,
    pub wgpu: WGPU,
    pub timer: Timer,

    state: ApplicationState,

    quad_bind_group_layout: BindGroupLayout,
    quad_render_pipeline: RenderPipeline,

    raytracer: RayTracer,
}

impl System {
    // TODO: Need some way to use RayTracer and render it properly without & vs &mut issues in `run`
    pub async fn new(width: u32, height: u32) -> Self {
        let sdl2 = Self::init_sdl2(width, height);
        let wgpu = Self::init_wgpu(&sdl2.window).await;
        let timer = Timer::from_sdl2_context(&sdl2.sdl2_context);

        // NOTE: bind_group_layouts MUST be SHARED. Creating the same one multiple times will cause errors.
        // Hence storing this in System.
        let quad_bind_group_layout = Quad::bind_group_layout(&wgpu.device);
        let quad_render_pipeline = Quad::create_render_pipeline(&wgpu.device, &quad_bind_group_layout, wgpu.sc_desc.format, None);

        let raytracer = RayTracer::new(&wgpu.device, width, height, 100);
        
        let state = ApplicationState::new();

        Self {
            sdl2,
            wgpu,
            timer,
            state,

            quad_bind_group_layout,
            quad_render_pipeline,

            raytracer,
        }
    }

    fn init_sdl2(width: u32, height: u32) -> SDL2 {
        let sdl2_context = sdl2::init().unwrap();
    
        let video = sdl2_context.video().unwrap();
    
        let window = video.window("Ray Tracing", width, height)
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
            // Using Vulkan-style shaders
            BackendBit::VULKAN,
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
            format: TextureFormat::Bgra8Unorm,
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
                        r: 0.1, g: 0.05, b: 0.1, a: 1.0,
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

    fn resize(&mut self, width: u32, height: u32) {
        self.wgpu.sc_desc.width = width as u32;
        self.wgpu.sc_desc.height = height as u32;

        self.wgpu.swap_chain = self.wgpu.device.create_swap_chain(&self.wgpu.render_surface, &self.wgpu.sc_desc);

        // This will trigger the sample count reset
        self.raytracer.resize(&self.wgpu.device, width, height)
    }

    // TODO: A lot of this can probably be simplified
    pub fn run(&mut self) {
        let mut event_pump = self.sdl2.sdl2_context.event_pump().unwrap();
        
        // TODO: Finish implementing timer
        self.timer.start();

        self.state.init(&self.sdl2);

        'run: loop {
            if !self.raytracer.pause_rendering {
                if self.raytracer.sample_count() == self.raytracer.target_samples {
                    println!("Target sample count reached.");
                    self.raytracer.pause_rendering = true;
                } else {   
                    let frame_view = &self.wgpu.swap_chain.get_next_texture().unwrap().view;

                    // Render directly to the screen
                    self.raytracer.render_to_frame(&self.wgpu.device, &self.wgpu.queue, frame_view);

                    let (width, height) = self.sdl2.window.size();
                    crate::text::render_text(&mut self.wgpu, frame_view, width, height, &format!("Sample {}/{}\nFPS: {}", self.raytracer.sample_count(), self.raytracer.target_samples, 1000.0/self.timer.average_delta_time()));
                }
            }

            for event in event_pump.poll_iter() {
                match self.state.update(&self.sdl2, &mut self.raytracer, &event) {
                    Message::Quit => {
                        // Application should now quit
                        break 'run;
                    }
                    Message::ConsumeEvent => {
                        // Event was consumed, skip to next event
                        continue;
                    }
                    Message::RestartRender => {
                        self.raytracer.pause_rendering = false;
                        self.raytracer.reset_samples();
                        continue;
                    }
                    Message::Nothing => {
                        // No message was returned, nothing to do
                    }
                }

                // System-handled events
                match event {
                    Event::Quit { .. } => {
                        break 'run;
                    }

                    Event::Window { win_event: WindowEvent::Resized(width, height), .. } => {
                        println!("Resized window to {}x{}", width, height);

                        self.raytracer.pause_rendering = false;
                        self.resize(width as u32, height as u32);
                    } 

                    Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                        if self.raytracer.sample_count() != self.raytracer.target_samples {
                            self.raytracer.pause_rendering = !self.raytracer.pause_rendering;
                            println!("{} render", if self.raytracer.pause_rendering {"Paused"} else {"Resuming"});
                        }
                    }

                    Event::KeyDown { keycode: Some(Keycode::F11), .. } => {                       
                        println!("Toggle fullscreen");
                        
                        self.raytracer.pause_rendering = false;
                        
                        let (width, height) = self.sdl2.toggle_full_screen();
                        self.resize(width, height);
                    }

                    _ => {
                        // println!("Unhandled event: {:?}", event);
                    }
                }
            }

            let keys = event_pump.keyboard_state();
            
            self.state.fixed_update(&self.sdl2, &keys, &mut self.raytracer);
            
            let delta_time = self.timer.tick();
            Timer::await_fps(60, delta_time);            
        }
        println!("Quitting...");
    }

    pub fn create_texture_from_path<P: AsRef<std::path::Path>>(&self, path: P) -> crate::texture::Texture {
        // Flip textures for OpenGL coordinate system
        let (texture, commands) = crate::texture::Texture::from_image_path(&self.wgpu.device, path, true).unwrap();
        self.wgpu.queue.submit(&[commands]);

        texture
    }   
}

impl QuadBuilder for System {
    fn create_quad_full_screen(&self, texture: crate::texture::Texture) -> Quad {
        Quad::new_full_screen(
            &self.wgpu.device, 
            &self.quad_bind_group_layout,
            texture,
        )
    }

    fn create_quad(&self, texture: crate::texture::Texture) -> Quad {
        Quad::new(
            &self.wgpu.device, 
            &self.quad_bind_group_layout, 
            texture, 
            None
        )
    }
}