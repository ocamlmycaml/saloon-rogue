use specs::prelude::*;
use crate::components::{CombatStats, SufferDamage, Player, Name};
use crate::game_log::GameLog;

pub struct DamageSystem;

impl<'a> System<'a> for DamageSystem {
    type SystemData = ( WriteStorage<'a, CombatStats>,
                        WriteStorage<'a, SufferDamage> );

    fn run(&mut self, data: Self::SystemData) {
        let (mut stats, mut damage) = data;

        for (mut stats, damage) in (&mut stats, &damage).join() {
            stats.hp -= damage.amount;
        }

        damage.clear();
    }
}

pub fn delete_the_dead(ecs: &mut World) {
    let mut dead: Vec<Entity> = Vec::new();
    {
        let mut log = ecs.write_resource::<GameLog>();
        let entities = ecs.entities();
        let combat_stats = ecs.read_storage::<CombatStats>();
        let players = ecs.read_storage::<Player>();
        let names = ecs.read_storage::<Name>();

        for (entity, name, stats) in (&entities, &names, &combat_stats).join() {
            if stats.hp < 1 {
                match players.get(entity) {
                    None => {
                        log.entries.insert(0, format!("{} is dead", &name.name));
                        dead.push(entity);
                    },
                    Some(_) => {
                        log.entries.insert(0, "You are dead!".to_string());
                    }
                }
            }
        }
    }

    for victim in dead {
        ecs.delete_entity(victim).expect("Unable to delete victim!");
    }
}