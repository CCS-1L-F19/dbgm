use std::ops::{Add, Sub, Mul, Div};

use imgui::sys::ImVec2;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Vec2 { pub x: f32, pub y: f32 }

impl Vec2 {
    pub fn scale(self, other: impl Into<Vec2>) -> Vec2 {
        let other = other.into();
        Vec2 { x: self.x * other.x, y: self.y * other.y }
    }

    pub fn scale_inv(self, other: impl Into<Vec2>) -> Vec2 {
        let other = other.into();
        Vec2 { x: self.x / other.x, y: self.y / other.y }
    }

    pub fn max(v1: impl Into<Vec2>, v2: impl Into<Vec2>) -> Vec2 {
        let (v1, v2) = (v1.into(), v2.into());
        Vec2 { x: f32::max(v1.x, v2.x), y: f32::max(v1.y, v2.y) }
    }

    pub fn min(v1: impl Into<Vec2>, v2: impl Into<Vec2>) -> Vec2 {
        let (v1, v2) = (v1.into(), v2.into());
        Vec2 { x: f32::min(v1.x, v2.x), y: f32::min(v1.y, v2.y) }
    }
}

impl<T> Add<T> for Vec2 where T: Into<Vec2> {
    type Output = Vec2;
    fn add(self, other: T) -> Vec2 {
        let other = other.into();
        Vec2 { x: self.x + other.x, y: self.y + other.y }
    }
}

impl<T> Sub<T> for Vec2 where T: Into<Vec2> {
    type Output = Vec2;
    fn sub(self, other: T) -> Vec2 {
        let other = other.into();
        Vec2 { x: self.x - other.x, y: self.y - other.y }
    }
}

impl Mul<f32> for Vec2 {
    type Output = Vec2;
    fn mul(self, scale: f32) -> Vec2 {
        Vec2 { x: self.x * scale, y: self.y * scale }
    }
}

impl Div<f32> for Vec2 {
    type Output = Vec2;
    fn div(self, scale: f32) -> Vec2 {
        Vec2 { x: self.x / scale, y: self.y / scale }
    }
}

impl Mul<Vec2> for f32 {
    type Output = Vec2;
    fn mul(self, vec: Vec2) -> Vec2 {
        vec * self
    }
}

impl Div<Vec2> for f32 {
    type Output = Vec2;
    fn div(self, vec: Vec2) -> Vec2 {
        vec * self
    }
}

impl From<[f32; 2]> for Vec2 {
    fn from(vec: [f32; 2]) -> Vec2 {
        Vec2 { x: vec[0], y: vec[1] }
    }
}

impl From<Vec2> for [f32; 2] {
    fn from(vec: Vec2) -> [f32; 2] {
        [vec.x, vec.y]
    }
}

impl From<ImVec2> for Vec2 {
    fn from(vec: ImVec2) -> Vec2 {
        Vec2 { x: vec.x, y: vec.y }
    }
}

impl From<Vec2> for ImVec2 {
    fn from(vec: Vec2) -> ImVec2 {
        ImVec2 { x: vec.x, y: vec.y }
    }
}

macro_rules! vec2 {
    { $x:expr, $y:expr } => { Vec2::from([$x, $y]) }
}