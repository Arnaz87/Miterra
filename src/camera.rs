
use cgmath::{Rad, Deg, Vector3, Matrix4, SquareMatrix, PerspectiveFov, Transform};

pub struct Camera {
    projection: Matrix4<f32>,

    /// Position of the camera.
    pub pos: Vector3<f32>,

    /// Angle in the Y axis, left/right movement.
    pub yaw: Rad<f32>,

    /// Angle in the X axis, up/down movement.
    pub pitch: Rad<f32>,

    /// Vertical field of view.
    ///
    /// Horizontal FOV is calculated using this and the aspect ratio.
    pub fov: Rad<f32>,

    /// Near clipping distance
    pub near: f32,

    /// Far clipping distance
    pub far: f32,

    /// Screen width in pixels
    pub width: f32,

    /// Screen height in pixels
    pub height: f32,

    /// Mouse sensitivity
    pub sensitivity: f32,

    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub front: bool,
    pub back: bool,
}

impl Camera {
    pub fn new (fov: f32, near: f32, far: f32) -> Self {
        let mut cam = Camera {
            projection: Matrix4::identity(),
            pos: Vector3{x: 0.0, y: 0.0, z: 0.0},
            yaw: Rad(0.0),
            pitch: Rad(0.0),
            fov: Rad::from(Deg(fov)),
            near: near,
            far: far,
            width: 1.0,
            height: 1.0,
            sensitivity: 1.0,

            up: false,
            down: false,
            left: false,
            right: false,
            front: false,
            back: false,
        };
        cam.recalculate_projection();
        cam
    }

    pub fn recalculate_projection (&mut self) {
        self.projection = Matrix4::from(PerspectiveFov{
            fovy: self.fov,
            near: self.near,
            far: self.far,
            aspect: self.width / self.height
        });
    }

    pub fn set_screen_size (&mut self, w: f32, h: f32) {
        self.width = w;
        self.height = h;
        self.recalculate_projection();
    }

    pub fn move_pixels (&mut self, pix_x: f32, pix_y: f32) {
        let fovx = self.fov * (self.width / self.height);
        let x = fovx     * (pix_x / self.width);
        let y = self.fov * (pix_y / self.height);
        self.yaw += x * self.sensitivity;
        if self.yaw > Rad( 80.0) { self.yaw = Rad( 80.0) }
        if self.yaw < Rad(-80.0) { self.yaw = Rad(-80.0) }
        self.pitch += y * self.sensitivity;
    }

    pub fn matrix (&self) -> Matrix4<f32> {
        self.projection
        * Matrix4::from_angle_x(self.pitch)
        * Matrix4::from_angle_y(self.yaw)
        * Matrix4::from_translation(-self.pos)
    }

    pub fn update (&mut self) {
        let mut mov = Vector3::new(0.0, 0.0, 0.0);
        if self.up    { mov.y += 1.0; }
        if self.down  { mov.y -= 1.0; }
        if self.front { mov.z -= 1.0; }
        if self.back  { mov.z += 1.0; }
        if self.left  { mov.x -= 1.0; }
        if self.right { mov.x += 1.0; }
        self.pos += Matrix4::from_angle_y(-self.yaw).transform_vector(mov * 0.1);
    }
}
