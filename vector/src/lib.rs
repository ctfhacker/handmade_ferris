//! Implementations of vector math operations

use std::ops::{Add, Sub};
use std::fmt::Debug;

/// A 2-dimensional Vector
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vector2<T: Clone> {
    /// First element in this vector
    pub x: T,

    /// Second element in this vector
    pub y: T
}

impl<T: Clone> Vector2<T> {
    /// Create a new [`Vector2`]
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    /// Convert [`Vector<T>`] into [`Vector<U>`]
    pub fn into<U: From<T> + Clone>(&self) -> Vector2<U> {
        Vector2 {
            x: self.x.clone().into(),
            y: self.y.clone().into(),
        }
    }
}

impl<T: Add<Output = T> + Clone> Add for Vector2<T> {
    type Output = Vector2<T>;

    fn add(self, right: Vector2<T>) -> Self::Output {
        Vector2 {
            x: self.x + right.x,
            y: self.y + right.y,
        }
    }
}

impl<T: Add<Output = T> + Copy + Clone> Add<T> for Vector2<T> {
    type Output = Vector2<T>;

    fn add(self, right: T) -> Self::Output {
        Self {
            x: self.x + right,
            y: self.y + right,
        }
    }
}

impl<T: Sub<Output = T> + Clone> Sub for Vector2<T> {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl<T: Sub<Output = T> + Copy + Clone> Sub<T> for Vector2<T> {
    type Output = Vector2<T>;

    fn sub(self, right: T) -> Self::Output {
        Self {
            x: self.x - right,
            y: self.y - right,
        }
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
