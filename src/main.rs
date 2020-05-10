mod rect;
mod game_map;
mod components;
mod visibility_system;
mod monster_ai_system;
mod map_ai_system;
mod melee_combat_system;
mod damage_system;
mod gui;
mod game_log;
mod spawner;
mod inventory_system;

use rltk::{
    Point, Console, GameState, Rltk, RGB,
    VirtualKeyCode, to_cp437, RandomNumberGenerator
};
use specs::prelude::*;
use std::cmp::{max, min};
use game_map::{GameMap, TileType};
use components::*;
use visibility_system::VisibilitySystem;
use monster_ai_system::MonsterAI;
use map_ai_system::MapIndexingSystem;
use melee_combat_system::MeleeCombatSystem;
use damage_system::DamageSystem;
use game_log::GameLog;
use inventory_system::{ItemCollectionSystem, ItemDropSystem, PotionUseSystem};

#[macro_use]
extern crate specs_derive;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    PreRun,
    AwaitingInput,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
    ShowDropItem
}

pub struct State {
    ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem;
        let mut mob = MonsterAI;
        let mut map_indexer = MapIndexingSystem;
        let mut melee = MeleeCombatSystem;
        let mut damage_system = DamageSystem;
        let mut item_collection_system = ItemCollectionSystem;
        let mut item_drop_system = ItemDropSystem;
        let mut potion_use_system = PotionUseSystem;

        vis.run_now(&self.ecs);
        mob.run_now(&self.ecs);
        map_indexer.run_now(&self.ecs);
        melee.run_now(&self.ecs);
        damage_system.run_now(&self.ecs);
        item_collection_system.run_now(&self.ecs);
        item_drop_system.run_now(&self.ecs);
        potion_use_system.run_now(&self.ecs);

        self.ecs.maintain();
    }

    fn state_after_player_input(self: &mut State, ctx: &mut Rltk) -> RunState {
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::Left |
                VirtualKeyCode::Numpad4 |
                VirtualKeyCode::H => try_move_player(&mut self.ecs, -1, 0),

                VirtualKeyCode::Right |
                VirtualKeyCode::Numpad6 |
                VirtualKeyCode::L => try_move_player(&mut self.ecs, 1, 0),

                VirtualKeyCode::Up |
                VirtualKeyCode::Numpad8 |
                VirtualKeyCode::K => try_move_player(&mut self.ecs, 0, -1),

                VirtualKeyCode::Down |
                VirtualKeyCode::Numpad2 |
                VirtualKeyCode::J => try_move_player(&mut self.ecs, 0, 1),

                // Diagonals
                VirtualKeyCode::Numpad9 |
                VirtualKeyCode::Y => try_move_player(&mut self.ecs, 1, -1),

                VirtualKeyCode::Numpad7 |
                VirtualKeyCode::U => try_move_player(&mut self.ecs, -1, -1),

                VirtualKeyCode::Numpad3 |
                VirtualKeyCode::N => try_move_player(&mut self.ecs, 1, 1),

                VirtualKeyCode::Numpad1 |
                VirtualKeyCode::B => try_move_player(&mut self.ecs, -1, 1),

                VirtualKeyCode::G => get_item(&mut self.ecs),

                VirtualKeyCode::I => return RunState::ShowInventory,
                VirtualKeyCode::D => return RunState::ShowDropItem,

                _ => { return RunState::AwaitingInput }
            }
            RunState::PlayerTurn
        } else {
            RunState::AwaitingInput
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        {
            let map = self.ecs.fetch::<GameMap>();

            // draw box
            gui::draw_ui(&self.ecs, ctx);

            // first render map
            draw_map(&map, ctx);
        }

        let mut run_state;
        run_state = *self.ecs.fetch::<RunState>();
        match run_state {
            RunState::PreRun => {
                self.run_systems();
                run_state = RunState::AwaitingInput;
            },
            RunState::AwaitingInput => {
                run_state = self.state_after_player_input(ctx);
            },
            RunState::PlayerTurn => {
                self.run_systems();
                run_state = RunState::MonsterTurn;
            },
            RunState::MonsterTurn => {
                self.run_systems();
                run_state = RunState::AwaitingInput;
            },
            RunState::ShowInventory => {
                match gui::show_inventory(&mut self.ecs, ctx) {
                    gui::ItemMenuResult::Cancel => run_state = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {},
                    gui::ItemMenuResult::Selected(item) => {
                        let mut intents_to_drink = self.ecs.write_storage::<WantsToDrinkPotion>();
                        intents_to_drink.insert(*self.ecs.fetch::<Entity>(), WantsToDrinkPotion { potion: item })
                            .expect("Unable to insert intent to drink potion");
                        run_state = RunState::PlayerTurn;
                    }
                }
            },
            RunState::ShowDropItem => {
                match gui::drop_item_menu(&mut self.ecs, ctx) {
                    gui::ItemMenuResult::Cancel => run_state = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {},
                    gui::ItemMenuResult::Selected(item) => {
                        let mut intents_to_drink = self.ecs.write_storage::<WantsToDropItem>();
                        intents_to_drink.insert(*self.ecs.fetch::<Entity>(), WantsToDropItem { item })
                            .expect("Unable to insert intent to drop potion");
                        run_state = RunState::PlayerTurn;
                    }
                }
            }
        }

        {
            let mut run_writer = self.ecs.write_resource::<RunState>();
            *run_writer = run_state;
        }
        damage_system::delete_the_dead(&mut self.ecs);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        // render anything else that can be rendered
        let map = self.ecs.fetch::<GameMap>();
        for (pos, render) in (&positions, &renderables).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
        }
    }
}


