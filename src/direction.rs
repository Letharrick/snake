use bracket_terminal::prelude::{
    Point,
    VirtualKeyCode
};

use std::convert::TryFrom;

#[derive(Copy, Clone, PartialEq)]
pub enum Direction {
    North,
    East,
    South,
    West
}

impl TryFrom<VirtualKeyCode> for Direction {
    type Error = ();

    fn try_from(key: VirtualKeyCode) -> Result<Self, ()> {
        match key {
            VirtualKeyCode::W | VirtualKeyCode::Up => Ok(Self::North),
            VirtualKeyCode::A | VirtualKeyCode::Left => Ok(Self::West),
            VirtualKeyCode::S | VirtualKeyCode::Down => Ok(Self::South),
            VirtualKeyCode::D | VirtualKeyCode::Right => Ok(Self::East),
            _ => Err(())
        }
    }
}

impl Into<Point> for Direction {
    fn into(self) -> Point {
        Point::from(match self {
            Self::North => (0, -1),
            Self::East => (1, 0),
            Self::South => (0, 1),
            Self::West => (-1, 0),
        })
    }
}