use rltk::{Algorithm2D, BaseMap, Point, RandomNumberGenerator, Rltk, RGB};
use std::cmp::{max, min};

use crate::{Rect, World};

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

#[derive(Default)]
pub struct Map {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
}
impl Map {
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y * self.width + x) as usize
    }

    fn apply_room_to_map(&mut self, room: &Rect) {
        for y in (room.y1 + 1)..=room.y2 {
            for x in (room.x1 + 1)..room.x2 {
                let idx = self.xy_idx(x, y);
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    fn apply_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        let last = (self.width * self.height) as usize;
        for x in min(x1, x2)..=max(x1, x2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < last {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    fn apply_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        let last = (self.width * self.height) as usize;
        for y in min(y1, y2)..=max(y1, y2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < last {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    pub fn is_visible(&self, x: i32, y: i32) -> bool {
        self.visible_tiles[self.xy_idx(x, y)]
    }

    pub fn new_map_rooms_and_corridors() -> Map {
        let mut map = Map {
            tiles: vec![TileType::Wall; 80 * 50],
            rooms: Vec::new(),
            width: 80,
            height: 50,
            revealed_tiles: vec![false; 80 * 50],
            visible_tiles: vec![false; 80 * 50],
        };

        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        let mut rng = RandomNumberGenerator::new();

        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, 80 - w - 1) - 1;
            let y = rng.roll_dice(1, 50 - h - 1) - 1;
            let new_room = Rect::new(x, y, w, h);
            let mut ok = true;
            for other_room in map.rooms.iter() {
                if new_room.intersect(other_room) {
                    ok = false
                }
            }
            if ok {
                map.apply_room_to_map(&new_room);

                if !map.rooms.is_empty() {
                    let (new_x, new_y) = new_room.centre();
                    let (prev_x, prev_y) = map.rooms[map.rooms.len() - 1].centre();
                    if rng.range(0, 2) == 1 {
                        map.apply_horizontal_tunnel(prev_x, new_x, prev_y);
                        map.apply_vertical_tunnel(prev_y, new_y, new_x);
                    } else {
                        map.apply_vertical_tunnel(prev_y, new_y, prev_x);
                        map.apply_horizontal_tunnel(prev_x, new_x, new_y);
                    }
                }

                map.rooms.push(new_room);
            }
        }

        map
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx] == TileType::Wall
    }
}

pub fn draw_map(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();

    let cyan = RGB::from_f32(0.0, 0.5, 0.5);
    let green = RGB::from_f32(0.0, 1.0, 0.0);
    let black = RGB::from_f32(0., 0., 0.);
    let floor = rltk::to_cp437('.');
    let wall = rltk::to_cp437('#');

    let (mut x, mut y) = (0, 0);

    for (idx, tile) in map.tiles.iter().enumerate() {
        // Rendering a tile depending on tile type

        if map.revealed_tiles[idx] {
            let (gylph, mut fg) = match tile {
                TileType::Floor => (floor, cyan),
                TileType::Wall => (wall, green),
            };
            if !map.visible_tiles[idx] {
                fg = fg.to_greyscale();
            }
            ctx.set(x, y, fg, black, gylph);
        }

        // Move the coordinate
        x += 1;
        if x > 79 {
            x = 0;
            y += 1;
        }
    }
}
