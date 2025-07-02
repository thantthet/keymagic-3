//! Rule processing subsystem

mod rule_application;
mod recursive;
mod action_generator;

pub use rule_application::RuleProcessor;
pub use recursive::{RecursiveProcessor, should_stop_recursion};
pub use action_generator::ActionGenerator;