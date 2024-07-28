use std::time;

use winit::{
  application::ApplicationHandler,
  event::{ElementState, KeyEvent, StartCause, WindowEvent},
  event_loop::ActiveEventLoop,
  keyboard::{KeyCode, PhysicalKey},
  window::Window,
};

use crate::state::State;

const UPDATE_INTERVAL: time::Duration = time::Duration::from_millis(500);

#[derive(Default)]
pub struct Application<'w> {
  state: Option<State<'w>>,
  need_redraw: bool,
}

impl<'w> ApplicationHandler for Application<'w> {
  fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
    let window_attributes =
      Window::default_attributes().with_title("Life Game");

    let window = event_loop
      .create_window(window_attributes)
      .expect("Failed to create window");

    let state = pollster::block_on(async { State::new(window).await });

    self.state = Some(state);
    self.need_redraw = true;
  }

  fn window_event(
    &mut self,
    event_loop: &winit::event_loop::ActiveEventLoop,
    _window_id: winit::window::WindowId,
    event: winit::event::WindowEvent,
  ) {
    let state = match &mut self.state {
      Some(state) => state,
      None => return,
    };

    match event {
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
      WindowEvent::Resized(physical_size) => {
        state.resize(physical_size);
      }
      WindowEvent::RedrawRequested => {
        state.update();
        match state.render() {
          Ok(_) => {}
          // Surfaceが失われたり古くなったりした場合は、再構成する
          Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
            state.resize(state.size());
          }
          // メモリ不足の場合、アプリケーションを終了する
          Err(wgpu::SurfaceError::OutOfMemory) => {
            println!("Out of memory");
            event_loop.exit();
          }
          // フレームが表示されるまでに時間がかかりすぎる場合、警告を出して次のフレームに進む
          Err(wgpu::SurfaceError::Timeout) => {
            println!("Surface timeout");
          }
        }
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

    let state = match &mut self.state {
      Some(state) => state,
      None => return,
    };

    state.window().request_redraw();
    self.need_redraw = false;

    event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
      time::Instant::now() + UPDATE_INTERVAL,
    ));
  }
}
