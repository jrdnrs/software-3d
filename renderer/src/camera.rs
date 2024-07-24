use maths::linear::{Mat4f, Vec3f};

const UP: Vec3f = {
    Vec3f {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    }
};

#[derive(Clone)]
pub struct Camera {
    pub position: Vec3f,
    pub direction: Vec3f,
    view_transform: Mat4f,
}

impl Camera {
    pub fn new() -> Self {
        let position = Vec3f::new(0.0, 0.0, 0.0);
        let direction = Vec3f::new(0.0, 0.0, 1.0);
        let view = Self::look_at(&position, &direction, &UP);

        Self {
            position,
            direction,
            view_transform: view,
        }
    }

    pub fn view_transform(&self) -> &Mat4f {
        &self.view_transform
    }

    pub fn update_view(&mut self) {
        self.view_transform = Self::look_at(&self.position, &self.direction, &UP);
    }

    fn look_at(position: &Vec3f, direction: &Vec3f, up: &Vec3f) -> Mat4f {
        let p = position;
        let d = direction;
        let r = up.cross(*d).normalise();
        let u = d.cross(r).normalise();

        let mut matrix = Mat4f::IDENTITY;

        matrix[0][0] = r.x;
        matrix[1][0] = r.y;
        matrix[2][0] = r.z;
        matrix[3][0] = -r.dot(*p);

        matrix[0][1] = u.x;
        matrix[1][1] = u.y;
        matrix[2][1] = u.z;
        matrix[3][1] = -u.dot(*p);

        matrix[0][2] = d.x;
        matrix[1][2] = d.y;
        matrix[2][2] = d.z;
        matrix[3][2] = -d.dot(*p);

        matrix
    }

    fn look_at_inverted(position: &Vec3f, direction: &Vec3f, up: &Vec3f) -> Mat4f {
        let p = position;
        let d = direction;
        let r = up.cross(*d).normalise();
        let u = d.cross(r).normalise();

        let mut matrix = Mat4f::IDENTITY;

        matrix[0][0] = r.x;
        matrix[0][1] = r.y;
        matrix[0][2] = r.z;

        matrix[1][0] = u.x;
        matrix[1][1] = u.y;
        matrix[1][2] = u.z;

        matrix[2][0] = d.x;
        matrix[2][1] = d.y;
        matrix[2][2] = d.z;

        matrix[3][0] = p.x;
        matrix[3][1] = p.y;
        matrix[3][2] = p.z;

        matrix
    }
}
