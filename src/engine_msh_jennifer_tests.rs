//! Jennifer's whole faces and actual per-victim damage packets through owned paid actions.
use super::ability_world::CharacteristicWorld;
use super::assurance::big_helpers::{cast, checked, named, next_turn, setup};
use super::*;
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn idle(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.announcing.is_none()
        && g.state.resolving.is_none()
        && g.state.pending_triggers.is_empty()
}
fn run(g: &mut Game, target: Target, accept: bool, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..150 {
        if stop(g) {
            return;
        }
        let actions = g.legal_actions();
        let desired = [
            Action::ChooseTarget { target },
            Action::ChooseOptional { accept },
        ];
        let action = desired
            .into_iter()
            .find(|a| actions.contains(a))
            .or_else(|| {
                actions
                    .iter()
                    .find(|a| {
                        matches!(
                            a,
                            Action::Pass
                                | Action::PutTrigger { .. }
                                | Action::FinishTargets
                                | Action::PayCosts { .. }
                                | Action::ChooseManaLotSpend { .. }
                                | Action::FinishAttackers
                                | Action::FinishBlockers
                        )
                    })
                    .cloned()
            })
            .or_else(|| {
                actions
                    .iter()
                    .find(|a| {
                        matches!(a, Action::KeepLegend { object }
                if g.state.objects[object.card as usize].is_copy_token())
                    })
                    .cloned()
            })
            .or_else(|| actions.into_iter().find(|a| !matches!(a, Action::Concede)))
            .unwrap_or_else(|| panic!("Jennifer stalled {:?}", g.state.pending));
        checked(g, action);
    }
    panic!("Jennifer did not settle {:?}", g.state.pending);
}
fn settle(g: &mut Game) {
    let target = Target::Player(g.state.active.other());
    run(g, target, false, idle);
}
fn pay(g: &mut Game) {
    for _ in 0..24 {
        if g.state.announcing.is_none() {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::ChooseManaLotSpend { .. }
                )
            })
            .unwrap();
        checked(g, action);
    }
    panic!("unpaid Jennifer action");
}
fn spell(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, targets);
    id
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    let id = spell(g, p, name, targets);
    settle(g);
    id
}
fn jennifer(g: &mut Game, p: Player, back: bool) -> CardId {
    let id = named(g, p, "Jennifer Walters", Zone::Hand);
    checked(
        g,
        if back {
            Action::AnnounceBack { card: id }
        } else {
            Action::AnnounceCast { card: id }
        },
    );
    pay(g);
    settle(g);
    id
}
fn activation(id: CardId) -> Action {
    Action::AnnounceActivate {
        card: id,
        ability: 0,
        counter_payment: Counters::default(),
    }
}
fn optional(g: &mut Game, target: Target, accept: bool) {
    run(g, target, false, |g| {
        matches!(g.state.pending, Pending::EffectOptional { .. })
    });
    reject(g, Action::FinishTargets);
    checked(g, Action::ChooseOptional { accept });
}
fn packets(g: &Game) -> Vec<&SpellView> {
    g.state
        .stack
        .iter()
        .chain(g.state.pending_triggers.iter())
        .filter(|s| s.damage_received.is_some())
        .collect()
}
#[test]
fn msh_jennifer_paid_face_costs_sorcery_privacy_and_back_deck_rejection() {
    let front = Card::from_name("Jennifer Walters").unwrap();
    let back = front.back_face().unwrap();
    assert_eq!(back, Card::MshSensationalSheHulk);
    assert!(
        Game::new(Config {
            decks: [vec![back; 7], vec![Card::Forest; 7]],
            seed: 1,
            starting_player: Player::P0,
            deck_rules: DeckRules::Freeform
        })
        .is_err()
    );
    for p in [Player::P0, Player::P1] {
        for back in [false, true] {
            let mut g = setup(p);
            let id = named(&mut g, p, "Jennifer Walters", Zone::Hand);
            assert!(g.observe(p.other()).objects.iter().all(|o| o.id != id));
            g.state.active = p.other();
            reject(
                &mut g,
                if back {
                    Action::AnnounceBack { card: id }
                } else {
                    Action::AnnounceCast { card: id }
                },
            );
            g.state.active = p;
            let (required, generic) = if back {
                ([2, 0, 0, 0, 1, 0], 3)
            } else {
                ([1, 0, 0, 0, 0, 0], 1)
            };
            g.state.players[p.index()].mana = required;
            g.state.players[p.index()].mana[5] = generic;
            checked(
                &mut g,
                if back {
                    Action::AnnounceBack { card: id }
                } else {
                    Action::AnnounceCast { card: id }
                },
            );
            let req = g.announcement_payment_request(p).unwrap();
            assert_eq!((req.required, req.generic), (required, generic));
            assert!(g.announcement_payment_request(p.other()).is_none());
            reject(&mut g, Action::PayCosts { payment: [0; 6] });
            pay(&mut g);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            let view = g.object_view(id);
            assert_eq!(view.colors, if back { 17 } else { 1 });
            assert_eq!(view.card.fixed_mana_value(), if back { 6 } else { 2 });
            assert_eq!(
                (view.power, view.toughness),
                if back { (6, 6) } else { (2, 3) }
            );
            assert!(
                view.granted_subtypes
                    .has(view.card, if back { "Gamma" } else { "Advisor" })
            );
            assert_eq!(
                view.keywords
                    .contains(Keywords::REACH.union(Keywords::TRAMPLE)),
                back
            );
            if back {
                reject(&mut g, activation(id));
            }
        }
    }
}
#[test]
fn msh_jennifer_paid_transform_preserves_layers_and_spell_only_turn_lock() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = jennifer(&mut g, p, false);
        let boots = paid(&mut g, p, "Swiftfoot Boots", &[]);
        checked(&mut g, activation(boots));
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
        pay(&mut g);
        settle(&mut g);
        g.set_counters(
            id,
            Counters::of(CounterKind::PlusOne, 1).with(CounterKind::Menace, 1),
        );
        paid(&mut g, p, "Shock", &[Target::Card(id)]);
        g.state.objects[id as usize].tapped = true;
        let before = g.object_view(id);
        g.state.step = Step::BeginCombat;
        reject(&mut g, activation(id));
        g.state.step = Step::Main1;
        checked(&mut g, activation(id));
        let req = g.announcement_payment_request(p).unwrap();
        assert_eq!((req.required, req.generic), ([2, 0, 0, 0, 1, 0], 3));
        pay(&mut g);
        settle(&mut g);
        let after = g.object_view(id);
        assert_eq!(after.reference, before.reference);
        assert_eq!(after.counters, before.counters);
        assert_eq!(after.damage, 2);
        assert!(after.tapped);
        assert_eq!(after.entered_this_turn, before.entered_this_turn);
        assert_eq!((after.power, after.toughness), (7, 7));
        assert_eq!(
            g.state.objects[boots as usize].attached_to,
            Some(before.reference)
        );
        assert!(g.state.damage_sources.contains(&id));
        assert!(g.state.static_sources.contains(&id));
        // Opponent spells are locked, activated abilities remain legal.
        checked(&mut g, Action::Pass);
        let shock = named(&mut g, p.other(), "Shock", Zone::Hand);
        reject(&mut g, Action::AnnounceCast { card: shock });
        let lotus = named(&mut g, p.other(), "Gilded Lotus", Zone::Battlefield);
        let mana = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ActivateMana { card, .. } if *card == lotus))
            .unwrap();
        checked(&mut g, mana);
        checked(&mut g, Action::Pass);
        // Passing around an empty stack advances the phase. Reach the next legal
        // main phase and fund its fixture pool before casting a sorcery-speed Aura.
        run(&mut g, Target::Player(p.other()), false, |g| {
            g.state.step == Step::Main2 && idle(g)
        });
        for state in &mut g.state.players {
            state.mana = [30; 6];
        }
        assert!(g.static_casting_prohibited(p.other()));
        paid(&mut g, p, "Frozen in Ice", &[Target::Card(id)]);
        assert!(!g.static_casting_prohibited(p.other()));
        checked(&mut g, Action::Pass);
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: shock })
        );
        // Actually pay for the opponent's newly legal spell; respond with bounce
        // rather than passing around an empty stack and emptying both pools again.
        cast(&mut g, shock, None, &[Target::Player(p)]);
        paid(&mut g, p, "Unsummon", &[Target::Card(id)]);
        assert_ne!(g.object_ref(id), before.reference);
        assert_eq!(g.object_view(id).name(), "Jennifer Walters");
        assert_eq!(g.object_view(id).card.cost(), ([1, 0, 0, 0, 0, 0], 1));
    }
}
#[test]
fn msh_jennifer_paid_damage_optional_decline_accept_turn_and_fresh_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = jennifer(&mut g, p, true);
        let target = paid(&mut g, p, "Hill Giant", &[]);
        for accept in [false, true] {
            spell(&mut g, p, "Shock", &[Target::Card(target)]);
            optional(&mut g, Target::Player(p.other()), accept);
            settle(&mut g);
            assert_eq!(g.life(p.other()), if accept { 18 } else { 20 });
        }
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
        paid(&mut g, p, "Shock", &[Target::Card(id)]);
        assert_eq!(g.life(p.other()), 18);
        assert!(packets(&g).is_empty());
        let old = g.object_ref(id);
        paid(&mut g, p, "Unsummon", &[Target::Card(id)]);
        checked(&mut g, Action::AnnounceBack { card: id });
        pay(&mut g);
        settle(&mut g);
        assert_ne!(old, g.object_ref(id));
        spell(&mut g, p, "Shock", &[Target::Card(id)]);
        optional(&mut g, Target::Player(p.other()), true);
        settle(&mut g);
        assert_eq!(g.life(p.other()), 16);
        next_turn(&mut g);
        g.state.players[p.index()].mana = [30; 6];
        spell(&mut g, p, "Shock", &[Target::Card(id)]);
        optional(&mut g, Target::Player(p.other()), true);
        settle(&mut g);
        assert_eq!(g.life(p.other()), 14);
    }
}
#[test]
fn msh_jennifer_paid_multiblocker_simultaneous_sum_lethal_source_and_trample() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = jennifer(&mut g, p, true);
        let before = g.object_ref(id);
        let a = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let b = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        g.state.objects[id as usize].summoning_sick = false;
        g.state.active = p;
        g.state.priority = p;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: id });
        checked(&mut g, Action::FinishAttackers);
        run(&mut g, Target::Player(p.other()), false, |g| {
            g.state.pending == Pending::Blockers
        });
        checked(
            &mut g,
            Action::Block {
                blocker: a,
                attacker: id,
            },
        );
        checked(
            &mut g,
            Action::Block {
                blocker: b,
                attacker: id,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        run(&mut g, Target::Player(p.other()), false, |g| {
            !packets(g).is_empty()
        });
        assert_eq!(packets(&g).len(), 1);
        let packet = packets(&g)[0].damage_received.as_ref().unwrap();
        assert_eq!((packet.recipient, packet.amount), (before, 6));
        optional(&mut g, Target::Player(p.other()), true);
        settle(&mut g);
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        assert_eq!(g.life(p.other()), 14);
    }
}
#[test]
fn msh_jennifer_simultaneous_victims_queue_separately_and_acceptance_disables_siblings() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = jennifer(&mut g, p, true);
        let a = paid(&mut g, p, "Hill Giant", &[]);
        let b = paid(&mut g, p, "Hill Giant", &[]);
        let other = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let source = g.damage_source(id);
        g.apply_damage_batch(&[
            DamageEvent {
                source,
                target: Target::Card(a),
                amount: 1,
                combat: false,
                preventable: true,
            },
            DamageEvent {
                source,
                target: Target::Card(a),
                amount: 2,
                combat: false,
                preventable: true,
            },
            DamageEvent {
                source,
                target: Target::Card(b),
                amount: 2,
                combat: false,
                preventable: true,
            },
            DamageEvent {
                source,
                target: Target::Card(other),
                amount: 1,
                combat: false,
                preventable: true,
            },
        ]);
        assert_eq!(packets(&g).len(), 2);
        let mut hits: Vec<_> = packets(&g)
            .iter()
            .map(|s| {
                let d = s.damage_received.as_ref().unwrap();
                (d.recipient, d.amount)
            })
            .collect();
        hits.sort();
        assert_eq!(hits, vec![(g.object_ref(a), 3), (g.object_ref(b), 2)]);
        optional(&mut g, Target::Player(p.other()), true);
        run(&mut g, Target::Player(p.other()), true, idle);
        assert!(matches!(g.life(p.other()), 17 | 18));
        assert_eq!(
            g.state
                .triggered_this_turn
                .iter()
                .filter(|u| u.source == g.object_ref(id))
                .count(),
            1
        );
    }
}
#[test]
fn msh_jennifer_paid_departure_control_lki_and_no_damage_history_on_returned_object() {
    for p in [Player::P0, Player::P1] {
        for depart in [false, true] {
            let mut g = setup(p);
            let id = jennifer(&mut g, p, true);
            spell(&mut g, p, "Shock", &[Target::Card(id)]);
            run(&mut g, Target::Player(p.other()), false, |g| {
                !g.state.stack.is_empty()
                    && g.state.stack.iter().any(|s| s.damage_received.is_some())
            });
            assert!(!packets(&g)[0].source_keywords.contains(Keywords::LIFELINK));
            g.set_counters(id, Counters::of(CounterKind::Lifelink, 1));
            let old = g.object_ref(id);
            g.change_controller(old, p.other());
            if depart {
                spell(&mut g, p, "Unsummon", &[Target::Card(id)]);
                run(&mut g, Target::Player(p.other()), false, |g| {
                    g.object_view(id).zone == Zone::Hand
                });
                // The source leaves as a back; its queued facts retain that definition and lifelink.
                assert_eq!(g.object_view(id).name(), "Jennifer Walters");
            }
            run(&mut g, Target::Player(p.other()), false, |g| {
                matches!(g.state.pending, Pending::EffectOptional { .. })
            });
            assert_eq!(g.actor(), Some(p));
            checked(&mut g, Action::ChooseOptional { accept: true });
            settle(&mut g);
            assert_eq!(g.life(p), 20);
            assert_eq!(g.life(p.other()), 20); // loses two, gains two under its new controller.
            assert_eq!(g.state.players[p.other().index()].life_lost_this_turn, 2);
            assert_eq!(g.state.players[p.other().index()].life_gained_this_turn, 2);
            assert_eq!(g.object_view(id).dealt_damage_this_turn, !depart);
            if !depart {
                paid(&mut g, p, "Shock", &[Target::Card(id)]);
                assert!(packets(&g).is_empty()); // Control transfer did not reset the exact ability's allowance.
            }
        }
    }
}
#[test]
fn msh_jennifer_paid_prevention_doubler_ability_removal_and_prohibited_prevention() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = jennifer(&mut g, p, true);
        let panther = named(&mut g, p, "King T'Challa", Zone::Hand);
        checked(&mut g, Action::AnnounceBack { card: panther });
        pay(&mut g);
        settle(&mut g);
        paid(&mut g, p, "Shock", &[Target::Card(panther)]);
        assert!(packets(&g).is_empty());
        paid(&mut g, p, "Gratuitous Violence", &[]);
        spell(&mut g, p, "Shock", &[Target::Card(id)]);
        optional(&mut g, Target::Player(p.other()), true);
        settle(&mut g);
        assert_eq!(g.life(p.other()), 16); // incoming Shock is not a creature; outgoing She-Hulk damage doubles.
        let ice = paid(&mut g, p, "Frozen in Ice", &[Target::Card(id)]);
        assert!(!g.keywords(id).contains(Keywords::REACH));
        paid(&mut g, p, "Shock", &[Target::Card(id)]);
        assert!(packets(&g).is_empty());
        paid(&mut g, p, "Disenchant", &[Target::Card(ice)]);
        assert!(g.keywords(id).contains(Keywords::TRAMPLE));
    }
}
#[test]
fn msh_jennifer_paid_invalid_target_and_canceled_packet_do_not_consume_use() {
    for p in [Player::P0, Player::P1] {
        for counter in [false, true] {
            let mut g = setup(p);
            let id = jennifer(&mut g, p, true);
            let bear = paid(&mut g, p, "Bear Cub", &[]);
            spell(&mut g, p, "Shock", &[Target::Card(id)]);
            run(&mut g, Target::Card(bear), false, |g| {
                matches!(g.state.pending, Pending::Targets { .. })
            });
            let forest = named(&mut g, p, "Forest", Zone::Battlefield);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(forest),
                },
            );
            run(&mut g, Target::Card(bear), false, |g| {
                g.state.stack.iter().any(|s| s.damage_received.is_some())
            });
            if counter {
                // Private cancellation boundary: the admitted pool has no ability counterspell.
                // Removing a queued packet must not record an optional successful use.
                let index = g
                    .state
                    .stack
                    .iter()
                    .position(|s| s.damage_received.is_some())
                    .unwrap();
                g.state.stack.remove(index);
            } else {
                spell(&mut g, p, "Unsummon", &[Target::Card(bear)]);
            }
            settle(&mut g);
            assert!(
                !g.state
                    .triggered_this_turn
                    .iter()
                    .any(|u| u.source == g.object_ref(id))
            );
            spell(&mut g, p, "Shock", &[Target::Card(id)]);
            optional(&mut g, Target::Player(p.other()), true);
            settle(&mut g);
            assert_eq!(g.life(p.other()), 18);
        }
    }
}
#[test]
fn msh_jennifer_paid_copy_faces_retain_pair_and_independent_received_allowance() {
    for p in [Player::P0, Player::P1] {
        for back in [false, true] {
            let mut g = setup(p);
            let id = jennifer(&mut g, p, back);
            paid(&mut g, p, "Applied Geometry", &[Target::Card(id)]);
            let copy = (0..g.state.objects.len())
                .map(|i| i as CardId)
                .find(|&i| {
                    g.on_battlefield(i)
                        && g.object_view(i).is_token()
                        && g.object_view(i).face.is_some()
                })
                .unwrap();
            if !back {
                checked(&mut g, activation(copy));
                pay(&mut g);
                settle(&mut g);
            }
            assert_eq!(g.object_view(copy).name(), "The Sensational She-Hulk");
            assert_eq!(
                g.copy_recipe(g.object_ref(copy)).unwrap().definition,
                Card::MshSensationalSheHulk
            );
            spell(&mut g, p, "Shock", &[Target::Card(copy)]);
            optional(&mut g, Target::Player(p.other()), true);
            settle(&mut g);
            assert_eq!(g.life(p.other()), 18);
            assert_eq!(g.counter_count(copy, CounterKind::PlusOne), 6);
        }
    }
}
#[test]
fn msh_jennifer_paid_fight_records_actual_doubled_damage_before_lethal_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = jennifer(&mut g, p, true);
        let enemy = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        named(&mut g, p.other(), "Gratuitous Violence", Zone::Battlefield);
        let fight = named(&mut g, p, "Bushwhack", Zone::Hand);
        cast(
            &mut g,
            fight,
            Some(1),
            &[Target::Card(id), Target::Card(enemy)],
        );
        run(&mut g, Target::Player(p.other()), false, |g| {
            !packets(g).is_empty()
        });
        let packet = packets(&g)[0].damage_received.as_ref().unwrap();
        assert_eq!(packet.amount, 6);
        optional(&mut g, Target::Player(p.other()), true);
        settle(&mut g);
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
        assert_eq!(g.life(p.other()), 14);
    }
}
#[test]
fn msh_jennifer_acceptance_before_prevention_and_recursive_self_damage_latches_use() {
    for p in [Player::P0, Player::P1] {
        for prevent in [false, true] {
            let mut g = setup(p);
            let id = jennifer(&mut g, p, true);
            let panther = named(&mut g, p, "King T'Challa", Zone::Hand);
            checked(&mut g, Action::AnnounceBack { card: panther });
            pay(&mut g);
            settle(&mut g);
            let target = Target::Card(if prevent { panther } else { id });
            spell(&mut g, p, "Shock", &[Target::Card(id)]);
            optional(&mut g, target, true);
            run(&mut g, target, true, idle);
            assert_eq!(g.object_view(id).damage, if prevent { 2 } else { 4 });
            assert_eq!(g.object_view(panther).damage, 0);
            assert_eq!(
                g.state
                    .triggered_this_turn
                    .iter()
                    .filter(|u| u.source == g.object_ref(id))
                    .count(),
                1
            );
            assert!(packets(&g).is_empty());
        }
    }
}
#[test]
fn msh_jennifer_paid_suppression_before_event_and_after_queued_packet() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = jennifer(&mut g, p, true);
        let ice = paid(&mut g, p, "Frozen in Ice", &[Target::Card(id)]);
        paid(&mut g, p, "Shock", &[Target::Card(id)]);
        assert!(packets(&g).is_empty());
        assert!(g.state.triggered_this_turn.is_empty());
        paid(&mut g, p, "Disenchant", &[Target::Card(ice)]);
        spell(&mut g, p, "Shock", &[Target::Card(id)]);
        run(&mut g, Target::Player(p.other()), false, |g| {
            g.state.stack.iter().any(|s| s.damage_received.is_some())
        });
        let ice = spell(&mut g, p, "Fresh Start", &[Target::Card(id)]);
        run(&mut g, Target::Player(p.other()), false, |g| {
            g.on_battlefield(ice)
        });
        optional(&mut g, Target::Player(p.other()), true);
        settle(&mut g);
        assert_eq!(g.life(p.other()), 18);
    }
}
#[test]
fn msh_jennifer_paid_actual_trample_and_reach_combat() {
    for p in [Player::P0, Player::P1] {
        for attack in [false, true] {
            let mut g = setup(p);
            let id = jennifer(&mut g, p, true);
            let enemy = named(
                &mut g,
                p.other(),
                if attack { "Bear Cub" } else { "Serra Angel" },
                Zone::Battlefield,
            );
            let attacker = if attack { id } else { enemy };
            let active = if attack { p } else { p.other() };
            g.state.objects[attacker as usize].summoning_sick = false;
            g.state.active = active;
            g.state.priority = active;
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            checked(&mut g, Action::Attack { card: attacker });
            checked(&mut g, Action::FinishAttackers);
            run(&mut g, Target::Player(p.other()), false, |g| {
                g.state.pending == Pending::Blockers
            });
            checked(
                &mut g,
                Action::Block {
                    blocker: if attack { enemy } else { id },
                    attacker,
                },
            );
            checked(&mut g, Action::FinishBlockers);
            run(&mut g, Target::Player(p.other()), false, |g| {
                g.state.step == Step::Main2 && idle(g)
            });
            assert_eq!(g.life(p), 20);
            assert_eq!(g.life(p.other()), if attack { 16 } else { 20 });
            assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
            assert_eq!(g.object_view(id).damage, if attack { 2 } else { 4 });
        }
    }
}
#[test]
fn msh_jennifer_actual_simultaneous_lethal_victims_and_owned_public_packets() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        jennifer(&mut g, p, true);
        let own: [CardId; 2] =
            std::array::from_fn(|_| named(&mut g, p, "Hill Giant", Zone::Battlefield));
        let enemies: [CardId; 2] =
            std::array::from_fn(|_| named(&mut g, p.other(), "Hill Giant", Zone::Battlefield));
        for &id in &enemies {
            g.state.objects[id as usize].summoning_sick = false;
        }
        g.state.active = p.other();
        g.state.priority = p.other();
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        for &id in &enemies {
            checked(&mut g, Action::Attack { card: id });
        }
        checked(&mut g, Action::FinishAttackers);
        run(&mut g, Target::Player(p.other()), false, |g| {
            g.state.pending == Pending::Blockers
        });
        for (&blocker, &attacker) in own.iter().zip(&enemies) {
            checked(&mut g, Action::Block { blocker, attacker });
        }
        checked(&mut g, Action::FinishBlockers);
        run(&mut g, Target::Player(p.other()), false, |g| {
            packets(g).len() == 2
        });
        for item in packets(&g) {
            assert_eq!(item.controller, p);
            assert_eq!(item.damage_received.as_ref().unwrap().amount, 3);
        }
        // Public packet contains only public event identity/amount, identically visible to either seat.
        for viewer in [p, p.other()] {
            let view = g.observe(viewer);
            let bytes = serde_json::to_string(&view).unwrap();
            assert!(bytes.contains("damage_received"));
            assert!(bytes.contains("recipient"));
        }
        optional(&mut g, Target::Player(p.other()), true);
        run(&mut g, Target::Player(p.other()), true, |g| {
            g.state.step == Step::Main2 && idle(g)
        });
        assert_eq!(g.life(p.other()), 17);
        for id in own {
            assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        }
    }
}
#[test]
fn msh_jennifer_actual_double_strike_batches_decline_then_accept() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = jennifer(&mut g, p, true);
        let enemy = named(&mut g, p.other(), "Raging Redcap", Zone::Battlefield);
        g.state.objects[enemy as usize].summoning_sick = false;
        g.state.active = p.other();
        g.state.priority = p.other();
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: enemy });
        checked(&mut g, Action::FinishAttackers);
        run(&mut g, Target::Player(p.other()), false, |g| {
            g.state.pending == Pending::Blockers
        });
        checked(
            &mut g,
            Action::Block {
                blocker: id,
                attacker: enemy,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        optional(&mut g, Target::Player(p.other()), false);
        assert_eq!(g.state.step, Step::FirstStrikeDamage);
        assert_eq!(g.object_view(id).damage, 1);
        optional(&mut g, Target::Player(p.other()), true);
        assert_eq!(g.state.step, Step::CombatDamage);
        run(&mut g, Target::Player(p.other()), true, |g| {
            g.state.step == Step::Main2 && idle(g)
        });
        assert_eq!(g.life(p.other()), 19);
        assert_eq!(g.object_view(id).damage, 2);
        assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
    }
}
#[test]
fn msh_jennifer_paid_foreign_copy_observes_its_controllers_creatures() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = jennifer(&mut g, p, true);
        paid(
            &mut g,
            p,
            "Echocasting Symposium",
            &[Target::Player(p.other()), Target::Card(id)],
        );
        let copy = (0..g.state.objects.len())
            .map(|i| i as CardId)
            .find(|&i| {
                g.on_battlefield(i)
                    && g.object_view(i).is_token()
                    && g.object_view(i).controller == p.other()
                    && g.object_view(i).face.is_some()
            })
            .unwrap();
        spell(&mut g, p, "Shock", &[Target::Card(copy)]);
        run(&mut g, Target::Player(p), false, |g| !packets(g).is_empty());
        assert_eq!(packets(&g).len(), 1);
        assert_eq!(packets(&g)[0].object, g.object_ref(copy));
        assert_eq!(packets(&g)[0].controller, p.other());
        optional(&mut g, Target::Player(p), true);
        settle(&mut g);
        assert_eq!(g.life(p), 18);
        assert_eq!(g.life(p.other()), 20);
    }
}
#[test]
fn msh_jennifer_queued_departure_packet_does_not_rebind_new_front_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = jennifer(&mut g, p, true);
        spell(&mut g, p, "Shock", &[Target::Card(id)]);
        run(&mut g, Target::Player(p.other()), false, |g| {
            g.state.stack.iter().any(|s| s.damage_received.is_some())
        });
        let old = g.object_ref(id);
        // Private exact movement boundary, without admitting a new blink consumer.
        g.set_counters(id, Counters::of(CounterKind::Lifelink, 1));
        g.move_object(id, Zone::Exile);
        g.move_object(id, Zone::Battlefield);
        let new = g.object_ref(id);
        assert_ne!(new, old);
        assert_eq!(g.object_view(id).name(), "Jennifer Walters");
        assert_eq!(g.counter_count(id, CounterKind::Lifelink), 0);
        optional(&mut g, Target::Player(p.other()), true);
        settle(&mut g);
        assert_eq!(g.life(p), 22);
        assert_eq!(g.life(p.other()), 18);
        assert!(!g.object_view(id).dealt_damage_this_turn);
        assert!(g.state.triggered_this_turn.iter().any(|u| u.source == old));
        assert!(!g.state.triggered_this_turn.iter().any(|u| u.source == new));
        println!(
            "damage-received packet {} bytes; boxed slot {} bytes",
            std::mem::size_of::<DamageReceivedView>(),
            std::mem::size_of::<Option<Box<DamageReceivedView>>>()
        );
    }
}
#[test]
fn msh_jennifer_acceptance_is_recorded_before_owned_damage_replacement_choice() {
    for p in [Player::P0, Player::P1] {
        for reverse in [false, true] {
            let mut g = setup(p);
            let id = jennifer(&mut g, p, true);
            paid(&mut g, p, "Gratuitous Violence", &[]);
            paid(&mut g, p, "Twinflame Tyrant", &[]);
            spell(&mut g, p, "Shock", &[Target::Card(id)]);
            optional(&mut g, Target::Player(p.other()), true);
            assert!(matches!(g.state.pending, Pending::Replacement { .. }));
            assert_eq!(g.actor(), Some(p.other()));
            assert!(
                g.state
                    .triggered_this_turn
                    .iter()
                    .any(|u| u.source == g.object_ref(id))
            );
            assert_eq!(g.life(p.other()), 20);
            let source = g.object_ref(id);
            reject(
                &mut g,
                Action::ChooseReplacement {
                    source,
                    ability: u16::MAX,
                },
            );
            let choices: Vec<_> = g
                .legal_actions()
                .into_iter()
                .filter(|a| matches!(a, Action::ChooseReplacement { .. }))
                .collect();
            assert_eq!(choices.len(), 2);
            checked(&mut g, choices[usize::from(reverse)].clone());
            settle(&mut g);
            assert_eq!(g.life(p.other()), 12);
            paid(&mut g, p, "Shock", &[Target::Card(id)]);
            assert!(packets(&g).is_empty());
            assert_eq!(g.life(p.other()), 12);
        }
    }
}
#[test]
fn msh_jennifer_mixed_prevented_fragments_capture_actual_sum_and_post_lifelink_layers() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = jennifer(&mut g, p, true);
        let king = named(&mut g, p, "King T'Challa", Zone::Hand);
        checked(&mut g, Action::AnnounceBack { card: king });
        pay(&mut g);
        settle(&mut g);
        let equipment = paid(&mut g, p, "Aettir and Priwen", &[]);
        checked(&mut g, activation(equipment));
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
        pay(&mut g);
        settle(&mut g);
        g.set_counters(id, Counters::of(CounterKind::Lifelink, 1));
        assert_eq!(g.stats(id).power, 20);
        let recipient = g.object_ref(king);
        let source = g.damage_source(id);
        assert_eq!(source.facts.power, 20);
        // Private central commit boundary: identical live recipient and source,
        // one simultaneous batch with a prevented fragment and an actual one.
        g.apply_damage_batch(&[
            DamageEvent {
                source,
                target: Target::Card(king),
                amount: 5,
                combat: false,
                preventable: true,
            },
            DamageEvent {
                source,
                target: Target::Card(king),
                amount: 2,
                combat: false,
                preventable: false,
            },
        ]);
        // Scheduling replacements is a preview, not an actual damage event.
        assert_eq!(g.object_view(king).damage, 0);
        assert_eq!(g.life(p), 20);
        assert!(packets(&g).is_empty());
        // This is the same continuation driven by the actual combat pipeline.
        assert!(!g.advance_replacements(false));
        assert!(g.state.replacements.is_none());
        assert_eq!(g.object_view(king).damage, 2);
        assert_eq!(g.life(p), 22);
        assert_eq!(g.stats(id).power, 22);
        assert_eq!(g.state.players[p.index()].life_gain_events_this_turn, 1);
        assert_eq!(packets(&g).len(), 1);
        let item = packets(&g)[0];
        let event = item.damage_received.as_ref().unwrap();
        assert_eq!((event.recipient, event.amount), (recipient, 2));
        // The observer's source snapshot is taken after simultaneous lifelink
        // changes its real live layer-7b value; it must not retain pre-event20.
        assert_eq!(item.source_facts.power, 22);
        optional(&mut g, Target::Player(p.other()), true);
        settle(&mut g);
        assert_eq!(g.life(p.other()), 18);
        assert_eq!(g.life(p), 24);
        assert_eq!(g.stats(id).power, 24);
    }
}
