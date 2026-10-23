//! Final inherited W/U source audit: actual costs, timing, targets and durations.
use super::*;

fn resolve_mode(g: &mut Game, mode: u16, target: CardId) {
    g.prepare_triggers();
    for _ in 0..100 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        let actions = g.legal_actions();
        let action = [
            Action::ChooseMode { mode },
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        ]
        .into_iter()
        .find(|a| actions.contains(a))
        .or_else(|| {
            actions
                .iter()
                .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
                .cloned()
        })
        .or_else(|| {
            actions
                .iter()
                .find(|a| !matches!(a, Action::Concede))
                .cloned()
        })
        .unwrap();
        checked(g, action);
    }
    panic!("unsettled inherited mode");
}
fn convoke(
    g: &mut Game,
    p: Player,
    name: &str,
    targets: &[CardId],
    colored: usize,
    blue: bool,
) -> CardId {
    let cost = Card::from_name(name).unwrap().cost();
    let total = cost.0.into_iter().sum::<u32>() + cost.1;
    let ids = (0..total)
        .map(|i| {
            let definition = if i < colored as u32 {
                Card::from_name(if blue {
                    "Silvergill Mentor"
                } else {
                    "@token/c48e9e904360e4cee1deeed8"
                })
                .unwrap()
            } else {
                Card::GrizzlyBears
            };
            spawn(g, p, definition, Zone::Battlefield)
        })
        .collect::<Vec<_>>();
    for &id in &ids {
        g.state.objects[id as usize].summoning_sick = true;
    }
    let id = spawn(g, p, Card::from_name(name).unwrap(), Zone::Hand);
    g.state.players[p.index()].mana = [0; 6];
    checked(g, Action::AnnounceCast { card: id });
    if name == "Appeal to Eirdu" {
        reject_atomic(g, Action::FinishTargets);
    }
    for (index, &target) in targets.iter().enumerate() {
        checked(
            g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        if name == "Appeal to Eirdu" && index == 0 {
            reject_atomic(
                g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
        }
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        checked(g, Action::FinishTargets);
    }
    checked(g, Action::BeginConvoke);
    for (index, &card) in ids.iter().enumerate() {
        let object = g.object_ref(card);
        checked(
            g,
            Action::ChooseConvoker {
                object,
                contribution: if index < colored {
                    if blue {
                        ConvokeContribution::Blue
                    } else {
                        ConvokeContribution::White
                    }
                } else {
                    ConvokeContribution::Generic
                },
            },
        );
    }
    finish_cast_choices(g);
    assert!(ids.iter().all(|&id| g.object_view(id).tapped));
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
#[test]
fn inherited_final_appeal_one_or_two_targets_convoke_partial_failure_and_expiry() {
    for p in [Player::P0, Player::P1] {
        for two in [false, true] {
            let mut g = setup(p);
            let a = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
            let b = spawn(&mut g, p.other(), Card::HillGiant, Zone::Battlefield);
            let targets = if two { vec![a, b] } else { vec![a] };
            convoke(&mut g, p, "Appeal to Eirdu", &targets, 1, false);
            if two {
                g.move_object(b, Zone::Hand);
            }
            settle(&mut g);
            assert_eq!(
                g.stats(a),
                Stats {
                    power: 4,
                    toughness: 3
                }
            );
            super::super::ability_timeline_tests::next_turn(&mut g);
            assert_eq!(g.stats(a), Stats::from(2));
        }
    }
}
#[test]
fn inherited_final_aquitect_flash_own_target_hexproof_trigger_and_attachment_duration() {
    for p in [Player::P0, Player::P1] {
        for remove in [false, true] {
            let mut g = setup(p);
            let own = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
            let enemy = spawn(&mut g, p.other(), Card::HillGiant, Zone::Battlefield);
            g.state.active = p.other();
            g.state.step = Step::BeginCombat;
            g.state.priority = p;
            let aura = spawn(
                &mut g,
                p,
                Card::from_name("Aquitect's Defenses").unwrap(),
                Zone::Hand,
            );
            checked(&mut g, Action::AnnounceCast { card: aura });
            reject_atomic(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(own),
                },
            );
            finish_cast_choices(&mut g);
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            assert_eq!(
                g.stats(own),
                Stats {
                    power: 3,
                    toughness: 4
                }
            );
            if remove {
                g.move_object(aura, Zone::Graveyard);
            }
            settle(&mut g);
            assert!(g.keywords(own).contains(Keywords::HEXPROOF));
            assert_eq!(
                g.stats(own),
                if remove {
                    Stats::from(2)
                } else {
                    Stats {
                        power: 3,
                        toughness: 4,
                    }
                }
            );
            super::super::ability_timeline_tests::next_turn(&mut g);
            assert!(!g.keywords(own).contains(Keywords::HEXPROOF));
        }
    }
}
#[test]
fn inherited_final_lofty_dreams_convoke_enemy_attachment_draw_and_source_removal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let enemy = spawn(&mut g, p.other(), Card::GrizzlyBears, Zone::Battlefield);
        let hand = g.hand_count(p);
        let aura = convoke(&mut g, p, "Lofty Dreams", &[enemy], 2, true);
        settle(&mut g);
        assert_eq!(g.hand_count(p), hand + 1);
        assert_eq!(g.stats(enemy), Stats::from(4));
        assert!(g.keywords(enemy).contains(Keywords::FLYING));
        assert!(
            !g.observe(p.other())
                .objects
                .iter()
                .any(|o| o.zone == Zone::Hand && o.owner == p)
        );
        g.move_object(aura, Zone::Graveyard);
        assert_eq!(g.stats(enemy), Stats::from(2));
        assert!(!g.keywords(enemy).contains(Keywords::FLYING));
    }
}
#[test]
fn inherited_final_disruptor_and_recluse_optional_other_target_types_owner_and_flash() {
    for p in [Player::P0, Player::P1] {
        for (name, noncreature) in [("Disruptor of Currents", true), ("Rimekin Recluse", false)] {
            for decline in [false, true] {
                let mut g = setup(p);
                let land = spawn(&mut g, p.other(), Card::Forest, Zone::Battlefield);
                let target = spawn(
                    &mut g,
                    p.other(),
                    if noncreature {
                        Card::from_name("Goldvein Pick").unwrap()
                    } else {
                        Card::GrizzlyBears
                    },
                    Zone::Battlefield,
                );
                g.change_controller(g.object_ref(target), p);
                if noncreature {
                    g.state.active = p.other();
                    g.state.step = Step::BeginCombat;
                    g.state.priority = p;
                }
                let source = if noncreature {
                    convoke(&mut g, p, name, &[], 2, true)
                } else {
                    spell(&mut g, p, name, &[])
                };
                g.prepare_triggers();
                for _ in 0..30 {
                    if matches!(g.state.pending, Pending::Targets { .. }) {
                        break;
                    }
                    let a = g
                        .legal_actions()
                        .into_iter()
                        .find(|a| !matches!(a, Action::Concede))
                        .unwrap();
                    checked(&mut g, a);
                }
                reject_atomic(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(source),
                    },
                );
                reject_atomic(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(land),
                    },
                );
                if !decline {
                    checked(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(target),
                        },
                    );
                } else {
                    checked(&mut g, Action::FinishTargets);
                }
                settle(&mut g);
                assert_eq!(
                    g.object_view(target).zone,
                    if decline {
                        Zone::Battlefield
                    } else {
                        Zone::Hand
                    }
                );
                assert_eq!(g.object_view(target).owner, p.other());
                assert_eq!(
                    g.stats(source),
                    if noncreature {
                        Stats::from(3)
                    } else {
                        Stats {
                            power: 3,
                            toughness: 2,
                        }
                    }
                );
            }
        }
    }
}
#[test]
fn inherited_final_glamermite_both_modes_flash_flying_and_riverguard_first_strike_expiry() {
    for p in [Player::P0, Player::P1] {
        for tap in [false, true] {
            let mut g = setup(p);
            let target = spawn(&mut g, p.other(), Card::GrizzlyBears, Zone::Battlefield);
            g.state.objects[target as usize].tapped = !tap;
            g.state.active = p.other();
            g.state.step = Step::BeginCombat;
            g.state.priority = p;
            let source = spell(&mut g, p, "Glamermite", &[]);
            resolve_mode(&mut g, if tap { 0 } else { 1 }, target);
            assert_eq!(g.stats(source), Stats::from(2));
            assert!(g.keywords(source).contains(Keywords::FLYING));
            assert_eq!(g.object_view(target).tapped, tap);
            take_test_priority(&mut g, p);
            spell(&mut g, p, "Riverguard's Reflexes", &[Target::Card(target)]);
            settle(&mut g);
            assert!(!g.object_view(target).tapped);
            assert_eq!(g.stats(target), Stats::from(4));
            assert!(g.keywords(target).contains(Keywords::FIRST_STRIKE));
            super::super::ability_timeline_tests::next_turn(&mut g);
            assert_eq!(g.stats(target), Stats::from(2));
            assert!(!g.keywords(target).contains(Keywords::FIRST_STRIKE));
        }
    }
}
#[test]
fn inherited_final_keep_out_damage_tapped_recheck_enchantment_mode_and_protective_combat_convoke() {
    for p in [Player::P0, Player::P1] {
        for untap in [false, true] {
            let mut g = setup(p);
            let target = spawn(&mut g, p.other(), Card::HillGiant, Zone::Battlefield);
            let id = spawn(&mut g, p, Card::from_name("Keep Out").unwrap(), Zone::Hand);
            g.state.objects[target as usize].tapped = true;
            super::super::ability_timeline_tests::cast(
                &mut g,
                id,
                Some(0),
                &[Target::Card(target)],
            );
            if untap {
                g.state.objects[target as usize].tapped = false;
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(target).zone,
                if untap {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
        }
        let mut g = setup(p);
        let enchantment = spawn(
            &mut g,
            p.other(),
            Card::from_name("Boggart Mischief").unwrap(),
            Zone::Battlefield,
        );
        let id = spawn(&mut g, p, Card::from_name("Keep Out").unwrap(), Zone::Hand);
        super::super::ability_timeline_tests::cast(
            &mut g,
            id,
            Some(1),
            &[Target::Card(enchantment)],
        );
        settle(&mut g);
        assert_eq!(g.object_view(enchantment).zone, Zone::Graveyard);
        let mut g = setup(p);
        let attacker = spawn(&mut g, p, Card::HillGiant, Zone::Battlefield);
        thoughtweft_attack(&mut g, &[attacker]);
        take_test_priority(&mut g, p);
        convoke(&mut g, p, "Protective Response", &[attacker], 1, false);
        settle(&mut g);
        assert_eq!(g.object_view(attacker).zone, Zone::Graveyard);
    }
}
#[test]
fn inherited_final_shinestriker_live_vivid_and_summit_sentinel_death_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let token = spawn(
            &mut g,
            p,
            Card::from_name("@token/c48e9e904360e4cee1deeed8").unwrap(),
            Zone::Battlefield,
        );
        let source = spell(&mut g, p, "Shinestriker", &[]);
        g.prepare_triggers();
        for _ in 0..30 {
            if g.state
                .stack
                .iter()
                .any(|s| s.card == source && matches!(s.kind, StackKind::TriggeredAbility { .. }))
            {
                break;
            }
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| !matches!(a, Action::Concede))
                .unwrap();
            checked(&mut g, a);
        }
        g.move_object(token, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.hand_count(p), 1);
        assert_eq!(g.stats(source), Stats::from(3));
        assert!(g.keywords(source).contains(Keywords::FLYING));
        take_test_priority(&mut g, p);
        let sentinel = spell(&mut g, p, "Summit Sentinel", &[]);
        settle(&mut g);
        assert_eq!(
            g.stats(sentinel),
            Stats {
                power: 1,
                toughness: 3
            }
        );
        g.change_controller(g.object_ref(sentinel), p.other());
        take_test_priority(&mut g, p);
        spell(&mut g, p, "Lightning Bolt", &[Target::Card(sentinel)]);
        settle(&mut g);
        assert_eq!(g.object_view(sentinel).zone, Zone::Graveyard);
        assert_eq!(g.hand_count(p.other()), 1);
        assert_eq!(g.hand_count(p), 1);
    }
}
#[test]
fn inherited_final_stratosoarer_basiclandcycling_private_paid_and_flying_entry_expires() {
    for p in [Player::P0, Player::P1] {
        for find in [false, true] {
            let mut g = setup(p);
            let source = spawn(
                &mut g,
                p,
                Card::from_name("Stratosoarer").unwrap(),
                Zone::Hand,
            );
            g.state.active = p.other();
            g.state.step = Step::BeginCombat;
            g.state.priority = p;
            g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 1];
            activate(&mut g, source, 0, None);
            finish_cast_choices(&mut g);
            assert_eq!(g.object_view(source).zone, Zone::Graveyard);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            for _ in 0..30 {
                if g.observe(p).search_choice.is_some() {
                    break;
                }
                let a = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| !matches!(a, Action::Concede))
                    .unwrap();
                checked(&mut g, a);
            }
            assert!(g.observe(p.other()).search_choice.is_none());
            let basic = g.state.players[p.index()].library[0];
            if find {
                checked(&mut g, Action::ChooseSearch { card: basic });
            }
            checked(&mut g, Action::FinishSearch);
            settle(&mut g);
            assert_eq!(g.hand_count(p), usize::from(find));
            if find {
                assert!(
                    g.observe(p.other())
                        .objects
                        .iter()
                        .any(|o| o.reference.card == basic && o.zone == Zone::Hand)
                );
            }
            let mut g = setup(p);
            let target = spawn(&mut g, p.other(), Card::GrizzlyBears, Zone::Battlefield);
            let source = spell(&mut g, p, "Stratosoarer", &[]);
            inherited_wu_settle_target(&mut g, target);
            assert_eq!(
                g.stats(source),
                Stats {
                    power: 3,
                    toughness: 5
                }
            );
            assert!(g.keywords(source).contains(Keywords::FLYING));
            assert!(g.keywords(target).contains(Keywords::FLYING));
            super::super::ability_timeline_tests::next_turn(&mut g);
            assert!(!g.keywords(target).contains(Keywords::FLYING));
            assert!(g.keywords(source).contains(Keywords::FLYING));
        }
    }
}
#[test]
fn inherited_final_celebrant_convoke_vigilance_and_timid_repeated_group_snapshot() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let celebrant = convoke(&mut g, p, "Sun-Dappled Celebrant", &[], 2, false);
        settle(&mut g);
        assert_eq!(
            g.stats(celebrant),
            Stats {
                power: 5,
                toughness: 6
            }
        );
        assert!(g.keywords(celebrant).contains(Keywords::VIGILANCE));
        thoughtweft_attack(&mut g, &[celebrant]);
        assert!(!g.object_view(celebrant).tapped);
        let mut g = setup(p);
        let source = cast_named(&mut g, p, "Timid Shieldbearer");
        let ally = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        let enemy = spawn(&mut g, p.other(), Card::GrizzlyBears, Zone::Battlefield);
        g.state.active = p.other();
        g.state.step = Step::BeginCombat;
        g.state.priority = p;
        for _ in 0..2 {
            g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 4];
            activate(&mut g, source, 0, None);
            finish_cast_choices(&mut g);
            settle(&mut g);
            take_test_priority(&mut g, p);
        }
        let later = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        assert_eq!(g.stats(source), Stats::from(4));
        assert_eq!(g.stats(ally), Stats::from(4));
        assert_eq!(g.stats(enemy), Stats::from(2));
        assert_eq!(g.stats(later), Stats::from(2));
        super::super::ability_timeline_tests::next_turn(&mut g);
        assert_eq!(g.stats(source), Stats::from(2));
    }
}
#[test]
fn inherited_final_run_away_distinct_controllers_partial_return_and_snare_exact_mana_value() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        let same = spawn(&mut g, p, Card::HillGiant, Zone::Battlefield);
        let enemy = spawn(&mut g, p.other(), Card::GrizzlyBears, Zone::Battlefield);
        let id = spawn(
            &mut g,
            p,
            Card::from_name("Run Away Together").unwrap(),
            Zone::Hand,
        );
        checked(&mut g, Action::AnnounceCast { card: id });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        reject_atomic(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(same),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        finish_cast_choices(&mut g);
        g.move_object(enemy, Zone::Exile);
        settle(&mut g);
        assert_eq!(g.object_view(own).zone, Zone::Hand);
        assert_eq!(g.object_view(enemy).zone, Zone::Exile);
        for eligible in [false, true] {
            let mut g = setup(p.other());
            let targets = if eligible {
                vec![]
            } else {
                vec![Target::Player(p)]
            };
            let target = spell(
                &mut g,
                p.other(),
                if eligible { "Grizzly Bears" } else { "Shock" },
                &targets,
            );
            take_test_priority(&mut g, p);
            let snare = spawn(
                &mut g,
                p,
                Card::from_name("Spell Snare").unwrap(),
                Zone::Hand,
            );
            if eligible {
                checked(&mut g, Action::AnnounceCast { card: snare });
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Spell(target),
                    },
                );
                finish_cast_choices(&mut g);
                settle(&mut g);
                assert_eq!(g.object_view(target).zone, Zone::Graveyard);
            } else {
                reject_atomic(&mut g, Action::AnnounceCast { card: snare });
            }
        }
    }
}
#[test]
fn inherited_final_unwelcome_sprite_requires_own_spell_on_opponents_turn_and_private_surveil_survives_departure()
 {
    for p in [Player::P0, Player::P1] {
        for (opponent_turn, own_cast) in [(false, true), (true, false), (true, true)] {
            let mut g = setup(p);
            let sprite = cast_named(&mut g, p, "Unwelcome Sprite");
            assert_eq!(
                g.stats(sprite),
                Stats {
                    power: 2,
                    toughness: 1
                }
            );
            assert!(g.keywords(sprite).contains(Keywords::FLYING));
            g.state.active = if opponent_turn { p.other() } else { p };
            g.state.step = Step::Main1;
            let caster = if own_cast { p } else { p.other() };
            take_test_priority(&mut g, caster);
            spell(&mut g, caster, "Shock", &[Target::Player(caster.other())]);
            g.move_object(sprite, Zone::Hand);
            let expected = opponent_turn && own_cast;
            let mut chosen = 0;
            for _ in 0..100 {
                if g.state.stack.is_empty()
                    && g.state.pending == Pending::Priority
                    && g.state.pending_triggers.is_empty()
                {
                    break;
                }
                let actor = g.actor().unwrap();
                let view = g.observe(actor);
                let action = if let Some(choice) = view.library_choice {
                    assert_eq!(actor, p);
                    assert!(g.observe(p.other()).library_choice.is_none());
                    let card = choice.remaining[0].reference.card;
                    chosen += 1;
                    Action::ArrangeLibrary {
                        card,
                        destination: LibraryDestination::Graveyard,
                    }
                } else if g.state.pending == Pending::Priority {
                    Action::Pass
                } else {
                    g.legal_actions()
                        .into_iter()
                        .find(|a| !matches!(a, Action::Concede))
                        .unwrap()
                };
                checked(&mut g, action);
            }
            assert_eq!(chosen, if expected { 2 } else { 0 });
            assert_eq!(
                g.state.players[p.index()].library.len(),
                if expected { 18 } else { 20 }
            );
        }
    }
}