fn try_move_player(ecs: &mut World, delta_x: i32, delta_y: i32) {
    let mut positions = ecs.write_storage::<Position>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();
    let entities = ecs.entities();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();
    let map = ecs.fetch::<GameMap>();

    for (entity, _player, pos, viewshed) in (&entities, &players, &mut positions, &mut viewsheds).join() {
        let new_pos = Position {
            x: min(79, max(0, pos.x + delta_x)),
            y: min(49, max(0, pos.y + delta_y))
        };

        let destination_idx = map.xy_idx(new_pos.x, new_pos.y);

        for potential_target in map.tile_content[destination_idx].iter() {
            let target = combat_stats.get(*potential_target);
            if let Some(_target) = target {
                wants_to_melee.insert(entity, WantsToMelee { target: *potential_target })
                    .expect("Add target failed");
                return;
            }
        }

        if !map.blocked[destination_idx] {
            *pos = new_pos;
            viewshed.dirty = true;

            let mut ppos = ecs.write_resource::<Point>();
            ppos.x = pos.x;
            ppos.y = pos.y;
        }
    }
}


fn get_item(ecs: &mut World) {
    let player_pos = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut gamelog = ecs.fetch_mut::<GameLog>();

    let mut target_item: Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => gamelog.entries.insert(0, "There is nothing here to pick up.".to_string()),
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();
            pickup.insert(*player_entity, WantsToPickupItem { collected_by: *player_entity, item })
                .expect("Unable to insert want to pickup");
        }
    }
}


pub fn draw_map(map: &GameMap, ctx: &mut Rltk) {
    let mut y = 0;
    let mut x = 0;
    for (idx, tile) in map.tiles.iter().enumerate() {
        if map.revealed_tiles[idx] {
            let glyph;
            let mut fg;
            match tile {
                TileType::Floor => {
                    fg = RGB::from_f32(0.0, 0.5, 0.0);
                    glyph = to_cp437('#');
                },
                TileType::Wall => {
                    fg = RGB::from_f32(0.0, 1.0, 0.0);
                    glyph = to_cp437('$');
                }
            }

            if !map.visible_tiles[idx] {
                fg = fg.to_greyscale();
            }
            ctx.set(x, y, fg, RGB::from_f32(0.0, 0.0, 0.0), glyph);
        }

        x += 1;
        if x > map.width - 1 {
            x = 0;
            y += 1;
        }
    }
}


fn main() {
    use rltk::RltkBuilder;

    let mut context = RltkBuilder::simple80x50()
        .with_title("Hello bitches")
        .build();
    context.with_post_scanlines(true);

    let mut gs = State {
        ecs: World::new(),
    };

    let mut map = GameMap::new();
    map.populate_with_random_rooms();
    let (player_x, player_y) = map.rooms[0].center();

    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<SufferDamage>();
    gs.ecs.register::<Item>();
    gs.ecs.register::<Potion>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<WantsToDropItem>();
    gs.ecs.register::<WantsToDrinkPotion>();
    gs.ecs.register::<InBackpack>();

    gs.ecs.insert(Point::new(player_x, player_y));
    gs.ecs.insert(RunState::PreRun);
    gs.ecs.insert(GameLog { entries: vec!["Welcome to the Wild Wild West".to_string()]});
    gs.ecs.insert(RandomNumberGenerator::new());

    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut gs.ecs, room);
    }
    let player_entity = spawner::player(&mut gs.ecs, player_x, player_y);

    gs.ecs.insert(player_entity);
    gs.ecs.insert(map);

    rltk::main_loop(context, gs);
}
