pub struct LcgRng(u64);

impl LcgRng {
    pub fn new(seed: u64) -> Self {
        Self(seed | 1)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0
    }

    pub fn next_f32(&mut self) -> f32 {
        let val = self.next_u64() >> 11;
        (val as f64 / 9007199254740992.0) as f32
    }

    pub fn next_bool(&mut self, prob: f32) -> bool {
        self.next_f32() < prob
    }

    pub fn next_range(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BhopState {
    Playing,
    Dead,
    Respawning,
}

pub struct BhopGame {
    pub player_y: f32,
    pub player_vy: f32,
    pub obstacle_x: f32,
    pub score: usize,
    pub best: usize,
    pub speed: f32,
    pub state: BhopState,
    pub timer: f32,
    pub auto_skill: f32,
    pub width: f32,
    rng: LcgRng,
}

impl BhopGame {
    pub fn new(width: f32) -> Self {
        Self {
            player_y: 0.0,
            player_vy: 0.0,
            obstacle_x: width,
            score: 0,
            best: 0,
            speed: 250.0,
            state: BhopState::Playing,
            timer: 0.0,
            auto_skill: 0.70,
            width,
            rng: LcgRng::new(1337),
        }
    }

    pub fn tick(&mut self, delta: f32, jump_input: bool, auto_mode: bool) {
        self.timer += delta;
        match self.state {
            BhopState::Playing => {
                // Obstacle moves left
                self.obstacle_x -= self.speed * delta;
                if self.obstacle_x < 0.0 {
                    self.obstacle_x = self.width;
                    self.score += 1;
                    self.speed = (self.speed + 10.0).min(600.0);
                }

                // AI Auto Jump
                if auto_mode {
                    let trigger_dist = self.speed * 0.15;
                    if self.obstacle_x < trigger_dist && self.obstacle_x > 10.0 && self.player_y <= 0.0 {
                        if self.rng.next_bool(self.auto_skill) {
                            self.player_vy = 12.0 + self.auto_skill * 2.0;
                            self.auto_skill = (self.auto_skill + 0.003).min(0.98);
                        }
                    }
                }

                // Manual Jump
                if jump_input && self.player_y <= 0.0 {
                    self.player_vy = 14.0;
                    self.score += 2;
                    self.speed = (self.speed + 5.0).min(600.0);
                    self.auto_skill = (self.auto_skill + 0.002).min(0.98);
                }

                // Physics (Gravity)
                if self.player_y > 0.0 || self.player_vy > 0.0 {
                    self.player_y += self.player_vy * delta * 4.0;
                    self.player_vy -= 26.0 * delta * 4.0;
                    if self.player_y <= 0.0 {
                        self.player_y = 0.0;
                        self.player_vy = 0.0;
                    }
                }

                // Collision Detection
                let player_x = self.width * 0.15;
                let collision_threshold_x = 25.0;
                if (self.obstacle_x - player_x).abs() < collision_threshold_x && self.player_y < 30.0 {
                    self.state = BhopState::Dead;
                    self.timer = 0.0;
                    self.speed = 0.0;
                    self.auto_skill = (self.auto_skill * 0.92).max(0.65);
                }
            }
            BhopState::Dead => {
                if self.timer >= 1.5 {
                    self.state = BhopState::Respawning;
                    self.timer = 0.0;
                }
            }
            BhopState::Respawning => {
                if self.timer >= 1.0 {
                    if self.score > self.best {
                        self.best = self.score;
                    }
                    self.score = 0;
                    self.speed = 250.0;
                    self.obstacle_x = self.width;
                    self.player_y = 0.0;
                    self.player_vy = 0.0;
                    self.state = BhopState::Playing;
                    self.timer = 0.0;
                }
            }
        }
    }
}
