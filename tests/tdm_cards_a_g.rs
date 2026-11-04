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

// Initial gated authorship installs the actual immutable card before any event.
fn authored_spell(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    if card.is_supported() {
        return put(g, p, name, zone);
    }
    assert!(
        [
            "Aggressive Negotiations",
            "Caustic Exhale",
            "Channeled Dragonfire",
            "Desperate Measures",
            "Dragon's Prey",
            "Abzan Monument",
            "Aegis Sculptor",
            "Ainok Wayfarer",
            "All-Out Assault",
            "Ambling Stormshell",
            "Armament Dragon",
            "Attuned Hunter",
            "Barrensteppe Siege",
            "Betor, Kin to All",
            "Bone-Cairn Butcher",
            "Breaching Dragonstorm",
            "Call the Spirit Dragons",
        ]
        .contains(&name)
    );
    let id = put(g, p, "Plains", Zone::Hand);
    g.state.objects[id as usize].card = card;
    if zone != Zone::Hand {
        g.move_object(id, zone);
    }
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

fn spell_card(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    let id = authored_spell(g, p, name, Zone::Hand);
    if name == "Defibrillating Current" {
        act(g, Action::AnnounceCast { card: id });
        // Each exact pool in the twobrid table funds only its corresponding
        // printed route. The general damage case uses the ordinary generic route.
        choose(g, |a| matches!(a, Action::ChooseSpellCost { .. }));
        for &target in targets {
            act(g, Action::ChooseTarget { target });
        }
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(g, Action::FinishTargets);
        }
        pay(g);
    } else {
        announce(g, id, targets);
    }
    id
}
fn modal_card(g: &mut Game, p: Player, name: &str, mode: u16, targets: &[Target]) -> CardId {
    let id = authored_spell(g, p, name, Zone::Hand);
    act(g, Action::AnnounceCast { card: id });
    act(g, Action::ChooseMode { mode });
    for &target in targets {
        act(g, Action::ChooseTarget { target });
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
    id
}
fn clause(g: &mut Game, clause: CastCostClause) {
    let index = g
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|r| r.clause == clause && !r.paid)
        .unwrap();
    act(
        g,
        Action::ChooseCastCost {
            clause: index as u16,
        },
    );
}
fn hand_pick(g: &mut Game, card: CardId) {
    let object = g.object_ref(card);
    act(
        g,
        Action::ChooseHandCard {
            object: Some(object),
        },
    );
}
fn cleanup(g: &mut Game) {
    assert_eq!(g.state.pending, Pending::Priority);
    assert!(g.state.stack.is_empty());
    g.state.step = Step::Cleanup;
    g.finish_cleanup();
}

#[test]
fn spells_aggressive_negotiations_reveals_nonland_exile_and_optional_own_counter() {
    for p in [Player::P0, Player::P1] {
        for counter in [false, true] {
            let mut g = scenario(p);
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let chosen = put(&mut g, p.other(), "Shock", Zone::Hand);
            let land = put(&mut g, p.other(), "Mountain", Zone::Hand);
            assert!(!g.observe(p).objects.iter().any(|o| o.id == chosen));
            let targets = if counter {
                vec![Target::Player(p.other()), Target::Card(own)]
            } else {
                vec![Target::Player(p.other())]
            };
            spell_card(&mut g, p, "Aggressive Negotiations", &targets);
            ready(&mut g, |g| {
                matches!(g.state.pending, Pending::HandCardChoice { .. })
            });
            for viewer in [p, p.other()] {
                assert!(g.observe(viewer).known_hand.contains(&g.object_ref(chosen)));
                assert!(g.observe(viewer).known_hand.contains(&g.object_ref(land)));
            }
            assert!(g.observe(p.other()).hand_choice.is_none());
            let object = g.object_ref(land);
            reject(
                &mut g,
                Action::ChooseHandCard {
                    object: Some(object),
                },
            );
            hand_pick(&mut g, chosen);
            drain(&mut g);
            assert_eq!(g.object_view(chosen).zone, Zone::Exile);
            assert_eq!(g.object_view(land).zone, Zone::Hand);
            assert_eq!(
                g.counter_count(own, CounterKind::PlusOne),
                u64::from(counter)
            );
        }
    }
}

#[test]
fn spells_aggressive_negotiations_targets_only_opponent_and_own_creature_and_groups_are_independent()
 {
    let p = Player::P0;
    for invalidate_player in [false, true] {
        let mut g = scenario(p);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let chosen = put(&mut g, p.other(), "Shock", Zone::Hand);
        let spell = authored_spell(&mut g, p, "Aggressive Negotiations", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: spell });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        if invalidate_player {
            g.state.players[p.other().index()].hexproof_until_end_turn = true;
        } else {
            g.move_object(own, Zone::Exile);
            g.move_object(own, Zone::Battlefield);
        }
        if !invalidate_player {
            ready(&mut g, |g| {
                matches!(g.state.pending, Pending::HandCardChoice { .. })
            });
            hand_pick(&mut g, chosen);
        }
        drain(&mut g);
        assert_eq!(
            g.object_view(chosen).zone,
            if invalidate_player {
                Zone::Hand
            } else {
                Zone::Exile
            }
        );
        assert_eq!(
            g.counter_count(own, CounterKind::PlusOne),
            u64::from(invalidate_player)
        );
    }
    let mut g = scenario(p);
    let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    put(&mut g, p.other(), "Mountain", Zone::Hand);
    spell_card(
        &mut g,
        p,
        "Aggressive Negotiations",
        &[Target::Player(p.other()), Target::Card(own)],
    );
    drain(&mut g);
    assert_eq!(
        g.counter_count(own, CounterKind::PlusOne),
        1,
        "no nonland to exile does not prevent counter"
    );
}

#[test]
fn spells_caustic_exhale_beholds_hand_or_permanent_or_pays_one_and_gives_minus_three() {
    for p in [Player::P0, Player::P1] {
        for origin in [None, Some(Zone::Hand), Some(Zone::Battlefield)] {
            let mut g = scenario(p);
            let target = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
            let witness = origin.map(|zone| put(&mut g, p, "Shivan Dragon", zone));
            let spell = authored_spell(&mut g, p, "Caustic Exhale", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, u32::from(origin.is_none())];
            act(&mut g, Action::AnnounceCast { card: spell });
            act(
                &mut g,
                Action::ChooseCastBehold {
                    pay: witness.is_some(),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            pay(&mut g);
            if let Some(witness) = witness {
                clause(&mut g, CastCostClause::Behold("Dragon"));
                let object = g.object_ref(witness);
                act(&mut g, Action::ChooseBeholdCost { object });
                assert_eq!(g.object_view(witness).zone, origin.unwrap());
            }
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            drain(&mut g);
            assert_eq!(stats(&g, target), (2, 2));
            cleanup(&mut g);
            assert_eq!(stats(&g, target), (5, 5));
        }
    }
    let mut g = scenario(Player::P0);
    let spell = authored_spell(&mut g, Player::P0, "Caustic Exhale", Zone::Hand);
    put(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    g.state.players[0].mana = [0, 0, 1, 0, 0, 0];
    reject(&mut g, Action::AnnounceCast { card: spell });
}

#[test]
fn spells_channeled_dragonfire_costs_red_deals_two_to_any_creature_or_player() {
    for p in [Player::P0, Player::P1] {
        for player_target in [None, Some(p), Some(p.other())] {
            let mut g = scenario(p);
            let victim = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
            let target = player_target.map_or(Target::Card(victim), Target::Player);
            g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
            let spell = spell_card(&mut g, p, "Channeled Dragonfire", &[target]);
            drain(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            if let Some(player) = player_target {
                assert_eq!(g.state.players[player.index()].life, 18);
            } else {
                assert_eq!(g.state.objects[victim as usize].damage, 2);
            }
        }
    }
}

#[test]
fn spells_channeled_dragonfire_harmonize_five_red_red_optional_power_discount_and_exile() {
    for p in [Player::P0, Player::P1] {
        for help in [false, true] {
            let mut g = scenario(p);
            let helper = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let foreign = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.state.objects[helper as usize].summoning_sick = true;
            let spell = authored_spell(&mut g, p, "Channeled Dragonfire", Zone::Graveyard);
            g.state.players[p.index()].mana = [0, 0, 0, 2, 0, if help { 3 } else { 5 }];
            act(&mut g, Action::AnnounceHarmonize { card: spell });
            let object = g.object_ref(foreign);
            reject(
                &mut g,
                Action::ChooseHarmonizer {
                    object: Some(object),
                },
            );
            let reference = g.object_ref(helper);
            act(
                &mut g,
                Action::ChooseHarmonizer {
                    object: help.then_some(reference),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
            pay(&mut g);
            if help {
                clause(&mut g, CastCostClause::Harmonize(reference));
            }
            assert_eq!(g.object_view(helper).tapped, help);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            drain(&mut g);
            assert_eq!(g.object_view(spell).zone, Zone::Exile);
            assert_eq!(g.state.players[p.other().index()].life, 18);
        }
    }
}

#[test]
fn spells_desperate_measures_delayed_and_immediate_death_respect_controller_and_incarnation() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..5 {
            let mut g = scenario(p);
            let owner = if mode == 2 { p.other() } else { p };
            let target = put(
                &mut g,
                owner,
                if mode == 0 {
                    "Llanowar Elves"
                } else {
                    "Bear Cub"
                },
                Zone::Battlefield,
            );
            spell_card(&mut g, p, "Desperate Measures", &[Target::Card(target)]);
            drain(&mut g);
            if mode == 0 {
                assert_eq!(g.object_view(target).zone, Zone::Graveyard);
            } else {
                assert_eq!(stats(&g, target), (3, 1));
                if mode == 3 {
                    g.record_control(g.object_ref(target), p.other(), None);
                }
                if mode == 4 {
                    g.move_object(target, Zone::Exile);
                    g.move_object(target, Zone::Battlefield);
                }
                let murder = put(&mut g, p, "Murder", Zone::Hand);
                cast(&mut g, murder, Some(Target::Card(target)));
                drain(&mut g);
            }
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                if mode <= 1 { 2 } else { 0 },
                "mode{mode}"
            );
        }
    }
}

#[test]
fn spells_dragons_prey_pays_extra_two_for_effective_dragon_and_destroys_creature() {
    for p in [Player::P0, Player::P1] {
        for dragon in [false, true] {
            let mut g = scenario(p);
            let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            if dragon {
                g.apply_temporary_copy(
                    g.object_ref(target),
                    CopyRecipe::new(Card::from_name("Shivan Dragon").unwrap()),
                    CopyDeadline::EndOfTurn,
                );
            }
            let spell = authored_spell(&mut g, p, "Dragon's Prey", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, if dragon { 4 } else { 2 }];
            act(&mut g, Action::AnnounceCast { card: spell });
            let land = put(&mut g, p.other(), "Forest", Zone::Battlefield);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(land),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            assert_eq!(
                g.observe(p).payment_request.unwrap().generic,
                if dragon { 4 } else { 2 }
            );
            pay(&mut g);
            drain(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Graveyard);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}

#[test]
fn spells_aleshas_legacy_only_own_target_gains_deathtouch_and_indestructible_until_cleanup() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
        let spell = put(&mut g, p, "Alesha's Legacy", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: spell });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        pay(&mut g);
        drain(&mut g);
        assert!(
            g.keywords(own)
                .contains(Keywords::DEATHTOUCH.union(Keywords::INDESTRUCTIBLE))
        );
        let hits = [
            DamageEvent {
                preventable: true,
                source: g.damage_source(own),
                target: Target::Card(enemy),
                amount: 1,
                combat: false,
            },
            DamageEvent {
                preventable: true,
                source: g.damage_source(enemy),
                target: Target::Card(own),
                amount: 5,
                combat: false,
            },
        ];
        g.apply_damage_batch(&hits);
        g.state_based_actions();
        drain(&mut g);
        assert_eq!(g.object_view(own).zone, Zone::Battlefield);
        assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
        let murder = put(&mut g, p, "Murder", Zone::Hand);
        cast(&mut g, murder, Some(Target::Card(own)));
        drain(&mut g);
        assert_eq!(g.object_view(own).zone, Zone::Battlefield);
        cleanup(&mut g);
        assert!(!g.keywords(own).contains(Keywords::DEATHTOUCH));
        assert!(!g.keywords(own).contains(Keywords::INDESTRUCTIBLE));
    }
}

#[test]
fn spells_auroral_procession_returns_any_own_graveyard_card_and_rejects_foreign_or_stale_objects() {
    for p in [Player::P0, Player::P1] {
        for name in ["Mountain", "Bear Cub", "Shock"] {
            let mut g = scenario(p);
            let own = put(&mut g, p, name, Zone::Graveyard);
            let foreign = put(&mut g, p.other(), name, Zone::Graveyard);
            let spell = put(&mut g, p, "Auroral Procession", Zone::Hand);
            g.state.players[p.index()].mana = [0, 1, 0, 0, 1, 0];
            act(&mut g, Action::AnnounceCast { card: spell });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(foreign),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(own),
                },
            );
            pay(&mut g);
            drain(&mut g);
            assert_eq!(g.object_view(own).zone, Zone::Hand);
            assert_eq!(g.object_view(foreign).zone, Zone::Graveyard);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
    let mut g = scenario(Player::P0);
    let old = put(&mut g, Player::P0, "Bear Cub", Zone::Graveyard);
    spell_card(
        &mut g,
        Player::P0,
        "Auroral Procession",
        &[Target::Card(old)],
    );
    g.move_object(old, Zone::Hand);
    g.move_object(old, Zone::Graveyard);
    drain(&mut g);
    assert_eq!(g.object_view(old).zone, Zone::Graveyard);
}

#[test]
fn spells_bewildering_blizzard_draws_three_and_weakens_only_opponents_current_creatures_until_cleanup()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        spell_card(&mut g, p, "Bewildering Blizzard", &[]);
        drain(&mut g);
        assert_eq!(g.hand_count(p), 3);
        assert_eq!(g.library_card_count(p), 5);
        assert_eq!(stats(&g, own), (2, 2));
        assert_eq!(stats(&g, enemy), (0, 3));
        let later = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        assert_eq!(stats(&g, later), (2, 2));
        cleanup(&mut g);
        assert_eq!(stats(&g, enemy), (3, 3));
    }
}

