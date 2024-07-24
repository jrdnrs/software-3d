use maths::linear::Vec3f;
use platform::{Input, KeyCode};
use renderer::Camera;

pub struct FreeMovement {
    pitch: f32,
    yaw: f32,

    velocity: Vec3f,

    max_speed: f32,
    acceleration: f32,
    friction: f32,
    mouse_sensitivity: f32,
}

impl FreeMovement {
    pub fn new(max_speed: f32, acceleration: f32, friction: f32, mouse_sensitivity: f32) -> Self {
        Self {
            pitch: 0.0,
            yaw: 90.0,

            velocity: Vec3f::ZERO,
            max_speed,
            acceleration,
            friction,
            mouse_sensitivity
        }
    }

    pub fn update(&mut self, input: &Input, camera: &mut Camera, controls_active: bool, delta_time: f32) {
        if controls_active {
            self.handle_movement(input, camera, delta_time);
            self.handle_looking(input, camera, delta_time);
        }

        // apply friction
        let frict_mag = -self.friction * delta_time;
        let friction = (self.velocity / self.velocity.magnitude().max(1.0)) * frict_mag;
        self.velocity += friction;

        // limit max speed
        let speed = self.velocity.magnitude();
        if speed > self.max_speed {
            self.velocity /= speed;
            self.velocity *= self.max_speed;
        }

        // damping
        // self.velocity *= 0.99;

        camera.position += self.velocity * delta_time;

        camera.update_view();
    }

    fn handle_movement(&mut self, input: &Input, camera: &Camera, delta_time: f32) {
        let accel_mag = self.acceleration * delta_time;

        let mut acceleration = Vec3f::ZERO;

        if input.keyboard.is_key_held(KeyCode::W) {
            acceleration.z += camera.direction.z;
            acceleration.x += camera.direction.x;
        } else if input.keyboard.is_key_held(KeyCode::S) {
            acceleration.z -= camera.direction.z;
            acceleration.x -= camera.direction.x;
        }

        if input.keyboard.is_key_held(KeyCode::A) {
            acceleration.z += camera.direction.x;
            acceleration.x -= camera.direction.z;
        } else if input.keyboard.is_key_held(KeyCode::D) {
            acceleration.z -= camera.direction.x;
            acceleration.x += camera.direction.z;
        }

        if input.keyboard.is_key_held(KeyCode::Space) {
            acceleration.y += 1.0;
        } else if input.keyboard.is_key_held(KeyCode::ShiftLeft) {
            acceleration.y -= 1.0;
        }

        // normalise the direction, and scale by impulse
        acceleration = acceleration.normalise() * accel_mag;

        // apply acceleration
        self.velocity += acceleration;
    }

    fn handle_looking(&mut self, input: &Input, camera: &mut Camera, delta_time: f32) {
        let sensitivity = self.mouse_sensitivity * delta_time;

        let delta = input.mouse.delta();
        self.yaw -= delta.x * sensitivity;
        self.pitch -= delta.y * sensitivity;

        self.pitch = self.pitch.clamp(-89.0, 89.0);

        let pitch_rad = self.pitch.to_radians();
        let yaw_rad = self.yaw.to_radians();

        let pitch_cos = pitch_rad.cos();

        camera.direction.x = yaw_rad.cos() * pitch_cos;
        camera.direction.y = pitch_rad.sin();
        camera.direction.z = yaw_rad.sin() * pitch_cos;

        // Shouldn't need to normalise
        // camera.direction = camera.direction.normalise();
    }
}
