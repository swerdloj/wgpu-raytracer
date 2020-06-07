use sdl2::{keyboard::Keycode, event::Event, mouse::MouseButton};

use crate::system::{Message, Runnable, SDL2};
use crate::camera::Camera;
use crate::raytrace::RayTracer;

pub struct ApplicationState {    
    // Application state
    camera: Camera,
    pause_rendering: bool,
    target_reached: bool,
    relative_mouse_mode: bool,
}

impl ApplicationState {
    pub fn new() -> Self {
        let camera = Camera::new(0.015);
        
        Self {
            camera, 
            pause_rendering: false, 
            target_reached: false,
            relative_mouse_mode: true,
        }
    }

    fn toggle_relative_mouse_mode(&mut self, sdl2: &SDL2) {
        self.relative_mouse_mode = !self.relative_mouse_mode;
        sdl2.set_relative_mouse_mode(self.relative_mouse_mode);
    }
}

impl Runnable for ApplicationState {
    fn init(&mut self, sdl2: &SDL2) {
        // Always begin with relative_mouse_mode on
        sdl2.set_relative_mouse_mode(true);
    }

    fn update(&mut self, sdl2: &SDL2, raytracer: &mut RayTracer, event: &Event) -> Message {
        use Message::*;

        match event {
            Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                if self.relative_mouse_mode {
                    // Unfreeze mouse
                    self.toggle_relative_mouse_mode(sdl2);
                } else {
                    return Quit;
                }

                ConsumeEvent
            }

            Event::MouseButtonDown { mouse_btn: MouseButton::Left, .. } => {
                if !self.relative_mouse_mode {
                    self.toggle_relative_mouse_mode(sdl2);
                }

                ConsumeEvent
            }

            _ => {
                Nothing
            }
        }
    }

    fn fixed_update(&mut self, sdl2: &SDL2) {
        todo!()
    }    
}