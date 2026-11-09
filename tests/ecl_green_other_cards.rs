//! Paid Lorwyn Eclipsed card scenarios in the current workspace layout.
#[path = "support/ecl_scenarios.rs"]
mod ecl_scenarios;
mod support;
use rustmage as _;

#[path = "ecl_cards/green_colorless.rs"]
mod green_colorless;

#[path = "ecl_cards/color_funding.rs"]
mod color_funding;

#[path = "ecl_cards/funding_performance.rs"]
mod funding_performance;

#[path = "ecl_cards/twinflame.rs"]
mod twinflame;

#[path = "ecl_cards/twinflame_funding.rs"]
mod twinflame_funding;

#[path = "ecl_cards/lluwen.rs"]
mod lluwen;

#[path = "ecl_cards/morcant.rs"]
mod morcant;

#[path = "ecl_cards/raiding.rs"]
mod raiding;
