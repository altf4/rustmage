//! Hartford transforming fronts cannot use the ordinary modal back-cast route.
use super::big_helpers::checked;
use super::hartford_craft_core::{begin_craft, craft_clause, craft_mana, craft_source};
use super::*;

#[test]
fn transforming_hartford_fronts_reject_direct_back_cast_transactionally() {
    for player in [Player::P0, Player::P1] {
        for name in [
            "The Enigma Jewel",
            "Spring-Loaded Sawblades",
            "Braided Net",
            "Esper Origins",
            "Aang, Swift Savior",
        ] {
            let mut g = scenario(player);
            let card = put(&mut g, player, name, Zone::Hand);
            assert!(g.legal_actions().contains(&Action::AnnounceCast { card }));
            assert!(!g.legal_actions().contains(&Action::AnnounceBack { card }));
            reject(&mut g, Action::AnnounceBack { card });
            assert_eq!(g.object_view(card).zone, Zone::Hand);
            assert!(!g.object_view(card).face.unwrap().back);
        }
    }
}

#[test]
fn hartford_modal_back_remains_an_ordinary_paid_spell() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let card = put(&mut g, player, "Jennifer Walters", Zone::Hand);
        checked(&mut g, Action::AnnounceBack { card });
        pay(&mut g);
        settle(&mut g);
        let object = g.object_view(card);
        assert_eq!(object.zone, Zone::Battlefield);
        assert_eq!(object.card.name(), "The Sensational She-Hulk");
        assert!(object.face.unwrap().back);
    }
}

#[test]
fn hartford_craft_still_pays_costs_and_returns_the_transformed_source() {
    for player in [Player::P0, Player::P1] {
        for (front, back, ability, count) in [
            ("The Enigma Jewel", "Locus of Enlightenment", 1, 4),
            ("Spring-Loaded Sawblades", "Bladewheel Chariot", 0, 1),
            ("Braided Net", "Braided Quipu", 1, 1),
        ] {
            let mut g = scenario(player);
            let source = put(&mut g, player, front, Zone::Battlefield);
            let before = g.object_ref(source);
            let materials: Vec<_> = (0..count)
                .map(|_| put(&mut g, player, "Short Bow", Zone::Graveyard))
                .collect();
            begin_craft(&mut g, source, ability);
            craft_clause(&mut g, CastCostClause::Additional);
            for &material in &materials {
                let object = g.object_ref(material);
                checked(&mut g, Action::ChooseAdditionalCost { object });
            }
            if count == 4 {
                checked(&mut g, Action::FinishCraftMaterials);
            }
            craft_source(&mut g);
            assert_eq!(g.object_view(source).zone, Zone::Exile);
            craft_mana(&mut g);
            settle(&mut g);
            let object = g.object_view(source);
            assert_eq!(object.zone, Zone::Battlefield);
            assert_eq!(object.card.name(), back);
            assert!(object.face.unwrap().back);
            assert_ne!(object.reference, before);
            for material in materials {
                assert_eq!(g.object_view(material).zone, Zone::Exile);
            }
        }
    }
}
