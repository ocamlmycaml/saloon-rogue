use specs::prelude::*;
use rltk::{Point, RGB};

#[derive(Component, Debug)]
pub struct Position {
    pub x: i32,
    pub y: i32
}

#[derive(Component)]
pub struct Renderable {
    pub glyph: u8,
    pub fg: RGB,
    pub bg: RGB
}

#[derive(Component)]
pub struct Viewshed {
    pub visible_tiles: Vec<Point>,
    pub range: i32,
    pub dirty: bool
}

#[derive(Component, Debug)]
pub struct Player;

#[derive(Component, Debug)]
pub struct Monster;

#[derive(Component, Debug)]
pub struct Name {
    pub name: String
}

#[derive(Component, Debug)]
pub struct BlocksTile;

#[derive(Component, Debug)]
pub struct CombatStats {
    pub max_hp: i32,
    pub defense: i32,
    pub power: i32,
    pub hp: i32
}

#[derive(Component, Debug, Clone)]
pub struct WantsToMelee {
    pub target: Entity
}

#[derive(Component, Debug)]
pub struct SufferDamage {
    pub amount: i32
}

#[derive(Component, Debug)]
pub struct Item;

#[derive(Component, Debug)]
pub struct Potion {
    pub heal_amount: i32
}

#[derive(Component, Debug, Clone)]
pub struct InBackpack {
    pub owner: Entity
}

#[derive(Component, Debug, Clone)]
pub struct WantsToPickupItem {
    pub collected_by: Entity,
    pub item: Entity
}

#[derive(Component, Debug, Clone)]
pub struct WantsToDropItem {
    pub item: Entity
}

#[derive(Component, Debug, Clone)]
pub struct WantsToDrinkPotion {
    pub potion: Entity
}
