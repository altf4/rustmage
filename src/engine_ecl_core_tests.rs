//! Paid blight choices preserve exact objects and owned continuations.
use super::ability_timeline_tests::{checked, named, settle, setup};
use super::*;

fn hybrid_cast(g: &mut Game, card: CardId) {
    checked(g, Action::AnnounceCast { card });
    if let Some(a) = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::ChooseSpellCost { .. }))
    {
        checked(g, a);
    }
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    checked(g, a);
}

fn until_payment(g: &mut Game) {
    for _ in 0..30 {
        if matches!(g.state.pending, Pending::ResolutionMana { .. }) {
            return;
        }
        let a = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            g.legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PutTrigger { .. } | Action::FinishTargets))
                .unwrap()
        };
        checked(g, a);
    }
    panic!("missing mana choice");
}

#[test]
fn ecl_chaos_spewer_paid_or_blight_both_hybrid_routes_and_seats() {
    for p in [Player::P0, Player::P1] {
        for black in [false, true] {
            for pay in [false, true] {
                let mut g = setup(p);
                let own = named(&mut g, p, "Hill Giant", Zone::Battlefield);
                let foe = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
                let spewer = named(&mut g, p, "Chaos Spewer", Zone::Hand);
                let mut mana = [0; 6];
                mana[if black { 2 } else { 3 }] = 1;
                mana[5] = 2;
                g.state.players[p.index()].mana = mana;
                hybrid_cast(&mut g, spewer);
                assert_eq!(g.state.players[p.index()].mana, [0; 6]);
                until_payment(&mut g);
                // The trigger is independent of its source, including a later incarnation.
                g.move_object(spewer, Zone::Exile);

                if pay {
                    g.state.players[p.index()].mana[5] = 2;
                    let a = g
                        .legal_actions()
                        .into_iter()
                        .find(|a| matches!(a, Action::PayResolutionMana { .. }))
                        .unwrap();
                    checked(&mut g, a);
                } else {
                    checked(&mut g, Action::DeclineResolutionMana);
                    assert!(
                        matches!(g.state.pending, Pending::EffectChoice { player } if player == p)
                    );
                    let invalid = Action::Blight {
                        object: g.object_ref(foe),
                    };
                    let before = g.snapshot();
                    let rev = g.revision();
                    assert_eq!(g.apply(p, invalid), Err(Error::IllegalAction));
                    assert_eq!(g.snapshot(), before);
                    assert_eq!(g.revision(), rev);
                    let object = g.object_ref(own);
                    checked(&mut g, Action::Blight { object });
                }
                settle(&mut g);
                assert_eq!(
                    g.counter_count(own, CounterKind::MinusOne),
                    if pay { 0 } else { 2 }
                );
                assert_eq!(g.counter_count(foe, CounterKind::MinusOne), 0);
                assert_eq!(g.stats(own).toughness, if pay { 3 } else { 1 });
            }
        }
    }
}

#[test]
fn ecl_mandatory_blight_without_creatures_finishes_and_source_can_blight_itself() {
    for p in [Player::P0, Player::P1] {
        for departed in [false, true] {
            let mut g = setup(p);
            let source = named(&mut g, p, "Chaos Spewer", Zone::Hand);
            hybrid_cast(&mut g, source);
            until_payment(&mut g);
            if departed {
                g.move_object(source, Zone::Exile);
            }
            checked(&mut g, Action::DeclineResolutionMana);
            if !departed {
                let object = g.object_ref(source);
                checked(&mut g, Action::Blight { object });
            }
            settle(&mut g);
            assert_eq!(
                g.counter_count(source, CounterKind::MinusOne),
                if departed { 0 } else { 2 }
            );
        }
    }
}

