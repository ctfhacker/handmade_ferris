//! Implementations of vector math operations

use std::fmt::Debug;
use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub};

pub trait Primitive:
    Copy + Clone + Add<Output = Self> + Mul<Output = Self> + Sub<Output = Self>
{
}

impl Primitive for f32 {}
impl Primitive for f64 {}
impl Primitive for u16 {}
impl Primitive for u32 {}
impl Primitive for usize {}

/// A 2-dimensional Vector
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct Vector2<T: Primitive> {
    /// First element in this vector
    pub x: T,

    /// Second element in this vector
    pub y: T,
}

impl<T: Primitive> Vector2<T> {
    /// Create a new [`Vector2`]
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    /// Convert [`Vector<T>`] into [`Vector<U>`]
    pub fn into<U: From<T> + Primitive>(&self) -> Vector2<U> {
        Vector2 {
            x: self.x.into(),
            y: self.y.into(),
        }
    }

    /// Get the dot product of two Vector2
    pub fn dot(&self, other: Self) -> T {
        self.x * other.x + self.y * other.y
    }

    /// Get the length^2 of this vector by performing a dot product with itself
    pub fn len_squared(&self) -> T {
        self.dot(*self)
    }
}

impl Vector2<f32> {
    pub fn len(&self) -> f32 {
        self.len_squared().sqrt()
    }
}

impl Vector2<f64> {
    pub fn len(&self) -> f64 {
        self.len_squared().sqrt()
    }
}

impl<T: Primitive> Add for Vector2<T> {
    type Output = Vector2<T>;

    fn add(self, right: Vector2<T>) -> Self::Output {
        Vector2 {
            x: self.x + right.x,
            y: self.y + right.y,
        }
    }
}

impl<T: Primitive> Add<T> for Vector2<T> {
    type Output = Vector2<T>;

    fn add(self, right: T) -> Self::Output {
        Self {
            x: self.x + right,
            y: self.y + right,
        }
    }
}

impl<T: Primitive> AddAssign<Vector2<T>> for Vector2<T> {
    fn add_assign(&mut self, right: Vector2<T>) {
        self.x = self.x + right.x;
        self.y = self.y + right.y;
    }
}

impl<T: Primitive> Sub for Vector2<T> {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl<T: Primitive> Sub<T> for Vector2<T> {
    type Output = Vector2<T>;

    fn sub(self, right: T) -> Self::Output {
        Self {
            x: self.x - right,
            y: self.y - right,
        }
    }
}

impl<T: Primitive + Neg<Output = T>> Neg for Vector2<T> {
    type Output = Vector2<T>;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl<T: Primitive> Mul<T> for Vector2<T> {
    type Output = Vector2<T>;

    fn mul(self, right: T) -> Self::Output {
        Self {
            x: self.x * right,
            y: self.y * right,
        }
    }
}

impl<T: Primitive> MulAssign<T> for Vector2<T> {
    fn mul_assign(&mut self, right: T) {
        self.x = self.x * right;
        self.y = self.y * right;
    }
}

impl From<Vector2<usize>> for Vector2<u16> {
    fn from(other: Vector2<usize>) -> Vector2<u16> {
        Vector2 {
            x: other.x.try_into().unwrap(),
            y: other.y.try_into().unwrap(),
        }
    }
}

impl From<Vector2<u16>> for Vector2<f32> {
    fn from(other: Vector2<u16>) -> Vector2<f32> {
        Vector2 {
            x: other.x.try_into().unwrap(),
            y: other.y.try_into().unwrap(),
        }
    }
}