#[test]
fn spells_coordinated_maneuver_both_modes_use_live_count_and_destroy_enchantment() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let removed = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
        modal_card(&mut g, p, "Coordinated Maneuver", 0, &[Target::Card(enemy)]);
        g.move_object(removed, Zone::Hand);
        drain(&mut g);
        assert_eq!(
            g.state.objects[enemy as usize].damage, 1,
            "count sampled at resolution"
        );
        let aura = put(&mut g, p.other(), "Pacifism", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(own));
        modal_card(&mut g, p, "Coordinated Maneuver", 1, &[Target::Card(aura)]);
        drain(&mut g);
        assert_eq!(g.object_view(aura).zone, Zone::Graveyard);
        assert_eq!(g.object_view(own).zone, Zone::Battlefield);
    }
}

#[test]
fn spells_cruel_truths_surveils_two_privately_then_draws_two_and_loses_two_life() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let deeper = put(&mut g, p, "Mountain", Zone::Library);
        let mill = put(&mut g, p, "Island", Zone::Library);
        let keep = put(&mut g, p, "Plains", Zone::Library);
        spell_card(&mut g, p, "Cruel Truths", &[]);
        ready(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectLibrary { .. })
        });
        assert!(g.observe(p).library_choice.is_some());
        assert!(g.observe(p.other()).library_choice.is_none());
        act(
            &mut g,
            Action::ArrangeLibrary {
                card: mill,
                destination: LibraryDestination::Graveyard,
            },
        );
        act(
            &mut g,
            Action::ArrangeLibrary {
                card: keep,
                destination: LibraryDestination::Top,
            },
        );
        drain(&mut g);
        assert_eq!(g.object_view(mill).zone, Zone::Graveyard);
        assert_eq!(g.object_view(keep).zone, Zone::Hand);
        assert_eq!(g.object_view(deeper).zone, Zone::Hand);
        assert_eq!(g.hand_count(p), 2);
        assert_eq!(g.state.players[p.index()].life, 18);
        assert_eq!(g.state.players[p.other().index()].life, 20);
    }
}

#[test]
fn spells_death_begets_life_destroys_creatures_and_enchantments_counts_tokens_not_indestructible() {
    for p in [Player::P0, Player::P1] {
        for tokens in [false, true] {
            let mut g = scenario(p);
            let protected = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let aura = put(&mut g, p.other(), "Pacifism", Zone::Battlefield);
            g.state.objects[aura as usize].attached_to = Some(g.object_ref(enemy));
            let artifact = put(&mut g, p, "Short Bow", Zone::Battlefield);
            let land = put(&mut g, p, "Forest", Zone::Battlefield);
            spell_card(&mut g, p, "Alesha's Legacy", &[Target::Card(protected)]);
            drain(&mut g);
            if tokens {
                modal_card(&mut g, p, "Frontline Rush", 0, &[]);
                drain(&mut g);
            }
            spell_card(&mut g, p, "Death Begets Life", &[]);
            drain(&mut g);
            for id in [own, enemy, aura] {
                assert_eq!(g.object_view(id).zone, Zone::Graveyard);
            }
            for id in [protected, artifact, land] {
                assert_eq!(g.object_view(id).zone, Zone::Battlefield);
            }
            assert!(battlefield_tokens(&g).is_empty());
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                if tokens { 5 } else { 3 }
            );
            assert_eq!(g.hand_count(p), if tokens { 5 } else { 3 });
        }
    }
}

#[test]
fn spells_defibrillating_current_all_eight_twobrid_payments_deal_four_and_gain_two() {
    let payments = [
        [0, 0, 0, 0, 0, 6],
        [0, 0, 0, 1, 0, 4],
        [1, 0, 0, 0, 0, 4],
        [0, 0, 1, 0, 0, 4],
        [1, 0, 0, 1, 0, 2],
        [0, 0, 1, 1, 0, 2],
        [1, 0, 1, 0, 0, 2],
        [1, 0, 1, 1, 0, 0],
    ];
    for p in [Player::P0, Player::P1] {
        for payment in payments {
            let mut g = scenario(p);
            let enemy = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
            g.state.players[p.index()].mana = payment;
            spell_card(&mut g, p, "Defibrillating Current", &[Target::Card(enemy)]);
            drain(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6], "{payment:?}");
            assert_eq!(g.state.objects[enemy as usize].damage, 4);
            assert_eq!(g.state.players[p.index()].life, 22);
        }
    }
}

#[test]
fn spells_defibrillating_current_targets_planeswalker_and_fizzles_without_life_if_target_blinks() {
    for blink in [false, true] {
        let mut g = scenario(Player::P0);
        let enemy = put(
            &mut g,
            Player::P1,
            "Elspeth, Storm Slayer",
            Zone::Battlefield,
        );
        g.set_counters(enemy, Counters::of(CounterKind::Loyalty, 5));
        spell_card(
            &mut g,
            Player::P0,
            "Defibrillating Current",
            &[Target::Card(enemy)],
        );
        if blink {
            g.move_object(enemy, Zone::Exile);
            g.move_object(enemy, Zone::Battlefield);
        }
        drain(&mut g);
        assert_eq!(
            g.counter_count(enemy, CounterKind::Loyalty),
            if blink { 5 } else { 1 },
            "damage removes four; a blinked Elspeth enters with five printed loyalty"
        );
        assert_eq!(g.state.players[0].life, if blink { 20 } else { 22 });
    }
}

#[test]
fn spells_duty_beyond_death_pays_creature_sacrifice_and_grants_only_survivors_indestructible_plus_counter()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let fuel = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let land = put(&mut g, p, "Forest", Zone::Battlefield);
        let spell = put(&mut g, p, "Duty Beyond Death", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: spell });
        for id in [enemy, land] {
            let object = g.object_ref(id);
            reject(&mut g, Action::ChooseAdditionalCost { object });
        }
        let object = g.object_ref(fuel);
        act(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        assert_eq!(g.object_view(fuel).zone, Zone::Graveyard);
        drain(&mut g);
        assert_eq!(g.counter_count(own, CounterKind::PlusOne), 1);
        assert!(g.keywords(own).contains(Keywords::INDESTRUCTIBLE));
        assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 0);
        assert!(!g.keywords(enemy).contains(Keywords::INDESTRUCTIBLE));
        let later = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        assert!(!g.keywords(later).contains(Keywords::INDESTRUCTIBLE));
        cleanup(&mut g);
        assert_eq!(stats(&g, own), (3, 3));
        assert!(!g.keywords(own).contains(Keywords::INDESTRUCTIBLE));
    }
}

#[test]
fn spells_focus_the_mind_pays_five_or_three_after_real_prior_cast_draws_three_then_discards_one() {
    for p in [Player::P0, Player::P1] {
        for prior in [false, true] {
            let mut g = scenario(p);
            if prior {
                let first = put(&mut g, p, "Short Bow", Zone::Hand);
                cast(&mut g, first, None);
                drain(&mut g);
            }
            let discard = put(&mut g, p, "Mountain", Zone::Library);
            g.state.players[p.index()].mana = [0, 1, 0, 0, 0, if prior { 2 } else { 4 }];
            spell_card(&mut g, p, "Focus the Mind", &[]);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            ready(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectDiscard { .. })
            });
            assert_eq!(g.hand_count(p), 3);
            assert_eq!(g.object_view(discard).zone, Zone::Hand);
            act(&mut g, Action::Discard { card: discard });
            drain(&mut g);
            assert_eq!(g.hand_count(p), 2);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 3);
            assert_eq!(g.object_view(discard).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn spells_fire_rim_form_flash_attaches_gives_plus_two_and_temporary_first_strike() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p.other());
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        act(&mut g, Action::Pass);
        assert_eq!(g.actor(), Some(p));
        let aura = put(&mut g, p, "Fire-Rim Form", Zone::Hand);
        cast(&mut g, aura, Some(Target::Card(target)));
        drain(&mut g);
        assert_eq!(g.object_view(aura).attached_to, Some(g.object_ref(target)));
        assert_eq!(stats(&g, target), (4, 2));
        assert!(g.keywords(target).contains(Keywords::FIRST_STRIKE));
        g.move_object(aura, Zone::Graveyard);
        assert_eq!(stats(&g, target), (2, 2));
        assert!(
            g.keywords(target).contains(Keywords::FIRST_STRIKE),
            "entry grant survives Aura departure"
        );
        cleanup(&mut g);
        assert!(!g.keywords(target).contains(Keywords::FIRST_STRIKE));
    }
}

#[test]
fn spells_fresh_start_flash_reduces_power_five_removes_all_abilities_and_restores_on_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p.other());
        let target = put(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
        act(&mut g, Action::Pass);
        let aura = put(&mut g, p, "Fresh Start", Zone::Hand);
        cast(&mut g, aura, Some(Target::Card(target)));
        drain(&mut g);
        assert_eq!(g.object_view(aura).attached_to, Some(g.object_ref(target)));
        assert_eq!(stats(&g, target), (-4, 1));
        g.state.priority = p.other();
        reject(&mut g, Action::TapMana { card: target });
        g.move_object(aura, Zone::Graveyard);
        assert_eq!(stats(&g, target), (1, 1));
        assert!(g.object_has_rules_abilities(target));
        act(&mut g, Action::TapMana { card: target });
    }
}

#[test]
fn spells_frontline_rush_both_modes_create_red_goblins_or_grow_any_target_by_live_controlled_count()
{
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        modal_card(&mut g, p, "Frontline Rush", 0, &[]);
        drain(&mut g);
        let tokens = battlefield_tokens(&g);
        assert_eq!(tokens.len(), 2);
        for token in &tokens {
            assert_eq!(stats(&g, *token), (1, 1));
            assert_eq!(g.object_view(*token).colors, 8);
            assert!(g.has_subtype(*token, "Goblin"));
            assert!(!g.keywords(*token).contains(Keywords::HASTE));
        }
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        modal_card(&mut g, p, "Frontline Rush", 1, &[Target::Card(target)]);
        g.move_object(tokens[0], Zone::Exile);
        drain(&mut g);
        assert_eq!(stats(&g, target), (3, 3));
        cleanup(&mut g);
        assert_eq!(stats(&g, target), (2, 2));
    }
}

#[test]
fn spells_channeled_dragonfire_targets_planeswalker_and_both_cast_routes_are_sorcery_speed() {
    let p = Player::P0;
    let mut g = scenario(p);
    let walker = put(
        &mut g,
        p.other(),
        "Elspeth, Storm Slayer",
        Zone::Battlefield,
    );
    g.set_counters(walker, Counters::of(CounterKind::Loyalty, 5));
    spell_card(&mut g, p, "Channeled Dragonfire", &[Target::Card(walker)]);
    drain(&mut g);
    assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 3);
    for origin in [Zone::Hand, Zone::Graveyard] {
        let spell = authored_spell(&mut g, p, "Channeled Dragonfire", origin);
        g.state.step = Step::BeginCombat;
        reject(
            &mut g,
            if origin == Zone::Hand {
                Action::AnnounceCast { card: spell }
            } else {
                Action::AnnounceHarmonize { card: spell }
            },
        );
    }
}

// These unrelated land definitions may remain gated. Only their real printed
// hand types are needed: install each into a hand fixture before any rules event,
// and never play it or invoke its abilities.
fn nonland_negative_hand_fixture(g: &mut Game, p: Player, name: &str) -> CardId {
    assert!(["Scene of the Crime", "Valgavoth's Lair"].contains(&name));
    let id = put(g, p, "Bear Cub", Zone::Hand);
    g.state.objects[id as usize].card = Card::from_name(name).unwrap();
    assert_ne!(g.current_kinds(id) & 2, 0);
    id
}
fn nonland_negative_lands(g: &mut Game, p: Player) -> [CardId; 3] {
    let ordinary = put(g, p, "Mountain", Zone::Hand);
    let artifact = nonland_negative_hand_fixture(g, p, "Scene of the Crime");
    let enchantment = nonland_negative_hand_fixture(g, p, "Valgavoth's Lair");
    assert_ne!(g.current_kinds(artifact) & 16, 0);
    assert_ne!(g.current_kinds(enchantment) & 32, 0);
    [ordinary, artifact, enchantment]
}
fn assert_revealed_hand_to_both(g: &Game, cards: &[CardId]) {
    for viewer in [Player::P0, Player::P1] {
        let observation = g.observe(viewer);
        for &card in cards {
            let reference = g.object_ref(card);
            assert!(observation.known_hand.contains(&reference));
            assert!(observation.objects.iter().any(|o| o.reference == reference));
        }
    }
}

#[test]
fn spells_aggressive_negotiations_nonland_filter_excludes_artifact_and_enchantment_lands() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let lands = nonland_negative_lands(&mut g, p.other());
        let nonland = put(&mut g, p.other(), "Shock", Zone::Hand);
        for &id in lands.iter().chain([&nonland]) {
            assert!(!g.observe(p).objects.iter().any(|o| o.id == id));
        }
        spell_card(
            &mut g,
            p,
            "Aggressive Negotiations",
            &[Target::Player(p.other()), Target::Card(own)],
        );
        ready(&mut g, |g| {
            matches!(g.state.pending, Pending::HandCardChoice { .. })
        });
        let mut hand = lands.to_vec();
        hand.push(nonland);
        assert_revealed_hand_to_both(&g, &hand);
        let view = g.observe(p).hand_choice.unwrap();
        assert_eq!(view.kind, HandChoiceKind::ExileRevealedNonland);
        assert_eq!(view.actor, p);
        assert_eq!(view.victim, Some(p.other()));
        assert_eq!(view.required, 1);
        assert!(!view.optional);
        assert!(view.inspected_cards.is_none());
        assert_eq!(view.candidates, vec![g.object_ref(nonland)]);
        assert!(g.observe(p.other()).hand_choice.is_none());
        reject(&mut g, Action::ChooseHandCard { object: None });
        for &id in &lands {
            let object = g.object_ref(id);
            reject(
                &mut g,
                Action::ChooseHandCard {
                    object: Some(object),
                },
            );
        }
        let before = g.snapshot();
        let revision = g.revision();
        let object = g.object_ref(nonland);
        assert_eq!(
            g.apply(
                p.other(),
                Action::ChooseHandCard {
                    object: Some(object)
                }
            ),
            Err(Error::WrongActor)
        );
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.revision(), revision);
        hand_pick(&mut g, nonland);
        drain(&mut g);
        assert_eq!(g.object_view(nonland).zone, Zone::Exile);
        for &id in &lands {
            assert_eq!(g.object_view(id).zone, Zone::Hand);
        }
        assert_eq!(g.counter_count(own, CounterKind::PlusOne), 1);
        assert_eq!(
            g.state.players[p.other().index()].cards_discarded_this_turn,
            0
        );
    }
}

