//! Paid casts, exact copy identities, retarget choices and owned continuation proofs.
use super::ability_timeline_tests::{checked, setup};
use super::tests::spawn;
use super::*;

fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    spawn(g, p, Card::from_name(name).unwrap(), zone)
}
fn announce(g: &mut Game, card: CardId, targets: &[Target]) {
    checked(g, Action::AnnounceCast { card });
    for _ in 0..20 {
        if let Some(range) = g.mana_lot_spend_range() {
            checked(
                g,
                Action::ChooseManaLotSpend {
                    amount: range.maximum,
                },
            );
            continue;
        }
        if g.state.announcing.is_none() {
            return;
        }
        let actions = g.legal_actions();
        let selected = g.state.announcing.as_ref().unwrap().item.targets.len();
        let action = targets
            .get(selected)
            .and_then(|target| {
                actions
                    .iter()
                    .find(|a| **a == Action::ChooseTarget { target: *target })
            })
            .or_else(|| {
                actions.iter().find(|a| {
                    matches!(
                        a,
                        Action::ChooseSpellCost { .. }
                            | Action::PayCosts { .. }
                            | Action::Discard { .. }
                            | Action::PayResolutionMana { .. }
                            | Action::FinishTargets
                    )
                })
            })
            .unwrap_or_else(|| panic!("cast choices {actions:?}"))
            .clone();
        checked(g, action);
    }
    panic!("unpaid cast");
}
fn progress(g: &mut Game, stop_retarget: bool) {
    for _ in 0..150 {
        if stop_retarget && matches!(g.state.pending, Pending::Retarget { .. }) {
            return;
        }
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        let actions = g.legal_actions();
        let action = actions
            .iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishStackTargets
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::Discard { .. }
                        | Action::PayResolutionMana { .. }
                )
            })
            .unwrap_or_else(|| panic!("settle {:?} {actions:?}", g.state.pending))
            .clone();
        checked(g, action);
    }
    panic!("unsettled");
}
fn teach(g: &mut Game, p: Player) {
    let id = named(g, p, "Teach by Example", Zone::Hand);
    announce(g, id, &[]);
    progress(g, false);
}
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    assert!(g.apply(g.actor().unwrap(), action).is_err());
    assert_eq!(g.snapshot(), before);
}
#[test]
fn teach_copies_next_qualifying_cast_once_and_retargets_without_casting_again() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        teach(&mut g, p);
        let body = named(&mut g, p, "Bear Cub", Zone::Hand);
        announce(&mut g, body, &[]);
        progress(&mut g, false);
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        announce(&mut g, shock, &[Target::Player(p.other())]);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 3);
        progress(&mut g, true);
        assert!(matches!(g.state.pending, Pending::Retarget { .. }));
        reject(
            &mut g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Spell(shock),
            },
        );
        checked(
            &mut g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Player(p),
            },
        );
        checked(&mut g, Action::FinishStackTargets);
        assert_eq!(g.state.stack.iter().filter(|s| s.is_copy).count(), 1);
        let copy = g.state.stack.iter().find(|s| s.is_copy).unwrap().clone();
        assert_ne!(copy.card, shock);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 3);
        assert!(g.legal_actions().iter().any(|a| matches!(a, Action::Pass)));
        progress(&mut g, false);
        assert_eq!(g.state.players[p.index()].life, 18);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert_eq!(g.state.objects[copy.card as usize].zone, Zone::Outside);
        assert_eq!(g.state.objects[shock as usize].zone, Zone::Graveyard);
        let second = named(&mut g, p, "Shock", Zone::Hand);
        announce(&mut g, second, &[Target::Player(p.other())]);
        progress(&mut g, false);
        assert_eq!(g.state.players[p.other().index()].life, 16);
        assert!(g.state.stack_copy_data.is_none());
    }
}
#[test]
fn storm_copies_fixed_prior_count_and_survives_source_and_original_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let first = named(&mut g, p, "Shock", Zone::Hand);
        announce(&mut g, first, &[Target::Player(p.other())]);
        progress(&mut g, false);
        let storm = named(&mut g, p, "Thousand-Year Storm", Zone::Battlefield);
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        announce(&mut g, shock, &[Target::Player(p.other())]);
        assert_eq!(
            g.state
                .stack
                .iter()
                .filter(|s| s.copy_event.is_some())
                .count(),
            1
        );
        // Countering the original does not invalidate the trigger's frozen spell.
        let id = g
            .state
            .stack
            .iter()
            .find(|s| s.card == shock && s.kind == StackKind::Spell)
            .unwrap()
            .id;
        assert!(g.counter_stack_item(id, false));
        g.move_object(storm, Zone::Graveyard);
        progress(&mut g, true);
        checked(&mut g, Action::FinishStackTargets);
        assert_eq!(g.state.stack.iter().filter(|s| s.is_copy).count(), 1);
        progress(&mut g, false);
        assert_eq!(g.state.players[p.other().index()].life, 16);
        assert_eq!(g.state.players[p.index()].instant_sorcery_cast_this_turn, 2);
    }
}
#[test]
fn copied_spell_has_its_own_counter_target_and_fizzles_independently() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        teach(&mut g, p);
        let a = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let b = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = named(&mut g, p, "Shock", Zone::Hand);
        announce(&mut g, spell, &[Target::Card(a)]);
        progress(&mut g, true);
        checked(
            &mut g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Card(b),
            },
        );
        checked(&mut g, Action::FinishStackTargets);
        let copy = g.state.stack.iter().find(|s| s.is_copy).unwrap().clone();
        let counter = named(&mut g, p, "Negate", Zone::Hand);
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: counter })
        );
        announce(&mut g, counter, &[Target::Stack(copy.id)]);
        progress(&mut g, false);
        assert_eq!(g.state.objects[b as usize].zone, Zone::Battlefield);
        assert_eq!(g.state.objects[a as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[copy.card as usize].zone, Zone::Outside);
    }
}
#[test]
fn bolt_bend_discounts_and_changes_an_ability_target_with_original_controller_legality() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Serra Angel", Zone::Battlefield);
        let protected = named(&mut g, p, "Sphinx of the Final Word", Zone::Battlefield);
        let archer = named(&mut g, p.other(), "Fanatical Firebrand", Zone::Battlefield);
        g.state.priority = p.other();
        let action = g.legal_actions().into_iter().find(|a| matches!(a, Action::Activate { card, target: Some(Target::Player(t)), .. } if *card == archer && *t == p)).unwrap();
        checked(&mut g, action);
        let ability = g.state.stack.last().unwrap().id;
        g.state.priority = p;
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
        let bend = named(&mut g, p, "Bolt Bend", Zone::Hand);
        announce(&mut g, bend, &[Target::Stack(ability)]);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        progress(&mut g, true);
        assert!(!g.legal_actions().contains(&Action::FinishStackTargets));
        reject(
            &mut g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Card(protected),
            },
        );
        reject(
            &mut g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Player(p),
            },
        );
        checked(
            &mut g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Player(p.other()),
            },
        );
        progress(&mut g, false);
        assert_eq!(g.state.players[p.index()].life, 20);
        assert_eq!(g.state.players[p.other().index()].life, 19);
    }
}
#[test]
fn copy_target_set_can_swap_slots_and_rejects_incomplete_duplicate_choices() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        teach(&mut g, p);
        let a = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        let b = named(&mut g, p, "Llanowar Elves", Zone::Graveyard);
        let waltz = named(&mut g, p, "Macabre Waltz", Zone::Hand);
        announce(&mut g, waltz, &[Target::Card(a), Target::Card(b)]);
        progress(&mut g, true);
        checked(
            &mut g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Card(b),
            },
        );
        reject(&mut g, Action::FinishStackTargets);
        checked(
            &mut g,
            Action::SetStackTarget {
                index: 1,
                target: Target::Card(a),
            },
        );
        assert_eq!(
            g.observe(p).retarget.as_ref().unwrap().item.targets.len(),
            2
        );
        checked(&mut g, Action::FinishStackTargets);
        let copy = g.state.stack.iter().find(|s| s.is_copy).unwrap();
        assert_eq!(
            copy.targets.iter().map(|b| b.target).collect::<Vec<_>>(),
            [Target::Card(b), Target::Card(a)]
        );
        progress(&mut g, false);
        assert_eq!(
            [a, b]
                .iter()
                .filter(|id| g.state.objects[**id as usize].zone == Zone::Hand)
                .count(),
            1
        );
        assert_eq!(g.state.objects[waltz as usize].zone, Zone::Graveyard);
    }
}
#[test]
fn nontargeted_flashback_copy_draws_without_recasting_or_creating_exile_permission() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let twice = named(&mut g, p, "Think Twice", Zone::Graveyard);
        teach(&mut g, p);
        checked(&mut g, Action::AnnounceFlashback { card: twice });
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        progress(&mut g, false);
        assert_eq!(g.state.objects[twice as usize].zone, Zone::Exile);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
        assert_eq!(g.state.players[p.index()].instant_sorcery_cast_this_turn, 2);
        assert!(g.state.stack_copy_data.is_none());
        assert!(
            g.state
                .objects
                .iter()
                .all(|o| !o.spell_copy || o.zone == Zone::Outside)
        );
    }
}
#[test]
fn delayed_copy_ignores_opponent_casts_and_expires_on_turn_change() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        teach(&mut g, p);
        assert_eq!(g.observe(p.other()).delayed_spell_copies.len(), 1);
        checked(&mut g, Action::Pass);
        let opponent = named(&mut g, p.other(), "Shock", Zone::Hand);
        announce(&mut g, opponent, &[Target::Player(p)]);
        progress(&mut g, false);
        assert_eq!(g.observe(p).delayed_spell_copies.len(), 1);
        assert_eq!(g.state.players[p.index()].life, 18);
        super::ability_timeline_tests::next_turn(&mut g);
        assert!(g.observe(p).delayed_spell_copies.is_empty());
    }
}
fn pay(g: &mut Game) {
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    checked(g, action);
}
#[test]
fn copies_preserve_mode_kicker_and_x_but_have_no_actual_mana_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        teach(&mut g, p);
        let burst = named(&mut g, p, "Burst Lightning", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: burst });
        checked(&mut g, Action::ChooseKicker { pay: true });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        pay(&mut g);
        progress(&mut g, true);
        let packet = &g.observe(p).retarget.unwrap().item;
        assert_eq!(packet.cast.unwrap().kicker, Some(true));
        assert_eq!(packet.cast.unwrap().payment, [0; 6]);
        reject(&mut g, Action::ChooseKicker { pay: false });
        checked(&mut g, Action::FinishStackTargets);
        progress(&mut g, false);
        assert_eq!(g.state.players[p.other().index()].life, 12);
        teach(&mut g, p);
        let charm = named(&mut g, p, "Boros Charm", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: charm });
        checked(&mut g, Action::ChooseMode { mode: 0 });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        pay(&mut g);
        progress(&mut g, true);
        assert_eq!(g.observe(p).retarget.unwrap().item.mode, Some(0));
        reject(&mut g, Action::ChooseMode { mode: 1 });
        checked(&mut g, Action::FinishStackTargets);
        progress(&mut g, false);
        assert_eq!(g.state.players[p.other().index()].life, 4);
        teach(&mut g, p);
        let finale = named(&mut g, p, "Finale of Revelation", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: finale });
        checked(&mut g, Action::ChooseX { value: 2 });
        pay(&mut g);
        for _ in 0..10 {
            if g.state.stack.iter().any(|s| s.is_copy) {
                break;
            }
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                .unwrap();
            checked(&mut g, action);
        }
        let copy = g.state.stack.iter().find(|s| s.is_copy).unwrap();
        assert_eq!(copy.x, Some(2));
        assert_eq!(copy.mana_value(), 4);
        assert_eq!(copy.cast.unwrap().payment, [0; 6]);
        progress(&mut g, false);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 4);
        assert_eq!(g.state.objects[finale as usize].zone, Zone::Exile);
    }
}
#[test]
fn inherited_target_triggers_a_separate_ward_bound_to_the_copy() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        teach(&mut g, p);
        let terror = named(&mut g, p.other(), "Tolarian Terror", Zone::Battlefield);
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        announce(&mut g, shock, &[Target::Card(terror)]);
        progress(&mut g, true); // The original spell's ward was paid first.
        checked(&mut g, Action::FinishStackTargets);
        let copy = g.state.stack.iter().find(|s| s.is_copy).unwrap().clone();
        for _ in 0..10 {
            if matches!(g.state.pending, Pending::ResolutionMana { .. }) {
                break;
            }
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                .unwrap();
            checked(&mut g, action);
        }
        assert_eq!(
            g.state
                .resolving
                .as_ref()
                .unwrap()
                .mana_payment
                .as_ref()
                .unwrap()
                .victim,
            Some(copy.id)
        );
        checked(&mut g, Action::DeclineResolutionMana);
        progress(&mut g, false);
        assert_eq!(g.state.objects[copy.card as usize].zone, Zone::Outside);
        assert_eq!(g.state.objects[terror as usize].damage, 2);
        assert_eq!(g.state.objects[shock as usize].zone, Zone::Graveyard);
    }
}
#[test]
fn storm_counts_only_earlier_casts_and_freezes_count_before_a_later_response() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Thousand-Year Storm", Zone::Battlefield);
        for _ in 0..2 {
            let shock = named(&mut g, p, "Shock", Zone::Hand);
            announce(&mut g, shock, &[Target::Player(p.other())]);
            progress(&mut g, false);
        }
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        announce(&mut g, shock, &[Target::Player(p.other())]);
        let original_count = |g: &Game| {
            g.state.stack.iter().find_map(|s| {
                s.copy_event
                    .as_ref()
                    .filter(|e| e.spell.card == shock)
                    .map(|e| e.amount)
            })
        };
        assert_eq!(original_count(&g), Some(2));
        let draw = named(&mut g, p, "Think Twice", Zone::Hand);
        announce(&mut g, draw, &[]);
        assert_eq!(original_count(&g), Some(2));
        progress(&mut g, false);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 4);
        assert_eq!(g.state.players[p.other().index()].life, 8);
        assert_eq!(g.state.players[p.index()].instant_sorcery_cast_this_turn, 4);
    }
}
#[test]
fn copied_teach_creates_its_own_delayed_trigger_without_consuming_the_next_cast() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        teach(&mut g, p);
        teach(&mut g, p);
        assert_eq!(g.observe(p).delayed_spell_copies.len(), 2);
        assert!(g.observe(p).delayed_spell_copies.iter().all(|s| !s.is_copy));
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        announce(&mut g, shock, &[Target::Player(p.other())]);
        progress(&mut g, false);
        assert_eq!(g.state.players[p.other().index()].life, 14);
        assert_eq!(g.state.players[p.index()].instant_sorcery_cast_this_turn, 3);
        assert!(g.observe(p).delayed_spell_copies.is_empty());
    }
}
fn tap_goggles(g: &mut Game, card: CardId) {
    checked(
        g,
        Action::ActivateMana {
            card,
            ability: 0,
            production: [0, 0, 0, 1, 0, 0],
            payment: [0; 6],
            counter_payment: Counters::default(),
        },
    );
}
#[test]
fn goggles_tagged_mana_copies_after_producer_departure_through_raw_paid_cast() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0; 6];
        let goggles = named(&mut g, p, "Pyromancer's Goggles", Zone::Battlefield);
        let source = g.object_ref(goggles);
        tap_goggles(&mut g, goggles);
        let lot = &g.observe(p).mana_lots[0];
        assert_eq!(lot.spell_bonus, ManaSpellBonusView::CopyRedInstantSorcery);
        assert_eq!(lot.copy_source.unwrap().object, source);
        g.move_object(goggles, Zone::Graveyard);
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        let cast = g.legal_actions().into_iter().find(|a| matches!(a, Action::Cast { card, target: Some(Target::Player(t)), .. } if *card == shock && *t == p.other())).unwrap();
        checked(&mut g, cast);
        let trigger = g
            .state
            .stack
            .iter()
            .find(|s| s.copy_event.is_some())
            .unwrap();
        assert_eq!(trigger.object, source);
        assert_eq!(trigger.source_card.name(), "Pyromancer's Goggles");
        assert_eq!(trigger.controller, p);
        progress(&mut g, false);
        assert_eq!(g.state.players[p.other().index()].life, 16);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 1);
        assert!(g.mana_lots(p).is_empty());
    }
}
#[test]
fn goggles_mana_is_unrestricted_and_only_spent_red_instant_or_sorcery_mana_copies() {
    for p in [Player::P0, Player::P1] {
        for name in ["Bear Cub", "Fanatical Firebrand", "Think Twice"] {
            let mut g = setup(p);
            g.state.players[p.index()].mana = [0; 6];
            let goggles = named(&mut g, p, "Pyromancer's Goggles", Zone::Battlefield);
            tap_goggles(&mut g, goggles);
            if name == "Bear Cub" {
                g.state.players[p.index()].mana[4] = 1;
            }
            if name == "Think Twice" {
                g.state.players[p.index()].mana[1] = 1;
            }
            let id = named(&mut g, p, name, Zone::Hand);
            announce(&mut g, id, &[]);
            progress(&mut g, false);
            assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 1);
            assert!(g.state.stack_copy_data.is_none());
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                u64::from(name == "Think Twice")
            );
        }
    }
}
#[test]
fn each_goggles_mana_unit_creates_an_independent_copy_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0; 6];
        let first = named(&mut g, p, "Pyromancer's Goggles", Zone::Battlefield);
        tap_goggles(&mut g, first);
        g.move_object(first, Zone::Graveyard);
        let second = named(&mut g, p, "Pyromancer's Goggles", Zone::Battlefield);
        tap_goggles(&mut g, second);
        assert_eq!(g.mana_lots(p).len(), 2);
        g.state.players[p.index()].mana[5] = 3;
        let burst = named(&mut g, p, "Burst Lightning", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: burst });
        checked(&mut g, Action::ChooseKicker { pay: true });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        pay(&mut g);
        let copies: Vec<_> = g
            .state
            .stack
            .iter()
            .chain(g.state.ordering_triggers.iter())
            .filter(|s| s.copy_event.is_some())
            .collect();
        assert_eq!(copies.len(), 2);
        assert_ne!(copies[0].object, copies[1].object);
        progress(&mut g, false);
        assert_eq!(g.state.players[p.other().index()].life, 8);
        assert_eq!(g.state.players[p.index()].instant_sorcery_cast_this_turn, 1);
    }
}
#[test]
fn mana_allocation_can_keep_goggles_mana_unspent_and_zero_payment_has_no_bonus() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0; 6];
        let goggles = named(&mut g, p, "Pyromancer's Goggles", Zone::Battlefield);
        tap_goggles(&mut g, goggles);
        let before = g.snapshot();
        let outcome = g.spend_mana_for(
            p,
            [0; 6],
            g.spell_mana_purpose(Card::from_name("Shock").unwrap()),
        );
        assert_eq!(g.snapshot(), before);
        assert!(!outcome.haste && outcome.copies.is_none());
        g.state.players[p.index()].mana[3] += 1;
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: shock });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        pay(&mut g);
        let range = g.mana_lot_spend_range().unwrap();
        assert_eq!((range.minimum, range.maximum), (0, 1));
        checked(&mut g, Action::ChooseManaLotSpend { amount: 0 });
        progress(&mut g, false);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert_eq!(g.mana_lots(p)[0].amounts[3], 1);
    }
}
