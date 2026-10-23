//! Paid Class levels and pre-event creature-departure observers.
use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::*;
use crate::cards::AnimationDuration;
fn reject(g: &mut Game, action: Action) {
    let old = g.snapshot();
    let rev = g.revision();
    assert!(g.apply(g.actor().unwrap(), action).is_err());
    assert_eq!(g.snapshot(), old);
    assert_eq!(g.revision(), rev);
}
fn level(g: &mut Game, card: CardId, index: u16) {
    checked(
        g,
        Action::AnnounceActivate {
            card,
            ability: index,
            counter_payment: Counters::default(),
        },
    );
    for _ in 0..8 {
        if g.state.announcing.is_none() {
            break;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
            .expect("Class level payment");
        checked(g, action);
    }
    settle(g);
}
#[test]
fn tmt_class_leader_paid_levels_are_designations_sorcery_only_and_exact_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let talent = named(&mut g, p, "Leader's Talent", Zone::Hand);
        cast(&mut g, talent, None, &[]);
        settle(&mut g);
        let original = g.object_ref(talent);
        assert_eq!(g.class_level(original), 1);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: talent,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        g.state.step = Step::Blockers;
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: talent,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        g.state.step = Step::Main1;
        let before = g.state.players[p.index()].mana;
        level(&mut g, talent, 0);
        assert_eq!(g.class_level(original), 2);
        assert_eq!(g.counter_amounts(talent).total(), 0);
        assert_ne!(g.state.players[p.index()].mana, before);
        assert_eq!(g.observe(p).class_levels, g.observe(p.other()).class_levels);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: talent,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        level(&mut g, talent, 1);
        assert_eq!(g.class_level(original), 3);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: talent,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        let bounce = named(&mut g, p, "Into the Roil", Zone::Hand);
        bounce_class(&mut g, bounce, talent);
        settle(&mut g);
        cast(&mut g, talent, None, &[]);
        settle(&mut g);
        assert_ne!(g.object_ref(talent), original);
        assert_eq!(g.class_level(g.object_ref(talent)), 1);
        assert!(g.observe(p).class_levels.is_empty());
        level(&mut g, talent, 0);
        assert_eq!(g.class_level(g.object_ref(talent)), 2);
    }
}
#[test]
fn tmt_class_leader_level_two_uses_any_counter_and_all_battlefield_departures() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let talent = named(&mut g, p, "Leader's Talent", Zone::Battlefield);
        let first = named(&mut g, p, "Savannah Lions", Zone::Battlefield);
        g.set_counters(first, Counters::of(CounterKind::Finality, 1));
        let murder = named(&mut g, p, "Murder", Zone::Hand);
        let life = g.state.players[p.index()].life;
        cast(&mut g, murder, None, &[Target::Card(first)]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life);
        level(&mut g, talent, 0);
        let second = named(&mut g, p, "Savannah Lions", Zone::Battlefield);
        g.set_counters(second, Counters::of(CounterKind::Finality, 1));
        let murder = named(&mut g, p, "Murder", Zone::Hand);
        cast(&mut g, murder, None, &[Target::Card(second)]);
        settle(&mut g);
        assert_eq!(g.state.objects[second as usize].zone, Zone::Exile);
        assert_eq!(g.state.players[p.index()].life, life + 2);
        let plain = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let bounce = named(&mut g, p, "Unsummon", Zone::Hand);
        cast(&mut g, bounce, None, &[Target::Card(plain)]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life + 2);
    }
}
#[test]
fn tmt_class_leader_simultaneous_bounce_keeps_departing_classes_level_and_observers() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let talent = named(&mut g, p, "Leader's Talent", Zone::Battlefield);
        level(&mut g, talent, 0);
        let first = named(&mut g, p, "Savannah Lions", Zone::Battlefield);
        let second = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.set_counters(first, Counters::of(CounterKind::Finality, 1));
        g.set_counters(second, Counters::of(CounterKind::Stun, 1));
        let life = g.state.players[p.index()].life;
        g.state.active = p.other();
        g.state.priority = p.other();
        let rebuke = named(&mut g, p.other(), "River's Rebuke", Zone::Hand);
        cast(&mut g, rebuke, None, &[Target::Player(p)]);
        settle(&mut g);
        for id in [talent, first, second] {
            assert_eq!(g.state.objects[id as usize].zone, Zone::Hand);
        }
        assert_eq!(g.state.players[p.index()].life, life + 4);
        assert!(g.observe(p).class_levels.is_empty());
    }
}
#[test]
fn tmt_class_leader_level_three_places_counters_only_on_current_controlled_creatures() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let talent = named(&mut g, p, "Leader's Talent", Zone::Battlefield);
        level(&mut g, talent, 0);
        level(&mut g, talent, 1);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, None, &[Target::Player(p.other())]);
        settle(&mut g);
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 1);
        assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 0);
        assert_eq!(g.counter_count(talent, CounterKind::PlusOne), 0);
        let blink = named(&mut g, p, "Into the Roil", Zone::Hand);
        bounce_class(&mut g, blink, talent);
        settle(&mut g);
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 2); // The cast triggered before the Class left.
    }
}