#[test]
fn spells_aggressive_negotiations_all_land_hand_is_revealed_and_counter_group_resumes() {
    for p in [Player::P0, Player::P1] {
        for counter in [false, true] {
            let mut g = scenario(p);
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let lands = nonland_negative_lands(&mut g, p.other());
            let targets = if counter {
                vec![Target::Player(p.other()), Target::Card(own)]
            } else {
                vec![Target::Player(p.other())]
            };
            let spell = spell_card(&mut g, p, "Aggressive Negotiations", &targets);
            drain(&mut g);
            assert_revealed_hand_to_both(&g, &lands);
            for viewer in [p, p.other()] {
                assert!(g.observe(viewer).hand_choice.is_none());
            }
            for id in lands {
                assert_eq!(g.object_view(id).zone, Zone::Hand);
            }
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            assert_eq!(
                g.counter_count(own, CounterKind::PlusOne),
                u64::from(counter)
            );
        }
    }
}

#[test]
fn spells_aggressive_negotiations_nonland_choice_is_exact_and_source_independent() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let stale = put(&mut g, p.other(), "Shock", Zone::Hand);
        let survivor = put(&mut g, p.other(), "Divination", Zone::Hand);
        let spell = spell_card(
            &mut g,
            p,
            "Aggressive Negotiations",
            &[Target::Player(p.other()), Target::Card(own)],
        );
        ready(&mut g, |g| {
            matches!(g.state.pending, Pending::HandCardChoice { .. })
        });
        let old = g.object_ref(stale);
        g.move_object(stale, Zone::Graveyard);
        g.move_object(stale, Zone::Hand);
        let new = g.object_ref(stale);
        assert_ne!(old, new);
        for object in [old, new] {
            reject(
                &mut g,
                Action::ChooseHandCard {
                    object: Some(object),
                },
            );
        }
        // The pending packet owns its chooser and continuation even if the source
        // incarnation departs while this owned resolving state is suspended.
        g.move_object(spell, Zone::Exile);
        g.move_object(spell, Zone::Hand);
        assert_eq!(
            g.observe(p).hand_choice.unwrap().candidates,
            vec![g.object_ref(survivor)]
        );
        hand_pick(&mut g, survivor);
        drain(&mut g);
        assert_eq!(g.object_view(survivor).zone, Zone::Exile);
        assert_eq!(g.object_view(stale).zone, Zone::Hand);
        assert_eq!(g.object_view(spell).zone, Zone::Hand);
        assert_eq!(g.counter_count(own, CounterKind::PlusOne), 1);
    }
}

#[test]
fn spells_aggressive_negotiations_nonland_choice_owns_raw_index_siblings_and_restore() {
    for p in [Player::P0, Player::P1] {
        let mut parent = scenario(p);
        let own = put(&mut parent, p, "Bear Cub", Zone::Battlefield);
        let lands = nonland_negative_lands(&mut parent, p.other());
        let first = put(&mut parent, p.other(), "Shock", Zone::Hand);
        let second = put(&mut parent, p.other(), "Divination", Zone::Hand);
        spell_card(
            &mut parent,
            p,
            "Aggressive Negotiations",
            &[Target::Player(p.other()), Target::Card(own)],
        );
        ready(&mut parent, |g| {
            matches!(g.state.pending, Pending::HandCardChoice { .. })
        });
        let action = Action::ChooseHandCard {
            object: Some(parent.object_ref(first)),
        };
        let index = parent
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        let saved = parent.snapshot();
        let revision = parent.revision();
        let mut raw = parent.clone();
        let mut indexed = parent.clone();
        let mut sibling = parent.clone();
        drop(parent);
        act(&mut raw, action.clone());
        indexed.apply_index(p, revision, index).unwrap();
        hand_pick(&mut sibling, second);
        for child in [&mut raw, &mut indexed, &mut sibling] {
            drain(child);
        }
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.object_view(first).zone, Zone::Exile);
        assert_eq!(raw.object_view(second).zone, Zone::Hand);
        assert_eq!(sibling.object_view(first).zone, Zone::Hand);
        assert_eq!(sibling.object_view(second).zone, Zone::Exile);
        for child in [&raw, &indexed, &sibling] {
            assert_eq!(child.counter_count(own, CounterKind::PlusOne), 1);
            for id in lands {
                assert_eq!(child.object_view(id).zone, Zone::Hand);
            }
        }
        indexed.restore(&saved).unwrap();
        act(&mut indexed, action);
        drain(&mut indexed);
        assert_eq!(indexed.snapshot(), raw.snapshot());
    }
}

fn enter_authored(g: &mut Game, p: Player, name: &str) -> CardId {
    let source = authored_spell(g, p, name, Zone::Hand);
    cast(g, source, None);
    source
}
fn begin_own_step(g: &mut Game, step: Step) {
    assert_eq!(g.state.pending, Pending::Priority);
    assert!(g.state.stack.is_empty());
    g.begin_step(step);
    g.give_priority();
}
fn effect_pick(g: &mut Game, id: CardId) {
    let object = g.object_ref(id);
    act(g, Action::Sacrifice { object });
}

#[test]
fn mechanics_abzan_monument_searches_only_basic_clan_lands_and_reveals_or_fails_to_find() {
    for p in [Player::P0, Player::P1] {
        for found in [None, Some("Plains"), Some("Swamp"), Some("Forest")] {
            let mut g = scenario(p);
            let plains = put(&mut g, p, "Plains", Zone::Library);
            let swamp = put(&mut g, p, "Swamp", Zone::Library);
            let forest = put(&mut g, p, "Forest", Zone::Library);
            let island = put(&mut g, p, "Island", Zone::Library);
            let nonbasic = put(&mut g, p, "Lush Portico", Zone::Library);
            enter_authored(&mut g, p, "Abzan Monument");
            ready(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectSearch { .. })
            });
            assert!(g.observe(p.other()).search_choice.is_none());
            for card in [island, nonbasic] {
                reject(&mut g, Action::ChooseSearch { card });
            }
            let rng = g.state.rng;
            let selected = found.map(|name| match name {
                "Plains" => plains,
                "Swamp" => swamp,
                "Forest" => forest,
                _ => unreachable!(),
            });
            if let Some(card) = selected {
                act(&mut g, Action::ChooseSearch { card });
            }
            if g.legal_actions().contains(&Action::FinishSearch) {
                act(&mut g, Action::FinishSearch);
            }
            drain(&mut g);
            assert_ne!(g.state.rng, rng, "even a failed search shuffles");
            for card in [plains, swamp, forest] {
                assert_eq!(
                    g.object_view(card).zone,
                    if selected == Some(card) {
                        Zone::Hand
                    } else {
                        Zone::Library
                    }
                );
            }
            if let Some(card) = selected {
                assert_revealed_hand_to_both(&g, &[card]);
            }
        }
    }
}

#[test]
fn mechanics_abzan_monument_pays_sorcery_tap_sacrifice_and_samples_greatest_current_toughness() {
    for p in [Player::P0, Player::P1] {
        for remove_tall in [false, true] {
            let mut g = scenario(p);
            let short = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let tall = put(&mut g, p, "Hill Giant", Zone::Battlefield);
            g.add_counter(tall, CounterKind::PlusOne, 4);
            let enemy = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            g.add_counter(enemy, CounterKind::PlusOne, 20);
            let source = enter_authored(&mut g, p, "Abzan Monument");
            ready(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectSearch { .. })
            });
            act(&mut g, Action::FinishSearch);
            drain(&mut g);
            let activation = Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            };
            g.state.step = Step::BeginCombat;
            reject(&mut g, activation.clone());
            g.state.step = Step::Main1;
            g.state.players[p.index()].mana = [1, 0, 1, 0, 1, 1];
            act(&mut g, activation);
            pay(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(g.object_view(source).zone, Zone::Graveyard);
            if remove_tall {
                g.move_object(tall, Zone::Hand);
            }
            drain(&mut g);
            let tokens = battlefield_tokens(&g);
            assert_eq!(tokens.len(), 1);
            let token = tokens[0];
            let size = if remove_tall { 2 } else { 7 };
            assert_eq!(stats(&g, token), (size, size));
            assert_eq!(
                g.counter_count(token, CounterKind::PlusOne),
                0,
                "X/X is a copiable base, not counters"
            );
            assert_eq!(g.object_view(token).colors, 1);
            assert!(g.has_subtype(token, "Spirit"));
            assert!(!g.keywords(token).contains(Keywords::FLYING));
            assert_eq!(stats(&g, short), (2, 2));
        }
    }
}

#[test]
fn mechanics_abzan_monument_without_creatures_creates_a_zero_zero_token_that_dies() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = enter_authored(&mut g, p, "Abzan Monument");
    ready(&mut g, |g| {
        matches!(g.state.pending, Pending::EffectSearch { .. })
    });
    act(&mut g, Action::FinishSearch);
    drain(&mut g);
    let before = g.state.objects.len();
    activate_plain(&mut g, source, 0);
    drain(&mut g);
    assert_eq!(g.state.objects.len(), before + 1);
    assert!(battlefield_tokens(&g).is_empty());
}

#[test]
fn mechanics_aegis_sculptor_upkeep_exiles_exactly_two_own_cards_or_places_no_counter() {
    for p in [Player::P0, Player::P1] {
        for (count, accept) in [(0, false), (1, false), (2, false), (2, true)] {
            let mut g = scenario(p);
            let source = enter_authored(&mut g, p, "Aegis Sculptor");
            drain(&mut g);
            assert_eq!(stats(&g, source), (2, 3));
            assert!(g.keywords(source).contains(Keywords::FLYING));
            let own: Vec<_> = (0..count)
                .map(|_| put(&mut g, p, "Forest", Zone::Graveyard))
                .collect();
            let foreign = put(&mut g, p.other(), "Forest", Zone::Graveyard);
            begin_own_step(&mut g, Step::Upkeep);
            if count >= 2 {
                ready(&mut g, |g| {
                    matches!(g.state.pending, Pending::EffectChoice { .. })
                });
                if accept {
                    let object = g.object_ref(foreign);
                    reject(&mut g, Action::Sacrifice { object });
                    for &id in &own {
                        effect_pick(&mut g, id);
                    }
                } else {
                    act(&mut g, Action::DeclineEffectPayment);
                }
            }
            drain(&mut g);
            assert_eq!(
                g.counter_count(source, CounterKind::PlusOne),
                u64::from(accept)
            );
            for id in own {
                assert_eq!(
                    g.object_view(id).zone,
                    if accept { Zone::Exile } else { Zone::Graveyard }
                );
            }
            assert_eq!(g.object_view(foreign).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn mechanics_aegis_sculptor_paid_upkeep_does_not_counter_a_new_source_incarnation() {
    let p = Player::P1;
    let mut g = scenario(p);
    let source = enter_authored(&mut g, p, "Aegis Sculptor");
    drain(&mut g);
    let a = put(&mut g, p, "Forest", Zone::Graveyard);
    let b = put(&mut g, p, "Mountain", Zone::Graveyard);
    begin_own_step(&mut g, Step::Upkeep);
    ready(&mut g, |g| {
        matches!(g.state.pending, Pending::EffectChoice { .. })
    });
    let old = g.object_ref(source);
    g.move_object(source, Zone::Hand);
    g.move_object(source, Zone::Battlefield);
    assert_ne!(g.object_ref(source), old);
    effect_pick(&mut g, a);
    effect_pick(&mut g, b);
    drain(&mut g);
    assert_eq!(g.counter_count(source, CounterKind::PlusOne), 0);
    assert_eq!(g.object_view(a).zone, Zone::Exile);
    assert_eq!(g.object_view(b).zone, Zone::Exile);
}

#[test]
fn mechanics_aegis_and_stormshell_actual_ward_two_is_paid_or_counters_only_the_opponents_spell() {
    for p in [Player::P0, Player::P1] {
        for name in ["Aegis Sculptor", "Ambling Stormshell"] {
            for paid in [false, true] {
                let mut g = scenario(p);
                let source = enter_authored(&mut g, p, name);
                drain(&mut g);
                act(&mut g, Action::Pass);
                let bounce = put(&mut g, p.other(), "Unsummon", Zone::Hand);
                g.state.players[p.other().index()].mana = [0, 1, 0, 0, 0, 2];
                cast(&mut g, bounce, Some(Target::Card(source)));
                ready(&mut g, |g| {
                    matches!(g.state.pending, Pending::ResolutionMana { .. })
                });
                assert_eq!(g.actor(), Some(p.other()));
                assert_eq!(
                    g.observe(p.other()).resolution_payment.unwrap().cost,
                    ManaCost {
                        colored: [0; 6],
                        generic: 2
                    }
                );
                act(
                    &mut g,
                    if paid {
                        Action::PayResolutionMana {
                            payment: [0, 0, 0, 0, 0, 2],
                        }
                    } else {
                        Action::DeclineResolutionMana
                    },
                );
                drain(&mut g);
                assert_eq!(
                    g.object_view(source).zone,
                    if paid { Zone::Hand } else { Zone::Battlefield }
                );
                assert_eq!(
                    g.state.players[p.other().index()].mana[5],
                    if paid { 0 } else { 2 }
                );
                assert_eq!(g.object_view(bounce).zone, Zone::Graveyard);
                if !paid {
                    let own_bounce = put(&mut g, p, "Unsummon", Zone::Hand);
                    cast(&mut g, own_bounce, Some(Target::Card(source)));
                    drain(&mut g);
                    assert_eq!(
                        g.object_view(source).zone,
                        Zone::Hand,
                        "own targeting does not trigger ward"
                    );
                }
            }
        }
    }
}

#[test]
fn mechanics_ainok_wayfarer_mills_three_recovers_only_that_land_or_gets_a_counter() {
    for p in [Player::P0, Player::P1] {
        for (eligible, recover) in [(true, true), (true, false), (false, false)] {
            let mut g = scenario(p);
            for id in g.state.players[p.index()].library.clone() {
                g.move_object(id, Zone::Outside);
            }
            let old_grave = put(&mut g, p, "Forest", Zone::Graveyard);
            let first = put(&mut g, p, "Bear Cub", Zone::Library);
            let second = put(&mut g, p, "Shock", Zone::Library);
            let third = put(
                &mut g,
                p,
                if eligible { "Mountain" } else { "Bear Cub" },
                Zone::Library,
            );
            let source = enter_authored(&mut g, p, "Ainok Wayfarer");
            if eligible {
                ready(&mut g, |g| {
                    matches!(g.state.pending, Pending::RecoverMilled { .. })
                });
                let object = g.object_ref(old_grave);
                reject(
                    &mut g,
                    Action::RecoverMilled {
                        object: Some(object),
                    },
                );
                let object = g.object_ref(third);
                act(
                    &mut g,
                    Action::RecoverMilled {
                        object: recover.then_some(object),
                    },
                );
            }
            drain(&mut g);
            assert_eq!(g.object_view(first).zone, Zone::Graveyard);
            assert_eq!(g.object_view(second).zone, Zone::Graveyard);
            assert_eq!(
                g.object_view(third).zone,
                if recover { Zone::Hand } else { Zone::Graveyard }
            );
            assert_eq!(g.object_view(old_grave).zone, Zone::Graveyard);
            assert_eq!(g.library_card_count(p), 0);
            assert_eq!(
                g.counter_count(source, CounterKind::PlusOne),
                u64::from(!recover)
            );
            assert_eq!(stats(&g, source), if recover { (1, 1) } else { (2, 2) });
        }
    }
}

#[test]
fn mechanics_all_out_assault_anthem_extra_combat_main_pair_and_only_next_attack_untap() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let attacker = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let idle = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let source = enter_authored(&mut g, p, "All-Out Assault");
        drain(&mut g);
        assert_eq!(stats(&g, attacker), (3, 3));
        assert_eq!(stats(&g, idle), (4, 4));
        assert_eq!(stats(&g, enemy), (3, 3));
        assert!(g.keywords(attacker).contains(Keywords::DEATHTOUCH));
        assert!(!g.keywords(enemy).contains(Keywords::DEATHTOUCH));
        g.state.objects[idle as usize].tapped = true;
        g.state.objects[enemy as usize].tapped = true;
        ready(&mut g, |g| g.state.pending == Pending::Attackers);
        act(&mut g, Action::Attack { card: attacker });
        act(&mut g, Action::FinishAttackers);
        drain(&mut g);
        assert!(!g.object_view(attacker).tapped && !g.object_view(idle).tapped);
        assert!(g.object_view(enemy).tapped);
        ready(&mut g, |g| g.state.pending == Pending::Attackers);
        act(&mut g, Action::Attack { card: attacker });
        act(&mut g, Action::FinishAttackers);
        drain(&mut g);
        assert!(
            g.object_view(attacker).tapped,
            "delayed untap fires only on next attack"
        );
        ready(&mut g, |g| {
            g.state.step == Step::Main2 && g.state.pending == Pending::Priority
        });
        let mut extra = 0;
        for _ in 0..48 {
            if g.state.step == Step::End {
                break;
            }
            progress(&mut g);
            if g.state.step == Step::BeginCombat {
                extra += 1;
            }
        }
        assert_eq!(g.state.step, Step::End);
        assert_eq!(extra, 0, "exactly one additional combat/main pair");
        g.move_object(source, Zone::Graveyard);
        assert_eq!(stats(&g, attacker), (2, 2));
        assert!(!g.keywords(attacker).contains(Keywords::DEATHTOUCH));
    }
}

#[test]
fn mechanics_all_out_assault_off_main_entry_gives_anthem_without_extra_phases() {
    let p = Player::P0;
    let mut g = scenario(p);
    let creature = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let source = authored_spell(&mut g, p, "All-Out Assault", Zone::Hand);
    g.state.step = Step::BeginCombat;
    g.move_object(source, Zone::Battlefield);
    drain(&mut g);
    assert_eq!(stats(&g, creature), (3, 3));
    assert!(g.keywords(creature).contains(Keywords::DEATHTOUCH));
    assert!(g.observe(p).inserted_phases.is_none());
    assert!(g.observe(p).turn_callbacks.is_none());
}

#[test]
fn mechanics_ambling_stormshell_attack_draws_three_stuns_and_each_turtle_cast_attempts_untap() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = enter_authored(&mut g, p, "Ambling Stormshell");
        drain(&mut g);
        assert_eq!(stats(&g, source), (5, 9));
        g.state.objects[source as usize].summoning_sick = false;
        attack(&mut g, source);
        drain(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::Stun), 3);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 3);
        assert!(g.object_view(source).tapped);
        ready(&mut g, |g| {
            g.state.step == Step::Main2 && g.state.pending == Pending::Priority
        });
        g.state.players[p.index()].mana = [8; 6];
        for (remaining, tapped) in [(2, true), (1, true), (0, true), (0, false)] {
            let turtle = put(&mut g, p, "Aegis Turtle", Zone::Hand);
            cast(&mut g, turtle, None);
            drain(&mut g);
            assert_eq!(g.counter_count(source, CounterKind::Stun), remaining);
            assert_eq!(g.object_view(source).tapped, tapped);
        }
    }
}

