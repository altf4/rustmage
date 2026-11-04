//! Actual Tarkir: Dragonstorm A–G definitions and their printed outcomes.
mod support;
use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

fn activate_target(g: &mut Game, source: CardId, ability: u16, target: CardId) {
    act(
        g,
        Action::AnnounceActivate {
            card: source,
            ability,
            counter_payment: Counters::default(),
        },
    );
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    pay(g);
}
fn progress(g: &mut Game) {
    match g.state.pending {
        Pending::Priority => act(g, Action::Pass),
        Pending::OrderTriggers { .. } => choose(g, |a| matches!(a, Action::PutTrigger { .. })),
        Pending::Attackers => act(g, Action::FinishAttackers),
        Pending::Blockers => act(g, Action::FinishBlockers),
        _ => panic!("explicit choice required: {:?}", g.state.pending),
    }
}
fn ready(g: &mut Game, predicate: impl Fn(&Game) -> bool) {
    for _ in 0..96 {
        if predicate(g) {
            return;
        }
        progress(g);
    }
    panic!("96-action scenario budget exhausted: {:?}", g.state.pending);
}
fn drain(g: &mut Game) {
    ready(g, |g| {
        g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending == Pending::Priority
    });
}
fn attack(g: &mut Game, source: CardId) {
    assert_eq!(g.state.pending, Pending::Priority);
    assert!(g.state.stack.is_empty());
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    g.state.priority = g.state.active;
    act(g, Action::Attack { card: source });
    act(g, Action::FinishAttackers);
}
fn renew_cost(name: &str) -> Mana {
    match name {
        "Adorned Crocodile" => [0, 0, 1, 0, 0, 0],
        "Agent of Kotis" => [0, 1, 0, 0, 0, 3],
        "Alchemist's Assistant" => [0, 0, 1, 0, 0, 1],
        "Champion of Dusan" => [0, 0, 0, 0, 1, 1],
        "Constrictor Sage" => [0, 1, 0, 0, 0, 2],
        _ => unreachable!(),
    }
}
const RENEW: &[(&str, &[(CounterKind, u64)])] = &[
    ("Adorned Crocodile", &[(CounterKind::PlusOne, 1)]),
    ("Agent of Kotis", &[(CounterKind::PlusOne, 2)]),
    ("Alchemist's Assistant", &[(CounterKind::Lifelink, 1)]),
    (
        "Champion of Dusan",
        &[(CounterKind::PlusOne, 1), (CounterKind::Trample, 1)],
    ),
    ("Constrictor Sage", &[(CounterKind::Stun, 1)]),
];

#[test]
fn renew_each_actual_definition_pays_printed_cost_exiles_source_and_places_exact_counters() {
    for p in [Player::P0, Player::P1] {
        for &(name, counters) in RENEW {
            let mut g = scenario(p);
            let source = put(&mut g, p, name, Zone::Graveyard);
            let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.state.players[p.index()].mana = renew_cost(name);
            activate_target(&mut g, source, 0, target);
            assert_eq!(
                g.object_view(source).zone,
                Zone::Exile,
                "{name}: cost is paid at activation"
            );
            assert_eq!(
                g.state.players[p.index()].mana,
                [0; 6],
                "{name}: exact printed mana cost"
            );
            drain(&mut g);
            for &(kind, amount) in counters {
                assert_eq!(g.counter_count(target, kind), amount, "{name}: {kind:?}");
            }
            match name {
                "Alchemist's Assistant" => assert!(g.keywords(target).contains(Keywords::LIFELINK)),
                "Champion of Dusan" => {
                    assert_eq!(stats(&g, target), (3, 3));
                    assert!(g.keywords(target).contains(Keywords::TRAMPLE));
                }
                "Constrictor Sage" => {
                    assert!(g.object_view(target).tapped);
                    g.state.active = p.other();
                    g.begin_turn();
                    assert!(
                        g.object_view(target).tapped,
                        "stun replaces one real turn untap"
                    );
                    assert_eq!(g.counter_count(target, CounterKind::Stun), 0);
                    g.begin_turn();
                    assert!(!g.object_view(target).tapped);
                }
                _ => (),
            }
        }
    }
}

