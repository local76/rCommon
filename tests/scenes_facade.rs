//! Smoke tests for the `scenes` façade surface.
//!
//! Each migrated effect gets a smoke test that constructs it, runs a few
//! `update` ticks, then renders into a `TerminalCell` grid at several
//! canonical sizes (80x24, 60x40, 200x60). The assertions are minimal:
//! the effect must construct without panicking, must accept update ticks
//! at different sizes, and must populate the grid without panicking on
//! index math. Visual correctness is verified by the human-eye test
//! pattern (run a release build, look at it), not by these tests.
//!
//! As more effects migrate into `library::scenes::*`, add one test per
//! effect here. The matrix test exists from 4.1.0; the other 9 land
//! alongside their migration in 4.1.x patch releases.

use std::time::Duration;

use library::core::screensaver::Screensaver;
use library::core::TerminalCell;
use library::role::application::scenes::beams::Beams;
use library::role::application::scenes::bhop::BhopDashboard;
use library::role::application::scenes::fire::FireEffect;
use library::role::application::scenes::fireflies::Fireflies;
use library::role::application::scenes::fireworks::Fireworks;
use library::role::application::scenes::life::LifeEffect;
use library::role::application::scenes::matrix::{Matrix, Matrix as MatrixEffect};
use library::role::application::scenes::party::Party;
use library::role::application::scenes::pour::Pour;
use library::role::application::scenes::unstable::Unstable;

fn render_grid(effect: &mut dyn Screensaver, cols: usize, rows: usize) -> Vec<TerminalCell> {
    // Run a few update ticks so internal state settles into a non-degenerate
    // configuration (e.g. matrix initializes its rain drop list on the first
    // non-zero-sized update).
    for _ in 0..3 {
        effect.update(Duration::from_millis(16), cols, rows);
    }
    let mut grid = vec![TerminalCell::default(); cols * rows];
    effect.draw(&mut grid, cols, rows);
    grid
}

#[test]
fn matrix_constructs_and_renders_at_80x24() {
    let mut m = Matrix::new();
    let grid = render_grid(&mut m, 80, 24);
    assert_eq!(grid.len(), 80 * 24);
}

#[test]
fn matrix_constructs_and_renders_at_60x40() {
    let mut m = Matrix::new();
    let grid = render_grid(&mut m, 60, 40);
    assert_eq!(grid.len(), 60 * 40);
}

#[test]
fn matrix_constructs_and_renders_at_200x60() {
    let mut m = Matrix::new();
    let grid = render_grid(&mut m, 200, 60);
    assert_eq!(grid.len(), 200 * 60);
}

#[test]
fn matrix_handles_resize_between_updates() {
    let mut m = MatrixEffect::new();
    let _ = render_grid(&mut m, 80, 24);
    // Resize larger, then back smaller — should not panic.
    let _ = render_grid(&mut m, 200, 60);
    let _ = render_grid(&mut m, 40, 20);
}

#[test]
fn matrix_handles_zero_size_grid() {
    // A 0x0 grid is degenerate but should not panic. Useful as a
    // regression net for "index = y * cols + x" math when cols=0.
    let mut m = Matrix::new();
    let mut grid: Vec<TerminalCell> = Vec::new();
    m.update(Duration::from_millis(16), 0, 0);
    m.draw(&mut grid, 0, 0);
    assert!(grid.is_empty());
}

#[test]
fn beams_constructs_and_renders_at_80x24() {
    let mut b = Beams::new();
    let grid = render_grid(&mut b, 80, 24);
    assert_eq!(grid.len(), 80 * 24);
}

#[test]
fn beams_constructs_and_renders_at_60x40() {
    let mut b = Beams::new();
    let grid = render_grid(&mut b, 60, 40);
    assert_eq!(grid.len(), 60 * 40);
}

#[test]
fn beams_constructs_and_renders_at_200x60() {
    let mut b = Beams::new();
    let grid = render_grid(&mut b, 200, 60);
    assert_eq!(grid.len(), 200 * 60);
}

