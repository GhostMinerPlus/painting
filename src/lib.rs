mod line;

use std::io::{self, Error};

use cgmath::Vector3;
use wgpu::{util::DeviceExt, Instance, Surface};
use winit::dpi::PhysicalSize;

// Public
pub mod camera;
pub mod point;

pub trait AsCanvas {
    fn get_size(&self) -> &PhysicalSize<u32>;

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>);

    fn render(&mut self) -> Result<(), wgpu::SurfaceError>;

    /// Point in view
    fn push_point(&mut self, pt: point::Point);

    /// Point in view
    fn start_line(&mut self, pt: point::Point);

    fn end_line(&mut self);

    fn cancle_line(&mut self);

    fn set_aspect(&mut self, aspect: f32);

    fn clear(&mut self);

    fn move_content(&mut self, x: f32, y: f32, z: f32);
}

pub struct Canvas {
    device: wgpu::Device,
    queue: wgpu::Queue,

    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,

    render_pipeline: wgpu::RenderPipeline,

    s_line: Option<line::Line>,
    next_id: u64,
    lines: std::collections::BTreeMap<u64, line::LineBuffer>,

    camera: camera::Camera,
    camera_uniform: camera::CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
}

impl Canvas {
    pub async fn create(
        instance: &Instance,
        surface: Surface,
        size: PhysicalSize<u32>,
    ) -> io::Result<Self> {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or(Error::new(io::ErrorKind::Other, "no adapter"))?;
        log::info!("found adapter: {:?}", adapter.get_info());

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None, // Trace path
            )
            .await
            .map_err(|e| Error::new(io::ErrorKind::Other, e))?;
        log::info!("found device: {:?}", device);

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .filter(|f| f.describe().srgb)
            .next()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);
        log::info!("prepared surface: {:?}", config);

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let camera = camera::Camera::new(
            // position the camera one unit up and 2 units back
            // +z is out of the screen
            (0.0, 0.0, 1.0).into(),
            // have it look at the origin
            (0., 0., 0.).into(),
            // which way is "up"
            cgmath::Vector3::unit_y(),
            config.width as f32 / config.height as f32,
            90.0,
            0.1,
            100.0,
        );

        // in new() after creating `camera`

        let mut camera_uniform = camera::CameraUniform::new();
        camera_uniform.update(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",           // 1.
                buffers: &[line::Vertex::desc()], // 2.
            },
            fragment: Some(wgpu::FragmentState {
                // 3.
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    // 4.
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1,                         // 2.
                mask: !0,                         // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
        });

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            next_id: 1,
            s_line: None,
            lines: std::collections::BTreeMap::new(),
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
        })
    }
}

impl AsCanvas for Canvas {
    fn get_size(&self) -> &PhysicalSize<u32> {
        &self.size
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width <= 0 || new_size.height <= 0 {
            return;
        }

        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);

        self.set_aspect((new_size.width as f32) / (new_size.height as f32));
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    // This is what @location(0) in the fragment shader targets
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 1.,
                                g: 1.,
                                b: 1.,
                                a: 1.0,
                            }),
                            store: true,
                        },
                    }),
                ],
                depth_stencil_attachment: None,
            });
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_pipeline(&self.render_pipeline); // 2.
            for (_, line) in &self.lines {
                line.draw_self(&mut render_pass);
            }
        }
        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    fn push_point(&mut self, mut pt: point::Point) {
        let o_pt = camera::untransform_point(&self.camera.vm, &pt.pos);
        pt.pos = o_pt;
        self.s_line.as_mut().unwrap().push_point(pt);
        self.lines.insert(
            self.next_id,
            line::LineBuffer::new(self.s_line.as_ref().unwrap(), self),
        );
    }

    fn start_line(&mut self, mut pt: point::Point) {
        let o_pt = camera::untransform_point(&self.camera.vm, &pt.pos);
        pt.pos = o_pt;
        self.s_line = Some(line::Line::new(pt));
    }

    fn end_line(&mut self) {
        self.s_line = None;
        self.next_id += 1;
    }

    fn cancle_line(&mut self) {
        self.lines.remove(&self.next_id);
    }

    fn set_aspect(&mut self, aspect: f32) {
        self.camera.set_aspect(aspect);
        self.camera_uniform.update(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    fn clear(&mut self) {
        self.lines.clear();
    }

    fn move_content(&mut self, x: f32, y: f32, z: f32) {
        self.camera.vm = cgmath::Matrix4::from_translation(Vector3::new(x, y, z)) * self.camera.vm;
        self.camera_uniform.update(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }
}
