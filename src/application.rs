use sdl2::{keyboard::Keycode, event::Event, mouse::MouseButton};

use crate::system::{Message, Runnable, System};
use crate::camera::Camera;

pub struct Application {
    // The application backend (gpu, windowing, etc.)
    system: System,
    
    // Application state
    camera: Camera,
    pause_rendering: bool,
    target_reached: bool,
    freeze_mouse: bool,
}

impl Application {
    pub fn new(width: u32, height: u32) -> Self {
        let system = futures::executor::block_on(System::new(width, height));
        let camera = Camera::new(0.01);

        let freeze_mouse = true;
        system.set_relative_mouse_mode(freeze_mouse);
        
        Self {
            system, 
            camera, 
            pause_rendering: false, 
            target_reached: false,
            freeze_mouse,
        }
    }

    pub fn start(&mut self) {
        // TODO: Integrate this with the system (needs refactor)
        self.system.run();
    }

    fn toggle_relative_mouse_mode(&mut self) {
        self.freeze_mouse = !self.freeze_mouse;
        self.system.set_relative_mouse_mode(self.freeze_mouse);
    }
}

impl Runnable for Application {
    fn update(&mut self, event: &Event) -> Message {
        use Message::*;

        match event {
            Event::KeyDown {keycode: Some(Keycode::Escape), ..} => {
                if self.freeze_mouse {
                    // Unfreeze mouse
                    self.toggle_relative_mouse_mode();
                } else {
                    return Quit;
                }

                ConsumeEvent
            }

            Event::MouseButtonDown {mouse_btn: MouseButton::Left, ..} => {
                if !self.freeze_mouse {
                    self.toggle_relative_mouse_mode();
                }

                Nothing
            }

            _ => {
                Nothing
            }
        }
    }

    fn fixed_update(&mut self) {
        todo!()
    }
    
}