#[test]
fn mechanics_armament_dragon_distributes_three_among_one_two_or_three_own_targets_without_reallocation()
 {
    for p in [Player::P0, Player::P1] {
        for count in 1..=3 {
            for blink in [false, true] {
                let mut g = scenario(p);
                let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
                let b = put(&mut g, p, "Bear Cub", Zone::Battlefield);
                let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
                let source = enter_authored(&mut g, p, "Armament Dragon");
                ready(&mut g, |g| {
                    matches!(g.state.pending, Pending::Targets { .. })
                });
                assert_eq!(stats(&g, source), (3, 4));
                assert!(g.keywords(source).contains(Keywords::FLYING));
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(enemy),
                    },
                );
                let ids = [source, a, b];
                for &id in ids.iter().take(count) {
                    act(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(id),
                        },
                    );
                }
                if g.legal_actions().contains(&Action::FinishTargets) {
                    act(&mut g, Action::FinishTargets);
                }
                let amounts: &[u64] = match count {
                    1 => &[3],
                    2 => &[2, 1],
                    3 => &[1, 1, 1],
                    _ => unreachable!(),
                };
                reject(&mut g, Action::AllocateTargetAmount { amount: 0 });
                for &amount in amounts {
                    act(&mut g, Action::AllocateTargetAmount { amount });
                }
                if blink {
                    g.move_object(a, Zone::Exile);
                    g.move_object(a, Zone::Battlefield);
                }
                drain(&mut g);
                for (index, &id) in ids.iter().enumerate() {
                    let expected = if index >= count || (blink && id == a) {
                        0
                    } else {
                        amounts[index]
                    };
                    assert_eq!(
                        g.counter_count(id, CounterKind::PlusOne),
                        expected,
                        "targets{count} blink{blink} recipient{index}"
                    );
                }
                assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 0);
            }
        }
    }
}

#[test]
fn mechanics_attuned_hunter_counts_one_owned_graveyard_batch_only_during_your_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let hunter = enter_authored(&mut g, p, "Attuned Hunter");
        drain(&mut g);
        assert_eq!(stats(&g, hunter), (3, 3));
        assert!(g.keywords(hunter).contains(Keywords::TRAMPLE));
        let a = put(&mut g, p, "Forest", Zone::Graveyard);
        let b = put(&mut g, p, "Mountain", Zone::Graveyard);
        let sunshield = put(&mut g, p, "Arashin Sunshield", Zone::Hand);
        cast(&mut g, sunshield, None);
        ready(&mut g, |g| {
            matches!(g.state.pending, Pending::Targets { .. })
        });
        for id in [a, b] {
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(id),
                },
            );
        }
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        drain(&mut g);
        assert_eq!(g.object_view(a).zone, Zone::Exile);
        assert_eq!(g.object_view(b).zone, Zone::Exile);
        assert_eq!(
            g.counter_count(hunter, CounterKind::PlusOne),
            1,
            "two noncreature cards leaving together trigger once"
        );
        let instant_return = put(&mut g, p, "Forest", Zone::Graveyard);
        g.state.active = p.other();
        g.state.priority = p;
        spell_card(
            &mut g,
            p,
            "Auroral Procession",
            &[Target::Card(instant_return)],
        );
        drain(&mut g);
        assert_eq!(g.object_view(instant_return).zone, Zone::Hand);
        assert_eq!(
            g.counter_count(hunter, CounterKind::PlusOne),
            1,
            "another player's turn does not trigger"
        );
        g.state.active = p;
        g.state.priority = p;
        g.state.players[p.index()].mana = [8; 6];
        let enemy_grave = put(&mut g, p.other(), "Forest", Zone::Graveyard);
        let sunshield = put(&mut g, p, "Arashin Sunshield", Zone::Hand);
        cast(&mut g, sunshield, None);
        ready(&mut g, |g| {
            matches!(g.state.pending, Pending::Targets { .. })
        });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy_grave),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        drain(&mut g);
        assert_eq!(
            g.counter_count(hunter, CounterKind::PlusOne),
            1,
            "opponent-owned graveyard cards do not trigger"
        );
    }
}

fn cast_barrensteppe(g: &mut Game, p: Player, mode: u16) -> CardId {
    let source = enter_authored(g, p, "Barrensteppe Siege");
    ready(g, |g| matches!(g.state.pending, Pending::EntryValue { .. }));
    act(
        g,
        Action::ChooseEntryValue {
            value: EntryChoiceValue::NamedMode(mode),
        },
    );
    drain(g);
    source
}

#[test]
fn mechanics_barrensteppe_siege_abzan_gives_each_own_creature_a_counter_only_on_own_end_step() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let source = cast_barrensteppe(&mut g, p, 0);
        assert_eq!(
            g.object_view(source).entry_choice,
            Some(EntryChoiceValue::NamedMode(0))
        );
        begin_own_step(&mut g, Step::End);
        drain(&mut g);
        for id in [a, b] {
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1);
        }
        assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 0);
        g.state.active = p.other();
        begin_own_step(&mut g, Step::End);
        drain(&mut g);
        for id in [a, b] {
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1);
        }
    }
}

#[test]
fn mechanics_barrensteppe_siege_mardu_requires_own_creature_death_and_opponent_chooses_sacrifice() {
    for p in [Player::P0, Player::P1] {
        for own_death in [false, true] {
            let mut g = scenario(p);
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let survivor = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            cast_barrensteppe(&mut g, p, 1);
            let murder = put(&mut g, p, "Murder", Zone::Hand);
            cast(
                &mut g,
                murder,
                Some(Target::Card(if own_death { own } else { enemy })),
            );
            drain(&mut g);
            begin_own_step(&mut g, Step::End);
            if own_death {
                ready(&mut g, |g| {
                    matches!(g.state.pending, Pending::EffectChoice { .. })
                });
                assert_eq!(g.actor(), Some(p.other()));
                effect_pick(&mut g, enemy);
            }
            drain(&mut g);
            assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
            assert_eq!(g.object_view(survivor).zone, Zone::Battlefield);
            assert_eq!(
                g.object_view(own).zone,
                if own_death {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(
                g.counter_count(survivor, CounterKind::PlusOne),
                0,
                "Mardu does not acquire the Abzan mode"
            );
        }
    }
}

#[test]
fn mechanics_betor_checks_ten_twenty_forty_live_total_toughness_and_rounds_half_life_up() {
    for p in [Player::P0, Player::P1] {
        for total in [9, 10, 19, 20, 39, 40] {
            let mut g = scenario(p);
            let source = enter_authored(&mut g, p, "Betor, Kin to All");
            drain(&mut g);
            assert_eq!(stats(&g, source), (5, 7));
            assert!(g.keywords(source).contains(Keywords::FLYING));
            let support = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.state.objects[support as usize].growth.toughness = total - 9;
            let enemy = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            g.state.objects[enemy as usize].growth.toughness = 100;
            g.state.players[p.other().index()].life = 21;
            g.state.objects[source as usize].tapped = true;
            g.state.objects[support as usize].tapped = true;
            g.state.objects[enemy as usize].tapped = true;
            begin_own_step(&mut g, Step::End);
            drain(&mut g);
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                u64::from(total >= 10),
                "total{total}"
            );
            assert_eq!(g.object_view(source).tapped, total < 20);
            assert_eq!(g.object_view(support).tapped, total < 20);
            assert!(g.object_view(enemy).tapped);
            assert_eq!(
                g.state.players[p.other().index()].life,
                if total >= 40 { 10 } else { 21 }
            );
        }
    }
}

#[test]
fn mechanics_betor_rechecks_intervening_threshold_and_later_thresholds_at_resolution() {
    for remove in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = enter_authored(&mut g, p, "Betor, Kin to All");
        drain(&mut g);
        let support = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[support as usize].growth.toughness = 31;
        g.state.objects[source as usize].tapped = true;
        begin_own_step(&mut g, Step::End);
        ready(&mut g, |g| {
            !g.state.stack.is_empty() && g.state.pending == Pending::Priority
        });
        if remove {
            g.move_object(support, Zone::Hand);
        } else {
            g.state.objects[support as usize].growth.toughness = 1;
        }
        drain(&mut g);
        assert_eq!(g.state.players[0].cards_drawn_this_turn, u64::from(!remove));
        assert!(g.object_view(source).tapped);
        assert_eq!(g.state.players[1].life, 20);
    }
}

#[test]
fn mechanics_bone_cairn_butcher_mobilizes_two_and_grants_deathtouch_only_to_attacking_own_tokens() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        modal_card(&mut g, p, "Frontline Rush", 0, &[]);
        drain(&mut g);
        let idle = battlefield_tokens(&g);
        let source = enter_authored(&mut g, p, "Bone-Cairn Butcher");
        drain(&mut g);
        assert_eq!(stats(&g, source), (4, 4));
        g.state.objects[source as usize].summoning_sick = false;
        attack(&mut g, source);
        drain(&mut g);
        let tokens = battlefield_tokens(&g);
        assert_eq!(tokens.len(), 4);
        assert_eq!(g.state.attackers.len(), 3);
        assert!(!g.keywords(source).contains(Keywords::DEATHTOUCH));
        for token in &tokens {
            let mobilized = !idle.contains(token);
            assert_eq!(g.keywords(*token).contains(Keywords::DEATHTOUCH), mobilized);
            if mobilized {
                assert_eq!(stats(&g, *token), (1, 1));
                assert!(g.object_view(*token).tapped && g.object_view(*token).attacking);
                assert_eq!(g.object_view(*token).colors, 8);
                assert!(g.has_subtype(*token, "Warrior"));
            }
        }
        g.move_object(source, Zone::Exile);
        for &token in &tokens {
            assert!(!g.keywords(token).contains(Keywords::DEATHTOUCH));
        }
        ready(&mut g, |g| {
            g.state.step == Step::Main2 && g.state.pending == Pending::Priority
        });
        begin_own_step(&mut g, Step::End);
        drain(&mut g);
        assert_eq!(
            battlefield_tokens(&g),
            idle,
            "only the exact mobilized Warriors are sacrificed"
        );
    }
}

