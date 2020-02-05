use rltk::{Point, RandomNumberGenerator, Algorithm2D, BaseMap};
use specs::prelude::*;
use std::cmp::{min, max};
use crate::rect::Rect;


#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall, Floor
}

pub struct GameMap {
    pub tiles: Vec<TileType>,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub blocked: Vec<bool>,
    pub tile_content: Vec<Vec<Entity>>,
    pub rooms: Vec<Rect>,
    pub width: i32,
    pub height: i32,
}

impl GameMap {
    fn draw_room(&mut self, room: &Rect) {
        for y in room.y1 + 1 ..= room.y2 {
            for x in room.x1 + 1 ..= room.x2 {
                let index = self.xy_idx(x, y);
                self.tiles[index] = TileType::Floor;
            }
        }
    }

    fn draw_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        let minx = min(x1, x2);
        let maxx = max(x1, x2);
        self.draw_room(&Rect::new(minx, y, maxx - minx, 1));
    }

    fn draw_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        let miny = min(y1, y2);
        let maxy = max(y1, y2);
        self.draw_room(&Rect::new(x, miny, 1, maxy - miny));
    }

    pub fn new(width: i32, height: i32) -> GameMap {
        GameMap {
            tiles: vec![TileType::Wall; (width * height) as usize],
            revealed_tiles: vec![false; (width * height) as usize],
            visible_tiles: vec![false; (width * height) as usize],
            blocked: vec![true; (width * height) as usize],
            tile_content: vec![Vec::<Entity>::new(); (width * height) as usize],
            rooms: Vec::<Rect>::new(),
            width,
            height
        }
    }

    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y * self.width + x) as usize
    }

    pub fn populate_with_random_rooms(&mut self) {
        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        let mut rng = RandomNumberGenerator::new();
        let mut rooms = Vec::<Rect>::new();

        'outer: for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, self.width - w - 1) - 1;
            let y = rng.roll_dice(1, self.height - h - 1) - 1;
            let new_room = Rect::new(x, y, w, h);

            for room in rooms.iter() {
                if new_room.intersect(room) {
                    continue 'outer;
                }
            }

            self.draw_room(&new_room);
            if !rooms.is_empty() {
                let (new_x, new_y) = new_room.center();
                let (prev_x, prev_y) = rooms[rooms.len() - 1].center();
                if rng.roll_dice(1, 2) == 1 {
                    self.draw_horizontal_tunnel(prev_x, new_x, prev_y);
                    self.draw_vertical_tunnel(prev_y, new_y, new_x);
                } else {
                    self.draw_vertical_tunnel(prev_y, new_y, prev_x);
                    self.draw_horizontal_tunnel(prev_x, new_x, new_y);
                }
            }

            rooms.push(new_room);
        }

        self.rooms = rooms;
    }

    pub fn populate_blocked(&mut self) {
        for (i, &tile) in self.tiles.iter().enumerate() {
            self.blocked[i] = tile == TileType::Wall;
        }
    }

    pub fn clear_content_index(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }

    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if x < 1 || x > self.width - 1 || y < 1 || y> self.height - 1 {
            false
        } else {
            let idx = self.xy_idx(x, y);
            !self.blocked[idx]
        }
    }
}

impl Algorithm2D for GameMap {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl BaseMap for GameMap {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx as usize] == TileType::Wall
    }

    fn get_available_exits(&self, idx: usize) -> Vec<(usize, f32)> {
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        let w = self.width;

        (-1..=1).flat_map(|dx| (-1..=1).map(move |dy| (dx, dy)))
            .filter(|&(dx, dy)| dx != 0 || dy != 0)
            .filter(|&(dx, dy)| self.is_exit_valid(x + dx, y + dy))
            .map(|(dx, dy)| (
                (idx as i32 + (w * dy) + dx) as usize,
                if dx == 0 || dy == 0 { 1.0 } else { 1.45 }
            ))
            .collect()
    }
}