fn bounce_class(g: &mut Game, card: CardId, target: CardId) {
    checked(g, Action::AnnounceCast { card });
    checked(g, Action::ChooseKicker { pay: false });
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    for _ in 0..8 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
            .expect("bounce payment");
        checked(g, a);
    }
}

#[test]
fn tmt_class_leader_attack_ability_targets_an_attacker_at_every_level() {
    for p in [Player::P0, Player::P1] {
        for levels in 0..3 {
            let mut g = setup(p);
            let talent = named(&mut g, p, "Leader's Talent", Zone::Battlefield);
            for index in 0..levels {
                level(&mut g, talent, index);
            }
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let bystander = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            checked(&mut g, Action::Attack { card: bear });
            checked(&mut g, Action::FinishAttackers);
            assert!(!g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(bystander)
            }));
            settle(&mut g);
            assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 1);
            assert_eq!(g.counter_count(bystander, CounterKind::PlusOne), 0);
        }
    }
}

#[test]
fn tmt_class_pending_level_resolves_after_ability_loss_but_gained_triggers_stay_removed() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let talent = named(&mut g, p, "Leader's Talent", Zone::Battlefield);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: talent,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        while g.state.announcing.is_some() {
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
                .unwrap();
            checked(&mut g, a);
        }
        let source = g.object_ref(talent);
        g.remove_abilities_exact(&[source], AnimationDuration::Permanent);
        settle(&mut g);
        assert_eq!(g.class_level(source), 2);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: talent,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        let lion = named(&mut g, p, "Savannah Lions", Zone::Battlefield);
        g.set_counters(lion, Counters::of(CounterKind::Stun, 1));
        let murder = named(&mut g, p, "Murder", Zone::Hand);
        let life = g.state.players[p.index()].life;
        cast(&mut g, murder, None, &[Target::Card(lion)]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life);
    }
}

