use winit::dpi::PhysicalSize;

pub struct Point {
    pub pos: cgmath::Point3<f32>,
    pub color: [f32; 4],
    pub width: f32,
}

pub struct Pen {
    width: f32,
    color: [f32; 4],
}

impl Pen {
    pub fn new() -> Self {
        Self {
            width: 0.002,
            color: [0., 0., 0., 1.],
        }
    }

    pub fn px2point(&self, x: f32, y: f32, force: f32, sz: PhysicalSize<u32>) -> Point {
        let unit = (sz.height as f32) / 2.;
        let ratio = (sz.width as f32) / (sz.height as f32);
        let point = Point {
            pos: [x / unit - ratio, -y / unit + 1.0, 0.].into(),
            color: self.color,
            width: self.width * (1.0 + force * 2.),
        };
        point
    }
}

impl Clone for Pen {
    fn clone(&self) -> Self {
        Self {
            width: self.width,
            color: self.color.clone(),
        }
    }
}
