use rand::Rng;
use wgpu::RequestAdapterOptions;
use winit::{
    event::{ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

pub struct Window {
    window: winit::window::Window,
    background_color: wgpu::Color,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    _event_loop: Option<EventLoop<()>>,
}

fn random_rgb_val() -> f64 {
    rand::thread_rng().gen_range(0.0..1.0)
}

impl Window {
    pub async fn new() -> Self {
        Window::configure(EventLoop::new()).await
    }

    async fn configure(event_loop: EventLoop<()>) -> Self {
        let window = {
            let w = winit::window::Window::new(&event_loop).unwrap();
            w.set_title(":P");
            w
        };
        let instance = wgpu::Instance::new(wgpu::Backend::Metal.into());
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("failed to bind to adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let size = window.inner_size();

        let config = wgpu::SurfaceConfiguration {
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            format: surface.get_supported_formats(&adapter)[0],
        };
        surface.configure(&device, &config);
        Window {
            window,
            surface,
            device,
            queue,
            config,
            size,
            background_color: wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },
            _event_loop: Some(event_loop),
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
    }

    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {
        // todo!()
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

        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(self.background_color),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub async fn spawn(mut self) -> ! {
        self._event_loop
            .take()
            .expect("event loop not found :(")
            .run(move |event, _, control_flow| match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == self.window.id() => match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(new_size) => {
                        self.resize(*new_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        self.resize(**new_inner_size);
                    }
                    WindowEvent::MouseInput { button, state, .. } => {
                        if matches!(button, MouseButton::Left)
                            && matches!(state, ElementState::Pressed)
                        {
                            self.background_color = wgpu::Color {
                                r: random_rgb_val(),
                                g: random_rgb_val(),
                                b: random_rgb_val(),
                                a: 1.0,
                            };
                        }
                    }
                    _ => {}
                },
                Event::RedrawRequested(window_id) if window_id == self.window.id() => {
                    self.update();
                    match self.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => self.resize(self.size),
                        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
                Event::MainEventsCleared => {
                    self.window.request_redraw();
                }
                _ => {}
            })
    }
}
