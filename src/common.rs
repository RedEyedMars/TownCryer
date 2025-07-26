use bevy::prelude::*;
use rand::Rng;

#[derive(Component, Clone, Debug)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    pub fn random(rng: &mut impl Rng) -> Self {
        match rng.random_range(0..4) {
            0 => Direction::North,
            1 => Direction::South,
            2 => Direction::East,
            _ => Direction::West,
        }
    }

    pub fn x(&self) -> i32 {
        match self {
            Direction::East => 1,
            Direction::West => -1,
            _ => 0,
        }
    }

    pub fn y(&self) -> i32 {
        match self {
            Direction::North => 1,
            Direction::South => -1,
            _ => 0,
        }
    }

    pub fn from_vector(vector: Vec3) -> Self {
        if vector.x > 0.0 {
            Direction::East
        } else if vector.x < 0.0 {
            Direction::West
        } else if vector.y > 0.0 {
            Direction::North
        } else {
            Direction::South
        }
    }
}
