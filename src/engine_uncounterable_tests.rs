use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    assert!(card.is_supported(), "{name}");
    spawn(g, p, card, zone)
}
fn replay(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let rev = g.revision();
    let p = g.actor().unwrap();
    let mut fork = g.clone();
    act(&mut fork, a.clone());
    act(g, a.clone());
    let expected = g.snapshot();
    assert_eq!(expected, fork.snapshot());
    g.restore(&saved).unwrap();
    assert_eq!(g.apply_index(p, rev, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), expected);
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for player in &mut g.state.players {
        player.mana = [20; 6];
    }
    g
}
fn bear_spell(g: &mut Game, p: Player) -> CardId {
    let id = named(g, p, "Gigantic Big Bear", Zone::Hand);
    cast(g, id, None);
    id
}
#[test]
fn self_spell_protection_keeps_counter_targets_legal_and_resolves_complete_bear() {
    for p in [Player::P0, Player::P1] {
        for counter in ["Counterspell", "Essence Scatter"] {
            let mut g = setup(p);
            let id = bear_spell(&mut g, p);
            act(&mut g, Action::Pass);
            let c = named(&mut g, p.other(), counter, Zone::Hand);
            cast(&mut g, c, Some(Target::Spell(id)));
            pass_pair(&mut g);
            assert_eq!(g.object_view(c).zone, Zone::Graveyard);
            assert_eq!(g.object_view(id).zone, Zone::Stack);
            pass_pair(&mut g);
            let b = g.object_view(id);
            assert_eq!((b.zone, b.power, b.toughness), (Zone::Battlefield, 10, 7));
            assert!(g.keywords(id).contains(Keywords::HEXPROOF));
            assert!(g.keywords(id).contains(Keywords::HASTE));
            // Hexproof only affects opposing permanent targets after resolution.
            let removal = named(&mut g, p.other(), "Unsummon", Zone::Hand);
            if g.actor() != Some(p.other()) {
                act(&mut g, Action::Pass);
            }
            assert!(!g.legal_actions().iter().any(|a|matches!(a,Action::Cast{card,target:Some(Target::Card(t)),..}if *card==removal&&*t==id)));
        }
    }
}
#[test]
fn failed_counters_preserve_other_instructions_and_pay_or_decline_branches() {
    for p in [Player::P0, Player::P1] {
        for counter in ["No More Lies", "It'll Quench Ya!", "Don't Make a Sound"] {
            for paid in [false, true] {
                let mut g = setup(p);
                let id = bear_spell(&mut g, p);
                act(&mut g, Action::Pass);
                let c = named(&mut g, p.other(), counter, Zone::Hand);
                cast(&mut g, c, Some(Target::Spell(id)));
                pass_pair(&mut g);
                assert_eq!(g.observe(p).decision, "resolution_mana");
                let a = if paid {
                    g.legal_actions()
                        .into_iter()
                        .find(|a| matches!(a, Action::PayResolutionMana { .. }))
                        .unwrap()
                } else {
                    Action::DeclineResolutionMana
                };
                replay(&mut g, a);
                if counter == "Don't Make a Sound" && paid {
                    assert!(g.observe(p.other()).library_choice.is_some());
                    assert!(g.observe(p).library_choice.is_none());
                    while let Some(choice) = g.observe(p.other()).library_choice {
                        replay(
                            &mut g,
                            Action::ArrangeLibrary {
                                card: choice.remaining[0].id,
                                destination: LibraryDestination::Top,
                            },
                        );
                    }
                }
                assert_eq!(g.object_view(id).zone, Zone::Stack);
                assert_eq!(g.object_view(c).zone, Zone::Graveyard);
                pass_pair(&mut g);
                assert_eq!(g.object_view(id).zone, Zone::Battlefield);
            }
        }
    }
}
#[test]
fn eject_survives_all_ward_payment_types_and_returns_target_then_draws() {
    for p in [Player::P0, Player::P1] {
        for ward in [
            "Hulking Raptor",
            "Spectral Snatcher",
            "Vein Ripper",
            "Sire of Seven Deaths",
        ] {
            for paid in [false, true] {
                let mut g = setup(p);
                let resource = named(
                    &mut g,
                    p,
                    "Grizzly Bears",
                    if ward == "Vein Ripper" {
                        Zone::Battlefield
                    } else {
                        Zone::Hand
                    },
                );
                let target = named(&mut g, p.other(), ward, Zone::Battlefield);
                let id = named(&mut g, p, "Eject", Zone::Hand);
                let hand_before = g
                    .state
                    .objects
                    .iter()
                    .filter(|o| o.owner == p && o.zone == Zone::Hand)
                    .count();
                cast(&mut g, id, Some(Target::Card(target)));
                assert_eq!(g.state.stack.len(), 2);
                pass_pair(&mut g);
                let action = match ward {
                    "Hulking Raptor" => {
                        if paid {
                            g.legal_actions()
                                .into_iter()
                                .find(|a| matches!(a, Action::PayResolutionMana { .. }))
                                .unwrap()
                        } else {
                            Action::DeclineResolutionMana
                        }
                    }
                    "Sire of Seven Deaths" => Action::ChooseOptional { accept: paid },
                    _ if !paid => Action::DeclineEffectPayment,
                    "Vein Ripper" => Action::Sacrifice {
                        object: g.object_ref(resource),
                    },
                    _ => Action::Discard { card: resource },
                };
                replay(&mut g, action);
                assert_eq!(g.object_view(id).zone, Zone::Stack);
                // Paying sacrifice queues Vein Ripper's death trigger; finish it first.
                for _ in 0..20 {
                    if g.object_view(id).zone != Zone::Stack {
                        break;
                    }
                    let a = g
                        .legal_actions()
                        .into_iter()
                        .find(|a| {
                            matches!(a, Action::PutTrigger { .. } | Action::ChooseTarget { .. })
                        })
                        .unwrap_or(Action::Pass);
                    replay(&mut g, a);
                }
                assert_eq!(g.object_view(id).zone, Zone::Graveyard);
                assert_eq!(g.object_view(target).zone, Zone::Hand);
                let hands = g
                    .state
                    .objects
                    .iter()
                    .filter(|o| o.owner == p && o.zone == Zone::Hand)
                    .count();
                assert_eq!(
                    hands,
                    hand_before - usize::from(paid && ward == "Spectral Snatcher")
                );
                assert_eq!(
                    g.state.players[p.index()].life,
                    20 - if paid && ward == "Sire of Seven Deaths" {
                        7
                    } else {
                        0
                    } - if paid && ward == "Vein Ripper" { 2 } else { 0 }
                );
            }
        }
    }
}
#[test]
fn protection_does_not_prevent_target_invalid_resolution_or_protect_abilities() {
    for name in ["Eject", "Long Goodbye"] {
        let mut g = setup(Player::P0);
        let target = named(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
        let id = named(&mut g, Player::P0, name, Zone::Hand);
        cast(&mut g, id, Some(Target::Card(target)));
        let hand = g.state.players[0].cards_drawn_this_turn;
        g.move_object(target, Zone::Hand);
        pass_pair(&mut g);
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        assert_eq!(g.state.players[0].cards_drawn_this_turn, hand);
    }
    let mut g = setup(Player::P0);
    let id = bear_spell(&mut g, Player::P0);
    let old = g.state.stack[0].id;
    assert!(!g.counter_stack_item(old, true));
    assert_eq!(g.object_view(id).zone, Zone::Stack);
    // Synthetic ability with the same source definition isolates spell-only scope.
    g.state.stack[0].kind = StackKind::ActivatedAbility { ability: 0 };
    assert!(g.counter_stack_item(old, true));
    assert!(g.state.stack.is_empty());
    assert_eq!(g.object_view(id).zone, Zone::Stack);
    assert!(!g.counter_stack_item(old, false));
}
#[test]
fn long_goodbye_checks_union_mana_value_and_destroy_protection() {
    for (name, legal) in [
        ("Grizzly Bears", true),
        ("Sunset Saboteur", true),
        ("Hill Giant", false),
        ("Professor Dellian Fel", false),
        ("Ajani, Caller of the Pride", true),
        ("Gigantic Big Bear", false),
        ("Forest", false),
        ("Vampiric Rites", false),
    ] {
        let mut g = setup(Player::P0);
        let t = named(&mut g, Player::P0, name, Zone::Battlefield);
        let definition = g.object_view(t).card;
        if definition.is_planeswalker() {
            g.state.objects[t as usize].counters.loyalty = definition.starting_loyalty();
        }
        let id = named(&mut g, Player::P0, "Long Goodbye", Zone::Hand);
        assert_eq!(g.legal_actions().iter().any(|a|matches!(a,Action::Cast{card,target:Some(Target::Card(tid)),..}if *card==id&&*tid==t)),legal,"{name}");
        if legal {
            cast(&mut g, id, Some(Target::Card(t)));
            assert_eq!(g.object_view(t).zone, Zone::Battlefield);
            pass_pair(&mut g);
            assert_eq!(g.object_view(t).zone, Zone::Graveyard);
        }
    }
}
#[test]
fn out_cold_zero_one_two_targets_and_partial_or_total_invalidity() {
    for count in 0..=2 {
        for invalid in 0..=count {
            let mut g = setup(Player::P0);
            let targets: Vec<_> = (0..count)
                .map(|_| named(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield))
                .collect();
            let id = named(&mut g, Player::P0, "Out Cold", Zone::Hand);
            replay(&mut g, Action::AnnounceCast { card: id });
            for &t in &targets {
                replay(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(t),
                    },
                );
            }
            if g.legal_actions().contains(&Action::FinishTargets) {
                replay(&mut g, Action::FinishTargets);
            }
            let pay = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap();
            replay(&mut g, pay);
            for &t in targets.iter().take(invalid) {
                g.move_object(t, Zone::Hand);
            }
            pass_pair(&mut g);
            assert_eq!(g.object_view(id).zone, Zone::Graveyard);
            for &t in targets.iter().skip(invalid) {
                assert!(g.object_view(t).tapped);
                assert_eq!(g.object_view(t).counters.stun, 1);
            }
            assert_eq!(
                g.state
                    .objects
                    .iter()
                    .filter(|o| o.zone == Zone::Battlefield && o.card.has_subtype("Clue"))
                    .count(),
                usize::from(count == 0 || invalid < count)
            );
        }
    }
}
#[test]
fn negate_and_refute_target_protected_instants_and_refute_still_loots() {
    for p in [Player::P0, Player::P1] {
        for counter in ["Negate", "Refute"] {
            let mut g = setup(p);
            let t = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
            let id = named(&mut g, p, "Eject", Zone::Hand);
            cast(&mut g, id, Some(Target::Card(t)));
            act(&mut g, Action::Pass);
            let c = named(&mut g, p.other(), counter, Zone::Hand);
            let drawn = g.state.players[p.other().index()].cards_drawn_this_turn;
            cast(&mut g, c, Some(Target::Spell(id)));
            pass_pair(&mut g);
            if counter == "Refute" {
                assert_eq!(
                    g.state.players[p.other().index()].cards_drawn_this_turn,
                    drawn + 1
                );
                assert_eq!(g.observe(p.other()).decision, "effect_discard");
                let a = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::Discard { .. }))
                    .unwrap();
                replay(&mut g, a);
            }
            assert_eq!(g.object_view(c).zone, Zone::Graveyard);
            assert_eq!(g.object_view(id).zone, Zone::Stack);
            pass_pair(&mut g);
            assert_eq!(g.object_view(t).zone, Zone::Hand);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        }
    }
}
#[test]
fn long_goodbye_accepts_tokens_but_destroy_still_respects_indestructible() {
    for indestructible in [false, true] {
        let mut g = setup(Player::P0);
        let token = Card::catalogue()
            .find(|c| c.token_info().is_some_and(|t| t.subtypes == "Golem"))
            .unwrap();
        let t = spawn(&mut g, Player::P1, token, Zone::Battlefield);
        if indestructible {
            // Synthetic keyword grant isolates destroy's prohibition from countering.
            g.state.objects[t as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
        }
        let id = named(&mut g, Player::P0, "Long Goodbye", Zone::Hand);
        cast(&mut g, id, Some(Target::Card(t)));
        pass_pair(&mut g);
        assert_eq!(
            g.object_view(t).zone,
            if indestructible {
                Zone::Battlefield
            } else {
                Zone::Outside
            }
        );
    }
}

#[test]
fn investigated_clue_has_its_full_paid_sacrifice_draw_ability() {
    let mut g = setup(Player::P0);
    let id = named(&mut g, Player::P0, "Out Cold", Zone::Hand);
    replay(&mut g, Action::AnnounceCast { card: id });
    if g.legal_actions().contains(&Action::FinishTargets) {
        replay(&mut g, Action::FinishTargets);
    }
    let pay = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    replay(&mut g, pay);
    pass_pair(&mut g);
    let clue = g
        .state
        .objects
        .iter()
        .position(|o| o.zone == Zone::Battlefield && o.card.has_subtype("Clue"))
        .unwrap() as CardId;
    let before = g.state.players[0].cards_drawn_this_turn;
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::Activate { card, .. } if *card == clue))
        .unwrap();
    replay(&mut g, action);
    assert_eq!(g.object_view(clue).zone, Zone::Outside);
    pass_pair(&mut g);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, before + 1);
}
