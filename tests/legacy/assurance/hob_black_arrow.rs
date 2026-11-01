//! Actual damage, post-prevention Dragon destruction and paid Equipment wiring.
use super::*;
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..64 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Black Arrow decision budget: {:?}", g.state.pending);
}
fn drive(g: &mut Game) {
    until(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
    });
}
fn start(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "The Black Arrow", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
    cast(g, id, None);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    until(g, |g| matches!(g.state.pending, Pending::Targets { .. }));
    id
}
fn bind(g: &mut Game, target: Target) {
    act(g, Action::ChooseTarget { target });
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}
fn equip(g: &mut Game, p: Player, arrow: CardId, target: CardId) {
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
    act(
        g,
        Action::AnnounceActivate {
            card: arrow,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    bind(g, Target::Card(target));
    pay(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}
#[test]
fn black_arrow_paid_flash_entry_any_player_creature_planeswalker_and_mandatory_legality() {
    for p in [Player::P0, Player::P1] {
        for row in 0..3 {
            let mut g = scenario(p);
            let bear = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let walker = put(
                &mut g,
                p.other(),
                "Ajani, Caller of the Pride",
                Zone::Battlefield,
            );
            g.state.objects[walker as usize].counters.loyalty = 5;
            let land = put(&mut g, p, "Forest", Zone::Battlefield);
            g.state.active = p.other();
            g.state.step = Step::End;
            g.state.priority = p;
            let arrow = start(&mut g, p);
            assert!(g.keywords(arrow).contains(Keywords::FLASH));
            assert_eq!(g.object_view(arrow).zone, Zone::Battlefield);
            for id in [land, arrow] {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(id),
                    },
                );
            }
            reject(&mut g, Action::FinishTargets);
            bind(
                &mut g,
                match row {
                    0 => Target::Player(p.other()),
                    1 => Target::Card(bear),
                    _ => Target::Card(walker),
                },
            );
            drive(&mut g);
            assert_eq!(
                g.state.players[p.other().index()].life,
                if row == 0 { 19 } else { 20 }
            );
            assert_eq!(
                g.state.objects[bear as usize].damage,
                if row == 1 { 1 } else { 0 }
            );
            assert_eq!(
                g.counter_count(walker, CounterKind::Loyalty),
                if row == 2 { 4 } else { 5 }
            );
        }
    }
}
#[test]
fn black_arrow_actual_positive_damage_destroys_dragon_but_shield_and_indestructible_differ() {
    for p in [Player::P0, Player::P1] {
        for row in 0..3 {
            let mut g = scenario(p);
            let dragon = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
            // Existing damage must not qualify a subsequently prevented Arrow.
            let shock = put(&mut g, p, "Shock", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
            cast(&mut g, shock, Some(Target::Card(dragon)));
            drive(&mut g);
            assert_eq!(g.state.objects[dragon as usize].damage, 2);
            if row == 1 {
                g.add_counter(dragon, CounterKind::Shield, 1);
            }
            if row == 2 {
                g.add_counter(dragon, CounterKind::Indestructible, 1);
            }
            start(&mut g, p);
            bind(&mut g, Target::Card(dragon));
            drive(&mut g);
            assert_eq!(
                g.object_view(dragon).zone,
                if row == 0 {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            if row == 1 {
                assert_eq!(g.state.objects[dragon as usize].damage, 2);
                assert_eq!(g.counter_count(dragon, CounterKind::Shield), 0);
            }
            if row == 2 {
                assert_eq!(g.state.objects[dragon as usize].damage, 3);
            }
        }
    }
}
#[test]
fn black_arrow_token_dragon_dies_and_source_departure_does_not_cancel_queued_damage() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let dragon = put(&mut g, p.other(), "@token/hob-dragon", Zone::Battlefield);
        let arrow = start(&mut g, p);
        bind(&mut g, Target::Card(dragon));
        g.move_object(arrow, Zone::Graveyard);
        g.give_priority();
        drive(&mut g);
        assert_eq!(g.object_view(dragon).zone, Zone::Outside);
        assert_eq!(g.object_view(arrow).zone, Zone::Graveyard);
    }
}
#[test]
fn black_arrow_stale_recipient_is_never_damaged_or_destroyed_after_owned_restore() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let dragon = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
        start(&mut g, p);
        bind(&mut g, Target::Card(dragon));
        let saved = g.snapshot();
        let mut valid = g.clone();
        let mut stale = g.clone();
        drop(g);
        drive(&mut valid);
        assert_eq!(valid.object_view(dragon).zone, Zone::Graveyard);
        stale.move_object(dragon, Zone::Hand);
        stale.move_object(dragon, Zone::Battlefield);
        stale.give_priority();
        let fresh = stale.object_ref(dragon);
        drive(&mut stale);
        assert_eq!(stale.object_ref(dragon), fresh);
        assert_eq!(stale.object_view(dragon).zone, Zone::Battlefield);
        assert_eq!(stale.state.objects[dragon as usize].damage, 0);
        stale.restore(&saved).unwrap();
        drive(&mut stale);
        assert_eq!(stale.snapshot(), valid.snapshot());
    }
}
#[test]
fn black_arrow_damage_replacement_choice_owns_tracker_and_never_destroys_prevented_dragon() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let dragon = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
        g.add_counter(dragon, CounterKind::Shield, 1);
        put(&mut g, p, "Twinflame Tyrant", Zone::Battlefield);
        start(&mut g, p);
        bind(&mut g, Target::Card(dragon));
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Replacement { .. })
        });
        assert_eq!(g.actor(), Some(p.other()));
        contracts::branch_contract(g.clone());
        let saved = g.snapshot();
        let mut shield = g.clone();
        let mut doubled = g.clone();
        drop(g);
        choose(&mut shield, |a| {
            matches!(a, Action::ChooseDamagePrevention { .. })
        });
        drive(&mut shield);
        choose(&mut doubled, |a| {
            matches!(a, Action::ChooseReplacement { .. })
        });
        drive(&mut doubled);
        for child in [&shield, &doubled] {
            assert_eq!(child.object_view(dragon).zone, Zone::Battlefield);
            assert_eq!(child.state.objects[dragon as usize].damage, 0);
            assert_eq!(child.counter_count(dragon, CounterKind::Shield), 0);
        }
        let expected = shield.snapshot();
        doubled.restore(&saved).unwrap();
        choose(&mut doubled, |a| {
            matches!(a, Action::ChooseDamagePrevention { .. })
        });
        drive(&mut doubled);
        assert_eq!(doubled.snapshot(), expected);
    }
}
#[test]
fn black_arrow_paid_equip_controller_sorcery_reach_stats_source_and_target_departure() {
    for p in [Player::P0, Player::P1] {
        for stale in [false, true] {
            let mut g = scenario(p);
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let land = put(&mut g, p, "Forest", Zone::Battlefield);
            let arrow = start(&mut g, p);
            bind(&mut g, Target::Player(p.other()));
            drive(&mut g);
            let announce = Action::AnnounceActivate {
                card: arrow,
                ability: 0,
                counter_payment: Counters::default(),
            };
            g.state.active = p.other();
            g.state.priority = p;
            reject(&mut g, announce.clone());
            g.state.active = p;
            g.state.step = Step::BeginCombat;
            reject(&mut g, announce.clone());
            g.state.step = Step::Main1;
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
            act(&mut g, announce);
            for id in [enemy, land] {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(id),
                    },
                );
            }
            bind(&mut g, Target::Card(own));
            pay(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            if stale {
                g.move_object(own, Zone::Hand);
                g.move_object(own, Zone::Battlefield);
                g.give_priority();
            }
            drive(&mut g);
            assert_eq!(stats(&g, own), if stale { (2, 2) } else { (3, 3) });
            assert_eq!(g.keywords(own).contains(Keywords::REACH), !stale);
            if stale {
                equip(&mut g, p, arrow, own);
                drive(&mut g);
            }
            let flyer = put(&mut g, p.other(), "Ravenhill Flock", Zone::Battlefield);
            g.state.active = p.other();
            assert!(g.can_block_pair(own, flyer));
            g.move_object(arrow, Zone::Graveyard);
            assert_eq!(stats(&g, own), (2, 2));
            assert!(!g.can_block_pair(own, flyer));
        }
    }
}
#[test]
fn black_arrow_shared_tracker_preserves_actual_goblin_negotiation_excess_tokens() {
    for p in [Player::P0, Player::P1] {
        for shield in [false, true] {
            let mut g = scenario(p);
            let target = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            if shield {
                g.add_counter(target, CounterKind::Shield, 1);
            }
            let spell = put(&mut g, p, "Goblin Negotiation", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 2, 0, 5];
            act(&mut g, Action::AnnounceCast { card: spell });
            act(&mut g, Action::ChooseX { value: 5 });
            bind(&mut g, Target::Card(target));
            pay(&mut g);
            drive(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            let goblins: Vec<_> = g
                .state
                .objects
                .iter()
                .enumerate()
                .filter_map(|(i, o)| {
                    (o.zone == Zone::Battlefield
                        && o.controller == p
                        && o.card.is_token()
                        && o.card.has_subtype("Goblin"))
                    .then_some(i as CardId)
                })
                .collect();
            assert_eq!(goblins.len(), if shield { 0 } else { 2 });
            for id in goblins {
                assert_eq!(stats(&g, id), (1, 1));
                assert_eq!(g.state.objects[id as usize].card.colors(), 8);
            }
            assert_eq!(
                g.object_view(target).zone,
                if shield {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
        }
    }
}
