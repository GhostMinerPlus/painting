mod structs;

use cgmath::*;
use std::f32::consts::PI;

use wgpu::{util::DeviceExt, Buffer, RenderPass};

use crate::{point::Point, Canvas};

fn point_v_to_vertex_v(point_v: &std::vec::Vec<Point>) -> std::vec::Vec<structs::Vertex> {
    let mut vertex_v = std::vec::Vec::new();
    for i in 0..point_v.len() {
        point_to_vertex(&mut vertex_v, &point_v[i]);
        if i > 0 {
            let r1 = point_v[(i - 1) as usize].width;
            let r2 = point_v[i as usize].width;
            let delta = r2 - r1;
            let o_p = &point_v[(i - 1) as usize].pos;
            let o1_p = &point_v[i as usize].pos;
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
            vertex_v.push(structs::Vertex {
                pos: a_p.into(),
                color: point_v[(i - 1) as usize].color,
            });
            vertex_v.push(structs::Vertex {
                pos: c_p.into(),
                color: point_v[(i - 1) as usize].color,
            });
            vertex_v.push(structs::Vertex {
                pos: b_p.into(),
                color: point_v[i as usize].color,
            });
            vertex_v.push(structs::Vertex {
                pos: b_p.into(),
                color: point_v[i as usize].color,
            });
            vertex_v.push(structs::Vertex {
                pos: c_p.into(),
                color: point_v[(i - 1) as usize].color,
            });
            vertex_v.push(structs::Vertex {
                pos: d_p.into(),
                color: point_v[i as usize].color,
            });
        }
    }
    vertex_v
}

fn point_to_vertex(vertex_v: &mut std::vec::Vec<structs::Vertex>, point: &Point) {
    let width = point.width;
    let num = get_number(width);
    let unit = 2. * PI / (num as f32);
    for i in 0..num {
        let alpha = i as f32 * unit;
        vertex_v.push(structs::Vertex {
            pos: [
                point.pos[0] + width * alpha.cos(),
                point.pos[1] + width * alpha.sin(),
                point.pos[2],
            ],
            color: point.color,
        });
        let alpha = alpha + unit;
        vertex_v.push(structs::Vertex {
            pos: [
                point.pos[0] + width * alpha.cos(),
                point.pos[1] + width * alpha.sin(),
                point.pos[2],
            ],
            color: point.color,
        });
        vertex_v.push(structs::Vertex {
            pos: point.pos.into(),
            color: point.color,
        });
    }
}

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

// Public
pub use structs::Vertex;

pub struct LineBuffer {
    vertex_buffer: Buffer,
    count: u32,
}

impl LineBuffer {
    pub fn new(line: &Line, canvas: &Canvas) -> Self {
        let points = point_v_to_vertex_v(&line.points);
        let vertex_buffer = canvas
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(points.as_slice()),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let count = points.len() as u32;
        Self {
            vertex_buffer,
            count,
        }
    }

    pub fn draw_self<'a, 'b>(&'a self, render_pass: &mut RenderPass<'b>)
    where
        'a: 'b,
    {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..self.count, 0..1); // 3.
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
}
