use std::env;

fn main() {
  let args: Vec<String> = env::args().collect();

  if args.len() < 2 {
    eprintln!("Usage: {} <workspace_member>", args[0]);
    return;
  }

  let target = &args[1];

  match target.as_str() {
    "tutorial/ch01-window" => {
      ch01_window::run().unwrap();
    }
    _ => {
      eprintln!("Not found: {}", target);
    }
  }
}