#[test]
fn mechanics_breaching_dragonstorm_exiles_lands_then_casts_up_to_eight_or_puts_nonland_in_hand() {
    for p in [Player::P0, Player::P1] {
        for (name, take) in [
            ("Bear Cub", false),
            ("Bear Cub", true),
            ("Zetalpa, Primal Dawn", true),
            ("Omniscience", false),
        ] {
            let mut g = scenario(p);
            let nonland = put(&mut g, p, name, Zone::Library);
            let first_land = put(&mut g, p, "Mountain", Zone::Library);
            let second_land = put(&mut g, p, "Forest", Zone::Library);
            let source = enter_authored(&mut g, p, "Breaching Dragonstorm");
            ready(&mut g, |g| {
                matches!(g.state.pending, Pending::ResolutionCasts { .. })
            });
            let object = g.object_ref(nonland);
            let free = Action::CastExiledDuringResolution {
                object,
                face: CastFace::Normal,
            };
            if name == "Omniscience" {
                assert!(
                    !g.legal_actions().contains(&free),
                    "the ten-mana spell cannot be cast by an eight-mana offer"
                );
            }
            if take {
                g.state.players[p.index()].mana = [0; 6];
                act(&mut g, free);
                if g.legal_actions().contains(&Action::FinishTargets) {
                    act(&mut g, Action::FinishTargets);
                }
                pay(&mut g);
            }
            act(&mut g, Action::FinishResolutionCasts);
            drain(&mut g);
            assert_eq!(g.object_view(first_land).zone, Zone::Exile);
            assert_eq!(g.object_view(second_land).zone, Zone::Exile);
            assert_eq!(
                g.object_view(nonland).zone,
                if take { Zone::Battlefield } else { Zone::Hand },
                "{name}"
            );
            assert_eq!(
                g.object_view(source).zone,
                Zone::Battlefield,
                "a non-Dragon entering does not return it"
            );
        }
        let mut g = scenario(p);
        let lands = g.state.players[p.index()].library.clone();
        enter_authored(&mut g, p, "Breaching Dragonstorm");
        drain(&mut g);
        assert_eq!(g.library_card_count(p), 0);
        for id in lands {
            assert_eq!(g.object_view(id).zone, Zone::Exile);
        }
    }
}

#[test]
fn mechanics_breaching_dragonstorm_returns_only_for_your_dragon_and_not_a_new_incarnation() {
    for p in [Player::P0, Player::P1] {
        for blink in [false, true] {
            let mut g = scenario(p);
            put(&mut g, p, "Bear Cub", Zone::Library);
            let source = enter_authored(&mut g, p, "Breaching Dragonstorm");
            ready(&mut g, |g| {
                matches!(g.state.pending, Pending::ResolutionCasts { .. })
            });
            act(&mut g, Action::FinishResolutionCasts);
            drain(&mut g);
            let enemy_dragon = put(&mut g, p.other(), "Shivan Dragon", Zone::Hand);
            g.move_object(enemy_dragon, Zone::Battlefield);
            drain(&mut g);
            assert_eq!(g.object_view(source).zone, Zone::Battlefield);
            let own_dragon = put(&mut g, p, "Shivan Dragon", Zone::Hand);
            cast(&mut g, own_dragon, None);
            ready(&mut g, |g| {
                g.state.stack.iter().any(|item| {
                    item.source.is_some_and(|s| s.id == source)
                        && matches!(item.kind, StackKind::TriggeredAbility { ability: 1 })
                })
            });
            if blink {
                g.move_object(source, Zone::Hand);
                g.move_object(source, Zone::Battlefield);
            }
            drain(&mut g);
            assert_eq!(
                g.object_view(source).zone,
                if blink { Zone::Battlefield } else { Zone::Hand }
            );
        }
    }
}

fn colored_dragon(g: &mut Game, p: Player, colors: u8) -> CardId {
    let id = put(g, p, "Shivan Dragon", Zone::Battlefield);
    g.change_object_form(
        g.object_ref(id),
        AnimationSpec {
            artifact: false,
            creature_types: AnimationCreatureTypes::Unchanged,
            base: None,
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        },
        characteristic_layers::AnimationTypeMode::PreserveCardTypes,
    );
    g.state
        .roles
        .as_mut()
        .unwrap()
        .animations
        .last_mut()
        .unwrap()
        .colors = Some(colors);
    id
}
fn color_counter_pick(g: &mut Game, id: CardId) {
    let object = g.object_ref(id);
    act(g, Action::ChooseColorCounter { object });
}

#[test]
fn mechanics_call_the_spirit_dragons_one_five_color_dragon_gets_five_counters_without_winning() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let dragon = colored_dragon(&mut g, p, 31);
        let enemy = colored_dragon(&mut g, p.other(), 31);
        let source = enter_authored(&mut g, p, "Call the Spirit Dragons");
        drain(&mut g);
        assert!(g.keywords(dragon).contains(Keywords::INDESTRUCTIBLE));
        assert!(!g.keywords(enemy).contains(Keywords::INDESTRUCTIBLE));
        let murder = put(&mut g, p, "Murder", Zone::Hand);
        cast(&mut g, murder, Some(Target::Card(dragon)));
        drain(&mut g);
        assert_eq!(g.object_view(dragon).zone, Zone::Battlefield);
        begin_own_step(&mut g, Step::Upkeep);
        ready(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectChoice { .. })
        });
        let object = g.object_ref(enemy);
        reject(&mut g, Action::ChooseColorCounter { object });
        reject(&mut g, Action::DeclineEffectPayment);
        for _ in 0..5 {
            color_counter_pick(&mut g, dragon);
        }
        drain(&mut g);
        assert_eq!(g.counter_count(dragon, CounterKind::PlusOne), 5);
        assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 0);
        assert_eq!(g.outcome(), None);
        g.move_object(source, Zone::Exile);
        assert!(!g.keywords(dragon).contains(Keywords::INDESTRUCTIBLE));
    }
}

#[test]
fn mechanics_call_the_spirit_dragons_five_distinct_dragons_get_counters_and_win_after_source_leaves()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let dragons: Vec<_> = (0..5).map(|i| colored_dragon(&mut g, p, 1 << i)).collect();
        let source = enter_authored(&mut g, p, "Call the Spirit Dragons");
        drain(&mut g);
        begin_own_step(&mut g, Step::Upkeep);
        ready(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectChoice { .. })
        });
        let wrong_color = g.object_ref(dragons[1]);
        reject(
            &mut g,
            Action::ChooseColorCounter {
                object: wrong_color,
            },
        );
        g.move_object(source, Zone::Exile);
        for &dragon in &dragons {
            color_counter_pick(&mut g, dragon);
        }
        assert_eq!(g.outcome(), Some(Outcome::Winner(p)));
        for dragon in dragons {
            assert_eq!(g.counter_count(dragon, CounterKind::PlusOne), 1);
        }
    }
}

#[test]
fn candidates_seventeen_actual_creature_casts_pay_exact_printed_mana_and_have_native_characteristics()
 {
    let cards = [
        ("Abzan Devotee", [0, 0, 1, 0, 0, 1], (2, 1), Keywords::NONE),
        (
            "Anafenza, Unyielding Lineage",
            [1, 0, 0, 0, 0, 2],
            (2, 2),
            Keywords::FLASH.union(Keywords::FIRST_STRIKE),
        ),
        (
            "Arashin Sunshield",
            [1, 0, 0, 0, 0, 3],
            (3, 4),
            Keywords::NONE,
        ),
        (
            "Avenger of the Fallen",
            [0, 0, 1, 0, 0, 2],
            (2, 4),
            Keywords::DEATHTOUCH,
        ),
        (
            "Bearer of Glory",
            [1, 0, 0, 0, 0, 1],
            (2, 1),
            Keywords::FIRST_STRIKE,
        ),
        (
            "Boulderborn Dragon",
            [0, 0, 0, 0, 0, 5],
            (3, 3),
            Keywords::FLYING.union(Keywords::VIGILANCE),
        ),
        (
            "Cori Mountain Stalwart",
            [1, 0, 0, 1, 0, 1],
            (3, 3),
            Keywords::NONE,
        ),
        (
            "Dalkovan Packbeasts",
            [1, 0, 0, 0, 0, 2],
            (0, 4),
            Keywords::VIGILANCE,
        ),
        (
            "Descendant of Storms",
            [1, 0, 0, 0, 0, 0],
            (2, 1),
            Keywords::NONE,
        ),
        (
            "Devoted Duelist",
            [0, 0, 0, 1, 0, 1],
            (2, 1),
            Keywords::HASTE,
        ),
        (
            "Dragon Sniper",
            [0, 0, 0, 0, 1, 0],
            (1, 1),
            Keywords::REACH
                .union(Keywords::VIGILANCE)
                .union(Keywords::DEATHTOUCH),
        ),
        (
            "Dragonback Lancer",
            [1, 0, 0, 0, 0, 3],
            (3, 3),
            Keywords::FLYING,
        ),
        (
            "Dragonstorm Forecaster",
            [0, 1, 0, 0, 0, 0],
            (0, 3),
            Keywords::NONE,
        ),
        (
            "Dusyut Earthcarver",
            [0, 0, 0, 0, 1, 5],
            (4, 4),
            Keywords::REACH,
        ),
        (
            "Effortless Master",
            [0, 1, 0, 1, 0, 2],
            (4, 3),
            Keywords::VIGILANCE.union(Keywords::MENACE),
        ),
        (
            "Equilibrium Adept",
            [0, 0, 0, 1, 0, 3],
            (2, 4),
            Keywords::NONE,
        ),
        (
            "Fortress Kin-Guard",
            [1, 0, 0, 0, 0, 1],
            (1, 2),
            Keywords::NONE,
        ),
    ];
    for p in [Player::P0, Player::P1] {
        for (name, cost, expected, keywords) in cards {
            let mut g = scenario(p);
            let source = put(&mut g, p, name, Zone::Hand);
            g.state.players[p.index()].mana = cost;
            cast(&mut g, source, None);
            assert_eq!(
                g.state.players[p.index()].mana,
                [0; 6],
                "{name}: exact printed cost"
            );
            ready(&mut g, |g| g.object_view(source).zone == Zone::Battlefield);
            assert_eq!(
                stats(&g, source),
                expected,
                "{name}: native stats before ETB trigger resolution"
            );
            assert_eq!(
                g.keywords(source),
                keywords,
                "{name}: actual native ability wiring"
            );
        }
    }
}

#[test]
fn candidates_abzan_devotee_filters_only_wbg_once_each_turn_without_tapping() {
    for p in [Player::P0, Player::P1] {
        for color in [0, 2, 4] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Abzan Devotee", Zone::Battlefield);
            g.state.objects[source as usize].summoning_sick = true;
            g.state.players[p.index()].mana = one_mana(5);
            let action = |production| Action::ActivateMana {
                card: source,
                ability: 0,
                payment: one_mana(5),
                counter_payment: Counters::default(),
                production,
            };
            for wrong in [1, 3, 5] {
                reject(&mut g, action(one_mana(wrong)));
            }
            act(&mut g, action(one_mana(color)));
            assert_eq!(g.state.players[p.index()].mana, one_mana(color));
            assert!(!g.object_view(source).tapped);
            assert!(
                g.state.stack.is_empty(),
                "mana ability does not use the stack"
            );
            g.state.players[p.index()].mana = one_mana(5);
            reject(&mut g, action(one_mana(color)));
            cleanup(&mut g);
            g.state.active = p.other();
            g.state.turn += 1;
            g.begin_turn();
            g.give_priority();
            if g.actor() != Some(p) {
                act(&mut g, Action::Pass);
            }
            g.state.players[p.index()].mana = one_mana(5);
            act(&mut g, action(one_mana(color)));
            assert_eq!(
                g.state.players[p.index()].mana,
                one_mana(color),
                "once each turn also permits an activation during the opponent's turn"
            );
        }
    }
}

#[test]
fn candidates_abzan_devotee_returns_only_its_exact_graveyard_incarnation_after_two_black() {
    for p in [Player::P0, Player::P1] {
        for stale in [false, true] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Abzan Devotee", Zone::Graveyard);
            g.state.active = p.other();
            g.state.priority = p;
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 1];
            reject(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: 1,
                    counter_payment: Counters::default(),
                },
            );
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 2];
            activate_plain(&mut g, source, 1);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            if stale {
                g.move_object(source, Zone::Exile);
                g.move_object(source, Zone::Graveyard);
            }
            drain(&mut g);
            assert_eq!(
                g.object_view(source).zone,
                if stale { Zone::Graveyard } else { Zone::Hand }
            );
        }
    }
}