#[test]
fn ecl_duelist_other_merfolk_lord_and_end_step_untap() {
    use super::ability_timeline_tests::cast;
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = named(&mut g, p, "Graveshifter", Zone::Battlefield);
        let foe = named(&mut g, p.other(), "Graveshifter", Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let duelist = named(&mut g, p, "Deepchannel Duelist", Zone::Hand);
        g.state.players[p.index()].mana = [1, 1, 0, 0, 0, 0];
        cast(&mut g, duelist, None, &[]);
        settle(&mut g);
        assert_eq!(g.stats(own).power, 3);
        assert_eq!(g.stats(foe).power, 2);
        assert_eq!(g.stats(bear).power, 2);
        assert_eq!(g.stats(duelist).power, 2);
        g.state.objects[own as usize].tapped = true;
        g.state.step = Step::Main2;
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert_eq!(g.state.step, Step::End);
        g.prepare_triggers();
        if let Some(a) = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PutTrigger { .. }))
        {
            checked(&mut g, a);
        }
        assert!(!g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(foe)
        }));
        assert!(!g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(bear)
        }));
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        // An already queued trigger keeps working after the lord departs.
        g.move_object(duelist, Zone::Graveyard);
        settle(&mut g);
        assert!(!g.state.objects[own as usize].tapped);
        assert_eq!(g.stats(own).power, 2);
    }
}

#[test]
fn ecl_tome_skimmer_optional_life_only_for_own_spell_on_opponents_turn() {
    use super::ability_timeline_tests::cast;
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            let mut g = setup(p);
            let skimmer = named(&mut g, p, "Voracious Tome-Skimmer", Zone::Hand);
            g.state.players[p.index()].mana = [0, 3, 0, 0, 0, 0];
            hybrid_cast(&mut g, skimmer);
            settle(&mut g);
            assert!(g.keywords(skimmer).contains(Keywords::FLYING));
            assert_eq!(g.stats(skimmer).toughness, 3);
            g.state.active = p.other();
            g.state.priority = p;
            let spell = named(&mut g, p, "Blossoming Defense", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
            let hand = g.hand_count(p);
            let life = g.state.players[p.index()].life;
            cast(&mut g, spell, None, &[Target::Card(skimmer)]);
            for _ in 0..30 {
                if matches!(g.state.pending, Pending::EffectOptional { .. }) {
                    break;
                }
                let a = if g.state.pending == Pending::Priority {
                    Action::Pass
                } else {
                    g.legal_actions()
                        .into_iter()
                        .find(|a| matches!(a, Action::PutTrigger { .. }))
                        .unwrap()
                };
                checked(&mut g, a);
            }
            assert!(matches!(g.state.pending, Pending::EffectOptional { .. }));
            g.move_object(skimmer, Zone::Exile);
            checked(&mut g, Action::ChooseOptional { accept: pay });
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, life - i128::from(pay));
            assert_eq!(g.hand_count(p), hand - 1 + usize::from(pay));
        }
    }
}

fn wait_for_library_choice(g: &mut Game) {
    for _ in 0..30 {
        if g.observe(g.actor().unwrap()).library_choice.is_some() {
            return;
        }
        let a = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            g.legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PutTrigger { .. }))
                .unwrap()
        };
        checked(g, a);
    }
    panic!("missing private library choice");
}

