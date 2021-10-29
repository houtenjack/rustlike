use tcod::colors;
use tcod::console::*;
use tcod::map::Map as FovMap;
mod global;
mod map;
mod objects;

struct Tcod {
    root: Root,
    con: Offscreen,
    fov: FovMap,
}
fn handle_keys(tcod: &mut Tcod, game: &map::Game, player: &mut objects::Object) -> bool {
    use tcod::input::Key;
    use tcod::input::KeyCode::*;

    let key = tcod.root.wait_for_keypress(true);
    match key {
        Key {
            code: Enter,
            alt: true,
            ..
        } => {
            // Alt+Enter: toggle fullscreen
            let fullscreen = tcod.root.is_fullscreen();
            tcod.root.set_fullscreen(!fullscreen);
        }
        Key { code: Escape, .. } => return true, // exit game

        // movement keys
        Key { code: Up, .. } => player.move_by(0, -1, game),
        Key { code: Down, .. } => player.move_by(0, 1, game),
        Key { code: Left, .. } => player.move_by(-1, 0, game),
        Key { code: Right, .. } => player.move_by(1, 0, game),

        _ => {}
    }

    false
}

fn render_all(tcod: &mut Tcod, game: &mut map::Game, objects: &[objects::Object]) {
    // draw all objects in the list
    for object in objects {
        if tcod.fov.is_in_fov(object.x, object.y) {
            object.draw(&mut tcod.con);
        }
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
    let mut game = map::Game {
        // generate map (at this point it's not drawn to the screen)
        map: map::generate(global::MAP_WIDTH, global::MAP_HEIGHT, 25, 23),
    };
    map::init_fov_map(&mut tcod.fov, &game);
    let mut previous_player_position = (-1, -1);

    // create object representing the player
    let player = objects::Object::new(25, 23, '@', colors::WHITE);

    // the list of objects with those two
    let mut objects = [player];

    while !tcod.root.window_closed() {
        tcod.con.clear();
        let player = &objects[0];
        let fov_recompute = previous_player_position != (player.x, player.y);
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

        let player = &mut objects[0];
        // handle keys and exit game if needed
        previous_player_position = (player.x, player.y);

        let exit = handle_keys(&mut tcod, &game, player);
        if exit {
            break;
        }
    }
}
