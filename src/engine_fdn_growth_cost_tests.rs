use super::ability_timeline_tests::{cast, checked, settle, setup};
use super::tests::spawn;
use super::*;

// Paid rules scenarios are shared by gated authoring and candidate validation.
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    spawn(g, p, Card::from_name(name).unwrap(), zone)
}
fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, &[]);
    settle(g);
    id
}
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert!(g.apply(g.actor().unwrap(), action).is_err());
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn activation(g: &mut Game, source: CardId, ability: u16, target: Option<Target>) {
    checked(
        g,
        Action::AnnounceActivate {
            card: source,
            ability,
            counter_payment: Counters::default(),
        },
    );
    if let Some(target) = target {
        checked(g, Action::ChooseTarget { target });
    }
    for _ in 0..8 {
        if g.state.announcing.is_none() {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
            .unwrap();
        checked(g, action);
    }
    panic!("activation payment did not finish");
}
fn tokens(g: &Game, p: Player, token: Card) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(i, o)| {
            (o.zone == Zone::Battlefield && o.controller == p && o.card == token)
                .then_some(i as CardId)
        })
        .collect()
}
#[test]
fn fdn_growth_ovika_cast_value_tokens_temporary_haste_and_mixed_ward() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ovika = paid(&mut g, p, "Ovika, Enigma Goliath");
        assert!(g.keywords(ovika).contains(Keywords::FLYING));
        let spell = named(&mut g, p, "Divination", Zone::Hand);
        cast(&mut g, spell, None, &[]);
        settle(&mut g);
        let goblin = Card::from_name("@token/phyrexian_goblin_1_1_red").unwrap();
        let made = tokens(&g, p, goblin);
        assert_eq!(made.len(), 3);
        for id in &made {
            let v = g.object_view(*id);
            assert_eq!((v.power, v.toughness, v.colors), (1, 1, 8));
            assert!(v.has_subtype("Phyrexian") && v.has_subtype("Goblin"));
            assert!(v.keywords.contains(Keywords::HASTE));
        }
        g.state.priority = p.other();
        let shock = named(&mut g, p.other(), "Shock", Zone::Hand);
        cast(&mut g, shock, None, &[Target::Card(ovika)]);
        for _ in 0..30 {
            if matches!(g.state.pending, Pending::ResolutionMana { .. }) {
                break;
            }
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::FinishTargets))
                .unwrap();
            checked(&mut g, a);
        }
        let beforelife = g.state.players[p.other().index()].life;
        let payment = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayResolutionMana { .. }))
            .unwrap();
        checked(&mut g, payment);
        assert_eq!(g.state.players[p.other().index()].life, beforelife - 3);
        settle(&mut g);
        assert_eq!(g.state.objects[ovika as usize].damage, 2);
        g.finish_cleanup();
        for id in made {
            assert!(!g.keywords(id).contains(Keywords::HASTE));
        }
    }
}
#[test]
fn fdn_growth_ovika_insufficient_life_declines_without_partial_mana_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let o = paid(&mut g, p, "Ovika, Enigma Goliath");
        g.state.priority = p.other();
        g.state.players[p.other().index()].life = 2;
        let s = named(&mut g, p.other(), "Shock", Zone::Hand);
        cast(&mut g, s, None, &[Target::Card(o)]);
        for _ in 0..30 {
            if matches!(g.state.pending, Pending::ResolutionMana { .. }) {
                break;
            }
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::FinishTargets))
                .unwrap();
            checked(&mut g, a);
        }
        reject(
            &mut g,
            Action::PayResolutionMana {
                payment: [0, 0, 0, 0, 0, 3],
            },
        );
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::PayResolutionMana { .. }))
        );
        checked(&mut g, Action::DeclineResolutionMana);
        settle(&mut g);
        assert_eq!(g.state.objects[s as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[o as usize].damage, 0);
    }
}
#[test]
fn fdn_growth_kellan_resolution_guards_persistent_types_stats_and_reset() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let k = paid(&mut g, p, "Kellan, Planar Trailblazer");
        activation(&mut g, k, 1, None);
        settle(&mut g);
        assert!(g.has_subtype(k, "Scout"));
        assert_eq!(g.stats(k).power, 2);
        activation(&mut g, k, 0, None);
        settle(&mut g);
        assert!(g.has_subtype(k, "Detective"));
        assert!(!g.has_subtype(k, "Scout"));
        assert_eq!(g.observe(p).temporary_abilities.len(), 1);
        activation(&mut g, k, 0, None);
        settle(&mut g);
        assert_eq!(g.observe(p).temporary_abilities.len(), 1);
        activation(&mut g, k, 1, None);
        settle(&mut g);
        assert!(g.has_subtype(k, "Rogue"));
        assert!(!g.has_subtype(k, "Detective"));
        assert_eq!(
            g.stats(k),
            Stats {
                power: 3,
                toughness: 2
            }
        );
        assert!(g.keywords(k).contains(Keywords::DOUBLE_STRIKE));
        g.finish_cleanup();
        assert_eq!(
            g.stats(k),
            Stats {
                power: 3,
                toughness: 2
            }
        );
        assert!(g.keywords(k).contains(Keywords::DOUBLE_STRIKE));
        assert_eq!(g.observe(p).temporary_abilities.len(), 1);
        g.move_object(k, Zone::Hand);
        g.move_object(k, Zone::Battlefield);
        assert_eq!(
            g.stats(k),
            Stats {
                power: 2,
                toughness: 1
            }
        );
        assert!(g.has_subtype(k, "Scout"));
        assert!(!g.keywords(k).contains(Keywords::DOUBLE_STRIKE));
        assert!(g.observe(p).temporary_abilities.is_empty());
    }
}
#[test]
fn fdn_growth_lathril_ten_distinct_other_elves_pay_as_cost_sick_elves_allowed() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let l = paid(&mut g, p, "Lathril, Blade of the Elves");
        g.state.objects[l as usize].summoning_sick = false;
        let elves: Vec<_> = (0..10)
            .map(|_| named(&mut g, p, "Llanowar Elves", Zone::Battlefield))
            .collect();
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: l,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let source = g.object_ref(l);
        reject(&mut g, Action::ChooseAdditionalCost { object: source });
        for (index, id) in elves.iter().enumerate() {
            let r = g.object_ref(*id);
            checked(&mut g, Action::ChooseAdditionalCost { object: r });
            if index < 9 {
                reject(&mut g, Action::ChooseAdditionalCost { object: r });
            }
            assert!(g.observe(p.other()).additional_cost.is_none());
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, action);
        assert!(g.state.objects[l as usize].tapped);
        for id in &elves {
            assert!(g.state.objects[*id as usize].tapped);
        }
        g.move_object(l, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 30);
        assert_eq!(g.state.players[p.other().index()].life, 10);
    }
}
#[test]
fn fdn_growth_preserver_nonhuman_optional_x_paid_reflexive_and_source_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let w = paid(&mut g, p, "Wildborn Preserver");
        assert!(
            g.keywords(w)
                .contains(Keywords::FLASH.union(Keywords::REACH))
        );
        let human = paid(&mut g, p, "Savannah Lions"); // Cat is a non-Human and must trigger.
        assert!(g.state.objects[human as usize].zone == Zone::Battlefield);
        // Default settle chooses X=0; now test a nonzero actual payment.
        let e = named(&mut g, p, "Llanowar Elves", Zone::Hand);
        // Keep exactly three generic mana after paying the entering Elf.
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 3];
        cast(&mut g, e, None, &[]);
        for _ in 0..50 {
            if matches!(g.state.pending, Pending::ResolutionX { .. }) {
                break;
            }
            let a = if matches!(g.state.pending, Pending::EffectOptional { .. }) {
                Action::ChooseOptional { accept: true }
            } else {
                g.legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::Pass | Action::FinishTargets))
                    .unwrap()
            };
            checked(&mut g, a);
        }
        checked(&mut g, Action::ChooseX { value: 3 });
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayResolutionMana { .. }))
            .unwrap();
        checked(&mut g, a);
        assert!(g.state.stack.iter().chain(&g.state.pending_triggers).any(|i|i.object.card==w && i.kind==StackKind::TriggeredAbility{ability:1}));
        settle(&mut g);
        assert_eq!(g.counter_count(w, CounterKind::PlusOne), 3);
    }
}
#[test]
fn fdn_growth_fishing_paid_grant_attachment_counter_and_actual_untap() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let pole = paid(&mut g, p, "Fishing Pole");
        let c = paid(&mut g, p, "Bear Cub");
        g.state.objects[c as usize].summoning_sick = false;
        activation(&mut g, pole, 0, Some(Target::Card(c)));
        settle(&mut g);
        let grantor = g.object_ref(pole);
        checked(
            &mut g,
            Action::AnnounceAttachedAbility {
                grantor,
                ability: 1,
            },
        );
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, a);
        settle(&mut g);
        assert!(g.state.objects[pole as usize].tapped && g.state.objects[c as usize].tapped);
        assert_eq!(g.counter_count(pole, CounterKind::Bait), 1);
        g.add_counter(c, CounterKind::Stun, 1);
        g.untap_objects(&[c], untap_rules::UntapCause::Effect);
        assert!(g.state.objects[c as usize].tapped);
        assert!(g.state.pending_triggers.is_empty());
        g.untap_objects(&[c], untap_rules::UntapCause::Effect);
        settle(&mut g);
        assert_eq!(g.counter_count(pole, CounterKind::Bait), 0);
        assert_eq!(
            tokens(
                &g,
                p,
                Card::from_name("@token/da26e55db634a438cfd544ee").unwrap()
            )
            .len(),
            1
        );
        g.untap_objects(&[c], untap_rules::UntapCause::Effect);
        settle(&mut g);
        assert_eq!(
            tokens(
                &g,
                p,
                Card::from_name("@token/da26e55db634a438cfd544ee").unwrap()
            )
            .len(),
            1
        );
    }
}
#[test]
fn fdn_growth_greatwurm_mixed_counter_cost_graveyard_and_normal_hand_routes() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let w = named(&mut g, p, "Quilled Greatwurm", Zone::Graveyard);
        let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        g.add_counter(a, CounterKind::Bait, 3);
        g.add_counter(b, CounterKind::PlusOne, 3);
        checked(&mut g, Action::AnnounceCounterGraveyard { card: w });
        for (id, kind) in [
            (a, CounterKind::Bait),
            (a, CounterKind::Bait),
            (a, CounterKind::Bait),
            (b, CounterKind::PlusOne),
            (b, CounterKind::PlusOne),
            (b, CounterKind::PlusOne),
        ] {
            let object = g.object_ref(id);
            checked(
                &mut g,
                Action::ChooseCreatureCounterCost {
                    object,
                    counter: kind,
                },
            );
        }
        let act = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, act);
        assert_eq!(g.counter_count(a, CounterKind::Bait), 0);
        assert_eq!(g.counter_count(b, CounterKind::PlusOne), 0);
        settle(&mut g);
        assert_eq!(g.state.objects[w as usize].zone, Zone::Battlefield);
        assert!(g.keywords(w).contains(Keywords::TRAMPLE));
    }
}
#[test]
fn fdn_growth_layout_hot_objects_and_pending_stay_bounded() {
    assert_eq!(std::mem::size_of::<Object>(), 144);
    assert_eq!(std::mem::size_of::<Pending>(), 40);
}

