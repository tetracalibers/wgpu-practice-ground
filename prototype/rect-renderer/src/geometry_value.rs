pub struct Point<T> {
  pub x: T,
  pub y: T,
}

pub struct Size<T> {
  pub width: T,
  pub height: T,
}

pub struct Bounds<T> {
  pub origin: Point<T>,
  pub size: Size<T>,
}

pub struct Corners<T> {
  pub top_left: T,
  pub top_right: T,
  pub bottom_right: T,
  pub bottom_left: T,
}
