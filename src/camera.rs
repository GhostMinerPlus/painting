use cgmath::{InnerSpace, Point3, Transform, Vector3};

#[rustfmt::skip]
const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

// Public
pub fn untransform_point(m: &cgmath::Matrix4<f32>, pt: &Point3<f32>) -> Point3<f32> {
    let origin = m.transform_point(Point3::new(0.0, 0.0, 0.0));
    let v = pt - m.transform_point(Point3::new(0.0, 0.0, 0.0));
    let vx = m.transform_point(Point3::new(1.0, 0.0, 0.0)) - origin;
    let vy = m.transform_point(Point3::new(0.0, 1.0, 0.0)) - origin;
    let vz = m.transform_point(Point3::new(0.0, 0.0, 1.0)) - origin;
    let x = v.dot(vx) / vx.magnitude2();
    let y = v.dot(vy) / vy.magnitude2();
    let z = v.dot(vz) / vz.magnitude2();
    Point3 { x, y, z }
}

pub fn untransform_vector(m: &cgmath::Matrix4<f32>, v: &Vector3<f32>) -> Vector3<f32> {
    let origin = m.transform_point(Point3::new(0.0, 0.0, 0.0));
    let vx = m.transform_point(Point3::new(1.0, 0.0, 0.0)) - origin;
    let vy = m.transform_point(Point3::new(0.0, 1.0, 0.0)) - origin;
    let vz = m.transform_point(Point3::new(0.0, 0.0, 1.0)) - origin;
    let x = v.dot(vx) / vx.magnitude2();
    let y = v.dot(vy) / vy.magnitude2();
    let z = v.dot(vz) / vz.magnitude2();
    Vector3 { x, y, z }
}

pub struct Camera {
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
    pub vm: cgmath::Matrix4<f32>,
}

impl Camera {
    pub fn new(
        eye: cgmath::Point3<f32>,
        target: cgmath::Point3<f32>,
        up: cgmath::Vector3<f32>,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        Self {
            aspect,
            fovy,
            znear,
            zfar,
            vm: cgmath::Matrix4::look_at_rh(eye, target, up),
        }
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
    }

    pub fn build_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        OPENGL_TO_WGPU_MATRIX * proj
    }
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // We can't use cgmath with bytemuck directly so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    proj_view: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            proj_view: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update(&mut self, camera: &Camera) {
        self.proj_view = (camera.build_projection_matrix() * camera.vm).into();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        let camera = super::Camera::new(
            // position the camera one unit up and 2 units back
            // +z is out of the screen
            (0.0, 0.0, 2.0).into(),
            // have it look at the origin
            (0., 0., 0.).into(),
            // which way is "up"
            cgmath::Vector3::unit_y(),
            1.,
            45.0,
            0.1,
            100.0,
        );
        let pos = camera.vm * cgmath::vec4(0., 1., 3., 1.);
        println!("{:?}", pos);
    }
}
