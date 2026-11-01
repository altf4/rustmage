//! Focused printed outcomes; shared engine invariants live in core tests.
//! Paid whole-card craft, transformed face, and vehicle actions.
use super::big_helpers::{cast, checked, settle};
use super::hartford_craft_core::{begin_craft, craft_clause, craft_mana, craft_source};
use super::*;

#[test]
fn sawblades_flash_entry_and_required_tapped_opponent_target() {
    {
        let player = Player::P0;
        let mut g = scenario(player);
        g.state.active = player.other();
        g.state.step = Step::End;
        let card = put(&mut g, player, "Spring-Loaded Sawblades", Zone::Hand);
        let victim = put(&mut g, player.other(), "Hill Giant", Zone::Battlefield);
        g.state.objects[victim as usize].tapped = true;
        cast(&mut g, card, None, &[]);
        settle(&mut g);
        assert_eq!(g.object_view(card).zone, Zone::Battlefield);
        assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
    }
}

#[test]
fn sawblades_crafts_from_either_zone_and_returns_under_owner() {
    for owner in [Player::P0, Player::P1] {
        for graveyard in [false, true] {
            let controller = owner.other();
            let mut g = scenario(controller);
            let source = put(&mut g, owner, "Spring-Loaded Sawblades", Zone::Battlefield);
            g.state.objects[source as usize].controller = controller;
            let material = put(
                &mut g,
                controller,
                "Short Bow",
                if graveyard {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                },
            );
            begin_craft(&mut g, source, 0);
            craft_clause(&mut g, CastCostClause::Additional);
            let source_ref = g.object_ref(source);
            reject(&mut g, Action::ChooseAdditionalCost { object: source_ref });
            let material_ref = g.object_ref(material);
            checked(
                &mut g,
                Action::ChooseAdditionalCost {
                    object: material_ref,
                },
            );
            craft_source(&mut g);
            craft_mana(&mut g);
            assert_eq!(g.object_view(source).zone, Zone::Exile);
            assert_eq!(g.object_view(material).zone, Zone::Exile);
            settle(&mut g);
            assert_eq!(g.object_view(source).zone, Zone::Battlefield);
            assert_eq!(g.object_view(source).controller, owner);
            assert_eq!(g.object_view(source).card.name(), "Bladewheel Chariot");
            assert!(!g.is_creature(source));
        }
    }
}
