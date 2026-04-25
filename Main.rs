use macroquad::prelude::*;
use std::fs;

#[derive(PartialEq, Clone, Copy)]
enum Difficulty { Easy, Normal, Hard }

impl Difficulty {
    fn get_life_time(&self) -> f32 {
        match self {
            Difficulty::Easy => 1.5,
            Difficulty::Normal => 1.0,
            Difficulty::Hard => 0.6,
        }
    }
    fn get_spawn_rate(&self) -> f32 {
        match self {
            Difficulty::Easy => 1.0,
            Difficulty::Normal => 0.7,
            Difficulty::Hard => 0.4,
        }
    }
}

#[derive(PartialEq)]
enum MoleType { Normal, Bomb }

#[derive(PartialEq)]
enum GameState { Title, Playing, GameOver }

struct Mole {
    x: f32,
    base_y: f32,
    active: bool,
    elapsed: f32,
    life_time: f32,
    mole_type: MoleType,
}

struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    lifetime: f32,
    color: Color,
}

struct HitText {
    x: f32,
    y: f32,
    lifetime: f32,
}

#[macroquad::main("Super Whack-A-Mole: Ultimate")]
async fn main() {
    let mut state = GameState::Title;
    let mut score: i32 = 0;
    let mut game_timer: f32 = 30.0;
    let mut selected_difficulty = Difficulty::Normal;
    let mut spawn_timer = 0.0;

    let mut moles = Vec::new();
    let mut particles: Vec<Particle> = Vec::new();
    let mut hit_texts: Vec<HitText> = Vec::new();

    let mut high_score: i32 = fs::read_to_string("high_score.txt")
        .unwrap_or_else(|_| "0".to_string())
        .trim().parse().unwrap_or(0);

    // 画面中央付近に配置
    let cell_size = 150.0;
    let offset_x = screen_width() / 2.0 - cell_size;
    let offset_y = screen_height() / 2.0 - cell_size + 50.0;

    for row in 0..3 {
        for col in 0..3 {
            moles.push(Mole {
                x: offset_x + col as f32 * cell_size,
                base_y: offset_y + row as f32 * cell_size,
                active: false,
                elapsed: 0.0,
                life_time: 1.0,
                mole_type: MoleType::Normal,
            });
        }
    }

    loop {
        clear_background(DARKGRAY);
        let delta = get_frame_time();
        let (mx, my) = mouse_position();
        let clicked = is_mouse_button_pressed(MouseButton::Left);

        match state {
            GameState::Title => {
                draw_text("SUPER WHACK-A-MOLE", screen_width()/2.0 - 230.0, 150.0, 50.0, GREEN);
                draw_text(&format!("High Score: {}", high_score), screen_width()/2.0 - 80.0, 200.0, 30.0, GOLD);
                draw_text("SELECT DIFFICULTY", screen_width()/2.0 - 130.0, 280.0, 30.0, WHITE);

                let diffs = [
                    (Difficulty::Easy, "EASY", GREEN, Color::new(0.0, 1.0, 0.0, 1.0)), 
                    (Difficulty::Normal, "NORMAL", BLUE, Color::new(0.5, 0.5, 1.0, 1.0)), 
                    (Difficulty::Hard, "HARD", RED, Color::new(1.0, 0.5, 0.5, 1.0))
                ];

                for (i, (d, label, col, hover_col)) in diffs.iter().enumerate() {
                    let bx = screen_width()/2.0 - 100.0;
                    let by = 320.0 + i as f32 * 60.0;
                    let is_hover = mx > bx && mx < bx + 200.0 && my > by && my < by + 50.0;
                    
                    let draw_color = if is_hover { *hover_col } else { *col };
                    draw_rectangle(bx, by, 200.0, 50.0, draw_color);
                    draw_text(label, bx + 60.0, by + 35.0, 30.0, WHITE);

                    if clicked && is_hover {
                        selected_difficulty = *d;
                        state = GameState::Playing;
                        score = 0;
                        game_timer = 30.0;
                        particles.clear();
                        hit_texts.clear();
                    }
                }
            }

            GameState::Playing => {
                game_timer -= delta;
                if game_timer <= 0.0 {
                    game_timer = 0.0;
                    state = GameState::GameOver;
                    if score > high_score {
                        high_score = score;
                        let _ = fs::write("high_score.txt", high_score.to_string());
                    }
                }

                spawn_timer += delta;
                if spawn_timer > selected_difficulty.get_spawn_rate() {
                    let idx = rand::gen_range(0, moles.len());
                    if !moles[idx].active {
                        moles[idx].active = true;
                        moles[idx].elapsed = 0.0;
                        moles[idx].life_time = selected_difficulty.get_life_time();
                        moles[idx].mole_type = if rand::gen_range(0, 5) == 0 { MoleType::Bomb } else { MoleType::Normal };
                    }
                    spawn_timer = 0.0;
                }

                for mole in moles.iter_mut() {
                    draw_circle(mole.x, mole.base_y, 60.0, BLACK);
                    if mole.active {
                        mole.elapsed += delta;
                        let t = mole.elapsed / mole.life_time;
                        if t > 1.0 { mole.active = false; continue; }

                        let height_offset = (t * std::f32::consts::PI).sin() * 70.0;
                        let current_y = mole.base_y - height_offset;

                        if clicked {
                            let dist = ((mx - mole.x).powi(2) + (my - current_y).powi(2)).sqrt();
                            if dist < 50.0 {
                                mole.active = false;
                                score += if mole.mole_type == MoleType::Normal { 10 } else { -50 };
                                for _ in 0..15 {
                                    particles.push(Particle {
                                        x: mole.x, y: current_y,
                                        vx: rand::gen_range(-200.0, 200.0), vy: rand::gen_range(-400.0, -100.0),
                                        lifetime: rand::gen_range(0.3, 0.6),
                                        color: if mole.mole_type == MoleType::Normal { YELLOW } else { RED },
                                    });
                                }
                                hit_texts.push(HitText { x: mole.x - 30.0, y: current_y - 20.0, lifetime: 0.5 });
                            }
                        }
                        let color = if mole.mole_type == MoleType::Normal { BROWN } else { RED };
                        draw_circle(mole.x, current_y, 50.0, color);
                        draw_circle(mole.x - 15.0, current_y - 10.0, 5.0, WHITE);
                        draw_circle(mole.x + 15.0, current_y - 10.0, 5.0, WHITE);
                    }
                }
            }

            GameState::GameOver => {
                draw_text("GAME OVER", screen_width()/2.0 - 140.0, screen_height()/2.0 - 50.0, 60.0, RED);
                draw_text(&format!("Final Score: {}", score), screen_width()/2.0 - 90.0, screen_height()/2.0 + 10.0, 30.0, WHITE);
                draw_text("Press [R] to Title", screen_width()/2.0 - 120.0, screen_height()/2.0 + 60.0, 30.0, YELLOW);
                if is_key_pressed(KeyCode::R) { state = GameState::Title; }
            }
        }

        // エフェクト更新（共通処理）
        particles.retain_mut(|p| {
            p.lifetime -= delta; p.x += p.vx * delta; p.y += p.vy * delta; p.vy += 500.0 * delta;
            if p.lifetime > 0.0 { draw_circle(p.x, p.y, p.lifetime * 5.0, p.color); true } else { false }
        });
        hit_texts.retain_mut(|ht| {
            ht.lifetime -= delta; ht.y -= 50.0 * delta;
            if ht.lifetime > 0.0 {
                draw_text("HIT!", ht.x, ht.y, 40.0, Color::new(1.0, 1.0, 0.0, ht.lifetime/0.5));
                true
            } else { false }
        });

        if state != GameState::Title {
            draw_text(&format!("Score: {}", score), 20.0, 40.0, 40.0, WHITE);
            draw_text(&format!("Time: {:.1}s", game_timer), 20.0, 80.0, 40.0, if game_timer < 5.0 { RED } else { WHITE });
        }
        next_frame().await
    }
}
