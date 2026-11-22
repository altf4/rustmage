//! Official November 2026 creature vocabulary reaches typed choices and live subtype checks.
use crate::support::*;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};

#[test]
fn star_trek_creature_types_are_legal_choices_and_changeling_types() {
    for name in ["Borg", "Officer", "Klingon", "Q", "Jem'Hadar", "Vau-N'Akat"] {
        assert!(CreatureType::from_name(name).is_some(), "missing {name}");
        assert!(is_creature_type(name), "not a creature subtype: {name}");
    }
    assert!(CreatureType::from_name("Time Lord").is_some());
    assert!(!is_creature_type("Planet"));
    assert!(!is_creature_type("Spacecraft"));

    let mut game = scenario(Player::P0);
    let changeling = put(
        &mut game,
        Player::P0,
        "Three Tree Mascot",
        Zone::Battlefield,
    );
    assert!(game.has_subtype(changeling, "Borg"));
    assert!(game.has_subtype(changeling, "Klingon"));
    assert!(!game.has_subtype(changeling, "Planet"));
}
