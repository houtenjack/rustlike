use tcod::colors;
use tcod::console::*;
use tcod::map::Map as FovMap;
mod global;
mod map;
mod objects;
mod utils;
use PlayerAction::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlayerAction {
    TookTurn,
    DidntTakeTurn,
    Exit,
}
struct Tcod {
    root: Root,
    con: Offscreen,
    fov: FovMap,
}

fn ai_take_turn(monster_id: usize, tcod: &Tcod, map: &map::Map, objects: &mut [objects::Object]) {
    // a basic monster takes its turn. If you can see it, it can see you
    let (monster_x, monster_y) = objects[monster_id].pos();
    if tcod.fov.is_in_fov(monster_x, monster_y) {
        if objects[monster_id].distance_to(&objects[global::PLAYER]) >= 2.0 {
            // move towards player if far away
            let (player_x, player_y) = objects[global::PLAYER].pos();
            objects::move_towards(monster_id, player_x, player_y, &map, objects);
        } else if objects[global::PLAYER].fighter.map_or(false, |f| f.hp > 0) {
            // close enough, attack! (if the player is still alive.)
            let (monster, player) = utils::mut_two(monster_id, global::PLAYER, objects);
            monster.attack(player);
        }
    }
}

fn handle_keys(tcod: &mut Tcod, map: &map::Map, objects: &mut [objects::Object]) -> PlayerAction {
    use tcod::input::Key;
    use tcod::input::KeyCode::*;

    let key = tcod.root.wait_for_keypress(true);
    let player_alive = &objects[global::PLAYER].alive;
    let ret = match (key, key.text(), player_alive) {
        (
            Key {
                code: Enter,
                alt: true,
                ..
            },
            _,
            _,
        ) => {
            // Alt+Enter: toggle fullscreen
            let fullscreen = tcod.root.is_fullscreen();
            tcod.root.set_fullscreen(!fullscreen);
            DidntTakeTurn
        }
        (Key { code: Escape, .. }, _, _) => return Exit, // exit game

        // movement keys
        (Key { code: Up, .. }, _, true) => {
            objects::player_move_or_attack(0, -1, &map, objects);
            TookTurn
        }
        (Key { code: Down, .. }, _, true) => {
            objects::player_move_or_attack(0, 1, &map, objects);
            TookTurn
        }
        (Key { code: Left, .. }, _, true) => {
            objects::player_move_or_attack(-1, 0, &map, objects);
            TookTurn
        }
        (Key { code: Right, .. }, _, true) => {
            objects::player_move_or_attack(1, 0, &map, objects);
            TookTurn
        }

        _ => DidntTakeTurn,
    };

    ret
}

fn render_all(tcod: &mut Tcod, game: &mut map::Game, objects: &[objects::Object]) {
    let mut to_draw: Vec<_> = objects
        .iter()
        .filter(|o| tcod.fov.is_in_fov(o.x, o.y))
        .collect();
    // sort so that non-blocking objects come first
    to_draw.sort_by(|o1, o2| o1.blocks.cmp(&o2.blocks));
    // draw the objects in the list
    for object in &to_draw {
        object.draw(&mut tcod.con);
    }
    // go through all tiles, and set their background color
    for y in 0..global::MAP_HEIGHT {
        for x in 0..global::MAP_WIDTH {
            let visible = tcod.fov.is_in_fov(x, y);
            let tile = &mut game.map[x as usize][y as usize];
            let explored = &mut tile.explored;

            let mut color = tile.color;
            if visible {
                *explored = true;
            } else {
                color = colors::lerp(tile.color, colors::BLACK, 0.4);
            }
            if *explored {
                tcod.con
                    .set_char_background(x, y, color, BackgroundFlag::Set);
            }
        }
    }
    blit(
        &tcod.con,
        (0, 0),
        (global::SCREEN_WIDTH, global::SCREEN_HEIGHT),
        &mut tcod.root,
        (0, 0),
        1.0,
        1.0,
    );
    // show the player's stats
    tcod.root.set_default_foreground(colors::WHITE);
    if let Some(fighter) = objects[global::PLAYER].fighter {
        tcod.root.print_ex(
            1,
            global::SCREEN_HEIGHT - 2,
            BackgroundFlag::None,
            TextAlignment::Left,
            format!("HP: {}/{} ", fighter.hp, fighter.max_hp),
        );
    }
}

fn main() {
    let root = Root::initializer()
        .font("arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(global::SCREEN_WIDTH, global::SCREEN_HEIGHT)
        .title("Rust/libtcod tutorial")
        .init();

    let mut tcod = Tcod {
        root,
        con: Offscreen::new(global::MAP_WIDTH, global::MAP_HEIGHT),
        fov: FovMap::new(global::MAP_WIDTH, global::MAP_HEIGHT),
    };
    tcod::system::set_fps(global::LIMIT_FPS);
    // create object representing the player
    let mut player = objects::Object::new(25, 23, '@', "player", colors::WHITE, false);
    player.alive = true;
    player.fighter = Some(objects::Fighter {
        max_hp: 30,
        hp: 30,
        defense: 2,
        power: 5,
        on_death: objects::DeathCallback::Player,
    });

    // the list of objects with those two
    let mut objects = vec![player];
    let mut game = map::Game {
        // generate map (at this point it's not drawn to the screen)
        map: map::generate(global::MAP_WIDTH, global::MAP_HEIGHT, 25, 23, &mut objects),
    };
    map::init_fov_map(&mut tcod.fov, &game);
    let mut previous_player_position = (-1, -1);

    while !tcod.root.window_closed() {
        tcod.con.clear();
        let player = &objects[global::PLAYER];
        let fov_recompute = previous_player_position != player.pos();
        if fov_recompute {
            tcod.fov.compute_fov(
                player.x,
                player.y,
                global::TORCH_RADIUS,
                global::FOV_LIGHT_WALLS,
                global::FOV_ALGO,
            );
        }
        render_all(&mut tcod, &mut game, &objects);
        tcod.root.flush();

        let player = &mut objects[global::PLAYER];
        // handle keys and exit game if needed
        previous_player_position = player.pos();

        let player_action = handle_keys(&mut tcod, &game.map, &mut objects);
        if player_action == PlayerAction::Exit {
            break;
        }
        // let monsters take their turn
        if objects[global::PLAYER].alive && player_action != PlayerAction::DidntTakeTurn {
            for id in 0..objects.len() {
                if objects[id].ai.is_some() {
                    ai_take_turn(id, &tcod, &game.map, &mut objects);
                }
            }
        }
    }
}