#[test]
fn candidates_arashin_sunshield_exiles_zero_one_or_two_from_one_graveyard_and_never_rebinds() {
    for p in [Player::P0, Player::P1] {
        for count in 0..=2 {
            let mut g = scenario(p);
            let a = put(&mut g, p.other(), "Forest", Zone::Graveyard);
            let b = put(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
            let foreign = put(&mut g, p, "Forest", Zone::Graveyard);
            let source = put(&mut g, p, "Arashin Sunshield", Zone::Hand);
            cast(&mut g, source, None);
            ready(&mut g, |g| {
                matches!(g.state.pending, Pending::Targets { .. })
            });
            if count > 0 {
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(a),
                    },
                );
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(foreign),
                    },
                );
                if count == 2 {
                    act(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(b),
                        },
                    );
                }
            }
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            if count == 2 {
                g.move_object(a, Zone::Hand);
                g.move_object(a, Zone::Graveyard);
            }
            g.move_object(source, Zone::Exile);
            drain(&mut g);
            assert_eq!(
                g.object_view(a).zone,
                if count == 1 {
                    Zone::Exile
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(
                g.object_view(b).zone,
                if count == 2 {
                    Zone::Exile
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(g.object_view(foreign).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn candidates_arashin_sunshield_pays_white_and_tap_to_tap_either_players_creature() {
    for p in [Player::P0, Player::P1] {
        for owner in [p, p.other()] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Arashin Sunshield", Zone::Battlefield);
            let target = put(&mut g, owner, "Bear Cub", Zone::Battlefield);
            let announce = Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            };
            g.state.objects[source as usize].summoning_sick = true;
            reject(&mut g, announce.clone());
            g.state.objects[source as usize].summoning_sick = false;
            g.state.players[p.index()].mana = [0; 6];
            reject(&mut g, announce);
            g.state.players[p.index()].mana = one_mana(0);
            activate_target(&mut g, source, 0, target);
            assert!(g.object_view(source).tapped);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            drain(&mut g);
            assert!(g.object_view(target).tapped);
        }
    }
}

#[test]
fn candidates_dusyut_and_fortress_enter_endure_with_exact_counter_or_white_spirit_size() {
    for p in [Player::P0, Player::P1] {
        for (name, amount, base) in [
            ("Dusyut Earthcarver", 3, (4, 4)),
            ("Fortress Kin-Guard", 1, (1, 2)),
        ] {
            for counters in [false, true] {
                let mut g = scenario(p);
                let source = put(&mut g, p, name, Zone::Hand);
                cast(&mut g, source, None);
                ready(&mut g, |g| {
                    matches!(g.state.pending, Pending::Endure { .. })
                });
                assert!(g.observe(p).endure_choice.is_some());
                assert!(g.observe(p.other()).endure_choice.is_none());
                act(
                    &mut g,
                    Action::ChooseEndure {
                        put_counters: counters,
                    },
                );
                drain(&mut g);
                assert_eq!(
                    g.counter_count(source, CounterKind::PlusOne),
                    if counters { amount as u64 } else { 0 }
                );
                assert_eq!(
                    stats(&g, source),
                    if counters {
                        (base.0 + amount, base.1 + amount)
                    } else {
                        base
                    }
                );
                let tokens = battlefield_tokens(&g);
                assert_eq!(tokens.len(), usize::from(!counters));
                for token in tokens {
                    assert_eq!(stats(&g, token), (amount, amount));
                    assert_eq!(g.object_view(token).colors, 1);
                    assert_eq!(g.object_view(token).controller, p);
                    assert!(g.has_subtype(token, "Spirit"));
                    assert_eq!(g.keywords(token), Keywords::NONE);
                }
            }
        }
    }
}

#[test]
fn candidates_descendant_attack_optional_one_white_payment_precedes_both_endure_branches() {
    for p in [Player::P0, Player::P1] {
        for outcome in [None, Some(false), Some(true)] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Descendant of Storms", Zone::Battlefield);
            attack(&mut g, source);
            ready(&mut g, |g| {
                matches!(g.state.pending, Pending::ResolutionMana { .. })
            });
            g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 1];
            if let Some(counters) = outcome {
                choose(&mut g, |a| matches!(a, Action::PayResolutionMana { .. }));
                assert_eq!(g.state.players[p.index()].mana, [0; 6]);
                ready(&mut g, |g| {
                    matches!(g.state.pending, Pending::Endure { .. })
                });
                act(
                    &mut g,
                    Action::ChooseEndure {
                        put_counters: counters,
                    },
                );
            } else {
                act(&mut g, Action::DeclineResolutionMana);
            }
            drain(&mut g);
            assert_eq!(
                g.counter_count(source, CounterKind::PlusOne),
                u64::from(outcome == Some(true))
            );
            let tokens = battlefield_tokens(&g);
            assert_eq!(tokens.len(), usize::from(outcome == Some(false)));
            for token in tokens {
                assert_eq!(stats(&g, token), (1, 1));
                assert!(!g.object_view(token).attacking);
                assert_eq!(g.object_view(token).colors, 1);
            }
        }
    }
}

#[test]
fn candidates_anafenza_flashes_during_opponents_turn_and_filters_other_nontoken_own_deaths() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        g.state.active = p.other();
        g.state.priority = p;
        let source = put(&mut g, p, "Anafenza, Unyielding Lineage", Zone::Hand);
        cast(&mut g, source, None);
        drain(&mut g);
        assert_eq!(stats(&g, source), (2, 2));
        assert!(g.keywords(source).contains(Keywords::FIRST_STRIKE));
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        // Resolving our flash creature returns priority to the active opponent.
        if g.actor() != Some(p) {
            act(&mut g, Action::Pass);
        }
        cast(&mut g, shock, Some(Target::Card(enemy)));
        drain(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 0);
        assert!(battlefield_tokens(&g).is_empty());
        // An actual token created by a normal spell is excluded from the death filter.
        g.state.active = p;
        g.state.priority = p;
        g.state.step = Step::Main1;
        g.state.players[p.index()].mana = [8; 6];
        modal_card(&mut g, p, "Frontline Rush", 0, &[]);
        drain(&mut g);
        let token = battlefield_tokens(&g)[0];
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, Some(Target::Card(token)));
        drain(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 0);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, Some(Target::Card(own)));
        ready(&mut g, |g| {
            matches!(g.state.pending, Pending::Endure { .. })
        });
        act(&mut g, Action::ChooseEndure { put_counters: true });
        drain(&mut g);
        assert_eq!(stats(&g, source), (4, 4));
        let tokens_before = battlefield_tokens(&g);
        let murder = put(&mut g, p, "Murder", Zone::Hand);
        cast(&mut g, murder, Some(Target::Card(source)));
        drain(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert_eq!(
            battlefield_tokens(&g),
            tokens_before,
            "its own death never endures"
        );
    }
}

#[test]
fn candidates_flurry_triggers_only_on_own_second_cast_on_either_players_turn() {
    for p in [Player::P0, Player::P1] {
        for own_turn in [false, true] {
            for name in [
                "Cori Mountain Stalwart",
                "Devoted Duelist",
                "Equilibrium Adept",
            ] {
                let mut g = scenario(p);
                if !own_turn {
                    g.state.active = p.other();
                }
                let source = put(&mut g, p, name, Zone::Battlefield);
                for ordinal in 1..=3 {
                    g.state.priority = p;
                    let spell = put(&mut g, p, "Quick Study", Zone::Hand);
                    cast(&mut g, spell, None);
                    drain(&mut g);
                    let triggered = ordinal >= 2;
                    let damage = if !triggered {
                        0
                    } else if name == "Cori Mountain Stalwart" {
                        2
                    } else if name == "Devoted Duelist" {
                        1
                    } else {
                        0
                    };
                    assert_eq!(g.state.players[p.other().index()].life, 20 - damage);
                    assert_eq!(
                        g.state.players[p.index()].life,
                        20 + if name == "Cori Mountain Stalwart" && triggered {
                            2
                        } else {
                            0
                        }
                    );
                    assert_eq!(
                        g.keywords(source).contains(Keywords::DOUBLE_STRIKE),
                        name == "Equilibrium Adept" && triggered
                    );
                }
                cleanup(&mut g);
                assert!(!g.keywords(source).contains(Keywords::DOUBLE_STRIKE));
            }
        }
    }
}

#[test]
fn candidates_flurry_ignores_opponents_second_cast_and_old_adept_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let sources: Vec<_> = [
            "Cori Mountain Stalwart",
            "Devoted Duelist",
            "Equilibrium Adept",
        ]
        .into_iter()
        .map(|name| put(&mut g, p, name, Zone::Battlefield))
        .collect();
        for _ in 0..2 {
            g.state.priority = p.other();
            let spell = put(&mut g, p.other(), "Quick Study", Zone::Hand);
            cast(&mut g, spell, None);
            drain(&mut g);
        }
        assert_eq!(g.state.players[p.index()].life, 20);
        assert_eq!(g.state.players[p.other().index()].life, 20);
        assert!(!g.keywords(sources[2]).contains(Keywords::DOUBLE_STRIKE));
        for ordinal in 1..=2 {
            g.state.priority = p;
            let spell = put(&mut g, p, "Quick Study", Zone::Hand);
            cast(&mut g, spell, None);
            if ordinal == 2 {
                g.move_object(sources[2], Zone::Hand);
                g.move_object(sources[2], Zone::Battlefield);
            }
            drain(&mut g);
        }
        assert_eq!(g.state.players[p.index()].life, 22);
        assert_eq!(g.state.players[p.other().index()].life, 17);
        assert!(!g.keywords(sources[2]).contains(Keywords::DOUBLE_STRIKE));
    }
}

#[test]
fn candidates_equilibrium_adept_actual_etb_authorizes_exact_exiled_land_after_source_departs() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let land = put(&mut g, p, "Mountain", Zone::Library);
        let source = put(&mut g, p, "Equilibrium Adept", Zone::Hand);
        cast(&mut g, source, None);
        drain(&mut g);
        assert_eq!(g.object_view(land).zone, Zone::Exile);
        g.move_object(source, Zone::Hand);
        let unrelated = put(&mut g, p, "Island", Zone::Exile);
        reject(&mut g, Action::PlayLand { card: unrelated });
        act(&mut g, Action::PlayLand { card: land });
        assert_eq!(g.object_view(land).zone, Zone::Battlefield);
        assert!(!g.object_view(land).tapped);
    }
}

#[test]
fn candidates_packbeasts_lancer_and_avenger_mobilize_exact_live_counts_and_sacrifice_only_their_warriors()
 {
    for p in [Player::P0, Player::P1] {
        for (name, initial, removed, expected) in [
            ("Dalkovan Packbeasts", 0, false, 3),
            ("Dragonback Lancer", 0, false, 1),
            ("Avenger of the Fallen", 0, false, 0),
            ("Avenger of the Fallen", 2, false, 2),
            ("Avenger of the Fallen", 2, true, 1),
        ] {
            let mut g = scenario(p);
            let source = put(&mut g, p, name, Zone::Battlefield);
            let dead: Vec<_> = (0..initial)
                .map(|_| put(&mut g, p, "Bear Cub", Zone::Graveyard))
                .collect();
            put(&mut g, p, "Forest", Zone::Graveyard);
            put(&mut g, p.other(), "Hill Giant", Zone::Graveyard);
            attack(&mut g, source);
            if removed {
                g.move_object(dead[0], Zone::Exile);
            }
            drain(&mut g);
            let tokens = battlefield_tokens(&g);
            assert_eq!(tokens.len(), expected);
            assert_eq!(g.object_view(source).tapped, name != "Dalkovan Packbeasts");
            for &token in &tokens {
                assert_eq!(stats(&g, token), (1, 1));
                assert_eq!(g.object_view(token).colors, 8);
                assert!(g.has_subtype(token, "Warrior"));
                assert!(g.object_view(token).tapped && g.object_view(token).attacking);
            }
            assert_eq!(g.state.attackers.len(), expected + 1);
            ready(&mut g, |g| {
                g.state.step == Step::Main2 && g.state.pending == Pending::Priority
            });
            begin_own_step(&mut g, Step::End);
            drain(&mut g);
            assert!(battlefield_tokens(&g).is_empty());
            assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        }
    }
}

#[test]
fn candidates_bearer_of_glory_has_first_strike_only_own_turn_and_boosts_current_own_creatures_until_cleanup()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Bearer of Glory", Zone::Battlefield);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        assert!(g.keywords(source).contains(Keywords::FIRST_STRIKE));
        g.state.active = p.other();
        assert!(!g.keywords(source).contains(Keywords::FIRST_STRIKE));
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 4];
        activate_plain(&mut g, source, 0);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        drain(&mut g);
        assert_eq!(stats(&g, source), (3, 2));
        assert_eq!(stats(&g, own), (3, 3));
        assert_eq!(stats(&g, enemy), (2, 2));
        let later = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        assert_eq!(stats(&g, later), (2, 2));
        g.move_object(source, Zone::Exile);
        assert_eq!(stats(&g, own), (3, 3));
        cleanup(&mut g);
        assert_eq!(stats(&g, own), (2, 2));
    }
}

#[test]
fn candidates_boulderborn_dragon_attack_surveils_one_privately_and_does_not_tap() {
    for p in [Player::P0, Player::P1] {
        for mill in [false, true] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Boulderborn Dragon", Zone::Battlefield);
            let top = put(&mut g, p, "Mountain", Zone::Library);
            attack(&mut g, source);
            ready(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectLibrary { .. })
            });
            assert!(!g.object_view(source).tapped);
            assert!(g.observe(p).library_choice.is_some());
            assert!(g.observe(p.other()).library_choice.is_none());
            let unrelated = put(&mut g, p, "Plains", Zone::Hand);
            reject(
                &mut g,
                Action::ArrangeLibrary {
                    card: unrelated,
                    destination: LibraryDestination::Graveyard,
                },
            );
            act(
                &mut g,
                Action::ArrangeLibrary {
                    card: top,
                    destination: if mill {
                        LibraryDestination::Graveyard
                    } else {
                        LibraryDestination::Top
                    },
                },
            );
            drain(&mut g);
            assert_eq!(
                g.object_view(top).zone,
                if mill { Zone::Graveyard } else { Zone::Library }
            );
            if !mill {
                assert_eq!(g.state.players[p.index()].library.last(), Some(&top));
            }
        }
    }
}

#[test]
fn candidates_dragon_sniper_reaches_a_flyer_and_its_one_deathtouch_damage_kills_it() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p.other());
        let source = put(&mut g, p, "Dragon Sniper", Zone::Battlefield);
        let attacker = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
        attack(&mut g, attacker);
        ready(&mut g, |g| g.state.pending == Pending::Blockers);
        act(
            &mut g,
            Action::Block {
                blocker: source,
                attacker,
            },
        );
        act(&mut g, Action::FinishBlockers);
        ready(&mut g, |g| {
            g.state.step == Step::Main2 && g.state.pending == Pending::Priority
        });
        assert_eq!(g.object_view(attacker).zone, Zone::Graveyard);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].life, 20);
    }
}

#[test]
fn candidates_effortless_master_counts_its_own_cast_for_two_spell_entry_and_menace_needs_two_blockers()
 {
    for p in [Player::P0, Player::P1] {
        for preceding in [0, 1] {
            let mut g = scenario(p);
            if preceding == 1 {
                let spell = put(&mut g, p, "Quick Study", Zone::Hand);
                cast(&mut g, spell, None);
                drain(&mut g);
            }
            let source = put(&mut g, p, "Effortless Master", Zone::Hand);
            cast(&mut g, source, None);
            drain(&mut g);
            assert_eq!(g.counter_count(source, CounterKind::PlusOne), preceding * 2);
            assert_eq!(
                stats(&g, source),
                if preceding == 0 { (4, 3) } else { (6, 5) }
            );
            g.state.objects[source as usize].summoning_sick = false;
            let a = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let b = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            attack(&mut g, source);
            assert!(!g.object_view(source).tapped);
            ready(&mut g, |g| g.state.pending == Pending::Blockers);
            act(
                &mut g,
                Action::Block {
                    blocker: a,
                    attacker: source,
                },
            );
            reject(&mut g, Action::FinishBlockers);
            act(
                &mut g,
                Action::Block {
                    blocker: b,
                    attacker: source,
                },
            );
            act(&mut g, Action::FinishBlockers);
        }
    }
}

