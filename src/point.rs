use winit::dpi::PhysicalSize;

#[derive(Clone, Debug)]
pub struct Point {
    pub pos: cgmath::Point3<f32>,
    pub color: [f32; 4],
    pub width: f32,
}

#[derive(Clone, Debug)]
pub struct Pen {
    width: f32,
    color: [f32; 4],
}

impl Pen {
    pub fn new(width: f32, color: [f32; 4]) -> Self {
        Self { width, color }
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

    pub fn set_width(&mut self, width: f32) {
        self.width = width;
    }

    pub fn set_color(&mut self, color: [f32; 4]) {
        self.color = color;
    }
}

impl Default for Pen {
    fn default() -> Self {
        Self {
            width: 0.002,
            color: [0., 0., 0., 1.],
        }
    }
}
