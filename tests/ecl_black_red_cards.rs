//! Paid Lorwyn Eclipsed card scenarios in the current workspace layout.
#[path = "support/ecl_scenarios.rs"]
mod ecl_scenarios;
mod support;
use rustmage as _;

#[path = "ecl_cards/black_red.rs"]
mod black_red;

#[path = "ecl_cards/inherited_br.rs"]
mod inherited_br;

#[path = "ecl_cards/bre.rs"]
mod bre;

#[path = "ecl_cards/dream_harvest.rs"]
mod dream_harvest;

#[path = "ecl_cards/goliath.rs"]
mod goliath;

#[path = "ecl_cards/maralen.rs"]
mod maralen;

#[path = "ecl_cards/shadow_urchin.rs"]
mod shadow_urchin;
