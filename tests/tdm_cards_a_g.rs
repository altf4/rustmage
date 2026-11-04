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

// Initial authorship runs keep the candidate out of public deck admission.
// Install its actual immutable definition into a hand fixture before any source
// registration; every subsequent land play and ability is the ordinary rules path.
// Once admitted, this helper is exactly the shared supported-card fixture.
fn authored_land_hand(g: &mut Game, p: Player, name: &str) -> CardId {
    let card = Card::from_name(name).unwrap();
    if card.is_supported() {
        return put(g, p, name, Zone::Hand);
    }
    assert_eq!(name, "Dalkovan Encampment");
    let id = put(g, p, "Plains", Zone::Hand);
    g.state.objects[id as usize].card = card;
    id
}

fn one_mana(color: usize) -> Mana {
    let mut value = [0; 6];
    value[color] = 1;
    value
}
fn mana_action(card: CardId, color: usize) -> Action {
    Action::ActivateMana {
        card,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production: one_mana(color),
    }
}
fn activate_plain(g: &mut Game, card: CardId, ability: u16) {
    act(
        g,
        Action::AnnounceActivate {
            card,
            ability,
            counter_payment: Counters::default(),
        },
    );
    pay(g);
}
fn battlefield_tokens(g: &Game) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| o.zone == Zone::Battlefield && o.is_token())
        .map(|(id, _)| id as CardId)
        .collect()
}

#[test]
fn lands_frontier_bivouac_enters_tapped_and_makes_exactly_each_printed_color() {
    for p in [Player::P0, Player::P1] {
        for color in [4, 1, 3] {
            let mut g = scenario(p);
            let land = put(&mut g, p, "Frontier Bivouac", Zone::Hand);
            g.state.players[p.index()].mana = [0; 6];
            act(&mut g, Action::PlayLand { card: land });
            assert!(g.object_view(land).tapped);
            reject(&mut g, mana_action(land, color));
            drain(&mut g);
            assert_eq!(g.state.players[p.index()].life, 20);
            g.state.objects[land as usize].tapped = false;
            reject(&mut g, mana_action(land, 0));
            reject(&mut g, mana_action(land, 2));
            act(&mut g, mana_action(land, color));
            assert_eq!(g.state.players[p.index()].mana, one_mana(color));
            assert!(g.object_view(land).tapped);
        }
    }
}

#[test]
fn lands_evolving_wilds_pays_tap_and_sacrifice_searches_only_own_basic_and_shuffles_on_decline() {
    for p in [Player::P0, Player::P1] {
        for find in [false, true] {
            let mut g = scenario(p);
            let wilds = put(&mut g, p, "Evolving Wilds", Zone::Hand);
            act(&mut g, Action::PlayLand { card: wilds });
            assert!(!g.object_view(wilds).tapped);
            let basic = put(&mut g, p, "Mountain", Zone::Library);
            let nonbasic = put(&mut g, p, "Frontier Bivouac", Zone::Library);
            let foreign = put(&mut g, p.other(), "Island", Zone::Library);
            activate_plain(&mut g, wilds, 0);
            assert_eq!(g.object_view(wilds).zone, Zone::Graveyard);
            ready(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectSearch { .. })
            });
            assert!(g.observe(p).search_choice.is_some());
            assert!(g.observe(p.other()).search_choice.is_none());
            reject(&mut g, Action::ChooseSearch { card: nonbasic });
            reject(&mut g, Action::ChooseSearch { card: foreign });
            let rng_before = g.state.rng;
            if find {
                act(&mut g, Action::ChooseSearch { card: basic });
            }
            if g.legal_actions().contains(&Action::FinishSearch) {
                act(&mut g, Action::FinishSearch);
            }
            drain(&mut g);
            assert_eq!(
                g.object_view(basic).zone,
                if find {
                    Zone::Battlefield
                } else {
                    Zone::Library
                }
            );
            if find {
                assert!(g.object_view(basic).tapped);
            }
            assert_ne!(
                g.state.rng, rng_before,
                "search shuffles even when no card is found"
            );
            assert_eq!(g.object_view(nonbasic).zone, Zone::Library);
        }
    }
}

#[test]
fn lands_clan_entries_use_controlled_subtypes_and_each_mana_ability_is_exact() {
    for p in [Player::P0, Player::P1] {
        for (name, qualifiers, color) in [
            ("Cori Mountain Monastery", ["Plains", "Island"], 3),
            ("Dalkovan Encampment", ["Swamp", "Mountain"], 0),
        ] {
            for (qualifier, foreign, tapped) in [
                (None, false, true),
                (Some(qualifiers[0]), false, false),
                (Some(qualifiers[1]), false, false),
                (Some(qualifiers[0]), true, true),
                (Some("Forest"), false, true),
            ] {
                let mut g = scenario(p);
                if let Some(q) = qualifier {
                    put(
                        &mut g,
                        if foreign { p.other() } else { p },
                        q,
                        Zone::Battlefield,
                    );
                }
                let land = authored_land_hand(&mut g, p, name);
                act(&mut g, Action::PlayLand { card: land });
                assert_eq!(
                    g.object_view(land).tapped,
                    tapped,
                    "{name} {qualifier:?} {foreign}"
                );
                if !tapped {
                    g.state.players[p.index()].mana = [0; 6];
                    act(&mut g, mana_action(land, color));
                    assert_eq!(g.state.players[p.index()].mana, one_mana(color));
                }
            }
        }
    }
}

