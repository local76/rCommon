//! The 10 screensaver effects, consolidated.
//!
//! **Taxonomy Classification**: System Role (Purpose - Application Software).
//!
//! In library 4.1 these effects migrated here from `trance-scenes/` (now
//! `screensavers/`). 4.1.0 ships `glyphs` (formerly `matrix`) as the canonical reference impl; the
//! other 9 land as 4.1.x patch releases. Each effect is a `Screensaver` impl
//! that renders a TerminalCell grid. The `screensavers/` workspace holds only
//! thin shim binaries that wire these effects into the GDI screen-saver
//! runtime.
//!
//! All 10 submodules are gated behind `feature = "scenes"`. library's default
//! features include `scenes`. Apps that don't need the effects can opt out
//! with `default-features = false`.
//!
//! To embed an effect in an app:
//!
//! ```no_run
//! use library::role::application::scenes::glyphs::Glyphs;
//! use library::core::screensaver::Screensaver;
//!
//! let mut effect = Glyphs::new();
//! effect.update(std::time::Duration::from_millis(16), 80, 24);
//! let mut grid = vec![library::core::TerminalCell::default(); 80 * 24];
//! effect.draw(&mut grid, 80, 24);
//! ```

pub mod beams;
pub mod bounce;
pub mod bursts;
pub mod chaos;
pub mod cosmos;
pub mod disco;
pub mod flame;
pub mod gnats;
pub mod glyphs;
pub mod storm;