#[test]
fn candidates_dragonstorm_forecaster_searches_only_either_printed_name_reveals_and_shuffles_even_on_failure()
 {
    for p in [Player::P0, Player::P1] {
        for found in [None, Some("Boulderborn Dragon"), Some("Dragonstorm Globe")] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Dragonstorm Forecaster", Zone::Battlefield);
            let dragon = put(&mut g, p, "Boulderborn Dragon", Zone::Library);
            let globe = if Card::from_name("Dragonstorm Globe").unwrap().is_supported() {
                put(&mut g, p, "Dragonstorm Globe", Zone::Library)
            } else {
                // The gated fixture installs its actual identity in hand before
                // any event. Admission switches this to the ordinary put above.
                let id = put(&mut g, p, "Plains", Zone::Hand);
                g.state.objects[id as usize].card = Card::from_name("Dragonstorm Globe").unwrap();
                g.move_object(id, Zone::Library);
                id
            };
            let other = put(&mut g, p, "Shivan Dragon", Zone::Library);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
            g.state.objects[source as usize].summoning_sick = true;
            reject(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            g.state.objects[source as usize].summoning_sick = false;
            activate_plain(&mut g, source, 0);
            assert!(g.object_view(source).tapped);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            ready(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectSearch { .. })
            });
            assert!(g.observe(p).search_choice.is_some());
            assert!(g.observe(p.other()).search_choice.is_none());
            reject(&mut g, Action::ChooseSearch { card: other });
            let selected = found.map(|name| {
                if name == "Boulderborn Dragon" {
                    dragon
                } else {
                    globe
                }
            });
            let rng = g.state.rng;
            if let Some(card) = selected {
                act(&mut g, Action::ChooseSearch { card });
            }
            if g.legal_actions().contains(&Action::FinishSearch) {
                act(&mut g, Action::FinishSearch);
            }
            drain(&mut g);
            assert_ne!(g.state.rng, rng);
            for id in [dragon, globe] {
                assert_eq!(
                    g.object_view(id).zone,
                    if selected == Some(id) {
                        Zone::Hand
                    } else {
                        Zone::Library
                    }
                );
            }
            if let Some(card) = selected {
                assert!(
                    g.observe(p.other()).objects.iter().any(|o| o.id == card),
                    "chosen search result is revealed"
                );
            }
        }
    }
}

#[test]
fn candidates_flurry_cast_ordinal_resets_for_retained_sources_on_the_next_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let stalwart = put(&mut g, p, "Cori Mountain Stalwart", Zone::Battlefield);
        let duelist = put(&mut g, p, "Devoted Duelist", Zone::Battlefield);
        let adept = put(&mut g, p, "Equilibrium Adept", Zone::Battlefield);
        for _ in 0..6 {
            put(&mut g, p, "Forest", Zone::Library);
        }
        for ordinal in 1..=2 {
            let spell = put(&mut g, p, "Quick Study", Zone::Hand);
            cast(&mut g, spell, None);
            drain(&mut g);
            assert_eq!(
                g.state.players[p.other().index()].life,
                if ordinal == 1 { 20 } else { 17 }
            );
        }
        cleanup(&mut g);
        g.state.active = p.other();
        g.state.turn += 1;
        g.begin_turn();
        g.give_priority();
        assert!(!g.keywords(adept).contains(Keywords::DOUBLE_STRIKE));
        g.state.players[p.index()].mana = [8; 6];
        for ordinal in 1..=2 {
            g.state.priority = p;
            let spell = put(&mut g, p, "Quick Study", Zone::Hand);
            cast(&mut g, spell, None);
            drain(&mut g);
            assert_eq!(
                g.state.players[p.other().index()].life,
                if ordinal == 1 { 17 } else { 14 }
            );
            assert_eq!(
                g.state.players[p.index()].life,
                if ordinal == 1 { 22 } else { 24 }
            );
        }
        assert_eq!(stats(&g, stalwart), (3, 3));
        assert!(g.keywords(duelist).contains(Keywords::HASTE));
        assert!(g.keywords(adept).contains(Keywords::DOUBLE_STRIKE));
    }
}

#[test]
fn candidates_equilibrium_adept_exile_permission_requires_mana_lasts_to_next_own_cleanup_and_is_exact()
 {
    for p in [Player::P0, Player::P1] {
        for cast_now in [false, true] {
            let mut g = scenario(p);
            let top = put(&mut g, p, "Bear Cub", Zone::Library);
            let source = put(&mut g, p, "Equilibrium Adept", Zone::Hand);
            cast(&mut g, source, None);
            drain(&mut g);
            assert!(g.can_play_exiled(p, top));
            g.move_object(source, Zone::Hand);
            let mut stale = g.clone();
            stale.move_object(top, Zone::Hand);
            stale.move_object(top, Zone::Exile);
            assert!(!stale.can_play_exiled(p, top));
            if cast_now {
                g.state.players[p.index()].mana = [0; 6];
                reject(&mut g, Action::AnnounceExile { card: top });
                g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
                act(&mut g, Action::AnnounceExile { card: top });
                pay(&mut g);
                drain(&mut g);
                assert_eq!(g.object_view(top).zone, Zone::Battlefield);
                assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            } else {
                cleanup(&mut g);
                assert!(g.can_play_exiled(p, top));
                g.state.active = p.other();
                g.state.turn += 1;
                g.begin_turn();
                assert!(g.can_play_exiled(p, top));
                g.state.active = p;
                g.state.turn += 1;
                g.begin_turn();
                assert!(g.can_play_exiled(p, top));
                cleanup(&mut g);
                assert!(!g.can_play_exiled(p, top));
                assert_eq!(g.object_view(top).zone, Zone::Exile);
            }
        }
    }
}
#[test]
fn growth_eight_new_definitions_cast_for_exact_printed_mana() {
    for p in [Player::P0, Player::P1] {
        for (name, cost, base) in [
            ("Clarion Conqueror", [1, 0, 0, 0, 0, 2], Some((3, 3))),
            ("Craterhoof Behemoth", [0, 0, 0, 0, 3, 5], Some((5, 5))),
            ("Delta Bloodflies", [0, 0, 1, 0, 0, 1], Some((1, 2))),
            ("Fleeting Effigy", [0, 0, 0, 1, 0, 0], Some((2, 2))),
            ("Formation Breaker", [0, 0, 0, 0, 1, 1], Some((2, 1))),
            ("Corroding Dragonstorm", [0, 0, 1, 0, 0, 1], None),
            ("Encroaching Dragonstorm", [0, 0, 0, 0, 1, 3], None),
            ("Dragonstorm Globe", [0, 0, 0, 0, 0, 3], None),
        ] {
            let mut g = scenario(p);
            g.state.players[p.index()].mana = cost;
            let source = enter_authored(&mut g, p, name);
            assert_eq!(g.state.players[p.index()].mana, [0; 6], "{name}");
            ready(&mut g, |g| g.object_view(source).zone == Zone::Battlefield);
            if let Some(expected) = base {
                assert_eq!(
                    stats(&g, source),
                    expected,
                    "{name}: native before ETB resolution"
                );
            }
        }
    }
}

#[test]
fn growth_clarion_conqueror_blocks_both_players_battlefield_kinds_including_mana_but_not_graveyard_or_lands()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = enter_authored(&mut g, p, "Clarion Conqueror");
        drain(&mut g);
        assert!(g.keywords(source).contains(Keywords::FLYING));
        for owner in [p, p.other()] {
            g.state.priority = owner;
            let elf = put(&mut g, owner, "Llanowar Elves", Zone::Battlefield);
            let globe = authored_spell(&mut g, owner, "Dragonstorm Globe", Zone::Battlefield);
            let elspeth = put(&mut g, owner, "Elspeth, Storm Slayer", Zone::Battlefield);
            g.add_counter(elspeth, CounterKind::Loyalty, 5);
            reject(&mut g, Action::TapMana { card: elf });
            reject(&mut g, mana_action(globe, 0));
            reject(
                &mut g,
                Action::AnnounceActivate {
                    card: elspeth,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            let land = put(&mut g, owner, "Forest", Zone::Battlefield);
            act(&mut g, Action::TapMana { card: land });
            assert!(g.object_view(land).tapped);
            let dead = put(&mut g, owner, "Abzan Devotee", Zone::Graveyard);
            activate_plain(&mut g, dead, 1);
            drain(&mut g);
            assert_eq!(g.object_view(dead).zone, Zone::Hand);
            g.state.priority = owner;
            g.move_object(source, Zone::Exile);
            act(&mut g, Action::TapMana { card: elf });
            act(&mut g, mana_action(globe, 0));
            g.move_object(source, Zone::Battlefield);
        }
    }
}

#[test]
fn growth_clarion_activation_lock_follows_live_ability_suppression_and_aura_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = enter_authored(&mut g, p, "Clarion Conqueror");
        drain(&mut g);
        let elf = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        reject(&mut g, Action::TapMana { card: elf });
        let aura = spell_card(&mut g, p, "Fresh Start", &[Target::Card(source)]);
        drain(&mut g);
        assert!(!g.keywords(source).contains(Keywords::FLYING));
        act(&mut g, Action::TapMana { card: elf });
        g.state.objects[elf as usize].tapped = false;
        g.move_object(aura, Zone::Graveyard);
        assert!(g.keywords(source).contains(Keywords::FLYING));
        reject(&mut g, Action::TapMana { card: elf });
        g.move_object(source, Zone::Hand);
        act(&mut g, Action::TapMana { card: elf });
    }
}

#[test]
fn growth_craterhoof_samples_live_own_creature_count_and_freezes_boost_and_trample_until_cleanup() {
    for p in [Player::P0, Player::P1] {
        for remove_source in [false, true] {
            let mut g = scenario(p);
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let leaving = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let source = enter_authored(&mut g, p, "Craterhoof Behemoth");
            ready(&mut g, |g| g.object_view(source).zone == Zone::Battlefield);
            g.move_object(leaving, Zone::Hand);
            if remove_source {
                g.move_object(source, Zone::Hand);
            }
            drain(&mut g);
            let x = if remove_source { 1 } else { 2 };
            assert_eq!(stats(&g, own), (2 + x, 2 + x));
            assert!(g.keywords(own).contains(Keywords::TRAMPLE));
            assert_eq!(stats(&g, enemy), (3, 3));
            assert!(!g.keywords(enemy).contains(Keywords::TRAMPLE));
            let later = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            assert_eq!(stats(&g, later), (2, 2));
            assert!(!g.keywords(later).contains(Keywords::TRAMPLE));
            if !remove_source {
                assert_eq!(stats(&g, source), (7, 7));
                assert!(
                    g.keywords(source)
                        .contains(Keywords::HASTE.union(Keywords::TRAMPLE))
                );
            }
            cleanup(&mut g);
            assert_eq!(stats(&g, own), (2, 2));
            assert!(!g.keywords(own).contains(Keywords::TRAMPLE));
            if !remove_source {
                assert_eq!(stats(&g, source), (5, 5));
                assert_eq!(g.keywords(source), Keywords::HASTE);
            }
        }
    }
}

#[test]
fn growth_delta_bloodflies_checks_a_counter_on_an_own_creature_at_both_trigger_and_resolution() {
    for p in [Player::P0, Player::P1] {
        for (location, change, expected) in [
            (0, false, 0),
            (1, false, 1),
            (2, false, 0),
            (3, false, 0),
            (4, false, 1),
            (1, true, 0),
            (0, true, 0),
        ] {
            let mut g = scenario(p);
            let source = enter_authored(&mut g, p, "Delta Bloodflies");
            drain(&mut g);
            g.state.objects[source as usize].summoning_sick = false;
            assert!(g.keywords(source).contains(Keywords::FLYING));
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let artifact = put(&mut g, p, "Short Bow", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let countered = match location {
                1 => Some(own),
                2 => Some(artifact),
                3 => Some(enemy),
                4 => Some(source),
                _ => None,
            };
            if let Some(id) = countered {
                g.add_counter(id, CounterKind::Stun, 1);
            }
            attack(&mut g, source);
            if change {
                if location == 1 {
                    g.move_object(own, Zone::Hand);
                } else {
                    g.add_counter(own, CounterKind::Lifelink, 1);
                }
            }
            drain(&mut g);
            assert_eq!(g.state.players[p.other().index()].life, 20 - expected);
            assert_eq!(g.state.players[p.index()].life, 20);
        }
    }
}

#[test]
fn growth_fleeting_effigy_pumps_repeatedly_at_instant_speed_and_returns_only_on_own_end_step() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = enter_authored(&mut g, p, "Fleeting Effigy");
        drain(&mut g);
        assert!(g.keywords(source).contains(Keywords::HASTE));
        g.state.active = p.other();
        g.state.priority = p;
        for expected in [4, 6] {
            g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 2];
            activate_plain(&mut g, source, 0);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            drain(&mut g);
            assert_eq!(stats(&g, source), (expected, 2));
            g.state.priority = p;
        }
        begin_own_step(&mut g, Step::End);
        drain(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        cleanup(&mut g);
        assert_eq!(stats(&g, source), (2, 2));
        g.state.active = p;
        g.state.turn += 1;
        g.begin_turn();
        begin_own_step(&mut g, Step::End);
        drain(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Hand);
    }
}

#[test]
fn growth_fleeting_effigy_end_trigger_and_pump_never_rebind_after_blink() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = enter_authored(&mut g, p, "Fleeting Effigy");
        drain(&mut g);
        activate_plain(&mut g, source, 0);
        g.move_object(source, Zone::Hand);
        g.move_object(source, Zone::Battlefield);
        drain(&mut g);
        assert_eq!(stats(&g, source), (2, 2));
        begin_own_step(&mut g, Step::End);
        ready(&mut g, |g| {
            !g.state.stack.is_empty() && g.state.pending == Pending::Priority
        });
        g.move_object(source, Zone::Hand);
        g.move_object(source, Zone::Battlefield);
        drain(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
    }
}