fn combat_to_blockers(g: &mut Game, attacker: CardId) {
    let p = g.state.objects[attacker as usize].controller;
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    g.state.objects[attacker as usize].summoning_sick = false;
    checked(g, Action::Attack { card: attacker });
    checked(g, Action::FinishAttackers);
    settle(g);
    while g.state.pending == Pending::Priority {
        checked(g, Action::Pass);
    }
    assert_eq!(g.state.pending, Pending::Blockers);
}
fn connect(g: &mut Game, attacker: CardId) {
    combat_to_blockers(g, attacker);
    checked(g, Action::FinishBlockers);
    checked(g, Action::Pass);
    checked(g, Action::Pass);
}
#[test]
fn fdn_growth_lathril_actual_combat_amount_and_menace_block_legality() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let l = paid(&mut g, p, "Lathril, Blade of the Elves");
        g.add_counter(l, CounterKind::PlusOne, 1);
        connect(&mut g, l);
        assert_eq!(g.state.players[p.other().index()].life, 17);
        assert_eq!(
            g.observe(p)
                .stack
                .iter()
                .find(|s| s.object.card == l)
                .unwrap()
                .damage_event
                .unwrap()
                .amount,
            3
        );
        g.move_object(l, Zone::Graveyard);
        settle(&mut g);
        let elf = Card::from_name("@token/ca8c0a7df61e962ddae7596f").unwrap();
        let made = tokens(&g, p, elf);
        assert_eq!(made.len(), 3);
        for id in made {
            let v = g.object_view(id);
            assert_eq!((v.power, v.toughness, v.colors), (1, 1, 16));
            assert!(v.has_subtype("Elf") && v.has_subtype("Warrior"));
        }
        let mut g = setup(p);
        let l = paid(&mut g, p, "Lathril, Blade of the Elves");
        let a = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let b = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        combat_to_blockers(&mut g, l);
        checked(
            &mut g,
            Action::Block {
                blocker: a,
                attacker: l,
            },
        );
        reject(&mut g, Action::FinishBlockers);
        checked(
            &mut g,
            Action::Block {
                blocker: b,
                attacker: l,
            },
        );
        checked(&mut g, Action::FinishBlockers);
    }
}
#[test]
fn fdn_growth_kellan_double_strike_connects_twice_and_real_exile_land_play() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let k = paid(&mut g, p, "Kellan, Planar Trailblazer");
        activation(&mut g, k, 0, None);
        settle(&mut g);
        activation(&mut g, k, 1, None);
        settle(&mut g);
        let top = named(&mut g, p, "Plains", Zone::Library);
        connect(&mut g, k);
        assert_eq!(g.state.step, Step::FirstStrikeDamage);
        assert_eq!(g.state.players[p.other().index()].life, 17);
        settle(&mut g);
        assert_eq!(g.state.objects[top as usize].zone, Zone::Exile);
        assert!(g.can_play_exiled(p, top));
        assert!(!g.can_play_exiled(p.other(), top));
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert_eq!(g.state.step, Step::CombatDamage);
        assert_eq!(g.state.players[p.other().index()].life, 14);
        settle(&mut g);
        assert_eq!(g.state.exile_permissions.len(), 2);
        // Advance the actual remaining combat windows to the next sorcery window.
        for _ in 0..12 {
            if g.state.step == Step::Main2 && g.actor() == Some(p) {
                break;
            }
            checked(&mut g, Action::Pass);
        }
        checked(&mut g, Action::PlayLand { card: top });
        assert_eq!(g.state.objects[top as usize].zone, Zone::Battlefield);
        g.finish_cleanup();
        assert!(g.state.exile_permissions.is_empty());
    }
}
#[test]
fn fdn_growth_kellan_later_ability_removal_retains_type_stats_but_suppresses_grants() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let k = paid(&mut g, p, "Kellan, Planar Trailblazer");
        activation(&mut g, k, 0, None);
        settle(&mut g);
        activation(&mut g, k, 1, None);
        settle(&mut g);
        g.state.priority = p.other();
        let stop = named(&mut g, p.other(), "Stop Cold", Zone::Hand);
        cast(&mut g, stop, None, &[Target::Card(k)]);
        settle(&mut g);
        assert!(g.has_subtype(k, "Rogue"));
        assert_eq!(
            g.stats(k),
            Stats {
                power: 3,
                toughness: 2
            }
        );
        assert!(!g.keywords(k).contains(Keywords::DOUBLE_STRIKE));
        assert!(!g.object_view(k).grant_available(
            g.object_ref(k),
            g.state.objects[k as usize].card,
            0,
            None
        ));
    }
}
#[test]
fn fdn_growth_greatwurm_actual_combat_player_creature_dead_dealer_and_other_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let w = paid(&mut g, p, "Quilled Greatwurm");
        let a = paid(&mut g, p, "Bear Cub");
        connect(&mut g, a);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        let event = g
            .state
            .stack
            .iter()
            .find(|s| s.object.card == w)
            .unwrap()
            .creature_combat_event
            .as_ref()
            .unwrap();
        assert_eq!((event.dealer, event.amount), (g.object_ref(a), 2));
        settle(&mut g);
        assert_eq!(g.counter_count(a, CounterKind::PlusOne), 2);
        let mut g = setup(p);
        let _w = paid(&mut g, p, "Quilled Greatwurm");
        let a = paid(&mut g, p, "Bear Cub");
        let b = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        combat_to_blockers(&mut g, a);
        checked(
            &mut g,
            Action::Block {
                blocker: b,
                attacker: a,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert_eq!(g.state.objects[a as usize].zone, Zone::Graveyard);
        assert!(g.state.stack.iter().any(|s| {
            s.creature_combat_event
                .as_ref()
                .is_some_and(|e| e.dealer.card == a && e.amount == 2)
        }));
        settle(&mut g);
        assert_eq!(g.counter_count(a, CounterKind::PlusOne), 0);
        let mut g = setup(p);
        let _w = paid(&mut g, p, "Quilled Greatwurm");
        let a = paid(&mut g, p, "Bear Cub");
        let b = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        combat_to_blockers(&mut g, b);
        checked(
            &mut g,
            Action::Block {
                blocker: a,
                attacker: b,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert!(
            !g.state
                .stack
                .iter()
                .any(|s| s.creature_combat_event.is_some())
        );
        settle(&mut g);
        assert_eq!(g.counter_count(a, CounterKind::PlusOne), 0);
    }
}
#[test]
fn fdn_growth_greatwurm_counter_and_mana_resources_cannot_be_spent_twice() {
    for p in [Player::P0, Player::P1] {
        for count in [10, 11] {
            let mut g = setup(p);
            let r = paid(&mut g, p, "Ramos, Dragon Engine");
            g.add_counter(r, CounterKind::PlusOne, count);
            let w = named(&mut g, p, "Quilled Greatwurm", Zone::Graveyard);
            g.state.players[p.index()].mana = [0; 6];
            let announce = Action::AnnounceCounterGraveyard { card: w };
            if count == 10 {
                reject(&mut g, announce);
                continue;
            }
            checked(&mut g, announce);
            let object = g.object_ref(r);
            for _ in 0..6 {
                checked(
                    &mut g,
                    Action::ChooseCreatureCounterCost {
                        object,
                        counter: CounterKind::PlusOne,
                    },
                );
            }
            let mana = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a,Action::ActivateMana{card,..}if *card==r))
                .unwrap();
            checked(&mut g, mana);
            assert_eq!(g.counter_count(r, CounterKind::PlusOne), 6);
            let pay = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap();
            checked(&mut g, pay);
            assert_eq!(g.counter_count(r, CounterKind::PlusOne), 0);
            settle(&mut g);
            assert_eq!(g.state.objects[w as usize].zone, Zone::Battlefield);
        }
    }
}
#[test]
fn fdn_growth_preserver_human_opponent_decline_and_paid_reflexive_new_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let w = paid(&mut g, p, "Wildborn Preserver");
        let human = paid(&mut g, p, "Ghitu Lavarunner");
        assert!(g.state.objects[human as usize].card.has_subtype("Human"));
        assert_eq!(g.counter_count(w, CounterKind::PlusOne), 0);
        let e = named(&mut g, p, "Llanowar Elves", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 2];
        cast(&mut g, e, None, &[]);
        for _ in 0..20 {
            if matches!(g.state.pending, Pending::EffectOptional { .. }) {
                break;
            }
            checked(&mut g, Action::Pass);
        }
        checked(&mut g, Action::ChooseOptional { accept: true });
        checked(&mut g, Action::ChooseX { value: 2 });
        let payment = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayResolutionMana { .. }))
            .unwrap();
        checked(&mut g, payment);
        g.move_object(w, Zone::Hand);
        g.move_object(w, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.counter_count(w, CounterKind::PlusOne), 0);
    }
}
#[test]
fn fdn_growth_fishing_grant_cost_respects_sickness_controller_and_exact_pole() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let pole = paid(&mut g, p, "Fishing Pole");
        let c = paid(&mut g, p, "Bear Cub");
        activation(&mut g, pole, 0, Some(Target::Card(c)));
        settle(&mut g);
        let grantor = g.object_ref(pole);
        reject(
            &mut g,
            Action::AnnounceAttachedAbility {
                grantor,
                ability: 1,
            },
        );
        g.state.objects[c as usize].summoning_sick = false;
        checked(
            &mut g,
            Action::AnnounceAttachedAbility {
                grantor,
                ability: 1,
            },
        );
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, a);
        g.move_object(pole, Zone::Hand);
        g.move_object(pole, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.counter_count(pole, CounterKind::Bait), 0);
    }
}
#[test]
fn fdn_growth_ovika_freezes_x_spell_value_and_token_doubling_after_source_leaves() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let o = paid(&mut g, p, "Ovika, Enigma Goliath");
        let _season = paid(&mut g, p, "Doubling Season");
        let goblin = Card::from_name("@token/phyrexian_goblin_1_1_red").unwrap();
        let earlier: Vec<_> = tokens(&g, p, goblin)
            .into_iter()
            .map(|id| g.object_ref(id))
            .collect();
        assert_eq!(
            earlier.len(),
            5,
            "casting Doubling Season triggers Ovika before Season resolves"
        );
        // The completed cast has mana value four, even after the spell and
        // observer have left. Every replacement-created token gets the effect.
        let spell = named(&mut g, p, "Mind Spring", Zone::Hand);
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 2];
        checked(&mut g, Action::AnnounceCast { card: spell });
        checked(&mut g, Action::ChooseX { value: 2 });
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        let packet = g
            .state
            .stack
            .iter()
            .find(|s| s.source_card == g.state.objects[o as usize].card)
            .unwrap();
        assert_eq!(packet.cast_event.as_ref().unwrap().mana_value, 4);
        let spell_id = g
            .state
            .stack
            .iter()
            .find(|s| s.kind == StackKind::Spell && s.card == spell)
            .unwrap()
            .id;
        assert!(g.counter_stack_item(spell_id, false));
        g.move_object(o, Zone::Hand);
        settle(&mut g);
        let made = tokens(
            &g,
            p,
            Card::from_name("@token/phyrexian_goblin_1_1_red").unwrap(),
        );
        assert_eq!(made.len(), 13);
        assert!(
            earlier
                .iter()
                .all(|r| made.contains(&r.card) && g.object_ref(r.card) == *r)
        );
        let newly_created: Vec<_> = made
            .iter()
            .copied()
            .filter(|id| !earlier.contains(&g.object_ref(*id)))
            .collect();
        assert_eq!(newly_created.len(), 8);
        assert!(
            made.iter()
                .all(|&id| g.keywords(id).contains(Keywords::HASTE))
        );
        g.finish_cleanup();
        assert!(
            made.iter()
                .all(|&id| !g.keywords(id).contains(Keywords::HASTE))
        );
    }
}
