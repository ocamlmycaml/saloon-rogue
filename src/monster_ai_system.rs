use specs::prelude::*;
use crate::components::{Viewshed, WantsToMelee, Monster, Position};
use crate::game_map::GameMap;
use super::RunState;
use rltk::Point;

pub struct MonsterAI;

impl<'a> System<'a> for MonsterAI {

    #[allow(clippy::type_complexity)]
    type SystemData = ( WriteExpect<'a, GameMap>,
                        ReadExpect<'a, Point>, // player position
                        ReadExpect<'a, RunState>,
                        ReadExpect<'a, Entity>, // player
                        Entities<'a>,
                        ReadStorage<'a, Monster>,
                        WriteStorage<'a, Viewshed>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, WantsToMelee> );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map,
            player_pos,
            run_state,
            player_entity,
            entities,
            monsters,
            mut viewsheds,
            mut positions,
            mut wants_to_melee
        ) = data;

        if *run_state == RunState::MonsterTurn {
            for (entity, _monster, mut viewshed, mut pos) in (&entities, &monsters, &mut viewsheds, &mut positions).join() {
                let distance_to_player = rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), *player_pos);
                if distance_to_player < 1.5 {
                    wants_to_melee.insert(entity, WantsToMelee { target: *player_entity }).expect("Unable to attack player");
                } else if viewshed.visible_tiles.contains(&*player_pos) {
                    let path = rltk::a_star_search(
                        map.xy_idx(pos.x, pos.y),
                        map.xy_idx(player_pos.x, player_pos.y),
                        &*map
                    );

                    if path.success && path.steps.len() > 1 {
                        let mut idx = map.xy_idx(pos.x, pos.y);
                        map.blocked[idx] = false;

                        pos.x = (path.steps[1] as i32) % map.width;
                        pos.y = (path.steps[1] as i32) / map.width;
                        idx = map.xy_idx(pos.x, pos.y);
                        map.blocked[idx] = true;

                        viewshed.dirty = true;
                    }
                }
            }
        }
    }

}