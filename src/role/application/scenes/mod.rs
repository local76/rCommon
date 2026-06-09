//! The 10 r* screensaver effects, consolidated.
//!
//! **Taxonomy Classification**: System Role (Purpose - Application Software).
//!
//! In library 4.1 these effects are migrating here from `trance-scenes/` (now
//! `screensavers/`). 4.1.0 ships `matrix` as the canonical reference impl; the
//! other 9 land as 4.1.x patch releases. Each effect is a `Screensaver` impl
//! that renders a TerminalCell grid. The `screensavers/` workspace will eventually
//! hold only thin shim binaries that wire these effects into the GDI
//! screen-saver runtime (also moving to library in a later 4.x release).
//!
//! All 10 submodules are gated behind `feature = "scenes"`. library's default
//! features include `scenes`. r* apps that don't need the effects can opt
//! out with `default-features = false`.
//!
//! To embed an effect in an r* app:
//!
//! ```no_run
//! use library::role::application::scenes::matrix::Matrix;
//! use library::core::screensaver::Screensaver;
//!
//! let mut effect = Matrix::new();
//! effect.update(std::time::Duration::from_millis(16), 80, 24);
//! let mut grid = vec![library::core::TerminalCell::default(); 80 * 24];
//! effect.draw(&mut grid, 80, 24);
//! ```

pub mod beams;
pub mod bhop;
pub mod fire;
pub mod fireflies;
pub mod fireworks;
pub mod life;
pub mod matrix;
pub mod party;
pub mod pour;
pub mod unstable;
