use sdl2::{
    keyboard::{Keycode, KeyboardState, Scancode}, 
    event::{Event, WindowEvent}, mouse::MouseButton,
};

use crate::system::{Message, Runnable, SDL2};
use crate::camera::Camera;
use crate::raytrace::RayTracer;

pub struct ApplicationState {    
    // Application state
    camera: Camera,
    relative_mouse_mode: bool,

    camera_changed_this_frame: bool,
}

impl ApplicationState {
    pub fn new() -> Self {
        let camera = Camera::new(0.02);
        
        Self {
            camera, 
            relative_mouse_mode: true,
            camera_changed_this_frame: false,
        }
    }

    fn toggle_relative_mouse_mode(&mut self, sdl2: &SDL2) {
        self.relative_mouse_mode = !self.relative_mouse_mode;
        sdl2.set_relative_mouse_mode(self.relative_mouse_mode);
    }

    fn set_relative_mouse_mode(&mut self, sdl2: &SDL2, on: bool) {
        self.relative_mouse_mode = on;
        sdl2.set_relative_mouse_mode(on);
    }
}

impl Runnable for ApplicationState {
    fn init(&mut self, sdl2: &SDL2) {
        // Always begin with relative_mouse_mode on
        sdl2.set_relative_mouse_mode(true);
    }

    fn update(&mut self, sdl2: &SDL2, raytracer: &mut RayTracer, event: &Event) -> Message {
        // Dereference the event so values are not behind references
        match *event {
            Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                if self.relative_mouse_mode {
                    // Unfreeze mouse
                    self.toggle_relative_mouse_mode(sdl2);
                } else {
                    return Message::Quit;
                }
                Message::ConsumeEvent
            }

            Event::Window { win_event: WindowEvent::FocusLost, .. } => {
                self.set_relative_mouse_mode(sdl2, false);
                Message::ConsumeEvent
            }

            Event::Window { win_event: WindowEvent::FocusGained, .. } => {
                self.set_relative_mouse_mode(sdl2, true);
                Message::ConsumeEvent
            }

            Event::MouseButtonDown { mouse_btn: MouseButton::Left, .. } => {
                if !self.relative_mouse_mode {
                    self.toggle_relative_mouse_mode(sdl2);
                }
                Message::ConsumeEvent
            }

            Event::MouseMotion { xrel, yrel, .. } => {
                if self.relative_mouse_mode {
                    self.camera.update_angle(xrel as f32, yrel as f32);
                    self.camera_changed_this_frame = true;

                    Message::RestartRender
                } else {
                    Message::Nothing
                }
            }

            Event::MouseWheel { y, .. } => {
                if self.camera.update_fov(-2.0 * y as f32) {
                    println!("Vertical FoV: {}", self.camera.v_fov);
                    self.camera_changed_this_frame = true;
                }
                Message::RestartRender
            }

            Event::KeyDown { keycode: Some(Keycode::R), .. } => {
                println!("Restarting render");

                raytracer.pause_rendering = false;
                raytracer.reset_samples();

                Message::ConsumeEvent
            }

            _ => { Message::Nothing }
        }
    }

    fn fixed_update(&mut self, _sdl2: &SDL2, keys: &KeyboardState, raytracer: &mut RayTracer) {
        let mut translation = cgmath::Vector3::new(0f32, 0.0, 0.0);
        if keys.is_scancode_pressed(Scancode::W) { // Forwards
            translation.z -= 0.05;
        }
        if keys.is_scancode_pressed(Scancode::A) { // Left
            translation.x -= 0.02;
        }
        if keys.is_scancode_pressed(Scancode::S) { // Backwards
            translation.z += 0.05;
        }
        if keys.is_scancode_pressed(Scancode::D) { // Right
            translation.x += 0.02;
        }
        if keys.is_scancode_pressed(Scancode::LShift) { // Up
            translation.y += 0.05;
        }
        if keys.is_scancode_pressed(Scancode::LCtrl) { // Down
            translation.y -= 0.05;
        }

        if translation != cgmath::Vector3::new(0.0, 0.0, 0.0) {
            self.camera.update_position(translation.x, translation.y, translation.z);
            self.camera_changed_this_frame = true;
            raytracer.pause_rendering = false;
        }

        if self.camera_changed_this_frame {
            raytracer.update_camera(&self.camera);
        }

        self.camera_changed_this_frame = false;
    }    
}