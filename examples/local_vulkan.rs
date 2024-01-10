use painting::*;
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("INFO")).init();

    let sz = PhysicalSize::new(1024, 1024);
    let event_loop = EventLoop::new();
    let window = {
        let window = WindowBuilder::new().build(&event_loop).unwrap();
        window.set_inner_size(sz);
        window
    };

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        dx12_shader_compiler: Default::default(),
    });
    let surface = unsafe { instance.create_surface(&window) }.unwrap();

    let mut canvas = pollster::block_on(canvas::Canvas::create(
        &instance,
        surface,
        window.inner_size(),
    ))
    .unwrap();
    canvas.start_line(point::Point {
        pos: [0., 0., 0.].into(),
        width: 0.1,
        color: [1., 0., 0., 1.],
    });
    canvas.push_point(point::Point {
        pos: [0., 1., 0.].into(),
        width: 0.2,
        color: [0., 1., 0., 1.],
    });
    canvas.push_point(point::Point {
        pos: [1., 0., 0.].into(),
        width: 0.2,
        color: [0., 0., 1., 1.],
    });
    canvas.end_line();

    event_loop.run(move |event, _target, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
            WindowEvent::Resized(physical_size) => canvas.resize(*physical_size),
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                canvas.resize(**new_inner_size)
            }
            WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                *control_flow = ControlFlow::Exit
            }
            _ => {}
        },
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            canvas.redraw();
        }
        _ => {}
    });
}
