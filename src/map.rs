use crate::global;
use crate::objects::Object;
use rand::Rng;
use std::cmp;
use tcod::colors::Color;
use tcod::map::Map as FovMap;

pub type Map = Vec<Vec<Tile>>;

pub struct Game {
    pub map: Map,
}

#[derive(Clone, Copy, Debug)]
struct Rect {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Rect {
            x1: x,
            y1: y,
            x2: x + w,
            y2: y + h,
        }
    }
    pub fn center(&self) -> (i32, i32) {
        let center_x = (self.x1 + self.x2) / 2;
        let center_y = (self.y1 + self.y2) / 2;
        (center_x, center_y)
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        (self.x1 < x) && (self.x2 > x) && (self.y1 < y) && (self.y2 > y)
    }

    pub fn intersects_with(&self, other: &Rect) -> bool {
        // returns true if this rectangle intersects with another one
        (self.x1 <= other.x2)
            && (self.x2 >= other.x1)
            && (self.y1 <= other.y2)
            && (self.y2 >= other.y1)
    }
}
fn create_room(room: Rect, map: &mut Map) {
    // go through the tiles in the rectangle and make them passable
    for x in (room.x1 + 1)..room.x2 {
        for y in (room.y1 + 1)..room.y2 {
            map[x as usize][y as usize] = Tile::empty();
        }
    }
}
fn create_h_tunnel(x1: i32, x2: i32, y: i32, map: &mut Map) {
    // horizontal tunnel. `min()` and `max()` are used in case `x1 > x2`
    for x in cmp::min(x1, x2)..(cmp::max(x1, x2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}
fn create_v_tunnel(y1: i32, y2: i32, x: i32, map: &mut Map) {
    // vertical tunnel
    for y in cmp::min(y1, y2)..(cmp::max(y1, y2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

fn create_random_room(width: i32, height: i32) -> Rect {
    let w = rand::thread_rng().gen_range(global::ROOM_MIN_SIZE, global::ROOM_MAX_SIZE + 1);
    let h = rand::thread_rng().gen_range(global::ROOM_MIN_SIZE, global::ROOM_MAX_SIZE + 1);
    // random position without going out of the boundaries of the map
    let x = rand::thread_rng().gen_range(0, width - w);
    let y = rand::thread_rng().gen_range(0, height - h);

    Rect::new(x, y, w, h)
}
pub fn generate(
    width: i32,
    height: i32,
    start_x: i32,
    start_y: i32,
    objs: &mut Vec<Object>,
) -> Map {
    // fill map with wall tiles
    let mut map = vec![vec![Tile::wall(); height as usize]; width as usize];

    let mut rooms = vec![];

    let mut first_room = Rect::new(0, 0, 0, 0);
    while !first_room.contains(start_x, start_y) {
        first_room = create_random_room(width, height);
    }
    create_room(first_room, &mut map);
    rooms.push(first_room);

    for _ in 0..global::MAX_ROOMS {
        // random width and height
        let new_room = create_random_room(width, height);

        // run through the other rooms and see if they intersect with this one
        let failed = rooms
            .iter()
            .any(|other_room| new_room.intersects_with(other_room));

        if !failed {
            // "paint" it to the map's tiles
            create_room(new_room, &mut map);
            if !rooms.is_empty() {
                let (new_x, new_y) = new_room.center();
                // center coordinates of the previous room
                let (prev_x, prev_y) = rooms[rooms.len() - 1].center();

                // toss a coin (random bool value -- either true or false)
                if rand::random() {
                    // first move horizontally, then vertically
                    create_h_tunnel(prev_x, new_x, prev_y, &mut map);
                    create_v_tunnel(prev_y, new_y, new_x, &mut map);
                } else {
                    // first move vertically, then horizontally
                    create_v_tunnel(prev_y, new_y, prev_x, &mut map);
                    create_h_tunnel(prev_x, new_x, new_y, &mut map);
                }
            }
            rooms.push(new_room);
            place_objects(new_room, &map, objs);
        }
    }

    map
}

pub fn init_fov_map(fov: &mut FovMap, game: &Game) {
    for y in 0..global::MAP_HEIGHT {
        for x in 0..global::MAP_WIDTH {
            fov.set(
                x,
                y,
                !game.map[x as usize][y as usize].block_sight,
                !game.map[x as usize][y as usize].blocked,
            );
        }
    }
}

/// A tile of the map and its properties
#[derive(Clone, Copy, Debug)]
pub struct Tile {
    pub blocked: bool,
    pub block_sight: bool,
    pub explored: bool,
    pub color: Color,
}

impl Tile {
    pub fn empty() -> Self {
        Tile {
            blocked: false,
            block_sight: false,
            explored: false,
            color: global::COLOR_DARK_GROUND,
        }
    }

    pub fn wall() -> Self {
        Tile {
            blocked: true,
            block_sight: true,
            explored: false,
            color: global::COLOR_DARK_WALL,
        }
    }
}
fn place_objects(room: Rect, map: &Map, objects: &mut Vec<Object>) {
    // choose random number of monsters
    let num_monsters = rand::thread_rng().gen_range(0, global::MAX_ROOM_MONSTERS + 1);

    for _ in 0..num_monsters {
        // choose random spot for this monster
        let x = rand::thread_rng().gen_range(room.x1 + 1, room.x2);
        let y = rand::thread_rng().gen_range(room.y1 + 1, room.y2);
        if !is_blocked(x, y, map, objects) {
            let mut monster = if rand::random::<f32>() < 0.8 {
                // 80% chance of getting an orc
                // create an orc
                Object::new(x, y, 'o', "orc", global::ORC_COLOR, true)
            } else {
                Object::new(x, y, 'T', "troll", global::TROLL_COLOR, true)
            };

            monster.alive=true;
            objects.push(monster);
        }
    }
}

pub fn is_blocked(x: i32, y: i32, map: &Map, objects: &[Object]) -> bool {
    // first test the map tile
    if map[x as usize][y as usize].blocked {
        return true;
    }
    // now check for any blocking objects
    objects
        .iter()
        .any(|object| object.blocks && object.pos() == (x, y))
}
