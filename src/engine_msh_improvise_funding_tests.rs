//! Small exhaustive funding domains; no mutation, imaginary mana or helper reuse.
use super::*;
use crate::engine::assurance::big_helpers::{named, setup};
const IRON: &str = "Ironheart, Clever Champion";
fn goal(p: Player, required: Mana, generic: u32) -> FundingGoal {
    FundingGoal {
        actor: p,
        required,
        generic,
        source: None,
        reserve: None,
    }
}
#[test]
fn msh_improvise_funding_small_exhaustive_generic_and_colored_oracle() {
    for p in [Player::P0, Player::P1] {
        for count in 0..=6 {
            for blue in 0..=2 {
                for colorless in 0..=6 {
                    let mut g = setup(p);
                    for _ in 0..count {
                        named(&mut g, p, "Hawkeye's Bow", Zone::Battlefield);
                    }
                    let id = named(&mut g, p, IRON, Zone::Hand);
                    g.state.players[p.index()].mana = [0, blue, 0, 0, 0, colorless];
                    let before = g.snapshot();
                    let purpose =
                        g.spell_mana_purpose_for_card(p, id, Card::from_name(IRON).unwrap());
                    let actual = g.can_fund_contributions_for(
                        goal(p, [0, 1, 0, 0, 0, 0], 4),
                        purpose,
                        false,
                    );
                    assert_eq!(
                        actual,
                        blue >= 1 && blue + colorless + count >= 5,
                        "{p:?} {count} {blue} {colorless}"
                    );
                    assert!(!g.can_fund_contributions_for(
                        goal(p, [0, 0, 0, 0, 0, colorless + 1], 0),
                        purpose,
                        false
                    ));
                    assert_eq!(g.snapshot(), before);
                }
            }
        }
    }
}
#[test]
fn msh_improvise_funding_union_deduplicates_artifact_creature() {
    for p in [Player::P0, Player::P1] {
        for artifact_creatures in 0..=2 {
            let mut g = setup(p);
            let iron = named(&mut g, p, IRON, Zone::Battlefield);
            let id = named(&mut g, p, "Unexpected Assistance", Zone::Hand);
            // A single real Ironheart is legendary. Extra same-definition objects are
            // not created; ordinary production animation makes an artifact creature.
            for _ in 0..artifact_creatures {
                let id = named(&mut g, p, "Hawkeye's Bow", Zone::Battlefield);
                let item = g.cast_preview(p, id, Zone::Battlefield, g.object_ref(id), 0, [0; 6]);
                g.animate_source(
                    &item,
                    crate::cards::AnimationSpec {
                        artifact: true,
                        creature_types: crate::cards::AnimationCreatureTypes::Unchanged,
                        base: Some((2, 2)),
                        keywords: Keywords::NONE,
                        duration: crate::cards::AnimationDuration::Permanent,
                    },
                );
            }
            g.state.players[p.index()].mana = [0; 6];
            let purpose = g.spell_mana_purpose_for_card(
                p,
                id,
                Card::from_name("Unexpected Assistance").unwrap(),
            );
            let before = g.snapshot();
            let need = goal(p, [0, 2, 0, 0, 0, 0], 0);
            assert!(!g.can_fund_contributions_for(need, purpose, true));
            let need = goal(p, [0, 1, 0, 0, 0, 0], 0);
            assert!(g.can_fund_contributions_for(need, purpose, true));
            assert!(g.can_fund_contributions_for(
                goal(p, [0, 1, 0, 0, 0, 0], artifact_creatures),
                purpose,
                true
            ));
            assert!(!g.can_fund_contributions_for(
                goal(p, [0, 1, 0, 0, 0, 0], artifact_creatures + 1),
                purpose,
                true
            ));
            assert_eq!(g.snapshot(), before);
            assert!(improvise::eligible(&g, p, g.object_ref(iron)));
        }
    }
}
#[test]
fn msh_improvise_funding_current_grant_recomputed_after_virtual_departure_and_restore() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let iron = named(&mut g, p, IRON, Zone::Battlefield);
        let fuel = named(&mut g, p, "Hawkeye's Bow", Zone::Battlefield);
        let spell = named(&mut g, p, "Divination", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        let purpose =
            g.spell_mana_purpose_for_card(p, spell, Card::from_name("Divination").unwrap());
        let need = goal(p, [0, 1, 0, 0, 0, 0], 2);
        let before = g.snapshot();
        let mut world = FundingState::from_game(&g, &need);
        world.set_purpose(purpose);
        assert!(world.improvise_available());
        assert!(world.reachable(&need, &mut HashSet::new()));
        let mut visits = 0;
        world.cost_departures(&[g.object_ref(iron)], Zone::Graveyard, &mut |after| {
            visits += 1;
            assert!(!after.improvise_available());
            assert!(!after.reachable(&need, &mut HashSet::new()));
            false
        });
        assert_eq!(visits, 1);
        assert!(world.improvise_available());
        assert!(improvise::eligible(&world, p, g.object_ref(fuel)));
        assert_eq!(g.snapshot(), before);
    }
}
