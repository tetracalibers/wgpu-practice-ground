use winit::{event::MouseButton, keyboard::ModifiersState};

use crate::action::Action;

pub struct Binding<T: Eq> {
  pub trigger: T,
  pub mods: ModifiersState,
  pub action: Action,
}

impl<T: Eq> Binding<T> {
  pub fn new(trigger: T, mods: ModifiersState, action: Action) -> Self {
    Self {
      trigger,
      mods,
      action,
    }
  }

  pub fn is_triggered_by(&self, trigger: &T, mods: &ModifiersState) -> bool {
    &self.trigger == trigger && &self.mods == mods
  }
}

pub fn modifiers_to_string(mods: ModifiersState) -> String {
  let mut mods_line = String::new();
  // Always add + since it's printed as a part of the bindings.
  for (modifier, desc) in [
    (ModifiersState::SUPER, "Super+"),
    (ModifiersState::ALT, "Alt+"),
    (ModifiersState::CONTROL, "Ctrl+"),
    (ModifiersState::SHIFT, "Shift+"),
  ] {
    if !mods.contains(modifier) {
      continue;
    }

    mods_line.push_str(desc);
  }
  mods_line
}

pub fn mouse_button_to_string(button: MouseButton) -> &'static str {
  match button {
    MouseButton::Left => "LMB",
    MouseButton::Right => "RMB",
    MouseButton::Middle => "MMB",
    MouseButton::Back => "Back",
    MouseButton::Forward => "Forward",
    MouseButton::Other(_) => "",
  }
}
