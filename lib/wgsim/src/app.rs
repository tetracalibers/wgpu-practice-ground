use std::{error::Error, sync::Arc};

use winit::{
  application::ApplicationHandler,
  dpi::LogicalSize,
  event::{ElementState, KeyEvent, StartCause, WindowEvent},
  event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
  keyboard::{KeyCode, PhysicalKey},
  window::{Window, WindowId},
};

use crate::{
  ctx::DrawingContext,
  render::{Render, RenderTarget},
  surface_cfg::SurfaceConfigBuilder,
};

pub struct App<'a, R>
where
  R: Render<'a>,
{
  window: Option<Arc<Window>>,
  window_title: &'a str,
  window_size: Option<LogicalSize<u32>>,
  initial: R::Initial,
  ctx: Option<DrawingContext<'a>>,
  surface_cfg_builder: Option<&'a SurfaceConfigBuilder<'a>>,
  sample_count: u32,
  renderer: Option<R>,
  render_start_time: Option<std::time::Instant>,
  update_interval: Option<std::time::Duration>,
  need_redraw: bool,
}

impl<'a, R> App<'a, R>
where
  R: Render<'a>,
{
  pub fn new(window_title: &'a str, initial: R::Initial) -> Self {
    Self {
      window: None,
      window_title,
      window_size: None,
      initial,
      sample_count: 1,
      ctx: None,
      surface_cfg_builder: None,
      renderer: None,
      render_start_time: None,
      update_interval: None,
      need_redraw: true,
    }
  }

  pub fn with_window_size(mut self, width: u32, height: u32) -> Self {
    self.window_size = Some(LogicalSize::new(width, height));
    self
  }

  pub fn with_update_interval(mut self, interval: std::time::Duration) -> Self {
    self.update_interval = Some(interval);
    self
  }

  pub fn with_msaa(mut self) -> Self {
    self.sample_count = 4;
    self
  }

  pub fn with_surface_cfg_builder(
    mut self,
    builder: &'a SurfaceConfigBuilder<'a>,
  ) -> Self {
    self.surface_cfg_builder = Some(builder);
    self
  }

  pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::builder().build()?;
    event_loop.run_app(self)?;

    Ok(())
  }

  fn window(&self) -> Option<&Window> {
    match &self.window {
      Some(window) => Some(window.as_ref()),
      None => None,
    }
  }

  async fn init(&mut self, window: Arc<Window>) {
    let surface_cfg_builder = match self.surface_cfg_builder {
      Some(builder) => builder,
      None => &SurfaceConfigBuilder::new(),
    };

    let ctx = DrawingContext::new_for_surface(window, &surface_cfg_builder)
      .await
      .with_sample_count(self.sample_count);
    self.ctx = Some(ctx);

    let renderer = R::new(self.ctx.as_ref().unwrap(), &self.initial).await;
    self.renderer = Some(renderer);
  }
}

impl<'a, R: Render<'a>> ApplicationHandler for App<'a, R> {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    let mut window_attributes =
      Window::default_attributes().with_title(self.window_title);

    if let Some(window_size) = self.window_size {
      window_attributes = window_attributes.with_max_inner_size(window_size);
    }

    let window = event_loop.create_window(window_attributes).unwrap();
    self.window = Some(Arc::new(window));

    pollster::block_on(self.init(self.window.as_ref().unwrap().clone()));

    self.render_start_time = Some(std::time::Instant::now());
    self.need_redraw = true;
  }

  fn window_event(
    &mut self,
    event_loop: &ActiveEventLoop,
    window_id: WindowId,
    event: WindowEvent,
  ) {
    let binding = self.window();
    let window = match &binding {
      Some(window) => window,
      None => return,
    };
    if window.id() != window_id {
      return;
    }

    let renderer = match &mut self.renderer {
      Some(renderer) => renderer,
      None => return,
    };
    if renderer.process_event(&event) {
      return;
    }

    let mut ctx = match &mut self.ctx {
      Some(ctx) => ctx,
      None => return,
    };

    match event {
      WindowEvent::Resized(size) => {
        renderer.resize(&mut ctx, size.into());
      }
      WindowEvent::RedrawRequested => {
        let ctx = match &mut self.ctx {
          Some(ctx) => ctx,
          None => {
            eprintln!("Context is not initialized");
            return;
          }
        };

        let surface = match ctx.surface() {
          Some(surface) => surface,
          None => {
            eprintln!("Surface is not initialized");
            return;
          }
        };

        let now = std::time::Instant::now();
        let dt = now - self.render_start_time.unwrap_or(now);
        renderer.update(ctx, dt);

        let mut command_encoder =
          ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None,
          });

        let result = renderer.draw(
          &mut command_encoder,
          RenderTarget::Surface(&surface),
          self.sample_count,
        );

        match result {
          Ok(frame) => renderer.submit(&ctx.queue, command_encoder, frame),
          Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
            renderer.resize(ctx, *ctx.size())
          }
          Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
          Err(e) => eprintln!("{:?}", e),
        }
      }
      WindowEvent::CloseRequested => {
        event_loop.exit();
      }
      WindowEvent::KeyboardInput {
        event:
          KeyEvent {
            physical_key: PhysicalKey::Code(KeyCode::Escape),
            state: ElementState::Pressed,
            ..
          },
        ..
      } => {
        event_loop.exit();
      }
      _ => {}
    }
  }

  fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
    if let StartCause::ResumeTimeReached { .. } = cause {
      self.need_redraw = true;
    }
  }

  fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
    if !self.need_redraw {
      return;
    }

    let binding = self.window();
    let window = match &binding {
      Some(window) => window,
      None => return,
    };
    window.request_redraw();

    if let Some(update_interval) = self.update_interval {
      self.need_redraw = false;

      event_loop.set_control_flow(ControlFlow::WaitUntil(
        std::time::Instant::now() + update_interval,
      ));
    }
  }
}