#[test]
fn renew_timing_wrong_zone_and_unaffordable_costs_are_atomic() {
    for &(name, _) in RENEW {
        for zone in [Zone::Hand, Zone::Battlefield, Zone::Graveyard] {
            let mut g = scenario(Player::P0);
            let source = put(&mut g, Player::P0, name, zone);
            put(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
            if zone == Zone::Graveyard {
                g.state.step = Step::BeginCombat;
            }
            reject(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
        }
        let mut g = scenario(Player::P0);
        let source = put(&mut g, Player::P0, name, Zone::Graveyard);
        put(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
        g.state.players[0].mana = [0; 6];
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
    }
}

#[test]
fn renew_paid_cost_survives_a_blinked_target_and_never_rebinds_it() {
    for &(name, counters) in RENEW {
        let mut g = scenario(Player::P0);
        let source = put(&mut g, Player::P0, name, Zone::Graveyard);
        let target = put(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
        activate_target(&mut g, source, 0, target);
        let old = g.object_ref(target);
        g.move_object(target, Zone::Exile);
        g.move_object(target, Zone::Battlefield);
        assert_ne!(g.object_ref(target), old);
        drain(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Exile, "{name}");
        for &(kind, _) in counters {
            assert_eq!(g.counter_count(target, kind), 0, "{name}");
        }
        assert!(!g.object_view(target).tapped);
    }
}

#[test]
fn renew_owned_pending_targets_raw_index_and_restore_remain_independent() {
    let mut parent = scenario(Player::P0);
    let source = put(
        &mut parent,
        Player::P0,
        "Champion of Dusan",
        Zone::Graveyard,
    );
    let own = put(&mut parent, Player::P0, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut parent, Player::P1, "Bear Cub", Zone::Battlefield);
    act(
        &mut parent,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    reject(
        &mut parent,
        Action::ChooseTarget {
            target: Target::Player(Player::P1),
        },
    );
    let action = Action::ChooseTarget {
        target: Target::Card(own),
    };
    let index = parent
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let saved = parent.snapshot();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let mut sibling = parent.clone();
    act(&mut raw, action.clone());
    indexed
        .apply_index(indexed.actor().unwrap(), indexed.revision(), index)
        .unwrap();
    act(
        &mut sibling,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    drop(parent);
    for child in [&mut raw, &mut indexed, &mut sibling] {
        pay(child);
        drain(child);
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.counter_count(own, CounterKind::Trample), 1);
    assert_eq!(raw.counter_count(enemy, CounterKind::Trample), 0);
    assert_eq!(sibling.counter_count(enemy, CounterKind::Trample), 1);
    assert_eq!(sibling.counter_count(own, CounterKind::Trample), 0);
    indexed.restore(&saved).unwrap();
    act(&mut indexed, action);
    pay(&mut indexed);
    drain(&mut indexed);
    assert_eq!(indexed.snapshot(), raw.snapshot());
}

#[test]
fn renew_native_lifelink_trample_and_granted_lifelink_wire_actual_characteristics() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let assistant = put(&mut g, p, "Alchemist's Assistant", Zone::Hand);
        cast(&mut g, assistant, None);
        drain(&mut g);
        assert_eq!(stats(&g, assistant), (2, 1));
        assert!(g.keywords(assistant).contains(Keywords::LIFELINK));
        g.state.objects[assistant as usize].summoning_sick = false;
        attack(&mut g, assistant);
        ready(&mut g, |g| {
            g.state.step == Step::Main2 && g.state.pending == Pending::Priority
        });
        assert_eq!(g.state.players[p.index()].life, 22);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        g.state.players[p.index()].mana = [8; 6];
        let champion = put(&mut g, p, "Champion of Dusan", Zone::Hand);
        cast(&mut g, champion, None);
        drain(&mut g);
        assert_eq!(stats(&g, champion), (4, 2));
        assert!(g.keywords(champion).contains(Keywords::TRAMPLE));
        let donor = put(&mut g, p, "Alchemist's Assistant", Zone::Graveyard);
        activate_target(&mut g, donor, 0, champion);
        drain(&mut g);
        let hit = DamageEvent {
            preventable: true,
            source: g.damage_source(champion),
            target: Target::Player(p.other()),
            amount: 4,
            combat: false,
        };
        g.apply_damage_batch(&[hit]);
        assert_eq!(
            g.state.players[p.index()].life,
            26,
            "actual lifelink counter creates life gain"
        );
    }
}

#[test]
fn renew_adorned_crocodile_actual_death_creates_one_black_zombie_druid() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let croc = put(&mut g, p, "Adorned Crocodile", Zone::Hand);
        cast(&mut g, croc, None);
        drain(&mut g);
        assert_eq!(stats(&g, croc), (5, 3));
        let murder = put(&mut g, p, "Murder", Zone::Hand);
        cast(&mut g, murder, Some(Target::Card(croc)));
        drain(&mut g);
        assert_eq!(g.object_view(croc).zone, Zone::Graveyard);
        let tokens: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| o.zone == Zone::Battlefield && o.is_token())
            .map(|(id, _)| id as CardId)
            .collect();
        assert_eq!(tokens.len(), 1);
        let token = tokens[0];
        assert_eq!(stats(&g, token), (2, 2));
        assert_eq!(g.object_view(token).controller, p);
        assert_eq!(g.object_view(token).colors, 4);
        assert!(g.has_subtype(token, "Zombie") && g.has_subtype(token, "Druid"));
    }
}

#[test]
fn renew_constrictor_sage_enters_targets_only_an_opponent_and_stuns_once() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let sage = put(&mut g, p, "Constrictor Sage", Zone::Hand);
        cast(&mut g, sage, None);
        ready(&mut g, |g| {
            matches!(g.state.pending, Pending::Targets { .. })
        });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        drain(&mut g);
        assert_eq!(stats(&g, sage), (4, 4));
        assert!(g.object_view(enemy).tapped);
        assert_eq!(g.counter_count(enemy, CounterKind::Stun), 1);
        assert!(!g.object_view(own).tapped);
    }
}

#[test]
fn renew_agent_of_kotis_casts_as_a_two_one() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let agent = put(&mut g, p, "Agent of Kotis", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 1];
        cast(&mut g, agent, None);
        drain(&mut g);
        assert_eq!(g.object_view(agent).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(stats(&g, agent), (2, 1));
    }
}
