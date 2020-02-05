use specs::prelude::*;
use crate::game_map::GameMap;
use crate::components::{Position, BlocksTile};

pub struct MapIndexingSystem;

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = ( WriteExpect<'a, GameMap>,
                        ReadStorage<'a, Position>,
                        ReadStorage<'a, BlocksTile>,
                        Entities<'a> );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, positions, blockers, entities) = data;

        map.populate_blocked();
        map.clear_content_index();
        for (entity, position) in (&entities, &positions).join() {
            let idx = map.xy_idx(position.x, position.y);

            // if they block, update the blocking list
            if let Some(_p) = blockers.get(entity) {
                map.blocked[idx] = true;
            }

            // push entity to the appropriate index. It's a copy type
            map.tile_content[idx].push(entity);
        }
    }
}