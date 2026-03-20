use macroquad::prelude::*;

mod game;
mod levels;

fn window_conf() -> Conf {
    Conf {
        window_title: "Cosmic Climber - Rust".to_owned(),
        window_width: 1280,
        window_height: 720,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    rand::srand(macroquad::miniquad::date::now() as u64);

    // 1. Load the texture FIRST
    let player_texture: Texture2D = load_texture("assets/astronaut.png").await.unwrap();
    player_texture.set_filter(FilterMode::Nearest);

    // 2. Pass it into the game constructor
    let mut game_state = game::Game::new(player_texture);

    loop {
        game_state.update();
        game_state.draw();
        next_frame().await
    }
}
