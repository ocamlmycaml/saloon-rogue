use rltk::{Point, RandomNumberGenerator, Algorithm2D, BaseMap};
use std::cmp::{min, max};
use crate::rect::Rect;


#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall, Floor
}

pub struct GameMap {
    pub tiles: Vec<TileType>,
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
        let tiles = vec![TileType::Wall; (width * height) as usize];
        let rooms = Vec::<Rect>::new();

        GameMap { tiles, rooms, width, height }
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
}