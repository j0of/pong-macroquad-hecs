use hecs::World;
use macroquad::prelude::*;

const STARTING_SPEED: f32 = 200.;
const SPEED_INCREASE: f32 = 25.;
const PLAYER_WIDTH: f32 = 30.;
const PLAYER_HEIGHT: f32 = 80.;
const BALL_SIZE: f32 = 30.;
const COLLISION_COOLDOWN: f32 = 0.2;

#[derive(PartialEq)]
enum State {
    PauseState,
    GameState,
    PlayerScoredState(i32),
}

fn conf() -> Conf {
    Conf {
        window_title: "Pong".to_string(),
        window_width: 500,
        window_height: 500,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(conf)]
async fn main() {
    let mut world = World::new();
    let mut state = State::PauseState;

    let mut elapsed = 0.;

    let mut score = (0, 0);

    let font = load_ttf_font("res/PressStart2P.ttf").await.unwrap();
    init_world(&mut world);
    loop {
        elapsed += get_frame_time();

        match state {
            State::PauseState => {
                if is_key_pressed(KeyCode::Space) {
                    state = State::GameState
                }
            }
            State::GameState => {
                execute_gamestate(&mut world, get_frame_time(), &mut elapsed, &mut score, &mut state);
            }
            State::PlayerScoredState(..) => {
                if is_key_pressed(KeyCode::Space) {
                    state = State::GameState
                }
                if is_key_pressed(KeyCode::R) {
                    system_reset(&world);
                    score = (0, 0);
                    state = State::PauseState;
                }
            }
        }

        clear_background(BLACK);
        match state {
            State::GameState => render_gamestate(&mut world, &font, score),
            State::PauseState => {
                draw_text_centre("PONG", 0., -100., 40, &font);
                draw_text_centre("Press SPACE to play", 0., 120., 20, &font);
            }

            State::PlayerScoredState(1) => {
                draw_text_centre("Player 1 Scored!", 0., -100., 20, &font);
                draw_text_centre("Press SPACE to continue", 0., 100., 15, &font);
                draw_text_centre("Press R to reset", 0., 125., 15, &font);
            }
            State::PlayerScoredState(2) => {
                draw_text_centre("Player 2 Scored!", 0., -50., 20, &font);
                draw_text_centre("Press SPACE to continue", 0., 100., 15, &font);
                draw_text_centre("Press R to reset", 0., 125., 15, &font);
            }
            _ => {
                println!("WTF how did that happen!?!? (invalid i32 associated with State::PlayerScoredState)");
                std::process::exit(1);
            }
        }

        render_gamestate(&mut world, &font, score);
        next_frame().await;
    }
}

// -- ==== ENTITIES ==== --

fn init_world(world: &mut World) {
    spawn_players(world);
    spawn_ball(world);
}

fn spawn_players(world: &mut World) {
    world.spawn((
        Bounds(Rect::new(
            20.,
            screen_height() / 2. - PLAYER_HEIGHT / 2.,
            PLAYER_WIDTH,
            PLAYER_HEIGHT,
        )),
        Speed(STARTING_SPEED),
        Velocity(Vec2::ZERO),
        Tint(WHITE),
        Player::Player1,
    ));

    world.spawn((
        Bounds(Rect::new(
            screen_width() - PLAYER_WIDTH - 20.,
            screen_height() / 2. - PLAYER_HEIGHT / 2.,
            PLAYER_WIDTH,
            PLAYER_HEIGHT,
        )),
        Speed(STARTING_SPEED),
        Velocity(Vec2::ZERO),
        Tint(WHITE),
        Player::Player2,
    ));
}

fn spawn_ball(world: &mut World) {
    world.spawn((
        Bounds(Rect::new(
            screen_width() / 2. - BALL_SIZE / 2.,
            screen_height() / 2. - BALL_SIZE / 2.,
            BALL_SIZE,
            BALL_SIZE,
        )),
        Speed(STARTING_SPEED),
        Velocity(vec2(rand::gen_range(-1., 1.), rand::gen_range(-1., 1.)).normalize()),
        Tint(WHITE),
        Ball,
    ));
}

// -- ==== COMPONENTS ==== --

struct Speed(f32);
struct Velocity(Vec2);
struct Bounds(Rect);
struct Tint(Color);

enum Player {
    Player1,
    Player2,
}

struct Ball;

// -- ==== SYSTEMS ==== --

fn execute_gamestate(world: &mut World, dt: f32, elapsed: &mut f32, score: &mut (i32, i32), state: &mut State) {
    system_apply_vel(world, dt);
    system_move_player(world);
    system_confine_bounds(world);
    system_bounce_ball_border(world);
    system_collide_ball_player(world, elapsed);
    system_score(world, score, state);
    if is_key_pressed(KeyCode::Escape) {
        *state = State::PauseState
    }
}

fn render_gamestate(world: &mut World, font: &Font, score: (i32, i32)) {
    draw_line(
        screen_width() / 2. - 1.,
        0.,
        screen_width() / 2. - 1.,
        screen_height(),
        1.,
        WHITE,
    );

    draw_text_centre(format!("{}", score.0).as_str(), -50., 0., 30, font);
    draw_text_centre(format!("{}", score.1).as_str(), 50., 0., 30, font);

    system_draw_bounds(world);
}

fn draw_text_centre(text: &str, x_offset: f32, y_offset: f32, size: u16, font: &Font) {
    let text_size = measure_text(text, Some(font), size, 1.);
    draw_text_ex(
        text,
        screen_width() / 2. - text_size.width / 2. + x_offset,
        screen_height() / 2. - text_size.height / 2. + y_offset,
        TextParams {
            font: Some(font),
            font_size: size,
            color: WHITE,
            ..Default::default()
        },
    );
}

fn system_apply_vel(world: &mut World, dt: f32) {
    for (_id, (bounds, vel, speed)) in world.query_mut::<(&mut Bounds, &Velocity, &Speed)>() {
        bounds.0.x += vel.0.x * speed.0 * dt;
        bounds.0.y += vel.0.y * speed.0 * dt;
    }
}

fn system_draw_bounds(world: &mut World) {
    for (_id, (rect, colour)) in world.query_mut::<(&Bounds, &Tint)>() {
        draw_rectangle(rect.0.x, rect.0.y, rect.0.w, rect.0.h, colour.0)
    }
}

fn system_move_player(world: &mut World) {
    for (_id, (vel, player)) in world.query_mut::<(&mut Velocity, &Player)>() {
        let (key_move_up, key_move_down) = match player {
            Player::Player1 => (KeyCode::W, KeyCode::S),
            Player::Player2 => (KeyCode::Up, KeyCode::Down),
        };

        if is_key_down(key_move_up) {
            vel.0.y = -1.
        } else if is_key_down(key_move_down) {
            vel.0.y = 1.
        } else {
            vel.0.y = 0.
        }
    }
}

fn system_confine_bounds(world: &mut World) {
    for (_id, (bounds, vel)) in world.query_mut::<(&mut Bounds, &mut Velocity)>() {
        if bounds.0.y < 0. {
            bounds.0.y = 0.;
            vel.0.y = 0.;
        } else if bounds.0.y + bounds.0.h > screen_width() {
            bounds.0.y = screen_height() - bounds.0.h;
            vel.0.y = 0.;
        }
    }
}

fn system_bounce_ball_border(world: &mut World) {
    for (_id, (bounds, vel, _)) in world.query_mut::<(&Bounds, &mut Velocity, &Ball)>() {
        if bounds.0.y == 0. {
            vel.0.y = 1.
        } else if bounds.0.y + bounds.0.h == screen_width() {
            vel.0.y = -1.
        }
    }
}

fn system_collide_ball_player(world: &World, elapsed: &mut f32) {
    for (_id, (player_bounds, player_speed,_player)) in world.query::<(&Bounds, &mut Speed, &Player)>().iter() {
        for (_id, (ball_bounds, ball_vel, ball_speed, _ball)) in world
            .query::<(&mut Bounds, &mut Velocity, &mut Speed, &Ball)>()
            .iter()
        {
            if *elapsed >= COLLISION_COOLDOWN {
                if let Some(intersect) = player_bounds.0.intersect(ball_bounds.0) {
                    if intersect.w < intersect.h {
                        if player_bounds.0.x < ball_bounds.0.x {
                            ball_bounds.0.x += intersect.w
                        } else {
                            ball_bounds.0.x -= intersect.w
                        }
                    } else {
                        if player_bounds.0.y < ball_bounds.0.y {
                            ball_bounds.0.y += intersect.h
                        } else {
                            ball_bounds.0.y -= intersect.h
                        }
                    }
                    ball_vel.0 = vec2(-ball_vel.0.x, rand::gen_range(-1., 1.)).normalize();
                    ball_speed.0 += SPEED_INCREASE;
                    player_speed.0 += SPEED_INCREASE;
                    *elapsed = 0.;
                }
            }
        }
    }
}

fn system_reset(world: &World) {
    for (_id, (bounds, vel, _ball)) in world.query::<(&mut Bounds, &mut Velocity, &Ball)>().iter() {
        bounds.0 = Rect::new(
            screen_width() / 2. - BALL_SIZE / 2.,
            screen_height() / 2. - BALL_SIZE / 2.,
            BALL_SIZE,
            BALL_SIZE,
        );

        vel.0 = vec2(rand::gen_range(-1., 1.), rand::gen_range(-1., 1.)).normalize();
    }
    for (_id, (bounds, player)) in world.query::<(&mut Bounds, &Player)>().iter() {
        match player {
            Player::Player1 => bounds.0 = Rect::new(
                20.,
                screen_height() / 2. - PLAYER_HEIGHT / 2.,
                PLAYER_WIDTH,
                PLAYER_HEIGHT,
            ),
            Player::Player2 => bounds.0 = Rect::new(
                screen_width() - PLAYER_WIDTH - 20.,
                screen_height() / 2. - PLAYER_HEIGHT / 2.,
                PLAYER_WIDTH,
                PLAYER_HEIGHT,
            ),
        }
    }
}

fn system_score(world: &World, score: &mut (i32, i32), state: &mut State) {
    let mut new_state = State::PlayerScoredState(-1);
    for (_id, (bounds, _ball)) in world.query::<(&Bounds, &Ball)>().iter() {
        if bounds.0.x > 0. && bounds.0.x + bounds.0.w < screen_width() {
            return
        } 

        if bounds.0.x < 0. {
            score.0 += 1;
            new_state = State::PlayerScoredState(1);
        } else if bounds.0.x + bounds.0.w > screen_width() {
            score.1 += 1;
            new_state = State::PlayerScoredState(2);
        }

    }
    system_reset(world);
    *state = new_state;
}
