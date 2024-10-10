use std::{error::Error, marker::PhantomData, mem, num::NonZeroU32, sync::Arc};

use log::info;
use winit::{
  dpi::{LogicalSize, PhysicalPosition, PhysicalSize},
  keyboard::ModifiersState,
  window::{Fullscreen, ResizeDirection, Window},
};

use crate::render::Render;

/// The amount of points to around the window for drag resize direction calculations.
const BORDER_SIZE: f64 = 20.;

pub struct WindowState<'a, T, R>
where
  R: Render<T>,
{
  /// The actual winit Window.
  window: Arc<Window>,
  /// Window modifiers state.
  pub modifiers: ModifiersState,
  /// Occlusion state of the window.
  occluded: bool,
  /// The amount of zoom into window.
  pub zoom: f64,
  /// The amount of rotation of the window.
  pub rotated: f32,
  /// The amount of pan of the window.
  pub panned: PhysicalPosition<f32>,
  /// Cursor position over the window.
  cursor_position: Option<PhysicalPosition<f64>>,

  renderer: &'a mut R,
  inputs: PhantomData<T>,
}

impl<'a, T, R> WindowState<'a, T, R>
where
  R: Render<T>,
{
  pub fn new(
    renderer: &'a mut R,
    window: Window,
  ) -> Result<Self, Box<dyn Error>> {
    let window = Arc::from(window);

    let size = Arc::clone(&window).inner_size();
    let mut state = Self {
      window,
      modifiers: Default::default(),
      occluded: Default::default(),
      rotated: Default::default(),
      panned: Default::default(),
      zoom: Default::default(),
      cursor_position: Default::default(),
      renderer,
      inputs: PhantomData,
    };

    state.resize(size);
    Ok(state)
  }

  pub fn window(&self) -> &Window {
    self.window.as_ref()
  }

  pub fn cursor_moved(&mut self, position: PhysicalPosition<f64>) {
    self.cursor_position = Some(position);
  }

  pub fn cursor_left(&mut self) {
    self.cursor_position = None;
  }

  pub fn minimize(&mut self) {
    self.window.set_minimized(true);
  }

  /// Toggle maximized.
  pub fn toggle_maximize(&self) {
    let maximized = self.window.is_maximized();
    self.window.set_maximized(!maximized);
  }

  /// Toggle window decorations.
  pub fn toggle_decorations(&self) {
    let decorated = self.window.is_decorated();
    self.window.set_decorations(!decorated);
  }

  /// Toggle window resizable state.
  pub fn toggle_resizable(&self) {
    let resizable = self.window.is_resizable();
    self.window.set_resizable(!resizable);
  }

  /// Toggle resize increments on a window.
  pub fn toggle_resize_increments(&mut self) {
    let new_increments: Option<PhysicalSize<u32>> =
      match self.window().resize_increments() {
        Some(_) => None,
        None => Some(
          LogicalSize::new(25.0, 25.0)
            .to_physical(self.window().scale_factor()),
        ),
      };
    info!("Had increments: {}", new_increments.is_none());
    self.window.set_resize_increments(new_increments);
  }

  /// Toggle fullscreen.
  pub fn toggle_fullscreen(&self) {
    let fullscreen = if self.window.fullscreen().is_some() {
      None
    } else {
      Some(Fullscreen::Borderless(None))
    };

    self.window.set_fullscreen(fullscreen);
  }

  /// Swap the window dimensions with `request_surface_size`.
  pub fn swap_dimensions(&mut self) {
    let old_surface_size = self.window().inner_size();
    let mut surface_size = old_surface_size;

    mem::swap(&mut surface_size.width, &mut surface_size.height);
    info!("Requesting resize from {old_surface_size:?} to {surface_size:?}");

    if let Some(new_surface_size) = self.window.request_inner_size(surface_size)
    {
      if old_surface_size == new_surface_size {
        info!("Inner size change got ignored");
      } else {
        self.resize(new_surface_size);
      }
    } else {
      info!("Requesting surface size is asynchronous");
    }
  }

  /// Resize the surface to the new size.
  pub fn resize(&mut self, size: PhysicalSize<u32>) {
    info!("Surface resized to {size:?}");
    {
      let (width, height) =
        match (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) {
          (Some(width), Some(height)) => (width, height),
          _ => return,
        };

      self.renderer.resize(width, height);
    }
    self.window.request_redraw();
  }

  /// Drag the window.
  pub fn drag_window(&self) {
    if let Err(err) = self.window.drag_window() {
      info!("Error starting window drag: {err}");
    } else {
      info!("Dragging window Window={:?}", self.window.id());
    }
  }

  /// Show window menu.
  pub fn show_menu(&self) {
    if let Some(position) = self.cursor_position {
      self.window.show_window_menu(position);
    }
  }

  /// Drag-resize the window.
  pub fn drag_resize_window(&self) {
    let position = match self.cursor_position {
      Some(position) => position,
      None => {
        info!("Drag-resize requires cursor to be inside the window");
        return;
      }
    };

    let win_size = self.window().inner_size();
    let border_size = BORDER_SIZE * self.window.scale_factor();

    let x_direction = if position.x < border_size {
      ResizeDirection::West
    } else if position.x > (win_size.width as f64 - border_size) {
      ResizeDirection::East
    } else {
      // Use arbitrary direction instead of None for simplicity.
      ResizeDirection::SouthEast
    };

    let y_direction = if position.y < border_size {
      ResizeDirection::North
    } else if position.y > (win_size.height as f64 - border_size) {
      ResizeDirection::South
    } else {
      // Use arbitrary direction instead of None for simplicity.
      ResizeDirection::SouthEast
    };

    let direction = match (x_direction, y_direction) {
      (ResizeDirection::West, ResizeDirection::North) => {
        ResizeDirection::NorthWest
      }
      (ResizeDirection::West, ResizeDirection::South) => {
        ResizeDirection::SouthWest
      }
      (ResizeDirection::West, _) => ResizeDirection::West,
      (ResizeDirection::East, ResizeDirection::North) => {
        ResizeDirection::NorthEast
      }
      (ResizeDirection::East, ResizeDirection::South) => {
        ResizeDirection::SouthEast
      }
      (ResizeDirection::East, _) => ResizeDirection::East,
      (_, ResizeDirection::South) => ResizeDirection::South,
      (_, ResizeDirection::North) => ResizeDirection::North,
      _ => return,
    };

    if let Err(err) = self.window.drag_resize_window(direction) {
      info!("Error starting window drag-resize: {err}");
    } else {
      info!("Drag-resizing window Window={:?}", self.window.id());
    }
  }

  /// Change window occlusion state.
  pub fn set_occluded(&mut self, occluded: bool) {
    self.occluded = occluded;
    if !occluded {
      self.window.request_redraw();
    }
  }

  pub fn draw(&mut self) -> Result<(), Box<dyn Error>> {
    if self.occluded {
      info!("Skipping drawing occluded window={:?}", self.window.id());
      return Ok(());
    }

    self.renderer.draw()?;

    Ok(())
  }
}
