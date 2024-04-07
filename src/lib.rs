use std::iter::once;

use futures::executor::block_on;
use log::{debug, error};
use wgpu::{include_wgsl, SurfaceConfiguration};
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};

struct State<'w> {
    surface: wgpu::Surface<'w>,
    surface_config: wgpu::SurfaceConfiguration,
    surface_view_descriptor: wgpu::TextureViewDescriptor<'w>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    size: PhysicalSize<u32>,
    pipeline: wgpu::RenderPipeline,
}

impl<'w> State<'w> {
    async fn new(window: &'w Window) -> Self {
        let size = window.inner_size();

        // an instance is the first object that wgpu needs to create
        // it is mainly used to create the surface and adapter
        let instance = wgpu::Instance::default();

        // a surface is a platform-specific object that is used to present rendered images
        // to the screen
        let surface = instance
            .create_surface(window)
            .expect("failed to create surface");
        debug!("surface: {:?}", surface);

        // an adapter is the actual handle to the gpu
        let backends = wgpu::util::backend_bits_from_env().unwrap_or_else(wgpu::Backends::all);
        debug!("backends: {:?}", backends);
        for adapter in instance.enumerate_adapters(backends) {
            let info = adapter.get_info();
            debug!("adapter: {:?}", info);
        }

        let adapter = wgpu::util::initialize_adapter_from_env_or_default(&instance, Some(&surface))
            .await
            .expect("failed to create adapter");

        // create the device and the queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    // we may request extra features that should be enabled
                    // devices limit the features they may have, so workarrounds need to be
                    // provided for unsupported hardware
                    required_features: wgpu::Features::empty(),
                    // they describe the limits of each type of resource we can create
                    required_limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .expect("failed to create device and queue");

        // configure the surface
        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities.formats[0];
        let surface_config = SurfaceConfiguration {
            // how will the texture be used
            // render_attachment specifies that it will be written to the screen
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            // how will it be stored on the gpu
            format: surface_format,
            width: size.width,
            height: size.height,
            // fifo is equivalent to vsync (guaranteed to be supported)
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![surface_format],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        // select the default texture view descriptor
        let surface_view_descriptor = wgpu::TextureViewDescriptor::default();

        // load the sample shader
        let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));

        // create the render pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("main render pipeline layout"),
            ..Default::default()
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("main render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vert",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "frag",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            multisample: wgpu::MultisampleState::default(),
            depth_stencil: None,
            multiview: None,
        });

        Self {
            surface,
            surface_config,
            surface_view_descriptor,
            device,
            queue,
            size,
            pipeline,
        }
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.size = new_size;
        self.surface_config.width = new_size.width;
        self.surface_config.height = new_size.height;
        self.surface.configure(&self.device, &self.surface_config);
    }

    fn input(&mut self, _event: &KeyEvent) -> bool {
        false
    }

    fn update(&mut self) {
        //
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // request a frame to render to
        let frame = self.surface.get_current_texture()?;

        // get a texture view into the frame
        let view = frame.texture.create_view(&self.surface_view_descriptor);

        // a command encoder sends the commands to the gpu
        // we store them in a command buffer before sending
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render command encoder"),
            });

        // create a render pass using the encoder
        // this has all the methods for drawing
        {
            let color_attachment = wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.3,
                        g: 0.5,
                        b: 0.9,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            };

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main render pass"),
                color_attachments: &[Some(color_attachment)],
                ..Default::default()
            });

            // example triangle
            render_pass.set_pipeline(&self.pipeline);
            render_pass.draw(0..3, 0..1);
        }

        self.queue.submit(once(encoder.finish()));
        frame.present();

        Ok(())
    }
}

pub fn run() {
    // initialize the appropiate logger
    env_logger::init();

    // create the main app components
    let event_loop = EventLoop::new().expect("Failed to create an event loop");
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(800, 800))
        .with_title("wgpu experiments")
        .build(&event_loop)
        .expect("failed to create a window");
    debug!("the main window was created");

    // create the application state
    let mut state = block_on(State::new(&window));

    // run the application loop
    event_loop
        .run(|event, control_flow| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::Resized(size) => state.resize(*size),
                WindowEvent::ScaleFactorChanged { .. } => {
                    state.resize(window.inner_size());
                }
                WindowEvent::CloseRequested => control_flow.exit(),
                WindowEvent::KeyboardInput { event, .. } => {
                    if state.input(event) {
                        window.request_redraw();
                        return;
                    }
                    if event.state.is_pressed() {
                        if let PhysicalKey::Code(KeyCode::Escape) = event.physical_key {
                            control_flow.exit();
                        }
                    }
                }
                WindowEvent::RedrawRequested => {
                    state.update();
                    match state.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                        Err(wgpu::SurfaceError::OutOfMemory) => control_flow.exit(),
                        Err(e) => error!("unhandled error {:?}", e),
                    }
                }
                _ => {}
            },
            _ => {}
        })
        .expect("failed to start the main application loop");
}
