use macroquad::prelude::*;
// We'll bring in Macroquad's color type for your flash/sky colors
use macroquad::prelude::Color;

// ── JS Bindings: Updated for 4-argument state transitions ──
#[cfg(target_arch = "wasm32")]
extern "C" {
    pub fn js_load_hi() -> i32;
    pub fn js_save_hi(s: i32);
    pub fn js_set_hud(sc: i32, lv: i32, lives: i32, hi: i32, fuel: i32);
    // Updated to match the new UI logic: state, score, level, timer
    pub fn js_set_state(state: i32, score: i32, level: i32, trans_timer: f32);
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn js_load_hi() -> i32 {
    0
}
#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn js_save_hi(_s: i32) {}
#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn js_set_hud(_sc: i32, _lv: i32, _l: i32, _h: i32, _f: i32) {}

// Update the stub to match 4 arguments!
#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn js_set_state(_state: i32, _score: i32, _level: i32, _trans_timer: f32) {}

// ── Constants ─────────────────────────────────────────
pub const SCREEN_W: f32 = 1280.0;
pub const SCREEN_H: f32 = 720.0;
pub const HALF_W: f32 = 640.0;
pub const HALF_H: f32 = 360.0;

pub const GRAVITY: f32 = 0.45;
pub const BASE_JUMP: f32 = -15.0;
pub const HOLD_JUMP_ADD: f32 = -4.5;
pub const PLAYER_SPEED: f32 = 7.5;
pub const FAST_FALL_SPD: f32 = 14.0;
pub const MAX_FALL: f32 = 20.0;
pub const JETPACK_THRUST: f32 = -0.9;
pub const JETPACK_FUEL: i32 = 180;
pub const BOUNCE_FACTOR: f32 = 1.55;
pub const SPRING_FORCE: f32 = -22.0;
pub const ICE_FRICTION: f32 = 0.98;

pub const LIVES_START: i32 = 3;
pub const NUM_LEVELS: usize = 10;

pub const LEVEL_THRESHOLDS: [f32; NUM_LEVELS] = [
    0.0, 800.0, 2000.0, 3500.0, 5000.0, 7500.0, 9800.0, 12400.0, 15400.0, 19000.0,
];

// ── Enums ─────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameState {
    Menu,
    Playing,
    Paused,
    GameOver,
    LevelTrans,
    Win,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlatformType {
    Normal,
    Moving,
    Bouncy,
    Crumble,
    Disappear,
    Spring,
    Conveyor,
    Lava,
    Cloud,
    Ice,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnemyType {
    Ghost,
    Turret,
    Chaser,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PowerupType {
    Jetpack,
    Shield,
    Star,
}

// ── Core Structs ──────────────────────────────────────

// We use `Default` so we can easily initialize a blank player, just like `memset(p, 0, sizeof(Player))`
#[derive(Default, Clone)]
pub struct Player {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub w: f32,
    pub h: f32,
    pub alive: bool,
    pub lives: i32,
    pub on_ground: bool,
    pub was_on_ground: bool,
    pub invincible: bool,
    pub inv_timer: i32,
    pub has_jetpack: bool,
    pub jet_fuel: i32,
    pub has_shield: bool,
    pub has_star: bool,
    pub star_timer: i32,
    pub shoot_cd: i32,
    pub anim_frame: i32,
    pub anim_timer: i32,
    pub face: i32,
    pub trail_x: [f32; 8],
    pub trail_y: [f32; 8],
    pub trail_head: usize,
}

#[derive(Clone)]
pub struct Platform {
    pub x: f32,
    pub y: f32,
    pub orig_x: f32,
    pub w: f32,
    pub h: f32,
    pub plat_type: PlatformType, // 'type' is a reserved keyword in Rust
    pub active: bool,
    pub vel: f32,
    pub range: f32,
    pub phase: f32,
    pub crumble: i32,
    pub crumble_t: i32,
    pub alpha: f32,
    pub visible: bool,
    pub conv_dir: i32,
    pub spring_ext: f32,
    pub has_pu: bool,
    pub pu_type: Option<PowerupType>, // Rust handles optional values beautifully!
    pub broken: bool,
}

// ── Secondary Game Objects ────────────────────────────

#[derive(Clone)]
pub struct Enemy {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub enemy_type: EnemyType, // 'type' is a reserved keyword in Rust
    pub alive: bool,
    pub hp: i32,
    pub anim_t: i32,
    pub shoot_t: f32,
    pub range: f32,
    pub orig_x: f32,
    pub bob: f32,
    pub phase: f32,
}

#[derive(Clone)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub max_life: f32,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub sz: f32,
    pub glow: bool,
}

#[derive(Clone)]
pub struct Bullet {
    pub x: f32,
    pub y: f32,
    pub vy: f32,
    pub active: bool,
    pub from_enemy: bool,
}

#[derive(Clone)]
pub struct Powerup {
    pub x: f32,
    pub y: f32,
    pub active: bool,
    pub pu_type: PowerupType, // 'type' is a reserved keyword
    pub bob: f32,
}

#[derive(Clone)]
pub struct Star {
    pub x: f32,
    pub y: f32,
    pub bri: f32,
    pub twinkle: f32,
}

// ── Level Configuration ───────────────────────────────
#[derive(Clone)]
pub struct LevelCfg {
    pub name: &'static str, // Replaces char name[24]
    pub desc: &'static str, // ADDED: For the transition subtitle
    pub scroll_spd: f32,
    pub gap_min: f32,
    pub gap_max: f32,
    pub plat_w_min: i32,
    pub plat_w_max: i32,
    pub type_weights: [i32; 10], // 10 is PLAT_TYPE_COUNT
    pub has_enemies: bool,
    pub enemy_chance: i32,
    pub sky_top: Color,
    pub sky_bot: Color, // Using Macroquad's Color instead of SDL_Color
    pub accent: Color,
    pub has_stars: bool,
    pub wind: f32,
}

// ── Main Game Struct ──────────────────────────────────
pub struct Game {
    pub player_tex: Texture2D,
    pub state: GameState,
    pub level: usize,
    pub score: i32,
    pub high_score: i32,

    pub cam_y: f32,
    pub max_height: f32,

    pub player: Player,

    // RUST UPGRADE: We replaced fixed arrays and manual counters
    // (like `Platform plats[MAX_PLATFORMS]; int plat_cnt;`) with Vectors!
    // They manage their own sizes dynamically.
    pub platforms: Vec<Platform>,
    pub enemies: Vec<Enemy>,
    pub particles: Vec<Particle>,
    pub bullets: Vec<Bullet>,
    pub powerups: Vec<Powerup>,
    pub stars: Vec<Star>,
    pub trans_timer: f32,

    pub gen_top: f32,
    pub world_top: f32,
    pub gen_prev_cx: f32,

    pub cp_x: f32,
    pub cp_y: f32,
    pub cp_valid: bool,
    pub cp_level: usize,

    pub shake: f32,
    pub flash_a: f32,
    pub flash_col: Color,

    // RUST UPGRADE: We completely removed the input tracking arrays
    // (key_held, key_just, key_time) because Macroquad tracks input for us natively!
    pub menu_sel: i32,
    pub menu_t: f32,
    pub trans_t: i32,
    pub trans_a: f32,

    pub dt: f32,
    pub time: f32,
}

// ── Rendering Colors ──────────────────────────────────
// This exactly matches your PLAT_COLS from render.c, converted to Macroquad's 0.0-1.0 Color scale
pub const PLAT_COLS: [Color; 10] = [
    Color::new(80. / 255., 200. / 255., 80. / 255., 1.0), // Normal
    Color::new(80. / 255., 140. / 255., 255. / 255., 1.0), // Moving
    Color::new(255. / 255., 180. / 255., 40. / 255., 1.0), // Bouncy
    Color::new(160. / 255., 100. / 255., 60. / 255., 1.0), // Crumble
    Color::new(200. / 255., 100. / 255., 220. / 255., 1.0), // Disappear
    Color::new(60. / 255., 200. / 255., 140. / 255., 1.0), // Spring
    Color::new(220. / 255., 160. / 255., 40. / 255., 1.0), // Conveyor
    Color::new(255. / 255., 60. / 255., 20. / 255., 1.0), // Lava
    Color::new(200. / 255., 220. / 255., 255. / 255., 1.0), // Cloud
    Color::new(160. / 255., 220. / 255., 255. / 255., 1.0), // Ice
];

impl Game {
    // ── Equivalent to game_init() ──
    pub fn new(player_tex: Texture2D) -> Self {
        // Build our starry background
        let mut stars = Vec::with_capacity(180);
        for _ in 0..180 {
            stars.push(Star {
                x: rand::gen_range(0.0, SCREEN_W),
                y: rand::gen_range(0.0, SCREEN_H * 3.0),
                bri: 0.3 + rand::gen_range(0.0, 0.7),
                twinkle: rand::gen_range(0.0, 6.28),
            });
        }

        // Create the base game object
        let mut game = Game {
            player_tex,
            state: GameState::Menu,
            level: 0,
            score: 0,
            high_score: unsafe { js_load_hi() },
            cam_y: 0.0,
            max_height: 0.0,
            player: Player::default(),

            platforms: Vec::new(),
            enemies: Vec::new(),
            particles: Vec::new(),
            bullets: Vec::new(),
            powerups: Vec::new(),
            stars,
            trans_timer: 0.0,

            gen_top: 0.0,
            world_top: 0.0,
            gen_prev_cx: 0.0,
            cp_x: 0.0,
            cp_y: 0.0,
            cp_valid: false,
            cp_level: 0,
            shake: 0.0,
            flash_a: 0.0,
            flash_col: WHITE,
            menu_sel: 0,
            menu_t: 0.0,
            trans_t: 0,
            trans_a: 0.0,
            dt: 0.0,
            time: 0.0,
        };

        // Initialize the first level
        game.reset(0);

        // ADD THIS LINE: Force the game back to the Menu state!
        game.state = GameState::Menu;

        game
    }

    // ── Equivalent to game_reset(int from_level) ──
    pub fn reset(&mut self, from_level: usize) {
        // Clear all dynamically sized vectors instead of memset
        self.platforms.clear();
        self.enemies.clear();
        self.particles.clear();
        self.bullets.clear();
        self.powerups.clear();

        self.score = 0;
        self.level = from_level;
        self.max_height = 0.0;

        // Reset player stats
        self.player = Player {
            x: SCREEN_W / 2.0 - 20.0,
            y: SCREEN_H - 120.0,
            w: 38.0,
            h: 44.0,
            vx: 0.0,
            vy: 0.0,
            alive: true,
            lives: LIVES_START,
            face: 1,
            ..Default::default() // Automatically fills the trail arrays with 0.0s
        };

        // Camera and generation tracking
        self.cam_y = self.player.y - SCREEN_H * 0.6;
        self.gen_top = SCREEN_H - 80.0; // The fix from your Changelog!
        self.gen_prev_cx = SCREEN_W / 2.0;

        // Checkpoint resets
        self.cp_x = self.player.x;
        self.cp_y = self.player.y;
        self.cp_valid = false;
        self.cp_level = from_level;

        // Seed the very first ground platform
        self.spawn_platform(
            SCREEN_W / 2.0 - 80.0,
            SCREEN_H - 80.0,
            160.0,
            PlatformType::Normal,
            false,
        );

        // TODO: We will write game_generate() in the next step!
        self.generate();

        self.state = GameState::Playing;
        self.shake = 0.0;
        self.flash_a = 0.0;
        self.time = 0.0;
    }

    // ── Equivalent to spawn_platform() ──
    pub fn spawn_platform(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        plat_type: PlatformType,
        has_pu: bool,
    ) {
        let vel = if plat_type == PlatformType::Moving {
            2.5 + rand::gen_range(0.0, 3.0)
        } else {
            0.0
        };
        let range = if plat_type == PlatformType::Moving {
            80.0 + rand::gen_range(0.0, 120.0)
        } else {
            0.0
        };
        let conv_dir = if rand::rand() % 2 == 0 { 1 } else { -1 };

        let pu_type = if has_pu {
            match rand::rand() % 3 {
                0 => Some(PowerupType::Jetpack),
                1 => Some(PowerupType::Shield),
                _ => Some(PowerupType::Star),
            }
        } else {
            None
        };

        // We just push it onto our dynamic Vector! No need to check MAX_PLATFORMS.
        self.platforms.push(Platform {
            x,
            y,
            orig_x: x,
            w,
            h: if plat_type == PlatformType::Spring {
                18.0
            } else {
                14.0
            },
            plat_type,
            active: true,
            alpha: 1.0,
            vel,
            range,
            conv_dir,
            has_pu,
            pu_type,
            phase: 0.0,
            crumble: 0,
            crumble_t: 0,
            visible: true,
            spring_ext: 0.0,
            broken: false,
        });
    }

    // ── Equivalent to spawn_enemy() ──
    pub fn spawn_enemy(&mut self, x: f32, y: f32, enemy_type: EnemyType) {
        let hp = if enemy_type == EnemyType::Turret {
            2
        } else {
            1
        };
        let vx = if enemy_type == EnemyType::Ghost || enemy_type == EnemyType::Chaser {
            2.0
        } else {
            0.0
        };

        self.enemies.push(Enemy {
            x,
            y,
            orig_x: x,
            vx,
            vy: 0.0,
            enemy_type,
            alive: true,
            hp,
            anim_t: 0,
            shoot_t: 120.0,
            range: 80.0 + rand::gen_range(0.0, 120.0),
            bob: 0.0,
            phase: 0.0,
        });
    }

    // ── Equivalent to game_generate() ──
    pub fn generate(&mut self) {
        let lc = &crate::levels::LEVEL_CFG[self.level];
        let top_needed = self.cam_y - SCREEN_H;

        let h_reach = 200.0 + (self.level as f32) * 20.0;

        while self.gen_top > top_needed {
            self.gen_top -= rand::gen_range(lc.gap_min, lc.gap_max);

            // Weighted random platform picking
            let total_weight: i32 = lc.type_weights.iter().sum();
            let pick = if total_weight > 0 {
                (rand::rand() % total_weight as u32) as i32
            } else {
                0
            };

            let mut plat_type = PlatformType::Normal;
            let mut acc = 0;
            let types = [
                PlatformType::Normal,
                PlatformType::Moving,
                PlatformType::Bouncy,
                PlatformType::Crumble,
                PlatformType::Disappear,
                PlatformType::Spring,
                PlatformType::Conveyor,
                PlatformType::Lava,
                PlatformType::Cloud,
                PlatformType::Ice,
            ];

            for (i, &weight) in lc.type_weights.iter().enumerate() {
                acc += weight;
                if pick < acc {
                    plat_type = types[i];
                    break;
                }
            }

            let pw = rand::gen_range(lc.plat_w_min as f32, lc.plat_w_max as f32);
            let half = pw * 0.5;

            // Constrain horizontal placement
            let mut cx_min = self.gen_prev_cx - h_reach;
            let mut cx_max = self.gen_prev_cx + h_reach;
            if cx_min < 30.0 + half {
                cx_min = 30.0 + half;
            }
            if cx_max > SCREEN_W - 30.0 - half {
                cx_max = SCREEN_W - 30.0 - half;
            }
            if cx_min > cx_max {
                cx_min = SCREEN_W * 0.5;
                cx_max = SCREEN_W * 0.5;
            }

            let span = cx_max - cx_min;
            let cx = cx_min
                + if span > 1.0 {
                    rand::gen_range(0.0, span)
                } else {
                    0.0
                };
            let px = cx - half;
            self.gen_prev_cx = cx;

            let has_pu = rand::rand() % 12 == 0;
            self.spawn_platform(px, self.gen_top, pw, plat_type, has_pu);

            // Enemy spawning
            if lc.has_enemies && lc.enemy_chance > 0 && (rand::rand() % lc.enemy_chance as u32) == 0
            {
                let et = match rand::rand() % 3 {
                    0 => EnemyType::Ghost,
                    1 => EnemyType::Turret,
                    _ => EnemyType::Chaser,
                };
                self.spawn_enemy(px + pw / 2.0, self.gen_top - 50.0, et);
            }
        }

        // --- THE RUST WAY TO CULL ARRAYS ---
        // Say goodbye to shifting arrays manually! `retain` keeps only the items that return `true`.
        let cam_y = self.cam_y;
        self.platforms.retain(|p| p.y - cam_y <= SCREEN_H + 200.0);
        self.enemies.retain(|e| e.y - cam_y <= SCREEN_H + 200.0);
    }

    // ── Particle Engine Logic ──
    pub fn spawn_particle(
        &mut self,
        x: f32,
        y: f32,
        vx: f32,
        vy: f32,
        life: f32,
        r: u8,
        g: u8,
        b: u8,
        sz: f32,
        glow: bool,
    ) {
        self.particles.push(Particle {
            x,
            y,
            vx,
            vy,
            life,
            max_life: life,
            r,
            g,
            b,
            sz,
            glow,
        });
    }

    pub fn update_particles(&mut self) {
        for p in &mut self.particles {
            p.x += p.vx;
            p.y += p.vy;
            p.vy += 0.12; // Gravity effect on particles
            p.vx *= 0.97; // Air friction
            p.life -= self.dt;
        }

        // Magically remove all particles whose life has run out!
        self.particles.retain(|p| p.life > 0.0);
    }

    // ── Equivalent to spawn_bullet() ──
    pub fn spawn_bullet(&mut self, x: f32, y: f32, from_enemy: bool) {
        let vy = if from_enemy { 8.0 } else { -18.0 };
        self.bullets.push(Bullet {
            x,
            y,
            vy,
            active: true,
            from_enemy,
        });
    }

    // ── Equivalent to update_bullets() ──
    pub fn update_bullets(&mut self) {
        let cam_y = self.cam_y;
        let px = self.player.x + self.player.w / 2.0;
        let py = self.player.y + self.player.h / 2.0;
        let mut player_hit = false;

        for b in &mut self.bullets {
            if !b.active {
                continue;
            }
            b.y += b.vy;

            // Off-screen cull
            if b.y < cam_y - 100.0 || b.y > cam_y + SCREEN_H + 100.0 {
                b.active = false;
            }

            // Enemy bullet hits player
            if b.from_enemy
                && self.player.alive
                && !self.player.invincible
                && !self.player.has_shield
            {
                if (b.x - px).abs() < 16.0 && (b.y - py).abs() < 16.0 {
                    b.active = false;
                    player_hit = true;
                }
            }
        }

        // Apply player damage outside the loop to keep the Borrow Checker happy!
        if player_hit {
            self.player.lives -= 1;
            self.player.invincible = true;
            self.player.inv_timer = 120;
            self.shake = self.shake.max(5.0);
            self.flash_col = Color::new(200. / 255., 50. / 255., 50. / 255., 1.0);
            self.flash_a = 0.6;
        }

        self.bullets.retain(|b| b.active);
    }

    // ── Equivalent to update_enemies() ──
    pub fn update_enemies(&mut self) {
        let px = self.player.x;
        let py = self.player.y;
        let pw = self.player.w;
        let ph = self.player.h;

        let mut player_hit = false;
        let mut new_enemy_bullets = Vec::new();

        // 1. Move enemies and check if they touch the player
        for e in &mut self.enemies {
            if !e.alive {
                continue;
            }
            e.anim_t += 1;
            e.bob = ((e.anim_t as f32) * 0.05).sin() * 6.0;

            match e.enemy_type {
                EnemyType::Ghost => {
                    e.phase += 0.025;
                    e.x = e.orig_x + e.phase.sin() * e.range;
                }
                EnemyType::Chaser => {
                    let dx = px - e.x;
                    e.vx += if dx > 0.0 { 0.12 } else { -0.12 };
                    e.vx *= 0.95;
                    e.x += e.vx;
                }
                EnemyType::Turret => {
                    e.shoot_t -= 1.0;
                    if e.shoot_t <= 0.0 {
                        new_enemy_bullets.push((e.x, e.y + 20.0));
                        e.shoot_t = 90.0 + rand::gen_range(0.0, 60.0);
                    }
                }
            }

            // Body collision with player
            if self.player.alive
                && !self.player.invincible
                && !self.player.has_shield
                && !self.player.has_star
            {
                let dx = (px + pw / 2.0) - e.x;
                let dy = (py + ph / 2.0) - e.y;
                if dx.abs() < 28.0 && dy.abs() < 28.0 {
                    player_hit = true;
                }
            }
        }

        // Spawn turret bullets
        for (bx, by) in new_enemy_bullets {
            self.spawn_bullet(bx, by, true);
        }

        if player_hit {
            self.player.lives -= 1;
            self.player.invincible = true;
            self.player.inv_timer = 120;
            self.player.vy = BASE_JUMP * 0.7;
            self.shake = self.shake.max(7.0);
            self.flash_col = Color::new(220. / 255., 50. / 255., 50. / 255., 1.0);
            self.flash_a = 0.7;
        }

        // 2. Check if player bullets hit enemies
        let lc_accent = crate::levels::LEVEL_CFG[self.level].accent;
        let mut enemies_killed = 0;

        for e in &mut self.enemies {
            if !e.alive {
                continue;
            }
            for b in &mut self.bullets {
                if !b.active || b.from_enemy {
                    continue;
                }
                let dx = b.x - e.x;
                let dy = b.y - e.y;
                if dx.abs() < 24.0 && dy.abs() < 24.0 {
                    b.active = false;
                    e.hp -= 1;
                    if e.hp <= 0 {
                        e.alive = false;
                        enemies_killed += 1;

                        // Death Explosion Particles!
                        for _ in 0..16 {
                            let vx = rand::gen_range(-10.0, 10.0);
                            let vy = rand::gen_range(-10.0, 10.0);
                            let life = 0.8 + rand::gen_range(0.0, 0.4);
                            self.particles.push(Particle {
                                x: e.x,
                                y: e.y,
                                vx,
                                vy,
                                life,
                                max_life: life,
                                r: (lc_accent.r * 255.0) as u8,
                                g: (lc_accent.g * 255.0) as u8,
                                b: (lc_accent.b * 255.0) as u8,
                                sz: 5.0,
                                glow: true,
                            });
                        }
                    }
                }
            }
        }

        if enemies_killed > 0 {
            self.shake = self.shake.max(2.0);
            self.score += 50 * (self.level as i32 + 1) * enemies_killed;
        }

        self.enemies.retain(|e| e.alive);
    }

    // ── Equivalent to update_camera() ──
    pub fn update_camera(&mut self) {
        let target_y = self.player.y - SCREEN_H * 0.55;
        // Only scroll up, never down
        if target_y < self.cam_y {
            self.cam_y += (target_y - self.cam_y) * 0.12;
        }

        // Track max height
        let h = -self.cam_y + SCREEN_H;
        if h > self.max_height {
            self.max_height = h;
        }
    }

    // ── Equivalent to update_player() ──
    pub fn update_player(&mut self) {
        if !self.player.alive {
            return;
        }

        let lc = crate::levels::LEVEL_CFG[self.level].clone();

        // Input: We use Macroquad's native input checks!
        let left = is_key_down(KeyCode::Left);
        let right = is_key_down(KeyCode::Right);
        let up = is_key_down(KeyCode::Up);
        let down = is_key_down(KeyCode::Down);
        let shoot = is_key_pressed(KeyCode::Enter)
            || is_key_pressed(KeyCode::KpEnter)
            || is_key_pressed(KeyCode::Space); // Added Spacebar for convenience!

        // In C you tracked 'just pressed' manually. Macroquad does this for us.
        let lj = is_key_pressed(KeyCode::Left);
        let rj = is_key_pressed(KeyCode::Right);

        // 1. Horizontal Movement
        if left || lj {
            self.player.vx -= PLAYER_SPEED * if left { 1.0 } else { 0.5 };
            self.player.face = -1;
        }
        if right || rj {
            self.player.vx += PLAYER_SPEED * if right { 1.0 } else { 0.5 };
            self.player.face = 1;
        }

        // 2. Friction (Ice vs Normal)
        let mut fric = 0.80;
        for p in &self.platforms {
            if p.active && p.plat_type == PlatformType::Ice && self.player.on_ground {
                fric = ICE_FRICTION;
            }
        }

        self.player.vx *= fric;
        if self.player.vx.abs() > PLAYER_SPEED * 1.5 {
            self.player.vx = self.player.vx.signum() * PLAYER_SPEED * 1.5;
        }

        // 3. Gravity / Jetpack
        if self.player.has_jetpack && self.player.jet_fuel > 0 && up {
            self.player.vy += JETPACK_THRUST;
            self.player.jet_fuel -= 2; // Increased consumption from 1 to 2

            if self.player.jet_fuel <= 0 {
                self.player.has_jetpack = false;
            }
        } else {
            self.player.vy += GRAVITY;
        }

        // Inside update_player in src/game.rs
        if self.player.has_shield {
            // Spawn shield particles during the update phase
            if rand::gen_range(0, 10) == 0 {
                let px = self.player.x + self.player.w / 2.0;
                let py = self.player.y + self.player.h / 2.0;
                let angle: f32 = rand::gen_range(0.0, 6.28);
                let dist: f32 = rand::gen_range(10.0, 32.0);

                // We use world coordinates here (no cam_y needed for spawning)
                self.spawn_particle(
                    px + angle.cos() * dist,
                    py + angle.sin() * dist,
                    0.0,
                    -0.5,
                    0.5,
                    100,
                    200,
                    255,
                    2.0,
                    true,
                );
            }
        }

        // Fast fall
        if down && self.player.vy > 0.0 {
            self.player.vy = (self.player.vy + 2.0).min(FAST_FALL_SPD);
        }

        // Wind
        if self.trans_timer <= 0.0 {
            self.player.vx += lc.wind * self.dt * 60.0;
        }

        // Terminal velocity
        if self.player.vy > MAX_FALL {
            self.player.vy = MAX_FALL;
        }

        // 4. Apply Velocity
        self.player.was_on_ground = self.player.on_ground;
        self.player.on_ground = false;

        // Store previous Y for collision detection
        let prev_y_bot = self.player.y + self.player.h;

        self.player.x += self.player.vx;
        self.player.y += self.player.vy;

        // Wrap horizontally
        if self.player.x + self.player.w < 0.0 {
            self.player.x = SCREEN_W;
        }
        if self.player.x > SCREEN_W {
            self.player.x = -self.player.w;
        }

        let py_bot = self.player.y + self.player.h;

        // 5. Platform Collisions (The tricky part!)
        // We iterate with mutability so we can change platform states (like crumbling)
        for i in 0..self.platforms.len() {
            let pl = &mut self.platforms[i];

            if !pl.active || pl.broken {
                continue;
            }
            if pl.plat_type == PlatformType::Lava {
                continue;
            } // Handled separately
            if pl.plat_type == PlatformType::Disappear && !pl.visible {
                continue;
            }
            if pl.plat_type == PlatformType::Cloud && self.player.vy <= 0.0 {
                continue;
            }

            let x_over =
                (self.player.x + self.player.w > pl.x + 4.0) && (self.player.x < pl.x + pl.w - 4.0);
            let y_cross = (py_bot >= pl.y) && (prev_y_bot <= pl.y + pl.h / 2.0 + 2.0);

            if x_over && y_cross && self.player.vy >= 0.0 {
                if pl.plat_type == PlatformType::Conveyor {
                    self.player.vx += (pl.conv_dir as f32) * 3.5;
                }

                // Bounce Logic
                match pl.plat_type {
                    PlatformType::Bouncy => {
                        self.player.vy = BASE_JUMP * BOUNCE_FACTOR;
                        if up {
                            self.player.vy += HOLD_JUMP_ADD;
                        }
                        self.shake = self.shake.max(3.0);
                    }
                    PlatformType::Spring => {
                        self.player.vy = SPRING_FORCE;
                        if up {
                            self.player.vy -= 4.0;
                        }
                        pl.spring_ext = 12.0;
                        self.shake = self.shake.max(2.0);
                    }
                    PlatformType::Crumble => {
                        self.player.vy = BASE_JUMP;
                        if up && !self.player.was_on_ground {
                            self.player.vy += HOLD_JUMP_ADD;
                        }
                        self.player.y = pl.y - self.player.h;
                        self.player.on_ground = true;
                        if pl.crumble == 0 {
                            pl.crumble = 1;
                        }
                    }
                    _ => {
                        self.player.vy = BASE_JUMP;
                        if up && !self.player.was_on_ground {
                            self.player.vy += HOLD_JUMP_ADD;
                        }
                        self.player.y = pl.y - self.player.h;
                        self.player.on_ground = true;

                        // Spawn small jump dust
                        // (RUST FIX: Push directly to the vector to avoid locking the whole 'self')
                        for _ in 0..5 {
                            let px = self.player.x + rand::gen_range(0.0, self.player.w);
                            let py = self.player.y + self.player.h;
                            let vx = rand::gen_range(-4.0, 4.0) * 0.5;
                            let vy = -rand::gen_range(0.0, 4.0);

                            self.particles.push(Particle {
                                x: px,
                                y: py,
                                vx,
                                vy,
                                life: 0.3,
                                max_life: 0.3,
                                r: 220,
                                g: 220,
                                b: 220,
                                sz: 2.5,
                                glow: false,
                            });
                        }
                    }
                }

                // Powerup Pickup
                if pl.has_pu {
                    pl.has_pu = false;
                    if let Some(pu) = pl.pu_type {
                        match pu {
                            PowerupType::Jetpack => {
                                self.player.has_jetpack = true;
                                self.player.jet_fuel = JETPACK_FUEL;
                            }
                            PowerupType::Shield => self.player.has_shield = true,
                            PowerupType::Star => {
                                self.player.has_star = true;
                                self.player.star_timer = 300;
                            }
                        }
                        self.flash_col = Color::new(0.4, 0.7, 1.0, 1.0); // Blue flash for shield
                        self.flash_a = 0.4;
                    }
                }

                // Checkpoint Logic (From your Changelog bugfix!)
                if self.player.y < self.cp_y || !self.cp_valid {
                    if pl.plat_type != PlatformType::Disappear
                        && pl.plat_type != PlatformType::Crumble
                    {
                        self.cp_x = self.player.x;
                        self.cp_y = self.player.y;
                        self.cp_valid = true;
                        self.cp_level = self.level;
                    }
                }
            }
        }

        // 6. Shooting Logic
        if self.player.shoot_cd > 0 {
            self.player.shoot_cd -= 1;
        }

        if shoot && self.player.shoot_cd <= 0 {
            let bx = self.player.x + self.player.w / 2.0;
            let by = self.player.y;
            // Push directly to bypass the borrow checker
            self.bullets.push(Bullet {
                x: bx,
                y: by,
                vy: -18.0,
                active: true,
                from_enemy: false,
            });
            self.player.shoot_cd = 20;

            // Muzzle flash particle
            self.particles.push(Particle {
                x: bx,
                y: by,
                vx: 0.0,
                vy: -5.0,
                life: 0.25,
                max_life: 0.25,
                r: 180,
                g: 220,
                b: 255,
                sz: 6.0,
                glow: true,
            });
        }

        // 7. Timers (Invincibility & Star)
        if self.player.invincible {
            self.player.inv_timer -= 1;
            if self.player.inv_timer <= 0 {
                self.player.invincible = false;
            }
        }

        if self.player.has_star {
            // Use trail_head for both arrays to keep them in sync
            self.player.trail_x[self.player.trail_head] = self.player.x;
            self.player.trail_y[self.player.trail_head] = self.player.y;

            // Increment the head index and wrap it around the 8-slot buffer
            self.player.trail_head = (self.player.trail_head + 1) % 8;
        }

        // Astronaut.png layout (768×64, 12 frames of 64×64):
        //   Idle frames : columns 0 and 1  (front-facing, 2-frame breathe cycle)
        //   Walk frames : columns 8, 9, 10 (facing right, 3-frame walk cycle)
        //
        // anim_frame stores the CYCLE INDEX (not the sheet column directly):
        //   Idle state → anim_frame cycles 0-1 (slow, every 20 ticks)
        //   Walk state → anim_frame cycles 0-2 (normal, every 6 ticks)
        // The draw function maps cycle index → sheet column.
        let walking = self.player.vx.abs() > 0.2 && self.player.on_ground;
        if walking {
            self.player.anim_timer += 1;
            if self.player.anim_timer >= 6 {
                self.player.anim_timer = 0;
                self.player.anim_frame = (self.player.anim_frame + 1) % 3; // 3 walk frames
            }
        } else {
            // Idle / airborne: slow 2-frame breathe animation
            self.player.anim_timer += 1;
            if self.player.anim_timer >= 20 {
                self.player.anim_timer = 0;
                self.player.anim_frame = (self.player.anim_frame + 1) % 2; // 2 idle frames
            }
        }
    }

    // ── Equivalent to update_platforms() ──
    pub fn update_platforms(&mut self) {
        for p in &mut self.platforms {
            if !p.active {
                continue;
            }

            // Moving platform logic
            if p.plat_type == PlatformType::Moving {
                p.phase += 0.03;
                p.x = p.orig_x + p.phase.sin() * p.range;
            }

            // Crumbling logic
            if p.plat_type == PlatformType::Crumble && p.crumble > 0 {
                p.crumble_t += 1;
                if p.crumble == 1 && p.crumble_t > 30 {
                    p.crumble = 2;
                    p.crumble_t = 0;
                    // TODO: Spawn debris particles here in the next step!
                }
                if p.crumble == 2 && p.crumble_t > 20 {
                    p.broken = true;
                    p.active = false;
                }
            }

            // Disappearing logic
            if p.plat_type == PlatformType::Disappear {
                p.phase += 0.04;
                let v = (p.phase.sin() + 1.0) * 0.5;
                p.visible = v > 0.4;
                p.alpha = v;
            }

            // Spring visual reset
            if p.spring_ext > 0.0 {
                p.spring_ext -= 0.8;
            }
        }
    }

    // ── Equivalent to check_death() ──
    pub fn check_death(&mut self) {
        // Fell below camera
        if self.player.y - self.cam_y > SCREEN_H + 100.0 && self.player.vy > 0.0 {
            if !self.player.invincible {
                self.player.lives -= 1;
                self.shake = self.shake.max(10.0);
                self.flash_col = Color::new(100. / 255., 100. / 255., 100. / 255., 1.0);
                self.flash_a = 0.9;

                if self.player.lives <= 0 {
                    self.player.alive = false;
                    self.state = GameState::GameOver;
                } else {
                    // Find a safe platform to respawn on
                    let mut found_safe = false;
                    let mut target_x = SCREEN_W / 2.0 - self.player.w / 2.0;
                    let mut target_y = self.cam_y + SCREEN_H * 0.5;

                    // Try checkpoint first
                    if self.cp_valid && self.cp_level == self.level {
                        for pl in &self.platforms {
                            if !pl.active || pl.broken || pl.plat_type == PlatformType::Lava {
                                continue;
                            }
                            if pl.plat_type == PlatformType::Disappear && !pl.visible {
                                continue;
                            }

                            if (pl.y - self.cp_y).abs() < 50.0 {
                                target_x = pl.x + pl.w / 2.0 - self.player.w / 2.0;
                                target_y = pl.y - self.player.h - 4.0;
                                found_safe = true;
                                break;
                            }
                        }
                    }

                    // Fallback: lowest visible platform
                    if !found_safe {
                        let mut best_y = -99999.0;
                        for pl in &self.platforms {
                            if !pl.active || pl.broken || pl.plat_type == PlatformType::Lava {
                                continue;
                            }
                            if pl.plat_type == PlatformType::Disappear && !pl.visible {
                                continue;
                            }

                            if pl.y - self.cam_y < -100.0 || pl.y - self.cam_y > SCREEN_H {
                                continue;
                            }

                            if pl.y > best_y {
                                best_y = pl.y;
                                target_x = pl.x + pl.w / 2.0 - self.player.w / 2.0;
                                target_y = pl.y - self.player.h - 4.0;
                            }
                        }
                    }

                    self.player.x = target_x;
                    self.player.y = target_y;
                    self.player.vx = 0.0;
                    self.player.vy = BASE_JUMP * 0.6;
                    self.player.invincible = true;
                    self.player.inv_timer = 180;
                    self.cam_y = self.player.y - SCREEN_H * 0.5;

                    // Update checkpoint
                    self.cp_x = self.player.x;
                    self.cp_y = self.player.y;
                    self.cp_valid = true;
                    self.cp_level = self.level;
                }
            }
        }
    }

    // ── The Master Game Loop Update ──
    pub fn update(&mut self) {
        self.dt = get_frame_time().min(0.05);
        self.time += self.dt;

        // 1. Tell the UI what state we are in immediately!
        unsafe {
            js_set_state(
                self.state as i32,
                self.score,
                self.level as i32,
                self.trans_timer,
            );
        }

        // State machine input handling
        let enter = is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter);
        let escape = is_key_pressed(KeyCode::Escape);

        match self.state {
            GameState::Menu => {
                if enter {
                    self.reset(0);
                }
                return;
            }
            GameState::GameOver => {
                if enter {
                    self.reset(0);
                }
                return;
            }
            GameState::Paused => {
                // Resume if Enter or Escape is pressed
                if enter || escape {
                    self.state = GameState::Playing;
                }
                return;
            }
            GameState::Playing => {
                if escape {
                    self.state = GameState::Paused;
                }
            }
            _ => {}
        }

        self.update_player();
        self.update_platforms();
        self.update_particles();
        self.update_bullets();
        self.update_enemies();
        self.update_camera();
        self.generate();
        self.check_death();

        // ── Transition Timer Countdown ──
        if self.trans_timer > 0.0 {
            self.trans_timer -= self.dt;
        }

        // ── FX Decay ──
        self.shake = (self.shake - 0.5).max(0.0);
        self.flash_a = (self.flash_a - 0.03).max(0.0);

        // ── Score Calculation ──
        let sc = (self.max_height / 10.0) as i32;
        if sc > self.score {
            self.score = sc;
        }
        if self.score > self.high_score {
            self.high_score = self.score;
            // (We will hook up the JS localStorage save here later!)
        }

        // ── Level Transition Check ──
        let mut new_lv = 0;
        for i in (0..crate::game::NUM_LEVELS).rev() {
            if self.max_height >= crate::game::LEVEL_THRESHOLDS[i] {
                new_lv = i;
                break;
            }
        }

        if new_lv != self.level {
            self.level = new_lv;
            self.trans_timer = 3.0; // Start the 3-second "Warning" state

            self.flash_col = Color::new(1.0, 1.0, 0.8, 1.0);
            self.flash_a = 0.6;
            self.shake = self.shake.max(4.0);
            self.cp_valid = false;
            self.cp_level = new_lv;
        }

        // ── JS Interop: Save & Update HUD ──
        if self.score > self.high_score {
            self.high_score = self.score;
            unsafe {
                js_save_hi(self.high_score);
            }
        }

        unsafe {
            let fuel = if self.player.has_jetpack {
                self.player.jet_fuel
            } else {
                0
            };

            // Update the HUD values
            js_set_hud(
                self.score,
                self.level as i32,
                self.player.lives,
                self.high_score,
                fuel,
            );

            // FIX: Pass the level and the trans_timer as the 3rd and 4th arguments
            js_set_state(
                self.state as i32,
                self.score,
                self.level as i32,
                self.trans_timer,
            );
        }
    }

    // ── Equivalent to render_frame() & render_platform() ──
    pub fn draw(&self) {
        let lc = &crate::levels::LEVEL_CFG[self.level];

        // 1. Background
        // For the POC, we'll clear with the sky_bot color
        clear_background(lc.sky_bot);

        // Draw Stars with parallax
        if lc.has_stars {
            for s in &self.stars {
                let mut sy = (s.y - self.cam_y * 0.2) % (SCREEN_H * 3.0);
                if sy < 0.0 {
                    sy += SCREEN_H * 3.0;
                }

                if sy <= SCREEN_H {
                    let twinkle = 0.5 + 0.5 * s.twinkle.sin();
                    let bri = twinkle * s.bri;
                    draw_circle(s.x, sy, 1.5, Color::new(bri, bri, bri * 0.9, 1.0));
                }
            }
        }

        // 2. Draw Platforms
        for p in &self.platforms {
            if !p.active {
                continue;
            }

            let shake_x = if self.shake > 0.0 {
                rand::gen_range(-2.0, 2.0) * self.shake * 0.1
            } else {
                0.0
            };
            let shake_y = if self.shake > 0.0 {
                rand::gen_range(-2.0, 2.0) * self.shake * 0.1
            } else {
                0.0
            };
            let sx = p.x + shake_x;
            let sy = p.y - self.cam_y + shake_y;

            let mut c = PLAT_COLS[p.plat_type as usize];
            if p.plat_type == PlatformType::Disappear {
                c.a = p.alpha;
            }

            match p.plat_type {
                PlatformType::Lava => {
                    // Lava: dark base + animated top
                    draw_rectangle(
                        sx,
                        sy,
                        p.w,
                        p.h,
                        Color::new(100. / 255., 20. / 255., 0.0, c.a),
                    );
                    let lava_g = (60.0 + (self.time * 6.0 + p.x).sin() * 30.0) / 255.0;
                    draw_rectangle(sx, sy, p.w, 4.0, Color::new(1.0, lava_g, 0.0, c.a));
                    draw_rectangle(
                        sx,
                        sy - 6.0,
                        p.w,
                        6.0,
                        Color::new(1.0, 80. / 255., 0.0, c.a / 3.0),
                    );
                }
                PlatformType::Cloud => {
                    // Fluffy cloud shape
                    draw_rectangle(sx, sy + 4.0, p.w, p.h, c);
                    draw_circle(sx + p.w / 4.0, sy + 4.0, p.h + 2.0, c);
                    draw_circle(sx + p.w / 2.0, sy, p.h + 4.0, c);
                    draw_circle(sx + 3.0 * p.w / 4.0, sy + 4.0, p.h + 2.0, c);
                }
                PlatformType::Spring => {
                    // Spring coil expansion
                    let ext = p.spring_ext;
                    draw_rectangle(sx, sy + ext, p.w, p.h - ext, c);
                    for i in 0..4 {
                        draw_rectangle(
                            sx + 4.0 + (i as f32) * (p.w - 8.0) / 4.0,
                            sy + 2.0 + ext,
                            (p.w - 8.0) / 4.0 - 2.0,
                            4.0,
                            Color::new(1.0, 1.0, 1.0, c.a * 0.6),
                        );
                    }
                }
                _ => {
                    // Standard platform body
                    draw_rectangle(sx, sy, p.w, p.h, c);
                    // Highlight top edge
                    draw_rectangle(
                        sx,
                        sy,
                        p.w,
                        3.0,
                        Color::new(
                            (c.r + 0.2).min(1.0),
                            (c.g + 0.2).min(1.0),
                            (c.b + 0.2).min(1.0),
                            c.a,
                        ),
                    );

                    // Conveyor arrows
                    if p.plat_type == PlatformType::Conveyor {
                        let offset = ((self.time * 40.0 * (p.conv_dir as f32)) as i32
                            % (p.w as i32 / 3))
                            .abs() as f32;
                        let mut ax = sx + offset;
                        while ax < sx + p.w - 6.0 {
                            if ax >= sx {
                                // Simplified conveyor visual for Macroquad
                                draw_line(
                                    ax,
                                    sy + p.h / 2.0,
                                    ax + 6.0,
                                    sy + p.h / 2.0,
                                    2.0,
                                    Color::new(1.0, 0.9, 0.2, 0.7),
                                );
                            }
                            ax += p.w / 3.0;
                        }
                    }

                    // Crumble cracks
                    if p.plat_type == PlatformType::Crumble && p.crumble >= 1 {
                        let shake_off = ((self.time * 30.0).sin() * 2.0) as f32;
                        draw_line(
                            sx + shake_off,
                            sy,
                            sx + p.w / 2.0 + shake_off,
                            sy + p.h,
                            2.0,
                            Color::new(0.3, 0.2, 0.1, 0.8),
                        );
                    }

                    // ── Draw Powerup Icon ──
                    if p.has_pu {
                        let px = sx + p.w / 2.0;
                        let py = sy - 12.0 + (self.time * 4.0).sin() * 4.0; // Floating animation

                        let col = match p.pu_type {
                            Some(PowerupType::Jetpack) => Color::new(1.0, 0.6, 0.0, 1.0), // Orange
                            Some(PowerupType::Shield) => Color::new(0.2, 0.8, 1.0, 1.0),  // Cyan
                            _ => Color::new(1.0, 1.0, 0.0, 1.0), // Yellow (Star)
                        };

                        // Draw a glowing diamond/droplet
                        draw_poly(px, py, 4, 8.0, 0.0, col);
                        draw_circle(px, py, 4.0, WHITE); // Sparkle center
                    }
                }
            }
        }

        // ── Draw Particles ──
        for p in &self.particles {
            let t = p.life / p.max_life; // Fades out as life approaches 0
            let a = t;

            let px = p.x;
            let py = p.y - self.cam_y;

            // Don't draw if off-screen
            if py < -20.0 || py > SCREEN_H + 20.0 {
                continue;
            }

            let mut sz = p.sz * t;
            if sz < 1.0 {
                sz = 1.0;
            }

            // Convert 0-255 RGB values to Macroquad's 0.0-1.0 Color format
            let base_color = Color::new(
                p.r as f32 / 255.0,
                p.g as f32 / 255.0,
                p.b as f32 / 255.0,
                a,
            );

            if p.glow {
                // Glow effect uses larger, highly transparent circles
                draw_circle(
                    px,
                    py,
                    sz + 3.0,
                    Color::new(base_color.r, base_color.g, base_color.b, a * 0.25),
                );
                draw_circle(
                    px,
                    py,
                    sz + 1.0,
                    Color::new(base_color.r, base_color.g, base_color.b, a * 0.5),
                );
            }
            draw_circle(px, py, sz, base_color);
        }

        // ── Draw Enemies ──
        for e in &self.enemies {
            let sx = e.x - 20.0;
            let sy = e.y + e.bob - self.cam_y - 20.0;

            match e.enemy_type {
                EnemyType::Ghost => {
                    draw_circle(
                        sx + 20.0,
                        sy + 20.0,
                        18.0,
                        Color::new(160. / 255., 60. / 255., 220. / 255., 0.8),
                    ); // Purple
                    draw_circle(sx + 13.0, sy + 15.0, 5.0, WHITE); // Left Eye
                    draw_circle(sx + 27.0, sy + 15.0, 5.0, WHITE); // Right Eye

                    let gaze = if e.vx > 0.0 { 2.0 } else { -2.0 };
                    draw_circle(sx + 13.0 + gaze, sy + 15.0, 3.0, DARKGRAY); // Pupils
                    draw_circle(sx + 27.0 + gaze, sy + 15.0, 3.0, DARKGRAY);
                }
                EnemyType::Turret => {
                    draw_rectangle(
                        sx + 8.0,
                        sy + 10.0,
                        24.0,
                        20.0,
                        Color::new(80. / 255., 80. / 255., 100. / 255., 1.0),
                    );
                    draw_rectangle(
                        sx + 12.0,
                        sy + 5.0,
                        16.0,
                        14.0,
                        Color::new(120. / 255., 120. / 255., 140. / 255., 1.0),
                    );
                    draw_circle(sx + 20.0, sy + 12.0, 8.0, LIGHTGRAY);
                    draw_rectangle(sx + 18.0, sy, 4.0, 14.0, DARKGRAY); // Barrel

                    let warn = (128.0 + 127.0 * (self.time * 8.0).sin()) / 255.0;
                    draw_circle(sx + 28.0, sy + 5.0, 4.0, Color::new(1.0, warn, 0.0, 1.0));
                    // Blinking light
                }
                EnemyType::Chaser => {
                    draw_circle(
                        sx + 20.0,
                        sy + 20.0,
                        16.0,
                        Color::new(200. / 255., 40. / 255., 40. / 255., 0.9),
                    );
                    for i in 0..6 {
                        let a = (i as f32) / 6.0 * 6.28 + self.time * 2.0;
                        let tx = sx + 20.0 + a.cos() * 22.0;
                        let ty = sy + 20.0 + a.sin() * 22.0;
                        draw_line(
                            sx + 20.0 + a.cos() * 14.0,
                            sy + 20.0 + a.sin() * 14.0,
                            tx,
                            ty,
                            2.0,
                            RED,
                        );
                    }
                    draw_circle(sx + 14.0, sy + 16.0, 5.0, WHITE);
                    draw_circle(sx + 26.0, sy + 16.0, 5.0, WHITE);
                }
            }
        }

        // ── Draw Bullets ──
        for b in &self.bullets {
            let bx = b.x;
            let by = b.y - self.cam_y;

            if b.from_enemy {
                draw_circle(bx, by, 5.0, ORANGE);
                draw_circle(bx, by, 3.0, YELLOW);
            } else {
                draw_rectangle(bx - 2.0, by - 12.0, 4.0, 24.0, SKYBLUE);
                draw_rectangle(bx - 1.0, by - 14.0, 2.0, 6.0, WHITE);
                // Glow
                draw_rectangle(
                    bx - 4.0,
                    by - 12.0,
                    8.0,
                    24.0,
                    Color::new(0.2, 0.5, 1.0, 0.3),
                );
            }
        }

        // ── Draw Player ─────────────────────────────────────────────────────────
        //
        // Three independent fixes for the "Astronaut Echo" ghosting bug:
        //
        // 1. PIXEL-SNAPPED CAMERA: cam_y is a smooth float (lerped 12% per frame).
        //    Subtracting it from world-Y produces sub-pixel screen coords.
        //    We floor cam_y ONCE here so every world→screen conversion below is
        //    integer-aligned.  (All other draw calls use self.cam_y directly —
        //    if you want to fix those too, compute render_cam_y once at the top
        //    of draw() and use it everywhere.)
        //
        // 2. UV HALF-PIXEL INSET: When the GPU samples the texture near a frame
        //    boundary — especially with flip_x — it can land exactly on the
        //    neighbouring frame's first/last column.  Shrinking the source rect
        //    by 0.5 px on each side keeps UV sampling safely inside the frame.
        //    dest_size stays at the full frame size, so the sprite renders at
        //    1:1; the 0.5-px inset is imperceptible at sprite scale.
        //
        // 3. INVINCIBILITY BLINK: The player is now hidden on every other 4-frame
        //    window while inv_timer is active, giving clear visual feedback.
        //    Previously the sprite always drew even when invincible.

        if self.player.alive {
            // Should the sprite be visible this frame? (invincibility blink)
            let blink_visible = if self.player.invincible {
                // Toggle every 4 frames using inv_timer
                (self.player.inv_timer / 4) % 2 == 0
            } else {
                true
            };

            if blink_visible {
                // ── 1. Pixel-snapped screen coordinates ──────────────────
                let render_cam_y = self.cam_y.floor();
                let px = self.player.x.floor();
                let py = (self.player.y - render_cam_y).floor();

                // ── 2. Frame selection ────────────────────────────────────
                // Astronaut.png: 768×64 px, 1 row, 12 columns, 64×64 per frame.
                //
                //  Sheet column → content
                //       0, 1   → Idle (front-facing, breathe animation)
                //       8, 9, 10 → Walk cycle (facing RIGHT)
                //
                // anim_frame is a cycle index (0-1 idle, 0-2 walk).
                // We map it to the actual sheet column here.
                //
                // flip_x: face < 0 means moving LEFT — flips the right-facing
                // walk sprite to face left. Idle frames are front-facing so the
                // flip is symmetric and looks fine.
                const FRAME_W: f32 = 64.0;
                const FRAME_H: f32 = 64.0; // sheet is a single row

                let walking = self.player.vx.abs() > 0.2 && self.player.on_ground;

                // sheet_col: the actual X column in the sprite sheet
                let sheet_col: f32 = if walking {
                    // Walk columns: 8, 9, 10  (anim_frame cycles 0-2)
                    8.0 + self.player.anim_frame as f32
                } else {
                    // Idle columns: 0, 1  (anim_frame cycles 0-1)
                    // Airborne also uses idle (col 0) — no jumping frame in this sheet
                    self.player.anim_frame as f32
                };

                // ── 3. UV inset — eliminate frame-boundary bleed ──────────
                // Inset source rect 1 px per edge so the GPU sampler never
                // reads a texel from a neighbouring frame, even when flip_x
                // reverses UV direction.
                // dest_size == source size → zero GPU scaling → no bleed.
                const UV_INSET: f32 = 1.0;
                let src_x = sheet_col * FRAME_W + UV_INSET;
                let src_y = UV_INSET; // single row → y = 0 + inset
                let src_w = FRAME_W - UV_INSET * 2.0; // 62.0
                let src_h = FRAME_H - UV_INSET * 2.0; // 62.0

                // ── 4. Draw ───────────────────────────────────────────────
                // Sprite is 62×62 (after inset); physics box is 38×44.
                // Centre sprite over hitbox:
                //   horizontal offset: (62 - 38) / 2 = 12 px left
                //   vertical offset:    62 - 44      = 18 px up (align feet)
                draw_texture_ex(
                    &self.player_tex,
                    px - 12.0, // centre 62px sprite over 38px hitbox
                    py - 18.0, // align sprite feet to hitbox bottom
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(src_w, src_h)), // 1:1, no GPU scaling
                        source: Some(Rect::new(src_x, src_y, src_w, src_h)),
                        flip_x: self.player.face < 0, // flip for left movement
                        ..Default::default()
                    },
                );
            }
        }

        if self.player.has_shield {
            let px = self.player.x + self.player.w / 2.0;
            let py = self.player.y + self.player.h / 2.0 - self.cam_y;

            let pulse = (self.time * 5.0).sin() * 3.0;
            let radius = 32.0 + pulse;

            // Only draw visual shapes here!
            draw_circle(px, py, radius, Color::new(0.2, 0.8, 1.0, 0.3));
            draw_circle_lines(px, py, radius, 2.0, Color::new(0.5, 0.9, 1.0, 0.6));
        }

        // ── Screen Flash ──
        if self.flash_a > 0.01 {
            draw_rectangle(
                0.0,
                0.0,
                SCREEN_W,
                SCREEN_H,
                Color::new(
                    self.flash_col.r,
                    self.flash_col.g,
                    self.flash_col.b,
                    self.flash_a,
                ),
            );
        }
    }
}
