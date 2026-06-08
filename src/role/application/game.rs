//! Mini-game state machines for TUI/screensaver integration.
//!
//! **Taxonomy Classification**: System Role (Purpose - Application Software).

// Canonical LcgRng is defined in core.rs so it cannot be accidentally
// changed by TUI-only or platform-specific code.
pub use crate::core::LcgRng;

// LcgRng implementation lives in core.rs (single source of truth across all
// Interface / Lifecycle / Platform / Role combinations).

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObstacleJumpState {
    Playing,
    Dead,
    Respawning,
}

pub struct ObstacleJumpGame {
    pub player_y: f32,
    pub player_vy: f32,
    pub obstacle_x: f32,
    pub score: usize,
    pub best: usize,
    pub speed: f32,
    pub state: ObstacleJumpState,
    pub timer: f32,
    pub auto_skill: f32,
    pub width: f32,
    pub active: bool,
    rng: LcgRng,
}

impl ObstacleJumpGame {
    pub fn new(width: f32) -> Self {
        Self {
            player_y: 0.0,
            player_vy: 0.0,
            obstacle_x: width,
            score: 0,
            best: 0,
            speed: 250.0,
            state: ObstacleJumpState::Playing,
            timer: 0.0,
            auto_skill: 0.70,
            width,
            active: true,
            rng: LcgRng::new(1337),
        }
    }

    pub fn tick(&mut self, delta: f32, jump_input: bool, auto_mode: bool) {
        if !self.active {
            return;
        }
        self.timer += delta;
        match self.state {
            ObstacleJumpState::Playing => {
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
                    if self.obstacle_x < trigger_dist && self.obstacle_x > 10.0 && self.player_y <= 0.0
                        && self.rng.next_bool(self.auto_skill) {
                            self.player_vy = 12.0 + self.auto_skill * 2.0;
                            self.auto_skill = (self.auto_skill + 0.003).min(0.98);
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
                    self.state = ObstacleJumpState::Dead;
                    self.timer = 0.0;
                    self.speed = 0.0;
                    self.auto_skill = (self.auto_skill * 0.92).max(0.65);
                }
            }
            ObstacleJumpState::Dead => {
                if self.timer >= 1.5 {
                    self.state = ObstacleJumpState::Respawning;
                    self.timer = 0.0;
                }
            }
            ObstacleJumpState::Respawning => {
                if self.timer >= 1.0 {
                    if self.score > self.best {
                        self.best = self.score;
                    }
                    self.score = 0;
                    self.speed = 250.0;
                    self.obstacle_x = self.width;
                    self.player_y = 0.0;
                    self.player_vy = 0.0;
                    self.state = ObstacleJumpState::Playing;
                    self.timer = 0.0;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_tick() {
        let mut game = ObstacleJumpGame::new(100.0);
        assert!(game.active);
        game.tick(0.1, false, false);
        assert!(game.timer > 0.0);

        game.active = false;
        let prev_timer = game.timer;
        game.tick(0.1, false, false);
        assert_eq!(game.timer, prev_timer);
    }
}
