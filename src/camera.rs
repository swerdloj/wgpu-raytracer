// Source: https://www.mauriciopoppe.com/notes/computer-graphics/viewing/camera/first-person/

/// First person camera that relies on relative mouse mode (set via sdl2)
pub struct Camera {
    sensitivity: f32,
    /// x-axis rotation (degrees)
    yaw: f32,
    /// y-axis rotation (degrees)
    pitch: f32,

    /// Vertical field of view in degrees
    pub v_fov: f32,

    /// Current position
    pub position: cgmath::Vector3<f32>,
    /// Facing direction
    pub target: cgmath::Vector3<f32>,
}

impl Camera {
    pub fn new(sensitivity: f32) -> Self {
        Camera {
            sensitivity,
            yaw: 0.0,
            pitch: 0.0,
            v_fov: 100.0,

            position: (0.0, 0.0, 5.0).into(),
            target: (0.0, 0.0, -1.0).into(),
        }
    }

    /// Returns true if fov was adjusted within bounds
    pub fn update_fov(&mut self, df: f32) -> bool {
        if self.v_fov + df > 160. || self.v_fov + df < 10. {
            false
        } else {   
            self.v_fov += df;
            true
        }
    }

    pub fn update_position(&mut self, dx: f32, dy: f32, dz: f32) {
        // TODO: There is probably a more concise way of doing these calculations

        // TODO: Want a 2D rotation so only x/z are moved (only using pitch), but don't want that amount to change based on pitch
        let forward = cgmath::Vector3::new(dz * -self.target.x, dz * self.target.y, dz * -self.target.z);
        self.position += forward;
        
        let strafe = self.target.cross(cgmath::Vector3::unit_y());
        self.position += dx * strafe;
        
        self.position.y += dy;
    }

    pub fn update_angle(&mut self, dx: f32, dy: f32) {
        self.yaw -= dx * self.sensitivity;
        self.pitch += dy * self.sensitivity;

        // Don't look up or down to the point of looking upside down
        if self.pitch > 89.0 {
            self.pitch = 89.0;
        } else if self.pitch < -89.0 {
            self.pitch = -89.0;
        }

        self.update_target();
    }

    fn update_target(&mut self) {
        let yaw_radians = self.yaw.to_radians();
        let pitch_radians = self.pitch.to_radians();

        self.target.x = -yaw_radians.sin() * pitch_radians.cos();
        self.target.y =  pitch_radians.sin();
        self.target.z = -yaw_radians.cos() * pitch_radians.cos();

        // println!("Target: {:?}", self.target);
    }
}