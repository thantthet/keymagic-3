//! Rule matching subsystem

mod matcher;
mod pattern;
mod context;
mod capture;

pub use matcher::RuleMatcher;
pub use pattern::{Pattern, PatternElement};
pub use context::MatchContext;
pub use capture::CaptureManager;