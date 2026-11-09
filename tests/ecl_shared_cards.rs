//! Paid Lorwyn Eclipsed card scenarios in the current workspace layout.
#[path = "support/ecl_scenarios.rs"]
mod ecl_scenarios;
mod support;
use rustmage as _;

#[path = "ecl_cards/commands.rs"]
mod commands;

#[path = "ecl_cards/core.rs"]
mod core;

#[path = "ecl_cards/doran.rs"]
mod doran;

#[path = "ecl_cards/evoke.rs"]
mod evoke;

#[path = "ecl_cards/figure.rs"]
mod figure;

#[path = "ecl_cards/kirol.rs"]
mod kirol;

#[path = "ecl_cards/navigator.rs"]
mod navigator;

#[path = "ecl_cards/oko.rs"]
mod oko;

#[path = "ecl_cards/shared_type_completion.rs"]
mod shared_type_completion;

#[path = "ecl_cards/students.rs"]
mod students;

#[path = "ecl_cards/transform.rs"]
mod transform;
