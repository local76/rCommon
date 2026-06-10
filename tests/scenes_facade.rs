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
//! effect here. The 10 scene structs are uniformly named `<SceneName>`
//! (Beams, Bounce, Bursts, Chaos, Cosmos, Disco, Flame, Glyphs, Gnats,
//! Storm); this file's tests are organized 1:1 with that naming.

use std::time::Duration;

use library::core::screensaver::Screensaver;
use library::core::TerminalCell;
use library::role::application::scenes::beams::Beams;
use library::role::application::scenes::bounce::Bounce;
use library::role::application::scenes::flame::Flame;
use library::role::application::scenes::gnats::Gnats;
use library::role::application::scenes::bursts::Bursts;
use library::role::application::scenes::cosmos::Cosmos;
use library::role::application::scenes::glyphs::Glyphs;
use library::role::application::scenes::disco::Disco;
use library::role::application::scenes::storm::Storm;
use library::role::application::scenes::chaos::Chaos;

fn render_grid(effect: &mut dyn Screensaver, cols: usize, rows: usize) -> Vec<TerminalCell> {
    // Run a few update ticks so internal state settles into a non-degenerate
    // configuration (e.g. glyphs initializes its rain drop list on the first
    // non-zero-sized update).
    for _ in 0..3 {
        effect.update(Duration::from_millis(16), cols, rows);
    }
    let mut grid = vec![TerminalCell::default(); cols * rows];
    effect.draw(&mut grid, cols, rows);
    grid
}

#[test]
fn glyphs_constructs_and_renders_at_80x24() {
    let mut g = Glyphs::new();
    let grid = render_grid(&mut g, 80, 24);
    assert_eq!(grid.len(), 80 * 24);
}

#[test]
fn glyphs_constructs_and_renders_at_60x40() {
    let mut g = Glyphs::new();
    let grid = render_grid(&mut g, 60, 40);
    assert_eq!(grid.len(), 60 * 40);
}

#[test]
fn glyphs_constructs_and_renders_at_200x60() {
    let mut g = Glyphs::new();
    let grid = render_grid(&mut g, 200, 60);
    assert_eq!(grid.len(), 200 * 60);
}

#[test]
fn glyphs_handles_resize_between_updates() {
    let mut g = Glyphs::new();
    let _ = render_grid(&mut g, 80, 24);
    // Resize larger, then back smaller — should not panic.
    let _ = render_grid(&mut g, 200, 60);
    let _ = render_grid(&mut g, 40, 20);
}

