// Source: https://www.mauriciopoppe.com/notes/computer-graphics/viewing/camera/first-person/

/// First person camera that relies on relative mouse mode (set via sdl2)
pub struct Camera {
    sensitivity: f32,
    yaw: f32,
    pitch: f32,

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

            position: (0.0, 0.0, 5.0).into(),
            target: (0.0, 0.0, -1.0).into(),
        }
    }

    pub fn update_position(&mut self, dx: f32, dy: f32, dz: f32) {
        self.position += (dx, dy, dz).into();
        println!("Position: {:?}", self.position);
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