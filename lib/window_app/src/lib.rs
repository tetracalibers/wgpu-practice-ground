pub mod action;
pub mod binding;
pub mod render;
mod state;

use std::{collections::HashMap, error::Error, sync::Arc};

use action::Action;
use binding::{modifiers_to_string, mouse_button_to_string, Binding};
use log::{error, info};
use render::Render;
use state::WindowState;
use winit::{
  application::ApplicationHandler,
  dpi::{PhysicalPosition, PhysicalSize},
  event::{DeviceEvent, DeviceId, MouseButton, MouseScrollDelta, WindowEvent},
  event_loop::ActiveEventLoop,
  keyboard::{Key, ModifiersState},
  window::{WindowAttributes, WindowId},
};

pub struct Application<'a, T, R>
where
  R: Render<T>,
{
  windows: HashMap<WindowId, WindowState<'a, T, R>>,
  key_bindings: &'a [Binding<&'static str>],
  mouse_bindings: &'a [Binding<MouseButton>],
  renderer: R,
  inputs: T,
}

impl<'a, T, R> Application<'a, T, R>
where
  R: Render<T>,
{
  pub fn new(renderer: R, inputs: T) -> Self {
    Self {
      windows: Default::default(),
      key_bindings: &[],
      mouse_bindings: &[],
      renderer,
      inputs,
    }
  }

  pub fn with_key_bindings(
    &mut self,
    key_bindings: &'a [Binding<&'static str>],
  ) {
    self.key_bindings = key_bindings;
  }

  pub fn with_mouse_bindings(
    &mut self,
    mouse_bindings: &'a [Binding<MouseButton>],
  ) {
    self.mouse_bindings = mouse_bindings;
  }

  fn create_window(
    &mut self,
    event_loop: &ActiveEventLoop,
  ) -> Result<WindowId, Box<dyn Error>> {
    let window_attributes =
      WindowAttributes::default().with_title("Winit window");

    let window = event_loop.create_window(window_attributes)?;

    R::new(Arc::from(&window), &self.inputs);

    let window_state = WindowState::new(&mut self.renderer, window)?;
    let window_id = window_state.window().id();
    info!("Created new window with id={window_id:?}");

    Ok(window_id)
  }

  fn handle_action_with_window(
    &mut self,
    event_loop: &ActiveEventLoop,
    window_id: WindowId,
    action: Action,
  ) {
    let window = self.windows.get_mut(&window_id).unwrap();
    info!("Executing action: {action:?}");
    match action {
      Action::CloseWindow => {
        let _ = self.windows.remove(&window_id);
      }
      Action::CreateNewWindow => {
        if let Err(err) = self.create_window(event_loop) {
          error!("Error creating new window: {err}");
        }
      }
      Action::ToggleResizeIncrements => window.toggle_resize_increments(),
      Action::ToggleResizable => window.toggle_resizable(),
      Action::ToggleDecorations => window.toggle_decorations(),
      Action::ToggleFullscreen => window.toggle_fullscreen(),
      Action::ToggleMaximize => window.toggle_maximize(),
      Action::Minimize => window.minimize(),
      Action::DragWindow => window.drag_window(),
      Action::DragResizeWindow => window.drag_resize_window(),
      Action::ShowWindowMenu => window.show_menu(),
      Action::PrintHelp => self.print_help(),
      Action::RequestResize => window.swap_dimensions(),
      Action::DumpMonitors => self.dump_monitors(event_loop),
    }
  }

  fn dump_monitors(&self, event_loop: &ActiveEventLoop) {
    info!("Monitors information");
    let primary_monitor = event_loop.primary_monitor();
    for monitor in event_loop.available_monitors() {
      let intro = if primary_monitor.as_ref() == Some(&monitor) {
        "Primary monitor"
      } else {
        "Monitor"
      };

      if let Some(name) = monitor.name() {
        info!("{intro}: {name}");
      } else {
        info!("{intro}: [no name]");
      }

      let PhysicalPosition { x, y } = monitor.position();
      info!("  Position: {x},{y}");

      info!("  Scale factor: {}", monitor.scale_factor());

      info!("  Available modes (width x height x bit-depth):");
      for mode in monitor.video_modes() {
        let PhysicalSize { width, height } = mode.size();
        let bits = mode.bit_depth();
        let m_hz = mode.refresh_rate_millihertz();
        info!("    {width}x{height}{bits}{m_hz}");
      }
    }
  }

  /// Process the key binding.
  fn process_key_binding(
    &self,
    key: &str,
    mods: &ModifiersState,
  ) -> Option<Action> {
    self.key_bindings.iter().find_map(|binding| {
      binding.is_triggered_by(&key, mods).then_some(binding.action)
    })
  }

  /// Process mouse binding.
  fn process_mouse_binding(
    &self,
    button: MouseButton,
    mods: &ModifiersState,
  ) -> Option<Action> {
    self.mouse_bindings.iter().find_map(|binding| {
      binding.is_triggered_by(&button, mods).then_some(binding.action)
    })
  }

  fn print_help(&self) {
    info!("Keyboard bindings:");
    for binding in self.key_bindings {
      info!(
        "{}{:<10} - {} ({})",
        modifiers_to_string(binding.mods),
        binding.trigger,
        binding.action,
        binding.action.help(),
      );
    }
    info!("Mouse bindings:");
    for binding in self.mouse_bindings {
      info!(
        "{}{:<10} - {} ({})",
        modifiers_to_string(binding.mods),
        mouse_button_to_string(binding.trigger),
        binding.action,
        binding.action.help(),
      );
    }
  }

  fn can_create_surfaces(&mut self, event_loop: &ActiveEventLoop) {
    info!("Ready to create surfaces");
    self.dump_monitors(event_loop);

    // Create initial window.
    self.create_window(event_loop).expect("failed to create initial window");

    self.print_help();
  }
}

