//! Paid Saga obligations; direct layer/incarnation fixtures are explicitly bounded.
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::msh_galactus_tests::{finish, reject, until};
use super::*;
const CIVIL: &str = "The Super Hero Civil War";
fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn chapter(g: &mut Game, s: CardId, n: u16) {
    until(g, |g| {
        g.state.announcing.as_ref().is_some_and(|a| {
            a.item.card == s && a.item.kind == StackKind::TriggeredAbility { ability: n - 1 }
        }) || (n == 2
            && g.state
                .stack
                .iter()
                .any(|i| i.card == s && i.kind == StackKind::TriggeredAbility { ability: 1 }))
    });
}
fn targets(g: &mut Game, ids: &[CardId]) {
    for &id in ids {
        checked(
            g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
    if g.state.announcing.is_some() {
        finish(g);
    }
}
fn announce(g: &mut Game, p: Player) -> CardId {
    priority(g, p);
    let s = named(g, p, CIVIL, Zone::Hand);
    g.state.players[p.index()].mana = [1, 0, 0, 1, 0, 3];
    cast(g, s, None, &[]);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    chapter(g, s, 1);
    s
}
fn saga(g: &mut Game, p: Player, ids: &[CardId]) -> CardId {
    let s = announce(g, p);
    targets(g, ids);
    settle(g);
    s
}
fn paid(g: &mut Game, p: Player, name: &str, mana: Mana, ids: &[CardId]) -> CardId {
    priority(g, p);
    let id = named(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = mana;
    cast(
        g,
        id,
        None,
        &ids.iter().copied().map(Target::Card).collect::<Vec<_>>(),
    );
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    settle(g);
    id
}
fn advance(g: &mut Game, p: Player, s: CardId, n: u16) {
    next_turn(g);
    next_turn(g);
    chapter(g, s, n);
    assert_eq!(g.counter_count(s, CounterKind::Lore), u64::from(n));
    assert_eq!(g.state.active, p);
}
fn control_rows(g: &Game) -> usize {
    g.source_linked_effect_views()
        .unwrap_or_default()
        .iter()
        .filter(|r| r.kind == SourceLinkedEffectKind::Control)
        .count()
}
#[test]
fn msh_civil_war_paid_zero_one_two_and_atomic_budget() {
    for p in [Player::P0, Player::P1] {
        for count in 0..=2 {
            let mut g = setup(p);
            let a = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let b = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let third = named(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
            let over = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
            let land = named(&mut g, p.other(), "Island", Zone::Battlefield);
            let s = announce(&mut g, p);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(land),
                },
            );
            if count > 0 {
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(a),
                    },
                );
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(a),
                    },
                );
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(over),
                    },
                );
            }
            if count == 2 {
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(b),
                    },
                );
                if g.state.announcing.is_some() {
                    reject(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(third),
                        },
                    );
                }
            }
            if g.state.announcing.is_some() {
                finish(&mut g);
            }
            settle(&mut g);
            assert_eq!(g.counter_count(s, CounterKind::Lore), 1);
            assert_eq!(
                g.object_view(a).controller,
                if count > 0 { p } else { p.other() }
            );
            assert_eq!(g.object_view(b).controller, p);
            assert_eq!(control_rows(&g), count);
            assert_eq!(g.object_view(a).summoning_sick, count > 0);
        }
    }
}
#[test]
fn msh_civil_war_paid_source_departure_before_or_after_and_blink() {
    for p in [Player::P0, Player::P1] {
        for before in [false, true] {
            for blink in [false, true] {
                let mut g = setup(p);
                let t = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
                let s = announce(&mut g, p);
                targets(&mut g, &[t]);
                if !before {
                    settle(&mut g);
                    assert_eq!(g.object_view(t).controller, p);
                }
                paid(&mut g, p, "Disenchant", [1, 0, 0, 0, 0, 1], &[s]);
                if blink {
                    g.move_object(s, Zone::Battlefield);
                    settle(&mut g);
                }
                assert_eq!(g.object_view(t).controller, p.other());
                assert_eq!(control_rows(&g), 0);
            }
        }
    }
}
#[test]
fn msh_civil_war_paid_suppression_and_source_control_do_not_end_duration() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let t = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let s = saga(&mut g, p, &[t]);
        // Bounded layer-two transfer of the noncreature Saga tests captured controller.
        let source = g.object_ref(s);
        g.record_control(source, p.other(), None);
        assert_eq!(g.object_view(s).controller, p.other());
        assert_eq!(g.object_view(t).controller, p);
        // Bounded type change lets the ordinary paid ability-removing Aura target it.
        super::msh_galactus_tests::animate(&mut g, s, false);
        paid(&mut g, p, "Witness Protection", [0, 1, 0, 0, 0, 0], &[s]);
        assert!(!g.printed_abilities_available(s));
        assert_eq!(g.object_view(t).controller, p);
        paid(&mut g, p, "Unsummon", [0, 1, 0, 0, 0, 0], &[s]);
        assert_eq!(g.object_view(t).controller, p.other());
    }
}
#[test]
fn msh_civil_war_control_timestamps_and_recipient_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let t = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let s = saga(&mut g, p, &[t]);
        // A later fixed control effect survives the earlier source's departure.
        let reference = g.object_ref(t);
        g.record_control(reference, p.other(), None);
        assert_eq!(g.object_view(t).controller, p.other());
        paid(&mut g, p, "Disenchant", [1, 0, 0, 0, 0, 1], &[s]);
        assert_eq!(g.object_view(t).controller, p.other());
        let s2 = saga(&mut g, p, &[t]);
        assert_eq!(g.object_view(t).controller, p);
        paid(&mut g, p, "Unsummon", [0, 1, 0, 0, 0, 0], &[t]);
        g.move_object(t, Zone::Battlefield); // Explicit new-incarnation fixture.
        assert_eq!(g.object_view(t).controller, p.other());
        assert_eq!(control_rows(&g), 0);
        assert!(g.on_battlefield(s2));
    }
}
#[test]
fn msh_civil_war_resolution_two_pass_current_copy_value_budget() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = setup(p);
            let a = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let b = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let s = announce(&mut g, p);
            targets(&mut g, &[a, b]);
            // Bounded layer-one replacement: a legal target now has MV6.
            // Physical identity/incarnation stays fixed, as for a temporary copy.
            let donor = Card::from_name("Shivan Dragon").unwrap();
            let reference = g.object_ref(b);
            g.apply_temporary_copy(reference, CopyRecipe::new(donor), CopyDeadline::EndOfTurn);
            if mode > 0 {
                g.move_object(b, Zone::Graveyard);
            }
            if mode == 2 {
                g.move_object(a, Zone::Graveyard);
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(a).controller,
                if mode == 1 { p } else { p.other() }
            );
            assert_eq!(control_rows(&g), usize::from(mode == 1));
            assert!(g.on_battlefield(s));
        }
    }
}
#[test]
fn msh_civil_war_paid_chapter_two_freezes_own_set_and_cleanup() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let stolen = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let own = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let s = saga(&mut g, p, &[stolen]);
        advance(&mut g, p, s, 2);
        finish(&mut g);
        settle(&mut g);
        assert_eq!((g.stats(stolen).power, g.stats(stolen).toughness), (3, 3));
        assert_eq!((g.stats(own).power, g.stats(own).toughness), (4, 4));
        assert_eq!(g.stats(enemy).power, 3);
        assert!(g.keywords(own).contains(Keywords::VIGILANCE));
        let later = paid(&mut g, p, "Bear Cub", [0, 0, 0, 0, 1, 1], &[]);
        assert_eq!(g.stats(later).power, 2);
        assert!(!g.keywords(later).contains(Keywords::VIGILANCE));
        g.begin_step(Step::BeginCombat);
        settle(&mut g);
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        checked(&mut g, Action::Attack { card: own });
        checked(&mut g, Action::FinishAttackers);
        assert!(!g.object_view(own).tapped);
        next_turn(&mut g);
        assert_eq!(g.stats(own).power, 3);
        assert!(!g.keywords(own).contains(Keywords::VIGILANCE));
    }
}
#[test]
fn msh_civil_war_paid_chapter_three_other_creature_optional_and_final_sacrifice() {
    for p in [Player::P0, Player::P1] {
        for fight in [false, true] {
            let mut g = setup(p);
            let own = named(&mut g, p, "Hill Giant", Zone::Battlefield);
            let victim = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let s = saga(&mut g, p, &[]);
            advance(&mut g, p, s, 2);
            finish(&mut g);
            settle(&mut g);
            advance(&mut g, p, s, 3);
            reject(&mut g, Action::FinishTargets);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(victim),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(own),
                },
            );
            if g.state.announcing.is_some() {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(own),
                    },
                );
                if fight {
                    checked(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(victim),
                        },
                    );
                }
                if g.state.announcing.is_some() {
                    finish(&mut g);
                }
            }
            assert!(g.on_battlefield(s));
            settle(&mut g);
            assert_eq!(g.object_view(s).zone, Zone::Graveyard);
            assert_eq!(
                g.object_view(victim).zone,
                if fight {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(g.object_view(own).zone, Zone::Battlefield);
        }
    }
}
#[test]
fn msh_civil_war_paid_fight_partial_illegality_does_not_deal_one_sided_damage() {
    for p in [Player::P0, Player::P1] {
        for remove_own in [false, true] {
            let mut g = setup(p);
            let own = named(&mut g, p, "Hill Giant", Zone::Battlefield);
            let other = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let s = saga(&mut g, p, &[]);
            // Bounded lore placement reaches the real final triggered packet.
            g.effect_counter(s, CounterKind::Lore, 2, replacements::ResumeMode::Pending);
            chapter(&mut g, s, 3);
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(own),
                },
            );
            targets(&mut g, &[other]);
            paid(
                &mut g,
                p,
                "Unsummon",
                [0, 1, 0, 0, 0, 0],
                &[if remove_own { own } else { other }],
            );
            assert_eq!(
                g.object_view(if remove_own { other } else { own }).damage,
                0
            );
            assert_eq!(g.object_view(s).zone, Zone::Graveyard);
        }
    }
}
#[test]
fn msh_civil_war_actual_and_funding_controlled_mana_disappears_with_saga() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let elf = named(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
        let s = saga(&mut g, p, &[elf]);
        g.state.objects[elf as usize].summoning_sick = false; // Explicit age boundary.
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0; 6];
        let goal = FundingGoal {
            actor: p,
            required: [0, 0, 0, 0, 1, 0],
            generic: 0,
            reserve: None,
            source: None,
        };
        let snap = g.snapshot();
        assert!(g.can_fund(goal.clone()));
        assert_eq!(g.snapshot(), snap);
        checked(&mut g, Action::TapMana { card: elf });
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 1, 0]);
        paid(&mut g, p, "Disenchant", [1, 0, 0, 0, 0, 1], &[s]);
        g.state.objects[elf as usize].tapped = false;
        g.state.objects[elf as usize].summoning_sick = false;
        g.state.players[p.index()].mana = [0; 6];
        assert!(!g.can_fund(goal));
        reject(&mut g, Action::TapMana { card: elf });
    }
}

