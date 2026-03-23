use boltffi::*;

/// A 2D point with double-precision coordinates.
#[data]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Point {
    /// Horizontal position.
    pub x: f64,
    /// Vertical position.
    pub y: f64,
}

#[data(impl)]
impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }

    pub fn origin() -> Self {
        Point { x: 0.0, y: 0.0 }
    }

    pub fn from_polar(r: f64, theta: f64) -> Self {
        Point {
            x: r * theta.cos(),
            y: r * theta.sin(),
        }
    }

    pub fn try_unit(x: f64, y: f64) -> Result<Self, String> {
        let len = (x * x + y * y).sqrt();
        if len == 0.0 {
            Err("cannot normalize zero vector".to_string())
        } else {
            Ok(Point { x: x / len, y: y / len })
        }
    }

    pub fn checked_unit(x: f64, y: f64) -> Option<Self> {
        let len = (x * x + y * y).sqrt();
        if len == 0.0 {
            None
        } else {
            Some(Point { x: x / len, y: y / len })
        }
    }

    pub fn distance(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn scale(&mut self, factor: f64) {
        self.x *= factor;
        self.y *= factor;
    }

    pub fn add(&self, other: Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    pub fn dimensions() -> u32 {
        2
    }
}

#[export]
pub fn echo_point(p: Point) -> Point {
    p
}

#[export]
pub fn try_make_point(x: f64, y: f64) -> Option<Point> {
    if x == 0.0 && y == 0.0 { None } else { Some(Point { x, y }) }
}

#[export]
pub fn make_point(x: f64, y: f64) -> Point {
    Point { x, y }
}

#[export]
pub fn add_points(a: Point, b: Point) -> Point {
    Point {
        x: a.x + b.x,
        y: a.y + b.y,
    }
}

#[data]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[export]
pub fn echo_color(c: Color) -> Color {
    c
}

#[export]
pub fn make_color(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color { r, g, b, a }
}