fn pay_class_level(g: &mut Game, card: CardId, index: u16) {
    checked(
        g,
        Action::AnnounceActivate {
            card,
            ability: index,
            counter_payment: Counters::default(),
        },
    );
    while g.state.announcing.is_some() {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
            .unwrap();
        checked(g, a);
    }
}
#[test]
fn tmt_class_cool_attack_private_optional_discard_draw_and_level_two_damage() {
    for p in [Player::P0, Player::P1] {
        for second in [false, true] {
            for accept in [false, true] {
                let mut g = setup(p);
                let class = named(&mut g, p, "Cool but Rude", Zone::Battlefield);
                if second {
                    level(&mut g, class, 0);
                }
                let attacker = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                let hand = named(&mut g, p, "Shock", Zone::Hand);
                let library = g.state.players[p.index()].library.len();
                let life = g.state.players[p.other().index()].life;
                g.state.step = Step::Attackers;
                g.state.pending = Pending::Attackers;
                checked(&mut g, Action::Attack { card: attacker });
                checked(&mut g, Action::FinishAttackers);
                for _ in 0..8 {
                    if matches!(g.state.pending, Pending::EffectChoice { .. }) {
                        break;
                    }
                    checked(&mut g, Action::Pass);
                }
                assert!(g.observe(p).effect_choice.is_some());
                assert!(g.observe(p.other()).effect_choice.is_none());
                if accept {
                    checked(&mut g, Action::Discard { card: hand });
                } else {
                    checked(&mut g, Action::DeclineEffectPayment);
                }
                settle(&mut g);
                assert_eq!(
                    g.state.objects[hand as usize].zone,
                    if accept { Zone::Graveyard } else { Zone::Hand }
                );
                assert_eq!(
                    g.state.players[p.index()].library.len(),
                    library - usize::from(accept)
                );
                assert_eq!(
                    g.state.players[p.other().index()].life,
                    life - if accept && second { 2 } else { 0 }
                );
            }
        }
    }
}
#[test]
fn tmt_class_cool_level_three_private_required_search_shuffle_and_random_discard() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let class = named(&mut g, p, "Cool but Rude", Zone::Battlefield);
        level(&mut g, class, 0);
        let a = named(&mut g, p, "Shock", Zone::Hand);
        let b = named(&mut g, p, "Bear Cub", Zone::Hand);
        let find = named(&mut g, p, "Opt", Zone::Library);
        let life = g.state.players[p.other().index()].life;
        pay_class_level(&mut g, class, 1);
        while !matches!(g.state.pending, Pending::EffectSearch { .. }) {
            checked(&mut g, Action::Pass);
        }
        assert!(g.observe(p).search_choice.is_some());
        assert!(g.observe(p.other()).search_choice.is_none());
        reject(&mut g, Action::FinishSearch);
        checked(&mut g, Action::ChooseSearch { card: find });
        checked(&mut g, Action::FinishSearch);
        settle(&mut g);
        assert_eq!(g.class_level(g.object_ref(class)), 3);
        assert_eq!(
            [a, b, find]
                .into_iter()
                .filter(|&id| g.state.objects[id as usize].zone == Zone::Graveyard)
                .count(),
            1
        );
        assert_eq!(g.state.players[p.other().index()].life, life - 2);
        if g.state.objects[find as usize].zone == Zone::Hand {
            assert!(!g.state.known_hand.contains(&g.object_ref(find)));
        }
    }
}
#[test]
fn tmt_class_cool_random_discard_capacity_failure_restores_search_hand_and_rng() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let class = named(&mut g, p, "Cool but Rude", Zone::Battlefield);
        level(&mut g, class, 0);
        let a = named(&mut g, p, "Shock", Zone::Hand);
        let b = named(&mut g, p, "Bear Cub", Zone::Hand);
        let find = named(&mut g, p, "Opt", Zone::Library);
        g.state.objects[a as usize].generation = u64::MAX;
        g.state.objects[b as usize].generation = u64::MAX;
        g.state.objects[find as usize].generation = u64::MAX - 1;
        pay_class_level(&mut g, class, 1);
        while !matches!(g.state.pending, Pending::EffectSearch { .. }) {
            checked(&mut g, Action::Pass);
        }
        checked(&mut g, Action::ChooseSearch { card: find });
        let before = g.snapshot();
        let revision = g.revision();
        let rng = g.state.rng;
        assert!(matches!(
            g.apply(p, Action::FinishSearch),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.state.rng, rng);
        assert_eq!(g.revision(), revision);
    }
}