#[test]
fn beams_handles_zero_size_grid() {
    let mut b = Beams::new();
    let mut grid: Vec<TerminalCell> = Vec::new();
    b.update(Duration::from_millis(16), 0, 0);
    b.draw(&mut grid, 0, 0);
    assert!(grid.is_empty());
}

#[test]
fn bhop_constructs_and_renders_at_80x24() {
    let mut b = BhopDashboard::new();
    let grid = render_grid(&mut b, 80, 24);
    assert_eq!(grid.len(), 80 * 24);
}

#[test]
fn bhop_constructs_and_renders_at_60x40() {
    let mut b = BhopDashboard::new();
    let grid = render_grid(&mut b, 60, 40);
    assert_eq!(grid.len(), 60 * 40);
}

#[test]
fn bhop_constructs_and_renders_at_200x60() {
    let mut b = BhopDashboard::new();
    let grid = render_grid(&mut b, 200, 60);
    assert_eq!(grid.len(), 200 * 60);
}

#[test]
fn bhop_handles_zero_size_grid() {
    let mut b = BhopDashboard::new();
    let mut grid: Vec<TerminalCell> = Vec::new();
    b.update(Duration::from_millis(16), 0, 0);
    b.draw(&mut grid, 0, 0);
    assert!(grid.is_empty());
}

#[test]
fn bhop_has_scanlines_returns_true() {
    let b = BhopDashboard::new();
    assert!(b.has_scanlines());
}

#[test]
fn fire_constructs_and_renders_at_80x24() {
    let mut f = FireEffect::new();
    let grid = render_grid(&mut f, 80, 24);
    assert_eq!(grid.len(), 80 * 24);
}

#[test]
fn fire_constructs_and_renders_at_60x40() {
    let mut f = FireEffect::new();
    let grid = render_grid(&mut f, 60, 40);
    assert_eq!(grid.len(), 60 * 40);
}

#[test]
fn fire_constructs_and_renders_at_200x60() {
    let mut f = FireEffect::new();
    let grid = render_grid(&mut f, 200, 60);
    assert_eq!(grid.len(), 200 * 60);
}

#[test]
fn fire_handles_zero_size_grid() {
    let mut f = FireEffect::new();
    let mut grid: Vec<TerminalCell> = Vec::new();
    f.update(Duration::from_millis(16), 0, 0);
    f.draw(&mut grid, 0, 0);
    assert!(grid.is_empty());
}

#[test]
fn fireflies_constructs_and_renders_at_80x24() {
    let mut f = Fireflies::new();
    let grid = render_grid(&mut f, 80, 24);
    assert_eq!(grid.len(), 80 * 24);
}

#[test]
fn fireflies_constructs_and_renders_at_60x40() {
    let mut f = Fireflies::new();
    let grid = render_grid(&mut f, 60, 40);
    assert_eq!(grid.len(), 60 * 40);
}

#[test]
fn fireflies_constructs_and_renders_at_200x60() {
    let mut f = Fireflies::new();
    let grid = render_grid(&mut f, 200, 60);
    assert_eq!(grid.len(), 200 * 60);
}

#[test]
fn fireflies_handles_zero_size_grid() {
    let mut f = Fireflies::new();
    let mut grid: Vec<TerminalCell> = Vec::new();
    f.update(Duration::from_millis(16), 0, 0);
    f.draw(&mut grid, 0, 0);
    assert!(grid.is_empty());
}

#[test]
fn fireworks_constructs_and_renders_at_80x24() {
    let mut f = Fireworks::new();
    let grid = render_grid(&mut f, 80, 24);
    assert_eq!(grid.len(), 80 * 24);
}

#[test]
fn fireworks_constructs_and_renders_at_60x40() {
    let mut f = Fireworks::new();
    let grid = render_grid(&mut f, 60, 40);
    assert_eq!(grid.len(), 60 * 40);
}

#[test]
fn fireworks_constructs_and_renders_at_200x60() {
    let mut f = Fireworks::new();
    let grid = render_grid(&mut f, 200, 60);
    assert_eq!(grid.len(), 200 * 60);
}

