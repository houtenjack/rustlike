use crate::global;
use crate::map;
use tcod::colors::Color;
use tcod::BackgroundFlag;
use tcod::Console;

/// This is a generic object: the player, a monster, an item, the stairs...
/// It's always represented by a character on screen.
#[derive(Debug)]
pub struct Object {
    pub x: i32,
    pub y: i32,
    character: char,
    color: Color,
    pub name: String,
    pub blocks: bool,
    pub alive: bool,
}

impl Object {
    pub fn new(x: i32, y: i32, character: char, name: &str, color: Color, blocks: bool) -> Self {
        Object {
            x: x,
            y: y,
            character: character,
            name: name.into(),
            color: color,
            blocks: blocks,
            alive: false,
        }
    }

    pub fn pos(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    pub fn set_pos(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    /// set the color and then draw the character that represents this object at its position
    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.character, BackgroundFlag::None);
    }
}

/// move by the given amount, if the destination is not blocked
pub fn move_by(id: usize, dx: i32, dy: i32, map: &map::Map, objects: &mut [Object]) {
    let (x, y) = objects[id].pos();
    if !map::is_blocked(x + dx, y + dy, map, objects) {
        objects[id].set_pos(x + dx, y + dy);
    }
}

pub fn player_move_or_attack(dx: i32, dy: i32, map: &map::Map, objects: &mut [Object]) {
    // the coordinates the player is moving to/attacking
    let x = objects[global::PLAYER].x + dx;
    let y = objects[global::PLAYER].y + dy;

    // try to find an attackable object there
    let target_id = objects.iter().position(|object| object.pos() == (x, y));

    // attack if target found, move otherwise
    match target_id {
        Some(target_id) => {
            println!(
                "The {} laughs at your puny efforts to attack him!",
                objects[target_id].name
            );
        }
        None => {
            move_by(global::PLAYER, dx, dy, &map, objects);
        }
    }
}