#[test]
fn tmt_class_machines_paid_entry_mills_then_draws_then_discards_two_private_cards() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let first = named(&mut g, p, "Shock", Zone::Library);
        let second = named(&mut g, p, "Opt", Zone::Library);
        let class = named(&mut g, p, "Does Machines", Zone::Hand);
        let library = g.state.players[p.index()].library.len();
        cast(&mut g, class, None, &[]);
        settle(&mut g);
        assert!(g.on_battlefield(class));
        assert_eq!(g.class_level(g.object_ref(class)), 1);
        assert_eq!(g.state.objects[first as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[second as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].library.len(), library - 4);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
        assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 2);
        assert_eq!(g.observe(p).players[p.index()].hand_count, 0);
        assert_eq!(g.observe(p.other()).players[p.index()].hand_count, 0);
    }
}
#[test]
fn tmt_class_machines_level_two_selects_zero_to_two_owned_grave_artifacts_and_keeps_partial_targets()
 {
    for p in [Player::P0, Player::P1] {
        for picks in 0..3 {
            let mut g = setup(p);
            let class = named(&mut g, p, "Does Machines", Zone::Battlefield);
            let a = named(&mut g, p, "Skateboard", Zone::Graveyard);
            let b = named(&mut g, p, "Mouser Foundry", Zone::Graveyard);
            let foreign = named(&mut g, p.other(), "Skateboard", Zone::Graveyard);
            let creature = named(&mut g, p, "Bear Cub", Zone::Graveyard);
            pay_class_level(&mut g, class, 0);
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(foreign),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(creature),
                },
            );
            if picks > 0 {
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(a),
                    },
                );
            }
            if picks > 1 {
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(b),
                    },
                );
            }
            if g.state.announcing.is_some() {
                checked(&mut g, Action::FinishTargets);
            }
            if picks == 2 {
                g.move_object(a, Zone::Exile);
            }
            settle(&mut g);
            assert_eq!(
                g.state.objects[a as usize].zone,
                match picks {
                    0 => Zone::Graveyard,
                    1 => Zone::Hand,
                    _ => Zone::Exile,
                }
            );
            assert_eq!(
                g.state.objects[b as usize].zone,
                if picks == 2 {
                    Zone::Hand
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(g.class_level(g.object_ref(class)), 2);
        }
    }
}
fn machines_combat(g: &mut Game, target: CardId) {
    assert_eq!(g.state.step, Step::Main1);
    checked(g, Action::Pass);
    checked(g, Action::Pass);
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    settle(g);
}
#[test]
fn tmt_class_machines_permanent_robot_animation_unattaches_equipment_and_survives_source_departure()
{
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let class = named(&mut g, p, "Does Machines", Zone::Battlefield);
        level(&mut g, class, 0);
        level(&mut g, class, 1);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let equipment = named(&mut g, p, "Skateboard", Zone::Battlefield);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: equipment,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        while g.state.announcing.is_some() {
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
                .unwrap();
            checked(&mut g, a);
        }
        settle(&mut g);
        assert_eq!(
            g.state.objects[equipment as usize].attached_to,
            Some(g.object_ref(bear))
        );
        machines_combat(&mut g, equipment);
        assert!(g.is_creature(equipment));
        assert!(g.object_view(equipment).has_subtype("Robot"));
        assert!(g.object_view(equipment).has_subtype("Equipment"));
        assert_eq!(
            g.stats(equipment),
            Stats {
                power: 3,
                toughness: 3
            }
        );
        assert!(g.state.objects[equipment as usize].attached_to.is_none());
        g.move_object(class, Zone::Hand);
        super::ability_timeline_tests::next_turn(&mut g);
        assert!(g.is_creature(equipment));
        assert_eq!(
            g.stats(equipment),
            Stats {
                power: 3,
                toughness: 3
            }
        );
        g.move_object(equipment, Zone::Hand);
        g.move_object(equipment, Zone::Battlefield);
        assert!(!g.is_creature(equipment));
        assert!(!g.object_view(equipment).has_subtype("Robot"));
    }
}
#[test]
fn tmt_class_machines_existing_creature_keeps_original_base_and_subtypes_with_counter_replacement()
{
    for p in [Player::P0, Player::P1] {
        for doubling in [false, true] {
            let mut g = setup(p);
            let class = named(&mut g, p, "Does Machines", Zone::Battlefield);
            level(&mut g, class, 0);
            level(&mut g, class, 1);
            let artifact = named(&mut g, p, "Solemn Simulacrum", Zone::Battlefield);
            if doubling {
                named(&mut g, p, "Doubling Season", Zone::Battlefield);
            }
            machines_combat(&mut g, artifact);
            let amount = if doubling { 6 } else { 3 };
            assert_eq!(g.counter_count(artifact, CounterKind::PlusOne), amount);
            assert_eq!(
                g.stats(artifact),
                Stats {
                    power: 2 + i128::from(amount),
                    toughness: 2 + i128::from(amount)
                }
            );
            assert!(g.object_view(artifact).has_subtype("Golem"));
            assert!(!g.object_view(artifact).has_subtype("Robot"));
            assert!(
                !g.observe(p)
                    .animations
                    .iter()
                    .any(|row| row.object.card == artifact)
            );
        }
    }
}
#[test]
fn tmt_class_machines_vehicle_animation_keeps_zero_base_after_actual_crew() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let class = named(&mut g, p, "Does Machines", Zone::Battlefield);
        level(&mut g, class, 0);
        level(&mut g, class, 1);
        let vehicle = named(&mut g, p, "Turtle Blimp", Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        machines_combat(&mut g, vehicle);
        assert_eq!(
            g.stats(vehicle),
            Stats {
                power: 3,
                toughness: 3
            }
        );
        assert!(g.object_view(vehicle).has_subtype("Vehicle"));
        assert!(g.object_view(vehicle).has_subtype("Robot"));
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: vehicle,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let object = g.object_ref(bear);
        checked(&mut g, Action::ChooseCrewer { object });
        checked(&mut g, Action::FinishCrew);
        settle(&mut g);
        assert!(g.state.objects[bear as usize].tapped);
        assert_eq!(
            g.stats(vehicle),
            Stats {
                power: 3,
                toughness: 3
            }
        );
    }
}
