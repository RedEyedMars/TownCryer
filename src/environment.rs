use bevy::prelude::*;
use rand::Rng;

#[derive(Component, Clone)]
pub enum Ground {
    Grass(u8),
    Dirt(u8),
    Water,
    Shore(usize),
}
#[derive(Component, Debug, Clone)]
pub struct FoodItem {
    pub name: String,
    pub nutrition: u32,
}
#[derive(Component, Debug, Clone)]
pub struct DrinkItem {
    pub name: String,
    pub hydration: u32,
}
#[derive(Component)]
pub struct Passable;

impl Ground {
    pub fn animation_index(&self, rng: &mut impl Rng) -> u32 {
        let mut weighed = rng.random_range(0..4);
        if weighed == 0 {
            weighed += 1;
        }
        weighed -= 1;
        match self {
            Ground::Grass(index) => weighed + *index as u32,
            Ground::Dirt(index) => weighed + *index as u32,
            Ground::Water => 4,
            Ground::Shore(index) => *index as u32,
        }
    }

    pub fn is_passable(&self) -> bool {
        match self {
            Ground::Grass(_) | Ground::Dirt(_) => true,
            Ground::Water | Ground::Shore(_) => false,
        }
    }

    pub fn random(rng: &mut impl Rng) -> Self {
        match rng.random_range(0..10) {
            0 | 1 | 2 => Ground::Grass(0),
            3 | 4 | 5 => Ground::Grass(3),
            6 => Ground::Dirt(0),
            7 => Ground::Dirt(3),
            _ => Ground::Water,
        }
    }

    pub fn tileset_index(&self) -> usize {
        match self {
            Ground::Dirt(_) => 0,
            Ground::Grass(_) => 1,
            Ground::Shore(_) => 2,
            Ground::Water => 3,
        }
    }
}