impl<T, R: Render<T>> ApplicationHandler for Application<'_, T, R> {
  fn window_event(
    &mut self,
    event_loop: &ActiveEventLoop,
    window_id: WindowId,
    event: WindowEvent,
  ) {
    let window = match self.windows.get_mut(&window_id) {
      Some(window) => window,
      None => return,
    };

    match event {
      WindowEvent::Resized(size) => {
        window.resize(size);
      }
      WindowEvent::Focused(focused) => {
        if focused {
          info!("Window={window_id:?} focused");
        } else {
          info!("Window={window_id:?} unfocused");
        }
      }
      WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
        info!("Window={window_id:?} changed scale to {scale_factor}");
      }
      WindowEvent::RedrawRequested => {
        if let Err(err) = window.draw() {
          error!("Error drawing window: {err}");
        }
      }
      WindowEvent::Occluded(occluded) => {
        window.set_occluded(occluded);
      }
      WindowEvent::CloseRequested => {
        info!("Closing Window={window_id:?}");
        self.windows.remove(&window_id);
      }
      WindowEvent::ModifiersChanged(modifiers) => {
        window.modifiers = modifiers.state();
        info!("Modifiers changed to {:?}", window.modifiers);
      }
      WindowEvent::MouseWheel { delta, .. } => match delta {
        MouseScrollDelta::LineDelta(x, y) => {
          info!("Mouse wheel Line Delta: ({x},{y})");
        }
        MouseScrollDelta::PixelDelta(px) => {
          info!("Mouse wheel Pixel Delta: ({},{})", px.x, px.y);
        }
      },
      WindowEvent::KeyboardInput {
        event,
        is_synthetic: false,
        ..
      } => {
        let mods = window.modifiers;

        // Dispatch actions only on press.
        if event.state.is_pressed() {
          let action = if let Key::Character(ch) = event.logical_key.as_ref() {
            Self::process_key_binding(self, &ch.to_uppercase(), &mods)
          } else {
            None
          };

          if let Some(action) = action {
            self.handle_action_with_window(event_loop, window_id, action);
          }
        }
      }
      WindowEvent::MouseInput { button, state, .. } => {
        info!("Pointer button {button:?} {state:?}");
        let mods = window.modifiers;
        if let Some(action) = state
          .is_pressed()
          .then(|| Self::process_mouse_binding(self, button, &mods))
          .flatten()
        {
          self.handle_action_with_window(event_loop, window_id, action);
        }
      }
      WindowEvent::CursorLeft { .. } => {
        info!("Pointer left Window={window_id:?}");
        window.cursor_left();
      }
      WindowEvent::CursorMoved { position, .. } => {
        info!("Moved pointer to {position:?}");
        window.cursor_moved(position);
      }
      WindowEvent::PinchGesture { delta, .. } => {
        window.zoom += delta;
        let zoom = window.zoom;
        if delta > 0.0 {
          info!("Zoomed in {delta:.5} (now: {zoom:.5})");
        } else {
          info!("Zoomed out {delta:.5} (now: {zoom:.5})");
        }
      }
      WindowEvent::RotationGesture { delta, .. } => {
        window.rotated += delta;
        let rotated = window.rotated;
        if delta > 0.0 {
          info!("Rotated counterclockwise {delta:.5} (now: {rotated:.5})");
        } else {
          info!("Rotated clockwise {delta:.5} (now: {rotated:.5})");
        }
      }
      WindowEvent::PanGesture { delta, phase, .. } => {
        window.panned.x += delta.x;
        window.panned.y += delta.y;
        info!("Panned ({delta:?})) (now: {:?}), {phase:?}", window.panned);
      }
      WindowEvent::DoubleTapGesture { .. } => {
        info!("Smart zoom");
      }
      _ => (),
    }
  }

  fn device_event(
    &mut self,
    _event_loop: &ActiveEventLoop,
    device_id: DeviceId,
    event: DeviceEvent,
  ) {
    info!("Device {device_id:?} event: {event:?}");
  }

  fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
    if self.windows.is_empty() {
      info!("No windows left, exiting...");
      event_loop.exit();
    }
  }

  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    self.can_create_surfaces(event_loop);
  }
}