#[test]
fn ecl_wary_farmer_frozen_entry_history_and_private_surveil() {
    use super::ability_timeline_tests::cast;
    for p in [Player::P0, Player::P1] {
        for another in [false, true] {
            let mut g = setup(p);
            let farmer = named(&mut g, p, "Wary Farmer", Zone::Hand);
            hybrid_cast(&mut g, farmer);
            settle(&mut g);
            assert_eq!(
                g.creature_entries_this_turn(p, g.object_ref(farmer), true),
                0
            );
            if another {
                let other = named(&mut g, p, "Bear Cub", Zone::Hand);
                g.state.players[p.index()].mana = [1, 0, 0, 0, 1, 0];
                cast(&mut g, other, None, &[]);
                settle(&mut g);
                g.move_object(other, Zone::Graveyard);
                assert_eq!(
                    g.creature_entries_this_turn(p, g.object_ref(farmer), true),
                    1
                );
            }
            let snapshot = g.snapshot();
            let log = g.observe(p).creature_entry_history;
            assert_eq!(log, g.observe(p.other()).creature_entry_history);
            g.state.step = Step::Main2;
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            if another {
                wait_for_library_choice(&mut g);
                assert!(g.observe(p.other()).library_choice.is_none());
                let card = g.observe(p).library_choice.unwrap().remaining[0].id;
                checked(
                    &mut g,
                    Action::ArrangeLibrary {
                        card,
                        destination: LibraryDestination::Graveyard,
                    },
                );
                assert_eq!(g.object_view(card).zone, Zone::Graveyard);
            } else {
                assert!(g.state.pending_triggers.is_empty());
                assert!(g.state.stack.is_empty());
                assert!(g.state.resolving.is_none());
            }
            g.restore(&snapshot).unwrap();
            assert_eq!(g.observe(p).creature_entry_history, log);
            g.begin_turn();
            assert!(g.state.creature_entry_history.is_none());
        }
    }
}

#[test]
fn ecl_entry_history_tracks_reincarnation_and_simultaneous_tokens() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let farmer = named(&mut g, p, "Wary Farmer", Zone::Hand);
        hybrid_cast(&mut g, farmer);
        settle(&mut g);
        let old = g.object_ref(farmer);
        g.move_object(farmer, Zone::Exile);
        g.move_object(farmer, Zone::Battlefield);
        assert_ne!(g.object_ref(farmer), old);
        assert_eq!(
            g.creature_entries_this_turn(p, g.object_ref(farmer), true),
            1
        );
        let parent = g.clone();
        let token = Card::from_name("@token/soldier_1_1_white").unwrap();
        g.create_tokens(p, token, 2, false);
        assert_eq!(
            g.creature_entries_this_turn(p, g.object_ref(farmer), true),
            3
        );
        assert_eq!(
            parent.creature_entries_this_turn(p, parent.object_ref(farmer), true),
            1
        );
        assert!(
            !g.state
                .creature_entry_history
                .as_ref()
                .unwrap()
                .ptr_eq(parent.state.creature_entry_history.as_ref().unwrap())
        );
    }
}

#[test]
fn ecl_flaring_cinder_optional_discard_cost_and_cast_threshold() {
    use super::ability_timeline_tests::cast;
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            let mut g = setup(p);
            let fodder = named(&mut g, p, "Island", Zone::Hand);
            let cinder = named(&mut g, p, "Flaring Cinder", Zone::Hand);
            hybrid_cast(&mut g, cinder);
            for _ in 0..30 {
                if matches!(g.state.pending, Pending::EffectChoice { .. }) {
                    break;
                }
                let a = if g.state.pending == Pending::Priority {
                    Action::Pass
                } else {
                    g.legal_actions()
                        .into_iter()
                        .find(|a| matches!(a, Action::PutTrigger { .. }))
                        .unwrap()
                };
                checked(&mut g, a);
            }
            assert!(g.observe(p).effect_choice.is_some());
            assert!(g.observe(p.other()).effect_choice.is_none());
            if pay {
                checked(&mut g, Action::Discard { card: fodder });
            } else {
                checked(&mut g, Action::DeclineEffectPayment);
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(fodder).zone,
                if pay { Zone::Graveyard } else { Zone::Hand }
            );
            let small = named(&mut g, p, "Bear Cub", Zone::Hand);
            g.state.players[p.index()].mana = [30; 6];
            cast(&mut g, small, None, &[]);
            assert!(g.state.pending_triggers.is_empty());
            settle(&mut g);
            let large = named(&mut g, p, "Hill Giant", Zone::Hand);
            cast(&mut g, large, None, &[]);
            assert_eq!(g.state.pending_triggers.len(), 1);
            settle(&mut g);
        }
    }
}
