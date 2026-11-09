//! Paid Lorwyn Eclipsed card scenarios in the current workspace layout.
#[path = "support/ecl_scenarios.rs"]
mod ecl_scenarios;
mod support;
use rustmage as _;

#[path = "ecl_cards/copy_entry.rs"]
mod copy_entry;

#[path = "ecl_cards/white_blue.rs"]
mod white_blue;

#[path = "ecl_cards/inherited.rs"]
mod inherited;