#[test]
fn growth_formation_breaker_live_counter_bonus_is_source_only_and_evasion_compares_current_power() {
    for p in [Player::P0, Player::P1] {
        for location in 0..=3 {
            let mut g = scenario(p);
            let source = enter_authored(&mut g, p, "Formation Breaker");
            drain(&mut g);
            g.state.objects[source as usize].summoning_sick = false;
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let artifact = put(&mut g, p, "Short Bow", Zone::Battlefield);
            let small = put(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
            let equal = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let high = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            if location != 0 {
                let id = match location {
                    1 => own,
                    2 => artifact,
                    _ => high,
                };
                g.add_counter(id, CounterKind::Lifelink, 1);
            }
            let boosted = location == 1;
            assert_eq!(stats(&g, source), if boosted { (3, 3) } else { (2, 1) });
            assert_eq!(
                stats(&g, own),
                (2, 2),
                "bonus belongs only to Formation Breaker"
            );
            attack(&mut g, source);
            ready(&mut g, |g| g.state.pending == Pending::Blockers);
            reject(
                &mut g,
                Action::Block {
                    blocker: small,
                    attacker: source,
                },
            );
            assert_eq!(
                g.legal_actions().contains(&Action::Block {
                    blocker: equal,
                    attacker: source
                }),
                !boosted
            );
            act(
                &mut g,
                Action::Block {
                    blocker: high,
                    attacker: source,
                },
            );
        }
    }
}

#[test]
fn growth_corroding_dragonstorm_loses_and_gains_two_then_surveils_two_privately() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let mill = put(&mut g, p, "Mountain", Zone::Library);
        let keep = put(&mut g, p, "Plains", Zone::Library);
        enter_authored(&mut g, p, "Corroding Dragonstorm");
        ready(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectLibrary { .. })
        });
        assert_eq!(g.state.players[p.index()].life, 22);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert!(g.observe(p).library_choice.is_some());
        assert!(g.observe(p.other()).library_choice.is_none());
        act(
            &mut g,
            Action::ArrangeLibrary {
                card: mill,
                destination: LibraryDestination::Graveyard,
            },
        );
        act(
            &mut g,
            Action::ArrangeLibrary {
                card: keep,
                destination: LibraryDestination::Top,
            },
        );
        drain(&mut g);
        assert_eq!(g.object_view(mill).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].library.last(), Some(&keep));
        assert_eq!(g.hand_count(p), 0);
    }
}

#[test]
fn growth_encroaching_dragonstorm_searches_zero_one_or_two_basic_lands_tapped_and_shuffles() {
    for p in [Player::P0, Player::P1] {
        for count in 0..=2 {
            let mut g = scenario(p);
            let a = put(&mut g, p, "Mountain", Zone::Library);
            let b = put(&mut g, p, "Plains", Zone::Library);
            let nonbasic = put(&mut g, p, "Frontier Bivouac", Zone::Library);
            enter_authored(&mut g, p, "Encroaching Dragonstorm");
            ready(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectSearch { .. })
            });
            assert!(g.observe(p).search_choice.is_some());
            assert!(g.observe(p.other()).search_choice.is_none());
            reject(&mut g, Action::ChooseSearch { card: nonbasic });
            let rng = g.state.rng;
            for card in [a, b].into_iter().take(count) {
                act(&mut g, Action::ChooseSearch { card });
            }
            act(&mut g, Action::FinishSearch);
            drain(&mut g);
            assert_ne!(g.state.rng, rng);
            for (i, card) in [a, b].into_iter().enumerate() {
                assert_eq!(
                    g.object_view(card).zone,
                    if i < count {
                        Zone::Battlefield
                    } else {
                        Zone::Library
                    }
                );
                if i < count {
                    assert!(g.object_view(card).tapped);
                }
            }
        }
    }
}

#[test]
fn growth_corroding_and_encroaching_return_only_on_an_own_dragon_entry() {
    for p in [Player::P0, Player::P1] {
        for name in ["Corroding Dragonstorm", "Encroaching Dragonstorm"] {
            let mut g = scenario(p);
            let source = enter_authored(&mut g, p, name);
            if name == "Corroding Dragonstorm" {
                ready(&mut g, |g| {
                    matches!(g.state.pending, Pending::EffectLibrary { .. })
                });
                for _ in 0..2 {
                    choose(&mut g, |a| {
                        matches!(
                            a,
                            Action::ArrangeLibrary {
                                destination: LibraryDestination::Top,
                                ..
                            }
                        )
                    });
                }
            } else {
                ready(&mut g, |g| {
                    matches!(g.state.pending, Pending::EffectSearch { .. })
                });
                act(&mut g, Action::FinishSearch);
            }
            drain(&mut g);
            let enemy = put(&mut g, p.other(), "Shivan Dragon", Zone::Hand);
            g.move_object(enemy, Zone::Battlefield);
            drain(&mut g);
            assert_eq!(g.object_view(source).zone, Zone::Battlefield);
            let dragon = put(&mut g, p, "Shivan Dragon", Zone::Hand);
            cast(&mut g, dragon, None);
            drain(&mut g);
            assert_eq!(g.object_view(source).zone, Zone::Hand);
        }
    }
}

#[test]
fn growth_dragonstorm_globe_taps_for_exactly_one_of_five_colors_without_a_stack() {
    for p in [Player::P0, Player::P1] {
        for color in 0..5 {
            let mut g = scenario(p);
            let source = enter_authored(&mut g, p, "Dragonstorm Globe");
            drain(&mut g);
            g.state.players[p.index()].mana = [0; 6];
            reject(&mut g, mana_action(source, 5));
            act(&mut g, mana_action(source, color));
            assert_eq!(g.state.players[p.index()].mana, one_mana(color));
            assert!(g.object_view(source).tapped);
            assert!(g.state.stack.is_empty());
            reject(&mut g, mana_action(source, color));
        }
    }
}

#[test]
fn growth_dragonstorm_globe_adds_entry_counters_to_own_dragon_cards_and_tokens_with_each_provider()
{
    for p in [Player::P0, Player::P1] {
        for doubled in [false, true] {
            let mut g = scenario(p);
            let a = enter_authored(&mut g, p, "Dragonstorm Globe");
            drain(&mut g);
            let b = enter_authored(&mut g, p, "Dragonstorm Globe");
            drain(&mut g);
            let own = put(&mut g, p, "Boulderborn Dragon", Zone::Hand);
            cast(&mut g, own, None);
            drain(&mut g);
            assert_eq!(g.counter_count(own, CounterKind::PlusOne), 2);
            assert_eq!(stats(&g, own), (5, 5));
            let enemy = put(&mut g, p.other(), "Shivan Dragon", Zone::Hand);
            g.move_object(enemy, Zone::Battlefield);
            assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 0);
            let bear = put(&mut g, p, "Bear Cub", Zone::Hand);
            cast(&mut g, bear, None);
            drain(&mut g);
            assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 0);
            if doubled {
                let elspeth = put(&mut g, p, "Elspeth, Storm Slayer", Zone::Battlefield);
                g.add_counter(elspeth, CounterKind::Loyalty, 5);
            }
            put(&mut g, p, "Dragonback Assault", Zone::Battlefield);
            let land = put(&mut g, p, "Forest", Zone::Hand);
            act(&mut g, Action::PlayLand { card: land });
            drain(&mut g);
            let tokens = battlefield_tokens(&g);
            assert_eq!(tokens.len(), if doubled { 2 } else { 1 });
            for token in tokens {
                assert_eq!(g.object_view(a).zone, Zone::Battlefield);
                assert_eq!(g.object_view(b).zone, Zone::Battlefield);
                assert!(g.has_subtype(token, "Dragon"));
                assert_eq!(g.object_view(token).colors, 8);
                assert!(matches!(
                    g.object_view(token).card.effect(),
                    Effect::Creature(4, 4)
                ));
                assert_eq!(
                    g.counter_count(token, CounterKind::PlusOne),
                    2,
                    "two live Globe providers must each add an entry counter to actual Dragonback Assault's4/4 red Dragon token"
                );
                assert_eq!(stats(&g, token), (6, 6));
            }
            g.move_object(a, Zone::Hand);
            g.move_object(b, Zone::Hand);
            let later = put(&mut g, p, "Shivan Dragon", Zone::Hand);
            cast(&mut g, later, None);
            drain(&mut g);
            assert_eq!(g.counter_count(later, CounterKind::PlusOne), 0);
            assert_eq!(g.counter_count(own, CounterKind::PlusOne), 2);
        }
    }
}

#[test]
fn growth_dragonback_assault_paid_entry_deals_three_once_to_each_creature_and_planeswalker() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let dragon = put(&mut g, p, "Shivan Dragon", Zone::Battlefield);
        let walker = put(
            &mut g,
            p.other(),
            "Elspeth, Storm Slayer",
            Zone::Battlefield,
        );
        g.add_counter(walker, CounterKind::Loyalty, 5);
        let equipment = put(&mut g, p, "Short Bow", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 1, 0, 1, 1, 3];
        let source = put(&mut g, p, "Dragonback Assault", Zone::Hand);
        cast(&mut g, source, None);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        drain(&mut g);
        assert_eq!(g.object_view(own).zone, Zone::Graveyard);
        assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
        assert_eq!(g.object_view(dragon).zone, Zone::Battlefield);
        assert_eq!(g.state.objects[dragon as usize].damage, 3);
        assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 2);
        assert_eq!(g.object_view(equipment).zone, Zone::Battlefield);
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].life, 20);
        assert_eq!(g.state.players[p.other().index()].life, 20);
    }
}

#[test]
fn growth_dragonback_assault_only_own_land_entry_creates_a_red_flying_four_four_even_after_departure()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Dragonback Assault", Zone::Hand);
        cast(&mut g, source, None);
        drain(&mut g);
        let enemy = put(&mut g, p.other(), "Forest", Zone::Hand);
        g.move_object(enemy, Zone::Battlefield);
        drain(&mut g);
        assert!(battlefield_tokens(&g).is_empty());
        let own = put(&mut g, p, "Forest", Zone::Hand);
        act(&mut g, Action::PlayLand { card: own });
        g.move_object(source, Zone::Hand);
        drain(&mut g);
        let tokens = battlefield_tokens(&g);
        assert_eq!(tokens.len(), 1);
        let token = tokens[0];
        assert_eq!(stats(&g, token), (4, 4));
        assert_eq!(g.object_view(token).controller, p);
        assert_eq!(g.object_view(token).colors, 8);
        assert!(g.has_subtype(token, "Dragon"));
        assert_eq!(g.keywords(token), Keywords::FLYING);
        assert!(!g.object_view(token).tapped);
    }
}

#[test]
fn growth_encroaching_private_search_owns_raw_index_siblings_and_restore() {
    for p in [Player::P0, Player::P1] {
        let mut parent = scenario(p);
        let a = put(&mut parent, p, "Mountain", Zone::Library);
        let b = put(&mut parent, p, "Plains", Zone::Library);
        enter_authored(&mut parent, p, "Encroaching Dragonstorm");
        ready(&mut parent, |g| {
            matches!(g.state.pending, Pending::EffectSearch { .. })
        });
        let action = Action::ChooseSearch { card: a };
        let revision = parent.revision();
        let index = parent
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        let saved = parent.snapshot();
        let mut raw = parent.clone();
        let mut indexed = parent.clone();
        let mut sibling = parent.clone();
        drop(parent);
        assert!(raw.observe(p.other()).search_choice.is_none());
        assert_eq!(raw.apply(p.other(), action.clone()), Err(Error::WrongActor));
        assert_eq!(raw.snapshot(), saved);
        act(&mut raw, action.clone());
        indexed.apply_index(p, revision, index).unwrap();
        act(&mut sibling, Action::ChooseSearch { card: b });
        for child in [&mut raw, &mut indexed, &mut sibling] {
            act(child, Action::FinishSearch);
            drain(child);
        }
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.object_view(a).zone, Zone::Battlefield);
        assert_eq!(raw.object_view(b).zone, Zone::Library);
        assert_eq!(sibling.object_view(a).zone, Zone::Library);
        assert_eq!(sibling.object_view(b).zone, Zone::Battlefield);
        indexed.restore(&saved).unwrap();
        assert_eq!(
            indexed.apply_index(p, revision, index),
            Err(Error::StaleRevision)
        );
        act(&mut indexed, action);
        act(&mut indexed, Action::FinishSearch);
        drain(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
    }
}

#[test]
fn growth_dragonstorm_return_clause_includes_its_own_entry_when_it_is_a_dragon() {
    // Only this external type provider is a generic immutable fixture. All three
    // copied Dragonstorms execute their actual canonical definitions unchanged.
    const DRAGON_TYPES: rustmage_core::cards::authored::CardRules =
        rustmage_core::cards::authored::CardRules {
            effect: Effect::Permanent,
            static_groups: &[StaticAbilityGroup {
                line: 0,
                first_layer: AbilityLayer::Type,
                fragments: &[StaticAbilityFragment::ControlledCreatureSubtype {
                    grant: CreatureSubtypeGrant::Named("Dragon"),
                    required_subtype: None,
                }],
            }],
            ..rustmage_core::cards::authored::CardRules::EMPTY
        };
    let _types = fixture_rules::activate(&[(Card::from_name("Short Bow").unwrap(), &DRAGON_TYPES)]);
    for name in [
        "Breaching Dragonstorm",
        "Corroding Dragonstorm",
        "Encroaching Dragonstorm",
    ] {
        for own_types in [false, true] {
            let p = Player::P0;
            let mut g = scenario(p);
            put(
                &mut g,
                if own_types { p } else { p.other() },
                "Short Bow",
                Zone::Battlefield,
            );
            let mut recipe = CopyRecipe::new(Card::from_name(name).unwrap())
                .with_sampled_size(copies::SampledTokenSize::new(2));
            recipe.overrides = Some(CopyOverrides {
                kinds: Some(1 | 32),
                ..CopyOverrides::NONE
            });
            g.stage_copy_creation(p, recipe, 1);
            assert!(!g.advance_replacements(false));
            g.advance_entries();
            let source = g
                .state
                .objects
                .iter()
                .enumerate()
                .find_map(|(id, object)| {
                    (object.zone == Zone::Battlefield && object.copy_kind == CopyKind::Token)
                        .then_some(id as CardId)
                })
                .unwrap();
            assert_eq!(g.object_view(source).card, Card::from_name(name).unwrap());
            assert_eq!(g.has_subtype(source, "Dragon"), own_types);
            for _ in 0..64 {
                if g.state.stack.is_empty()
                    && g.state.pending_triggers.is_empty()
                    && g.state.resolving.is_none()
                    && g.state.pending == Pending::Priority
                {
                    break;
                }
                match g.state.pending {
                    Pending::EffectLibrary { .. } => choose(&mut g, |action| {
                        matches!(
                            action,
                            Action::ArrangeLibrary {
                                destination: LibraryDestination::Top,
                                ..
                            }
                        )
                    }),
                    Pending::EffectSearch { .. } => act(&mut g, Action::FinishSearch),
                    _ => progress(&mut g),
                }
            }
            drain(&mut g);
            assert_eq!(
                g.object_view(source).zone,
                if own_types {
                    Zone::Outside
                } else {
                    Zone::Battlefield
                },
                "{name}: 'a Dragon' includes the entering source itself; returned token then ceases to exist"
            );
        }
    }
}
