use specs::prelude::*;
use crate::components::{
    WantsToPickupItem, WantsToDropItem,
    Name, InBackpack, Position,
    Potion, WantsToDrinkPotion, CombatStats
};
use crate::game_log::GameLog;

pub struct ItemCollectionSystem;
pub struct ItemDropSystem;
pub struct PotionUseSystem;

impl<'a> System<'a> for ItemCollectionSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, WantsToPickupItem>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, InBackpack> );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut gamelog,
            names,
            mut wants_pickups,
            mut positions,
            mut backpack
        ) = data;

        for pickup in wants_pickups.join() {
            positions.remove(pickup.item);
            backpack.insert(pickup.item, InBackpack { owner: pickup.collected_by }).expect("Unable to add item to backpack");

            if pickup.collected_by == *player_entity {
                gamelog.entries.insert(0, format!("You pick up the {}.", names.get(pickup.item).unwrap().name));
            }
        }

        wants_pickups.clear();
    }
}


impl<'a> System<'a> for PotionUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        Entities<'a>,
                        ReadStorage<'a, Name>,
                        ReadStorage<'a, Potion>,
                        WriteStorage<'a, WantsToDrinkPotion>,
                        WriteStorage<'a, CombatStats> );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut gamelog,
            entities,
            names,
            potions,
            mut wants_to_drink_potions,
            mut combat_stats
        ) = data;

        for (entity, want_to_drink_potion, stats) in (&entities, &wants_to_drink_potions, &mut combat_stats).join() {
            let potion = potions.get(want_to_drink_potion.potion);
            if let Some(potion) = potion {
                stats.hp = i32::min(stats.max_hp, stats.hp + potion.heal_amount);
                if entity == *player_entity {
                    gamelog.entries.insert(0, format!(
                        "You drink the {}, healing {} hp.",
                        names.get(want_to_drink_potion.potion).unwrap().name, potion.heal_amount
                    ));
                }
                entities.delete(want_to_drink_potion.potion).expect("Delete failed");
            }
        }

        wants_to_drink_potions.clear();
    }
}

impl<'a> System<'a> for ItemDropSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        Entities<'a>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, WantsToDropItem>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, InBackpack> );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut gamelog,
            entities,
            names,
            mut items_to_drop,
            mut positions,
            mut backpack_items
        ) = data;

        for (entity, to_drop) in (&entities, &items_to_drop).join() {
            let mut dropper_pos = Position{ x: 0, y: 0 };
            {
                let dropped_pos = positions.get(entity).unwrap();
                dropper_pos.x = dropped_pos.x;
                dropper_pos.y = dropped_pos.y;
            }
            positions.insert(to_drop.item, Position { x: dropper_pos.x, y: dropper_pos.y }).expect("Unable to insert dropped item into position");
            backpack_items.remove(to_drop.item);

            if entity == *player_entity {
                gamelog.entries.insert(0, format!("You drop the {}.", names.get(to_drop.item).unwrap().name));
            }
        }

        items_to_drop.clear();
    }
}
