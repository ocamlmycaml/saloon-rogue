mod rect;
mod game_map;
mod components;
mod visibility_system;

use rltk::{Console, GameState, Rltk, RGB, VirtualKeyCode, Point, to_cp437};
use specs::prelude::*;
use std::cmp::{max, min};
use game_map::{GameMap, TileType};
use components::*;
use visibility_system::VisibilitySystem;

#[macro_use]
extern crate specs_derive;

struct State {
    ecs: World
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem;

        vis.run_now(&self.ecs);

        // any systems go here, but we dont have any yet
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        handle_player_input(self, ctx);
        self.run_systems();

        // first render map
        draw_map(&self.ecs, ctx);

        // render anything else that can be rendered
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}


fn try_move_player(ecs: &mut World, delta_x: i32, delta_y: i32) {
    let mut positions = ecs.write_storage::<Position>();
    let players = ecs.read_storage::<Player>();
    let map = ecs.fetch::<GameMap>();

    for (_player, pos) in (&players, &mut positions).join() {
        let new_pos = Position {
            x: min(79, max(0, pos.x + delta_x)),
            y: min(49, max(0, pos.y + delta_y))
        };
        if map.tiles[map.xy_idx(new_pos.x, new_pos.y)] != TileType::Wall {
            *pos = new_pos
        }
    }
}


pub fn draw_map(ecs: &World, ctx: &mut Rltk) {
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let mut players = ecs.write_storage::<Player>();
    let map = ecs.fetch::<GameMap>();

    for (_player, viewshed) in (&mut players, &mut viewsheds).join() {
        let mut y = 0;
        let mut x = 0;
        for tile in map.tiles.iter() {
            let pt = Point::new(x, y);

            if viewshed.visible_tiles.contains(&pt) {
                match tile {
                    TileType::Floor => {
                        ctx.set(x, y, RGB::from_f32(0.5, 0.5, 0.5), RGB::from_f32(0.0, 0.0, 0.0), to_cp437('.'));
                    },
                    TileType::Wall => {
                        ctx.set(x, y, RGB::from_f32(0.0, 1.0, 0.0), RGB::from_f32(0.0, 0.0, 0.0), to_cp437('#'));
                    }
                }
            }

            x += 1;
            if x > 79 {
                x = 0;
                y += 1;
            }
        }
    }
}


fn handle_player_input(gs: &mut State, ctx: &mut Rltk) {
    if let Some(key) = ctx.key {
        match key {
            VirtualKeyCode::Left => try_move_player(&mut gs.ecs, -1, 0),
            VirtualKeyCode::Numpad4 => try_move_player(&mut gs.ecs, -1, 0),
            VirtualKeyCode::H => try_move_player(&mut gs.ecs, -1, 0),
            VirtualKeyCode::Right => try_move_player(&mut gs.ecs, 1, 0),
            VirtualKeyCode::Numpad6 => try_move_player(&mut gs.ecs, 1, 0),
            VirtualKeyCode::L => try_move_player(&mut gs.ecs, 1, 0),
            VirtualKeyCode::Up => try_move_player(&mut gs.ecs, 0, -1),
            VirtualKeyCode::Numpad8 => try_move_player(&mut gs.ecs, 0, -1),
            VirtualKeyCode::K => try_move_player(&mut gs.ecs, 0, -1),
            VirtualKeyCode::Down => try_move_player(&mut gs.ecs, 0, 1),
            VirtualKeyCode::Numpad2 => try_move_player(&mut gs.ecs, 0, 1),
            VirtualKeyCode::J => try_move_player(&mut gs.ecs, 0, 1),
            _ => {}
        }
    }
}


fn main() {
    use rltk::RltkBuilder;

    let context = RltkBuilder::simple80x50()
        .with_title("Hello bitches")
        .build();

    let mut gs = State {
        ecs: World::new()
    };

    let mut map = GameMap::new(80, 50);
    map.populate_with_random_rooms();
    let (player_x, player_y) = map.rooms[0].center();
    gs.ecs.insert(map);

    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();

    gs.ecs
        .create_entity()
        .with(Player)
        .with(Position { x: player_x, y: player_y })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::YELLOW)
        })
        .with(Viewshed { visible_tiles: Vec::new(), range: 8 })
        .build();

    rltk::main_loop(context, gs);
}
