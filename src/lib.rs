use cgmath::*;

#[derive(Debug)]
pub struct Point {
    pub pos: cgmath::Point3<f32>,
    pub color: [f32; 4],
    pub width: f32,
}

impl Point {
    fn get_number(width: f32) -> u32 {
        let num = (width.sqrt().sqrt() * 12.) as u32;
        1 << {
            if num < 2 {
                2
            } else if num > 12 {
                12
            } else {
                num
            }
        }
    }
}

pub struct Line {
    points: std::vec::Vec<Point>,
}

impl Line {
    pub fn new(point: Point) -> Self {
        Self {
            points: std::vec![point],
        }
    }

    pub fn push_point(&mut self, point: Point) {
        self.points.push(point);
    }

    fn build_points(&self) -> std::vec::Vec<canvas::structs::Point> {
        let mut points = std::vec::Vec::new();
        for i in 0..self.points.len() {
            Self::build_point(&mut points, &self.points[i]);
            if i > 0 {
                let r1 = self.points[(i - 1) as usize].width;
                let r2 = self.points[i as usize].width;
                let delta = r2 - r1;
                let o_p = &self.points[(i - 1) as usize].pos;
                let o1_p = &self.points[i as usize].pos;
                let v = o1_p - o_p;
                let l = v.magnitude();
                let x_v = v.normalize();
                let y_v = x_v.cross(Vector3 {
                    x: 0.,
                    y: 0.,
                    z: -1.,
                });
                let c_a = -delta / l;
                let s_a = (l * l - delta * delta).sqrt() / l;
                let v1 = x_v * c_a + y_v * s_a;
                let v2 = x_v * c_a - y_v * s_a;
                let a_p = o_p + v1 * r1;
                let b_p = o1_p + v1 * r2;
                let c_p = o_p + v2 * r1;
                let d_p = o1_p + v2 * r2;
                points.push(canvas::structs::Point {
                    pos: a_p.into(),
                    color: self.points[(i - 1) as usize].color,
                });
                points.push(canvas::structs::Point {
                    pos: c_p.into(),
                    color: self.points[(i - 1) as usize].color,
                });
                points.push(canvas::structs::Point {
                    pos: b_p.into(),
                    color: self.points[i as usize].color,
                });
                points.push(canvas::structs::Point {
                    pos: b_p.into(),
                    color: self.points[i as usize].color,
                });
                points.push(canvas::structs::Point {
                    pos: c_p.into(),
                    color: self.points[(i - 1) as usize].color,
                });
                points.push(canvas::structs::Point {
                    pos: d_p.into(),
                    color: self.points[i as usize].color,
                });
            }
        }
        points
    }

    fn build_point(points: &mut std::vec::Vec<canvas::structs::Point>, point: &Point) {
        let width = point.width;
        let num = Point::get_number(width);
        let unit = 2. * PI / (num as f32);
        for i in 0..num {
            let alpha = i as f32 * unit;
            points.push(canvas::structs::Point {
                pos: [
                    point.pos[0] + width * alpha.cos(),
                    point.pos[1] + width * alpha.sin(),
                    point.pos[2],
                ],
                color: point.color,
            });
            let alpha = alpha + unit;
            points.push(canvas::structs::Point {
                pos: [
                    point.pos[0] + width * alpha.cos(),
                    point.pos[1] + width * alpha.sin(),
                    point.pos[2],
                ],
                color: point.color,
            });
            points.push(canvas::structs::Point {
                pos: point.pos.into(),
                color: point.color,
            });
        }
    }
}

pub trait Frame {
    fn redraw(&mut self);

    fn push_point(&mut self, pt: Point);

    fn start_line(&mut self, pt: Point);

    fn end_line(&mut self);

    fn cancle_line(&mut self);

    fn set_aspect(&mut self, aspect: f32);
}

mod canvas;
use std::f32::consts::PI;

pub use canvas::Canvas;

pub mod tools;
