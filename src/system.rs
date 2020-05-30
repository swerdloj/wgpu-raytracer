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
    device: Device,
    queue: Queue,
    sc_desc: SwapChainDescriptor,
    swap_chain: SwapChain,

    // TODO:
    // quad_render_pipeline
    // raytrace_compute_pipeline
}

pub struct System {
    sdl2: SDL2,
    wgpu: WGPU,
}

impl System {
    pub async fn new() -> Self {
        let sdl2 = Self::init_sdl2();
        let wgpu = Self::init_wgpu(&sdl2.window).await;
        
        Self {
            sdl2,
            wgpu,
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

        // TODO: Quad program for rendering texture
        // let quad: Quad = todo!();

        let quad_render_pipeline = Quad::create_render_pipeline(&device, sc_desc.format, None);


        // TODO: Compute program that writes to texture

        WGPU {
            render_surface,
            adapter,
            device,
            queue,
            sc_desc,
            swap_chain,
        }
    }

    // TODO: Move this to `Application` struct which handles application state as well
    pub fn run (&mut self) {
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
        }
    }
}