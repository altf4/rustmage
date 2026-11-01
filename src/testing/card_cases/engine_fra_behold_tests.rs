//! Focused printed outcomes; shared engine invariants live in core tests.
//! Exact, non-targeting witnesses shared by behold consumers.
use super::*;

#[test]
fn countersculpt_both_cost_routes_counter_and_empower() {
    {
        let p = Player::P0;
        for route in [0, 1] {
            for hand_witness in [false, true] {
                let mut g = scenario(p.other());
                let target = put(&mut g, p.other(), "Bear Cub", Zone::Hand);
                let spell = put(&mut g, p, "Countersculpt", Zone::Hand);
                let witness = put(
                    &mut g,
                    p,
                    "Jace, Reality Sculptor",
                    if hand_witness {
                        Zone::Hand
                    } else {
                        Zone::Battlefield
                    },
                );
                if !hand_witness {
                    g.add_counter(witness, CounterKind::Loyalty, 2);
                }
                cast(&mut g, target, None);
                g.state.priority = p;
                g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 1];
                act(&mut g, Action::AnnounceCast { card: spell });
                act(&mut g, Action::ChooseSpellCost { cost: route });
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Spell(target),
                    },
                );
                if route == 0 {
                    let reference = g.object_ref(witness);
                    act(&mut g, Action::ChooseAdditionalCost { object: reference });
                }
                pay(&mut g);
                if route == 0 && hand_witness {
                    assert!(
                        g.observe(p.other())
                            .known_hand
                            .contains(&g.object_ref(witness))
                    );
                }
                assert_eq!(
                    g.object_view(witness).zone,
                    if hand_witness {
                        Zone::Hand
                    } else {
                        Zone::Battlefield
                    }
                );

                settle(&mut g);
                assert_eq!(g.object_view(target).zone, Zone::Graveyard);
                assert!(g.state.objects.iter().enumerate().any(|(id, o)| o.zone
                    == Zone::Battlefield
                    && o.controller == p
                    && o.is_token()
                    && g.has_subtype(id as CardId, "Jace")
                    && g.counter_count(id as CardId, CounterKind::Loyalty) == 1));
            }
        }
    }
}

#[test]
fn sanctum_tap_replacements_compose_in_chosen_order() {
    {
        let p = Player::P0;
        for intrinsic_first in [false, true] {
            let mut g = scenario(p);
            let spellunking = put(&mut g, p, "Spelunking", Zone::Battlefield);
            let witness = put(&mut g, p, "Jace, Reality Sculptor", Zone::Hand);
            let sanctum = put(&mut g, p, "Theorist's Sanctum", Zone::Hand);
            act(&mut g, Action::PlayLand { card: sanctum });
            assert!(matches!(g.state.pending, Pending::EntryReplacement { .. }));
            let source = if intrinsic_first {
                g.object_ref(sanctum)
            } else {
                g.object_ref(spellunking)
            };
            act(
                &mut g,
                Action::ChooseEntryReplacement {
                    source,
                    intrinsic: intrinsic_first,
                },
            );
            let witness_ref = g.object_ref(witness);
            if intrinsic_first {
                assert!(matches!(g.state.pending, Pending::EntryBehold { .. }));
                act(
                    &mut g,
                    Action::ChooseEntryBehold {
                        object: Some(witness_ref),
                    },
                );
            }
            // Declining the intrinsic replacement is overridden only when the
            // untapped replacement is applied after it.
            if !intrinsic_first {
                assert!(matches!(g.state.pending, Pending::EntryBehold { .. }));
                act(&mut g, Action::ChooseEntryBehold { object: None });
                assert!(g.object_view(sanctum).tapped);
            } else {
                assert!(!g.object_view(sanctum).tapped);
            }
        }
    }
}

#[test]
fn forge_borrows_sanctum_printed_and_land_type_mana_abilities() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let sanctum = put(&mut g, p.other(), "Theorist's Sanctum", Zone::Battlefield);
        let forge = put(&mut g, p, "Territory Forge", Zone::Hand);
        cast(&mut g, forge, None);
        let target = Action::ChooseTarget {
            target: Target::Card(sanctum),
        };
        let mut chose = false;
        for _ in 0..64 {
            g.prepare_triggers();
            let actions = g.legal_actions();
            if !chose && actions.contains(&target) {
                act(&mut g, target.clone());
                chose = true;
            } else if actions.contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            } else if chose
                && g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
                && g.state.pending_triggers.is_empty()
            {
                break;
            } else if let Some(action) = actions
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
            {
                act(&mut g, action);
            } else {
                panic!("unexpected Forge state {:?}", g.state.pending);
            }
        }
        assert!(chose);
        assert_eq!(g.object_view(sanctum).zone, Zone::Exile);
        let abilities = g.borrowed_ability_views();
        assert!(
            abilities
                .iter()
                .any(|a| a.source.card == forge && a.ability == 0)
        );
        assert!(
            abilities
                .iter()
                .any(|a| a.source.card == forge && a.ability == u16::MAX - 2)
        );
    }
}

#[test]
fn dual_land_type_mana_and_lone_mountain_borrowing() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let land = put(&mut g, p, "Hedge Maze", Zone::Battlefield);
        let actions = g.legal_actions();
        assert!(actions.contains(&Action::TapMana { card: land }));
        assert!(actions.iter().any(|a| matches!(a, Action::TapGrantedMana { card, production, .. } if *card == land && *production == [0, 0, 0, 0, 1, 0])));
        assert!(
            !actions
                .iter()
                .any(|a| matches!(a, Action::ActivateMana { card, .. } if *card == land))
        );
    }
}

#[test]
fn chosen_forest_produces_green_for_real_cast_with_or_without_ability_removal() {
    {
        let p = Player::P0;
        for removal in [false, true] {
            let mut g = scenario(p);
            let land = put(&mut g, p, "Multiversal Passage", Zone::Hand);
            let creature = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            if removal {
                let aura = put(&mut g, p.other(), "Fresh Start", Zone::Battlefield);
                g.state.objects[aura as usize].attached_to = Some(g.object_ref(creature));
            }
            act(&mut g, Action::PlayLand { card: land });
            act(
                &mut g,
                Action::ChooseEntryValue {
                    value: EntryChoiceValue::BasicLandType(crate::cards::BasicLandType::Forest),
                },
            );
            act(&mut g, Action::ChooseEntryLife { pay: true });
            g.state.players[p.index()].mana = [0; 6];
            assert!(g.can_fund(FundingGoal {
                actor: p,
                required: [0, 0, 0, 0, 1, 0],
                generic: 0,
                reserve: None,
                source: None
            }));
            let growth = put(&mut g, p, "Giant Growth", Zone::Hand);
            act(&mut g, Action::AnnounceCast { card: growth });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(creature),
                },
            );
            let tap = g.legal_actions().into_iter().find(|a| matches!(a, Action::TapGrantedMana { card, production, .. } if *card == land && *production == [0, 0, 0, 0, 1, 0])).expect("chosen Forest mana");
            act(&mut g, tap);
            pay(&mut g);
            assert_eq!(g.object_view(growth).zone, Zone::Stack);
        }
    }
}