#[test]
fn glyphs_handles_zero_size_grid() {
    // A 0x0 grid is degenerate but should not panic. Useful as a
    // regression net for "index = y * cols + x" math when cols=0.
    let mut g = Glyphs::new();
    let mut grid: Vec<TerminalCell> = Vec::new();
    g.update(Duration::from_millis(16), 0, 0);
    g.draw(&mut grid, 0, 0);
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
fn bounce_constructs_and_renders_at_80x24() {
    let mut b = Bounce::new();
    let grid = render_grid(&mut b, 80, 24);
    assert_eq!(grid.len(), 80 * 24);
}

#[test]
fn bounce_constructs_and_renders_at_60x40() {
    let mut b = Bounce::new();
    let grid = render_grid(&mut b, 60, 40);
    assert_eq!(grid.len(), 60 * 40);
}

#[test]
fn bounce_constructs_and_renders_at_200x60() {
    let mut b = Bounce::new();
    let grid = render_grid(&mut b, 200, 60);
    assert_eq!(grid.len(), 200 * 60);
}

#[test]
fn bounce_handles_zero_size_grid() {
    let mut b = Bounce::new();
    let mut grid: Vec<TerminalCell> = Vec::new();
    b.update(Duration::from_millis(16), 0, 0);
    b.draw(&mut grid, 0, 0);
    assert!(grid.is_empty());
}

#[test]
fn bounce_has_scanlines_returns_true() {
    let b = Bounce::new();
    assert!(b.has_scanlines());
}

#[test]
fn flame_constructs_and_renders_at_80x24() {
    let mut f = Flame::new();
    let grid = render_grid(&mut f, 80, 24);
    assert_eq!(grid.len(), 80 * 24);
}

#[test]
fn flame_constructs_and_renders_at_60x40() {
    let mut f = Flame::new();
    let grid = render_grid(&mut f, 60, 40);
    assert_eq!(grid.len(), 60 * 40);
}

#[test]
fn flame_constructs_and_renders_at_200x60() {
    let mut f = Flame::new();
    let grid = render_grid(&mut f, 200, 60);
    assert_eq!(grid.len(), 200 * 60);
}

#[test]
fn flame_handles_zero_size_grid() {
    let mut f = Flame::new();
    let mut grid: Vec<TerminalCell> = Vec::new();
    f.update(Duration::from_millis(16), 0, 0);
    f.draw(&mut grid, 0, 0);
    assert!(grid.is_empty());
}

#[test]
fn gnats_constructs_and_renders_at_80x24() {
    let mut g = Gnats::new();
    let grid = render_grid(&mut g, 80, 24);
    assert_eq!(grid.len(), 80 * 24);
}

#[test]
fn gnats_constructs_and_renders_at_60x40() {
    let mut g = Gnats::new();
    let grid = render_grid(&mut g, 60, 40);
    assert_eq!(grid.len(), 60 * 40);
}

#[test]
fn gnats_constructs_and_renders_at_200x60() {
    let mut g = Gnats::new();
    let grid = render_grid(&mut g, 200, 60);
    assert_eq!(grid.len(), 200 * 60);
}

#[test]
fn gnats_handles_zero_size_grid() {
    let mut g = Gnats::new();
    let mut grid: Vec<TerminalCell> = Vec::new();
    g.update(Duration::from_millis(16), 0, 0);
    g.draw(&mut grid, 0, 0);
    assert!(grid.is_empty());
}

#[test]
fn bursts_constructs_and_renders_at_80x24() {
    let mut b = Bursts::new();
    let grid = render_grid(&mut b, 80, 24);
    assert_eq!(grid.len(), 80 * 24);
}

#[test]
fn bursts_constructs_and_renders_at_60x40() {
    let mut b = Bursts::new();
    let grid = render_grid(&mut b, 60, 40);
    assert_eq!(grid.len(), 60 * 40);
}

#[test]
fn bursts_constructs_and_renders_at_200x60() {
    let mut b = Bursts::new();
    let grid = render_grid(&mut b, 200, 60);
    assert_eq!(grid.len(), 200 * 60);
}

#[test]
fn bursts_handles_zero_size_grid() {
    let mut b = Bursts::new();
    let mut grid: Vec<TerminalCell> = Vec::new();
    b.update(Duration::from_millis(16), 0, 0);
    b.draw(&mut grid, 0, 0);
    assert!(grid.is_empty());
}

#[test]
fn cosmos_constructs_and_renders_at_80x24() {
    let mut c = Cosmos::new();
    let grid = render_grid(&mut c, 80, 24);
    assert_eq!(grid.len(), 80 * 24);
}

#[test]
fn cosmos_constructs_and_renders_at_60x40() {
    let mut c = Cosmos::new();
    let grid = render_grid(&mut c, 60, 40);
    assert_eq!(grid.len(), 60 * 40);
}

#[test]
fn cosmos_constructs_and_renders_at_200x60() {
    let mut c = Cosmos::new();
    let grid = render_grid(&mut c, 200, 60);
    assert_eq!(grid.len(), 200 * 60);
}

#[test]
fn cosmos_handles_zero_size_grid() {
    let mut c = Cosmos::new();
    let mut grid: Vec<TerminalCell> = Vec::new();
    c.update(Duration::from_millis(16), 0, 0);
    c.draw(&mut grid, 0, 0);
    assert!(grid.is_empty());
}

#[test]
fn disco_constructs_and_renders_at_80x24() {
    let mut d = Disco::new();
    let grid = render_grid(&mut d, 80, 24);
    assert_eq!(grid.len(), 80 * 24);
}

#[test]
fn disco_constructs_and_renders_at_60x40() {
    let mut d = Disco::new();
    let grid = render_grid(&mut d, 60, 40);
    assert_eq!(grid.len(), 60 * 40);
}

#[test]
fn disco_constructs_and_renders_at_200x60() {
    let mut d = Disco::new();
    let grid = render_grid(&mut d, 200, 60);
    assert_eq!(grid.len(), 200 * 60);
}

#[test]
fn disco_handles_zero_size_grid() {
    let mut d = Disco::new();
    let mut grid: Vec<TerminalCell> = Vec::new();
    d.update(Duration::from_millis(16), 0, 0);
    d.draw(&mut grid, 0, 0);
    assert!(grid.is_empty());
}

#[test]
fn storm_constructs_and_renders_at_80x24() {
    let mut s = Storm::new();
    let grid = render_grid(&mut s, 80, 24);
    assert_eq!(grid.len(), 80 * 24);
}

#[test]
fn storm_constructs_and_renders_at_60x40() {
    let mut s = Storm::new();
    let grid = render_grid(&mut s, 60, 40);
    assert_eq!(grid.len(), 60 * 40);
}

#[test]
fn storm_constructs_and_renders_at_200x60() {
    let mut s = Storm::new();
    let grid = render_grid(&mut s, 200, 60);
    assert_eq!(grid.len(), 200 * 60);
}

#[test]
fn storm_handles_zero_size_grid() {
    let mut s = Storm::new();
    let mut grid: Vec<TerminalCell> = Vec::new();
    s.update(Duration::from_millis(16), 0, 0);
    s.draw(&mut grid, 0, 0);
    assert!(grid.is_empty());
}

#[test]
fn chaos_constructs_and_renders_at_80x24() {
    let mut c = Chaos::new();
    let grid = render_grid(&mut c, 80, 24);
    assert_eq!(grid.len(), 80 * 24);
}

#[test]
fn chaos_constructs_and_renders_at_60x40() {
    let mut c = Chaos::new();
    let grid = render_grid(&mut c, 60, 40);
    assert_eq!(grid.len(), 60 * 40);
}

#[test]
fn chaos_constructs_and_renders_at_200x60() {
    let mut c = Chaos::new();
    let grid = render_grid(&mut c, 200, 60);
    assert_eq!(grid.len(), 200 * 60);
}

#[test]
fn chaos_handles_zero_size_grid() {
    let mut c = Chaos::new();
    let mut grid: Vec<TerminalCell> = Vec::new();
    c.update(Duration::from_millis(16), 0, 0);
    c.draw(&mut grid, 0, 0);
    assert!(grid.is_empty());
}