#[test]
fn fireworks_handles_zero_size_grid() {
    let mut f = Fireworks::new();
    let mut grid: Vec<TerminalCell> = Vec::new();
    f.update(Duration::from_millis(16), 0, 0);
    f.draw(&mut grid, 0, 0);
    assert!(grid.is_empty());
}

#[test]
fn life_constructs_and_renders_at_80x24() {
    let mut l = LifeEffect::new();
    let grid = render_grid(&mut l, 80, 24);
    assert_eq!(grid.len(), 80 * 24);
}

#[test]
fn life_constructs_and_renders_at_60x40() {
    let mut l = LifeEffect::new();
    let grid = render_grid(&mut l, 60, 40);
    assert_eq!(grid.len(), 60 * 40);
}

#[test]
fn life_constructs_and_renders_at_200x60() {
    let mut l = LifeEffect::new();
    let grid = render_grid(&mut l, 200, 60);
    assert_eq!(grid.len(), 200 * 60);
}

#[test]
fn life_handles_zero_size_grid() {
    let mut l = LifeEffect::new();
    let mut grid: Vec<TerminalCell> = Vec::new();
    l.update(Duration::from_millis(16), 0, 0);
    l.draw(&mut grid, 0, 0);
    assert!(grid.is_empty());
}

#[test]
fn party_constructs_and_renders_at_80x24() {
    let mut p = Party::new();
    let grid = render_grid(&mut p, 80, 24);
    assert_eq!(grid.len(), 80 * 24);
}

#[test]
fn party_constructs_and_renders_at_60x40() {
    let mut p = Party::new();
    let grid = render_grid(&mut p, 60, 40);
    assert_eq!(grid.len(), 60 * 40);
}

#[test]
fn party_constructs_and_renders_at_200x60() {
    let mut p = Party::new();
    let grid = render_grid(&mut p, 200, 60);
    assert_eq!(grid.len(), 200 * 60);
}

#[test]
fn party_handles_zero_size_grid() {
    let mut p = Party::new();
    let mut grid: Vec<TerminalCell> = Vec::new();
    p.update(Duration::from_millis(16), 0, 0);
    p.draw(&mut grid, 0, 0);
    assert!(grid.is_empty());
}

#[test]
fn pour_constructs_and_renders_at_80x24() {
    let mut p = Pour::new();
    let grid = render_grid(&mut p, 80, 24);
    assert_eq!(grid.len(), 80 * 24);
}

#[test]
fn pour_constructs_and_renders_at_60x40() {
    let mut p = Pour::new();
    let grid = render_grid(&mut p, 60, 40);
    assert_eq!(grid.len(), 60 * 40);
}

#[test]
fn pour_constructs_and_renders_at_200x60() {
    let mut p = Pour::new();
    let grid = render_grid(&mut p, 200, 60);
    assert_eq!(grid.len(), 200 * 60);
}

#[test]
fn pour_handles_zero_size_grid() {
    let mut p = Pour::new();
    let mut grid: Vec<TerminalCell> = Vec::new();
    p.update(Duration::from_millis(16), 0, 0);
    p.draw(&mut grid, 0, 0);
    assert!(grid.is_empty());
}

#[test]
fn unstable_constructs_and_renders_at_80x24() {
    let mut u = Unstable::new();
    let grid = render_grid(&mut u, 80, 24);
    assert_eq!(grid.len(), 80 * 24);
}

#[test]
fn unstable_constructs_and_renders_at_60x40() {
    let mut u = Unstable::new();
    let grid = render_grid(&mut u, 60, 40);
    assert_eq!(grid.len(), 60 * 40);
}

#[test]
fn unstable_constructs_and_renders_at_200x60() {
    let mut u = Unstable::new();
    let grid = render_grid(&mut u, 200, 60);
    assert_eq!(grid.len(), 200 * 60);
}

#[test]
fn unstable_handles_zero_size_grid() {
    let mut u = Unstable::new();
    let mut grid: Vec<TerminalCell> = Vec::new();
    u.update(Duration::from_millis(16), 0, 0);
    u.draw(&mut grid, 0, 0);
    assert!(grid.is_empty());
}