#[test]
fn lands_cori_mountain_monastery_pays_three_red_taps_and_authorizes_paid_spell_or_land() {
    for p in [Player::P0, Player::P1] {
        for name in ["Bear Cub", "Mountain"] {
            let mut g = scenario(p);
            let land = put(&mut g, p, "Cori Mountain Monastery", Zone::Battlefield);
            let top = put(&mut g, p, name, Zone::Library);
            g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 3];
            activate_plain(&mut g, land, 1);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert!(g.object_view(land).tapped);
            g.move_object(land, Zone::Graveyard);
            drain(&mut g);
            assert_eq!(g.object_view(top).zone, Zone::Exile);
            let permission = g
                .state
                .exile_permissions
                .iter()
                .find(|r| r.object.card == top)
                .unwrap();
            assert_eq!(permission.player, p);
            assert_eq!(permission.expiry, ExilePlayExpiry::EndOfNextTurn);
            if name == "Mountain" {
                act(&mut g, Action::PlayLand { card: top });
            } else {
                reject(&mut g, Action::AnnounceExile { card: top });
                g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
                act(&mut g, Action::AnnounceExile { card: top });
                pay(&mut g);
                drain(&mut g);
            }
            assert_eq!(g.object_view(top).zone, Zone::Battlefield);
        }
    }
}

#[test]
fn lands_cori_permission_lasts_through_next_own_turn_then_expires_and_is_exact_incarnation() {
    let p = Player::P1;
    let mut g = scenario(p);
    let land = put(&mut g, p, "Cori Mountain Monastery", Zone::Battlefield);
    let top = put(&mut g, p, "Bear Cub", Zone::Library);
    activate_plain(&mut g, land, 1);
    drain(&mut g);
    assert!(g.can_play_exiled(p, top));
    let mut stale = g.clone();
    stale.move_object(top, Zone::Hand);
    stale.move_object(top, Zone::Exile);
    assert!(!stale.can_play_exiled(p, top));
    g.state.step = Step::Cleanup;
    g.finish_cleanup();
    assert!(g.can_play_exiled(p, top));
    g.state.active = p.other();
    g.state.turn += 1;
    g.begin_turn();
    assert!(g.can_play_exiled(p, top));
    g.state.active = p;
    g.state.turn += 1;
    g.begin_turn();
    assert!(g.can_play_exiled(p, top));
    g.state.step = Step::Cleanup;
    g.finish_cleanup();
    assert!(!g.can_play_exiled(p, top));
}

#[test]
fn lands_dalkovan_attack_callback_creates_two_tapped_attacking_red_warriors_after_land_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Swamp", Zone::Battlefield);
        let land = authored_land_hand(&mut g, p, "Dalkovan Encampment");
        act(&mut g, Action::PlayLand { card: land });
        let attacker = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 2];
        activate_plain(&mut g, land, 1);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert!(g.object_view(land).tapped);
        drain(&mut g);
        g.move_object(land, Zone::Graveyard);
        attack(&mut g, attacker);
        drain(&mut g);
        let tokens = battlefield_tokens(&g);
        assert_eq!(tokens.len(), 2);
        assert_eq!(g.state.attackers.len(), 3);
        for token in &tokens {
            assert_eq!(stats(&g, *token), (1, 1));
            assert!(g.object_view(*token).tapped && g.object_view(*token).attacking);
            assert_eq!(g.object_view(*token).controller, p);
            assert_eq!(g.object_view(*token).colors, 8);
            assert!(g.has_subtype(*token, "Warrior"));
        }
        // Finish this combat, then enter a second combat in the same turn.
        ready(&mut g, |g| {
            g.state.step == Step::Main2 && g.state.pending == Pending::Priority
        });
        g.state.objects[attacker as usize].tapped = false;
        g.state.attack_declared = false;
        g.state.attackers.clear();
        g.state.selected_attackers.clear();
        attack(&mut g, attacker);
        drain(&mut g);
        assert_eq!(
            battlefield_tokens(&g).len(),
            4,
            "registration triggers on each attack this turn"
        );
        ready(&mut g, |g| {
            g.state.step == Step::Main2 && g.state.pending == Pending::Priority
        });
        g.begin_step(Step::End);
        g.give_priority();
        drain(&mut g);
        assert!(
            battlefield_tokens(&g).is_empty(),
            "all four exact tokens sacrificed at next end step"
        );
        assert_eq!(g.object_view(attacker).zone, Zone::Battlefield);
    }
}
