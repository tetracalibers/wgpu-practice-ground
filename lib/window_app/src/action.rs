use std::fmt::{self, Debug};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
  CloseWindow,
  CreateNewWindow,
  ToggleResizeIncrements,
  ToggleDecorations,
  ToggleResizable,
  ToggleFullscreen,
  ToggleMaximize,
  Minimize,
  PrintHelp,
  DragWindow,
  DragResizeWindow,
  ShowWindowMenu,
  RequestResize,
  DumpMonitors,
}

impl Action {
  pub fn help(&self) -> &'static str {
    match self {
      Action::CloseWindow => "Close window",
      Action::CreateNewWindow => "Create new window",
      Action::ToggleDecorations => "Toggle decorations",
      Action::ToggleResizable => "Toggle window resizable state",
      Action::ToggleFullscreen => "Toggle fullscreen",
      Action::ToggleMaximize => "Maximize",
      Action::Minimize => "Minimize",
      Action::ToggleResizeIncrements => {
        "Use resize increments when resizing window"
      }
      Action::PrintHelp => "Print help",
      Action::DragWindow => "Start window drag",
      Action::DragResizeWindow => "Start window drag-resize",
      Action::ShowWindowMenu => "Show window menu",
      Action::RequestResize => "Request a resize",
      Action::DumpMonitors => "Dump monitor information",
    }
  }
}

impl fmt::Display for Action {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    Debug::fmt(&self, f)
  }
}