#[test]
fn msh_civil_war_paid_source_copy_does_not_end_resolved_control() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let t = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let s = saga(&mut g, p, &[t]);
        // Bounded layer-one overlay on the exact Saga; the resolved effect is external.
        let source = g.object_ref(s);
        g.apply_temporary_copy(
            source,
            CopyRecipe::new(Card::from_name("Bear Cub").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        assert_eq!(g.object_view(t).controller, p);
        paid(&mut g, p, "Unsummon", [0, 1, 0, 0, 0, 0], &[s]);
        assert_eq!(g.object_view(t).controller, p.other());
    }
}
#[test]
fn msh_civil_war_paid_x_and_token_targets_use_battlefield_mana_value() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        priority(&mut g, p.other());
        g.state.active = p.other();
        g.state.step = Step::Main1;
        let x = named(&mut g, p.other(), "Wildwood Scourge", Zone::Hand);
        g.state.players[p.other().index()].mana = [0, 0, 0, 0, 1, 9];
        checked(&mut g, Action::AnnounceCast { card: x });
        checked(&mut g, Action::ChooseX { value: 9 });
        finish(&mut g);
        settle(&mut g);
        g.create_tokens(p.other(), Card::MshBlueMerfolkToken, 1, false);
        settle(&mut g);
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(i, o)| {
                (o.zone == Zone::Battlefield && o.is_token() && o.card == Card::MshBlueMerfolkToken)
                    .then_some(i as CardId)
            })
            .unwrap();
        g.state.active = p;
        g.state.step = Step::Main1;
        let s = saga(&mut g, p, &[x, token]);
        assert_eq!(g.object_view(x).controller, p);
        assert_eq!(g.object_view(token).controller, p);
        assert_eq!(g.counter_count(x, CounterKind::PlusOne), 9);
        assert_eq!(control_rows(&g), 2);
        assert!(g.on_battlefield(s));
    }
}
#[test]
fn msh_civil_war_prospective_saga_departure_restores_control_before_next_mana() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let elf = named(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
        let s = saga(&mut g, p, &[elf]);
        super::msh_galactus_tests::animate(&mut g, s, false); // Bounded creature type permits equip.
        let ring = paid(&mut g, p, "Lotus Ring", [0, 0, 0, 0, 0, 3], &[]);
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: ring,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(s),
            },
        );
        finish(&mut g);
        settle(&mut g);
        g.state.objects[s as usize].summoning_sick = false;
        // The teammate cannot contribute before/after the required source sacrifice:
        // it is tapped beforehand and control returns to the opponent on departure.
        g.state.objects[elf as usize].tapped = true;
        g.state.players[p.index()].mana = [0; 6];
        let goal = |n| FundingGoal {
            actor: p,
            required: [0, 0, 0, 0, n, 0],
            generic: 0,
            reserve: None,
            source: None,
        };
        let before = g.snapshot();
        assert!(g.can_fund(goal(3)));
        assert!(!g.can_fund(goal(4)));
        assert_eq!(g.snapshot(), before);
        let grantor = g.object_ref(ring);
        checked(
            &mut g,
            Action::TapGrantedMana {
                card: s,
                grantor,
                production: [0, 0, 0, 0, 3, 0],
            },
        );
        assert_eq!(g.object_view(elf).controller, p.other());
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 3, 0]);
    }
}
#[test]
fn msh_civil_war_public_control_privacy_orphan_and_restore() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let t = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let s = saga(&mut g, p, &[t]);
        for viewer in [p, p.other()] {
            let value = serde_json::to_value(g.observe(viewer)).unwrap();
            assert_eq!(value["source_linked_effects"][0]["kind"], "Control");
            let mut a = g.clone();
            let mut b = g.clone();
            named(&mut a, viewer.other(), "Mountain", Zone::Hand);
            named(&mut b, viewer.other(), "Forest", Zone::Hand);
            assert_eq!(
                serde_json::to_value(a.observe(viewer)).unwrap(),
                serde_json::to_value(b.observe(viewer)).unwrap()
            );
        }
        priority(&mut g, p);
        let d = named(&mut g, p, "Disenchant", Zone::Hand);
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 1];
        cast(&mut g, d, None, &[Target::Card(s)]);
        let actor = g.actor().unwrap();
        let action = Action::Pass;
        let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
        g = super::msh_thirteenth_combined_tests::owned_checked_step(g, actor, index, action);
        settle(&mut g);
        assert_eq!(g.object_view(t).controller, p.other());
        assert!(
            serde_json::to_value(g.observe(p))
                .unwrap()
                .get("source_linked_effects")
                .is_none()
        );
    }
}

#[test]
fn msh_civil_war_final_fight_uses_control_before_saga_sacrifice() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let stolen = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let s = saga(&mut g, p, &[stolen]);
        advance(&mut g, p, s, 2);
        finish(&mut g);
        settle(&mut g);
        advance(&mut g, p, s, 3);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(stolen),
            },
        );
        targets(&mut g, &[enemy]);
        assert_eq!(g.object_view(stolen).controller, p);
        settle(&mut g);
        assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
        assert_eq!(g.object_view(stolen).zone, Zone::Battlefield);
        assert_eq!(g.object_view(stolen).controller, p.other());
        assert_eq!(g.object_view(s).zone, Zone::Graveyard);
    }
}
