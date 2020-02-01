mod rect;

use rltk::{Console, GameState, Rltk, RGB, VirtualKeyCode, RandomNumberGenerator};
use specs::prelude::*;
use std::cmp::{max, min};
use rect::Rect;

#[macro_use]
extern crate specs_derive;

#[derive(Component, Debug)]
struct Position {
    x: i32,
    y: i32
}

#[derive(Component)]
struct Renderable {
    glyph: u8,
    fg: RGB,
    bg: RGB
}

#[derive(PartialEq, Copy, Clone)]
enum TileType {
    Wall, Floor
}

#[derive(Component, Debug)]
struct Player;

struct State {
    ecs: World
}

impl State {
    fn run_systems(&mut self) {
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
        self.ecs.fetch::<GameMap>().draw_into(ctx);

        // render anything else that can be rendered
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}

fn draw_room(room: &Rect, tiles: &mut [TileType]) {
    for y in room.y1 + 1 ..= room.y2 {
        for x in room.x1 + 1 ..= room.x2 {
            tiles[xy_idx(x, y)] = TileType::Floor;
        }
    }
}

fn draw_horizontal_tunnel(x1: i32, x2: i32, y: i32, tiles: &mut [TileType]) {
    let minx = min(x1, x2);
    let maxx = max(x1, x2);
    draw_room(&Rect::new(minx, y, maxx - minx, 1), tiles);
}

fn draw_vertical_tunnel(y1: i32, y2: i32, x: i32, tiles: &mut [TileType]) {
    let miny = min(y1, y2);
    let maxy = max(y1, y2);
    draw_room(&Rect::new(x, miny, 1, maxy - miny), tiles);
}

struct GameMap {
    tiles: Vec<TileType>,
    rooms: Vec<Rect>,
}

impl GameMap {
    pub fn new() -> GameMap {
        let mut tiles = vec![TileType::Wall; 80 * 50];

        let mut rooms = Vec::<Rect>::new();
        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        let mut rng = RandomNumberGenerator::new();

        'outer: for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, 80 - w - 1) - 1;
            let y = rng.roll_dice(1, 50 - h - 1) - 1;
            let new_room = Rect::new(x, y, w, h);

            for room in rooms.iter() {
                if new_room.intersect(room) {
                    continue 'outer;
                }
            }

            draw_room(&new_room, &mut tiles);
            if !rooms.is_empty() {
                let (new_x, new_y) = new_room.center();
                let (prev_x, prev_y) = rooms[rooms.len() - 1].center();
                if rng.roll_dice(1, 2) == 1 {
                    draw_horizontal_tunnel(prev_x, new_x, prev_y, &mut tiles);
                    draw_vertical_tunnel(prev_y, new_y, new_x, &mut tiles);
                } else {
                    draw_vertical_tunnel(prev_y, new_y, prev_x, &mut tiles);
                    draw_horizontal_tunnel(prev_x, new_x, new_y, &mut tiles);
                }
            }

            rooms.push(new_room);
        }

        GameMap { tiles, rooms }
    }

    fn draw_into(&self, ctx: &mut Rltk) {
        let mut y = 0;
        let mut x = 0;
        for tile in self.tiles.iter() {
            match tile {
                TileType::Floor => {
                    ctx.set(x, y, RGB::from_f32(0.5, 0.5, 0.5), RGB::from_f32(0.0, 0.0, 0.0), rltk::to_cp437('.'));
                },
                TileType::Wall => {
                    ctx.set(x, y, RGB::from_f32(0.0, 1.0, 0.0), RGB::from_f32(0.0, 0.0, 0.0), rltk::to_cp437('#'));
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


fn try_move_player(ecs: &mut World, delta_x: i32, delta_y: i32) {
    let mut positions = ecs.write_storage::<Position>();
    let players = ecs.read_storage::<Player>();
    let map = ecs.fetch::<GameMap>();

    for (_player, pos) in (&players, &mut positions).join() {
        let new_pos = Position {
            x: min(79, max(0, pos.x + delta_x)),
            y: min(49, max(0, pos.y + delta_y))
        };
        if map.tiles[xy_idx(new_pos.x, new_pos.y)] != TileType::Wall {
            *pos = new_pos
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

pub fn xy_idx(x: i32, y: i32) -> usize {
    (y as usize * 80) + x as usize
}


fn main() {
    let mut gs = State {
        ecs: World::new()
    };

    let map = GameMap::new();
    let (player_x, player_y) = map.rooms[0].center();
    gs.ecs.insert(map);

    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();

    gs.ecs
        .create_entity()
        .with(Player)
        .with(Position { x: player_x, y: player_y })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::YELLOW)
        })
        .build();

    let context = Rltk::init_simple8x8(80, 50, "Hello bitches", "resources");
    rltk::main_loop(context, gs);
}
