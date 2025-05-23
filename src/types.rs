use std::ops;

use serde::Deserialize;

#[allow(clippy::upper_case_acronyms)]
pub type TODO = serde_json::Value;

#[derive(Deserialize, Debug, Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    x: u32,
    y: u32,
}

impl Position {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

impl ops::Add<Position> for Position {
    type Output = Position;

    fn add(self, rhs: Position) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl ops::AddAssign<Position> for Position {
    fn add_assign(&mut self, rhs: Position) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl ops::Sub<Position> for Position {
    type Output = Position;

    fn sub(self, rhs: Position) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl ops::SubAssign<Position> for Position {
    fn sub_assign(&mut self, rhs: Position) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl ops::Mul<u32> for Position {
    type Output = Position;
    fn mul(self, rhs: u32) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl ops::MulAssign<Position> for Position {
    fn mul_assign(&mut self, rhs: Position) {
        self.x *= rhs.x;
        self.y *= rhs.y;
    }
}
