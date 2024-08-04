struct Point<T> {
  x: T,
  y: T,
}

struct Size<T> {
  width: T,
  height: T,
}

pub struct Bounds<T> {
  origin: Point<T>,
  size: Size<T>,
}

pub struct Corners<T> {
  top_left: T,
  top_right: T,
  bottom_right: T,
  bottom_left: T,
}
