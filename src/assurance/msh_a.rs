//! Independent bounded Oracle scenarios for A.I.M. Labs through Echo.
use super::*;

// These advance only mandatory priority/trigger plumbing. Card choices remain
// explicit in each scenario; no policy, search or per-action branch checks run.
pub(super) fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..32 {
        if stop(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("32-action priority budget exhausted: {:?}", g.state.pending);
}
pub(super) fn resolved(g: &mut Game) {
    until(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
    });
}
fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        act(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn spell(g: &mut Game, p: Player, name: &str, mana: Mana, target: Option<Target>) -> CardId {
    priority(g, p);
    g.state.players[p.index()].mana = mana;
    let id = put(g, p, name, Zone::Hand);
    cast(g, id, target);
    assert_eq!(
        g.state.players[p.index()].mana,
        [0; 6],
        "{name}: printed cost"
    );
    id
}
fn activate(g: &mut Game, p: Player, id: CardId, ability: u16, payment: Mana, targets: &[Target]) {
    priority(g, p);
    g.state.players[p.index()].mana = payment;
    act(
        g,
        Action::AnnounceActivate {
            card: id,
            ability,
            counter_payment: Counters::default(),
        },
    );
    for _ in 0..4 {
        if !g
            .legal_actions()
            .iter()
            .any(|a| matches!(a, Action::ChooseActivationCost { .. }))
        {
            break;
        }
        choose(g, |a| matches!(a, Action::ChooseActivationCost { .. }));
    }
    for &target in targets {
        act(g, Action::ChooseTarget { target });
    }
    for _ in 0..8 {
        if g.state.announcing.is_none() {
            return;
        }
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(g, Action::FinishTargets);
        } else if g.legal_actions().contains(&Action::PayCosts { payment }) {
            act(g, Action::PayCosts { payment });
        } else {
            choose(g, |a| matches!(a, Action::ChooseActivationCost { .. }));
        }
    }
    panic!("8-action activation budget exhausted");
}
fn creatures(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(i, o)| {
            (o.zone == Zone::Battlefield && o.controller == p && g.is_creature(i as CardId))
                .then_some(i as CardId)
        })
        .collect()
}
pub(super) fn attack(g: &mut Game, cards: &[CardId]) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    for &card in cards {
        act(g, Action::Attack { card });
    }
    act(g, Action::FinishAttackers);
}

#[test]
fn msh_life_lands_enter_tapped_gain_one_and_produce_only_printed_colors() {
    // Four equivalent complete Oracle bodies, with independently specified colors.
    for (name, colors) in [
        ("A.I.M. Labs", [1, 2]),
        ("Asgardian Citadel", [3, 0]),
        ("Avengers Hangar", [0, 1]),
        ("Birnin Zana Plaza", [4, 0]),
    ] {
        for p in [Player::P0, Player::P1] {
            let mut g = scenario(p);
            let land = put(&mut g, p, name, Zone::Hand);
            act(&mut g, Action::PlayLand { card: land });
            resolved(&mut g);
            assert!(g.object_view(land).tapped, "{name}");
            assert_eq!(g.state.players[p.index()].life, 21, "{name}");
            assert!(
                !g.legal_actions()
                    .iter()
                    .any(|a| matches!(a,Action::ActivateMana{card,..} if *card==land)),
                "{name}: tapped source"
            );
            for color in colors {
                let mut g = scenario(p);
                g.state.players[p.index()].mana = [0; 6];
                let land = put(&mut g, p, name, Zone::Battlefield);
                let mut production = [0; 6];
                production[color] = 1;
                choose(
                    &mut g,
                    |a| matches!(a,Action::ActivateMana{card,production:m,..} if *card==land && *m==production),
                );
                assert_eq!(g.state.players[p.index()].mana, production, "{name}");
                assert!(g.object_view(land).tapped, "{name}");
            }
        }
    }
}

#[test]
fn msh_simple_power_up_family_cost_reduction_effect_and_once_limit() {
    // Full activation cost versus same-turn reduction is specified from Oracle.
    for (name, cast_mana, early, late, body, counters, draws) in [
        (
            "Aerial Doombot",
            [0, 1, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 5],
            [0, 1, 0, 0, 0, 5],
            (1, 1),
            3,
            0,
        ),
        (
            "Bold Biochemist",
            [0, 1, 0, 0, 0, 1],
            [0, 0, 0, 0, 0, 4],
            [0, 1, 0, 0, 0, 5],
            (1, 3),
            1,
            2,
        ),
        (
            "Brave Brawler",
            [1, 0, 0, 0, 0, 1],
            [0, 0, 0, 0, 0, 3],
            [1, 0, 0, 0, 0, 4],
            (2, 1),
            2,
            0,
        ),
        (
            "Captain Marvel, Earth's Protector",
            [2, 0, 0, 0, 0, 3],
            [0, 0, 0, 0, 0, 2],
            [2, 0, 0, 0, 0, 5],
            (5, 4),
            1,
            0,
        ),
    ] {
        for (same_turn, payment) in [(true, early), (false, late)] {
            let p = Player::P0;
            let mut g = scenario(p);
            if name == "Captain Marvel, Earth's Protector" {
                g.state.step = Step::BeginCombat;
            }
            let id = spell(&mut g, p, name, cast_mana, None);
            resolved(&mut g);
            assert_eq!(stats(&g, id), body, "{name}");
            if !same_turn {
                g.state.turn += 1;
                g.begin_turn();
            } // Real turn-start clears the same-turn cost reduction.
            let hand = g.hand_count(p);
            activate(&mut g, p, id, 0, payment, &[]);
            resolved(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6], "{name}");
            assert_eq!(
                g.counter_count(id, CounterKind::PlusOne),
                counters,
                "{name}"
            );
            assert_eq!(
                stats(&g, id),
                (body.0 + i128::from(counters), body.1 + i128::from(counters)),
                "{name}"
            );
            assert_eq!(g.hand_count(p), hand + draws, "{name}");
            if name == "Captain Marvel, Earth's Protector" {
                assert_eq!(g.counter_count(id, CounterKind::Indestructible), 1);
                let removal = spell(
                    &mut g,
                    p,
                    "Murder",
                    [0, 0, 2, 0, 0, 1],
                    Some(Target::Card(id)),
                );
                resolved(&mut g);
                assert_eq!(g.object_view(id).zone, Zone::Battlefield);
                assert_eq!(g.object_view(removal).zone, Zone::Graveyard);
            }
            g.state.players[p.index()].mana = late;
            assert!(
                !g.legal_actions().contains(&Action::AnnounceActivate {
                    card: id,
                    ability: 0,
                    counter_payment: Counters::default()
                }),
                "{name}: once per object"
            );
        }
    }
}

#[test]
fn msh_abomination_optional_fight_hybrid_cost_and_surviving_damage() {
    for fight in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let bear = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let id = spell(
            &mut g,
            p,
            "Abomination, Terrifying Titan",
            [0, 0, 0, 1, 0, 3],
            None,
        );
        resolved(&mut g);
        assert_eq!(stats(&g, id), (4, 4));
        let targets = if fight {
            vec![Target::Card(bear)]
        } else {
            vec![]
        };
        activate(&mut g, p, id, 0, [0, 0, 0, 1, 0, 2], &targets);
        resolved(&mut g);
        assert_eq!(stats(&g, id), (5, 5));
        assert_eq!(
            g.object_view(bear).zone,
            if fight {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
        assert_eq!(g.object_view(id).damage, if fight { 2 } else { 0 });
        assert!(g.keywords(id).contains(Keywords::TRAMPLE));
    }
}

#[test]
fn msh_attacks_alone_family_contrasts_one_and_two_attackers_and_expires() {
    for (name, benefit) in [
        ("Agent 13, Sharon Carter", 0),
        ("Agents of S.H.I.E.L.D.", 1),
        ("Black Widow, Double Agent", 2),
    ] {
        for alone in [true, false] {
            let p = Player::P0;
            let mut g = scenario(p);
            put(&mut g, p, name, Zone::Battlefield);
            let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let second = put(&mut g, p, "Hill Giant", Zone::Battlefield);
            let attackers = if alone {
                vec![bear]
            } else {
                vec![bear, second]
            };
            attack(&mut g, &attackers);
            resolved(&mut g);
            if benefit == 0 {
                let clues = g
                    .state
                    .objects
                    .iter()
                    .filter(|o| o.zone == Zone::Battlefield && o.card.has_subtype("Clue"))
                    .count();
                assert_eq!(clues, usize::from(alone), "{name}");
                if alone {
                    let clue = g
                        .state
                        .objects
                        .iter()
                        .position(|o| o.zone == Zone::Battlefield && o.card.has_subtype("Clue"))
                        .unwrap() as CardId;
                    let hand = g.hand_count(p);
                    activate(&mut g, p, clue, 0, [0, 0, 0, 0, 0, 2], &[]);
                    resolved(&mut g);
                    assert_eq!(
                        g.hand_count(p),
                        hand + 1,
                        "Clue costs two and sacrifices to draw one"
                    );
                    assert_ne!(g.object_view(clue).zone, Zone::Battlefield);
                }
            } else if benefit == 1 {
                assert_eq!(
                    stats(&g, bear),
                    if alone { (3, 3) } else { (2, 2) },
                    "{name}"
                );
            } else {
                assert_eq!(
                    g.keywords(bear).contains(Keywords::FIRST_STRIKE),
                    alone,
                    "{name}"
                );
                assert_eq!(g.keywords(bear).contains(Keywords::MENACE), alone, "{name}");
            }
            g.finish_cleanup();
            assert_eq!(stats(&g, bear), (2, 2), "{name}: expires");
            assert!(
                !g.keywords(bear).contains(Keywords::FIRST_STRIKE),
                "{name}: expires"
            );
        }
    }
}

#[test]
fn msh_agents_hydra_real_death_creates_printed_villain_and_zola_checks_graveyard() {
    let p = Player::P0;
    let mut g = scenario(p);
    let hydra = spell(&mut g, p, "Agents of HYDRA", [0, 0, 1, 0, 0, 1], None);
    resolved(&mut g);
    spell(
        &mut g,
        p,
        "Shock",
        [0, 0, 0, 1, 0, 0],
        Some(Target::Card(hydra)),
    );
    resolved(&mut g);
    assert_eq!(g.object_view(hydra).zone, Zone::Graveyard);
    let tokens: Vec<_> = creatures(&g, p)
        .into_iter()
        .filter(|&id| g.state.objects[id as usize].is_token())
        .collect();
    assert_eq!(tokens.len(), 1);
    assert_eq!(stats(&g, tokens[0]), (2, 1));
    assert!(g.object_view(tokens[0]).has_subtype("Villain"));
    assert!(g.keywords(tokens[0]).contains(Keywords::MENACE));
    let zola = put(&mut g, p, "Arnim Zola, Bio-Fanatic", Zone::Battlefield);
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
    let announcement = Action::AnnounceActivate {
        card: zola,
        ability: 0,
        counter_payment: Counters::default(),
    };
    assert!(!g.legal_actions().contains(&announcement));
    put(&mut g, p, "Bear Cub", Zone::Graveyard);
    activate(&mut g, p, zola, 0, [0, 0, 0, 0, 0, 3], &[]);
    resolved(&mut g);
    let token = creatures(&g, p)
        .into_iter()
        .find(|&id| id != zola && id != tokens[0])
        .unwrap();
    assert_eq!(stats(&g, token), (2, 1));
    assert!(g.object_view(token).tapped);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}

#[test]
fn msh_phil_coulson_counters_other_owned_heroes_only_and_taps() {
    let p = Player::P0;
    let mut g = scenario(p);
    let phil = put(&mut g, p, "Agent Phil Coulson", Zone::Battlefield);
    let hero = put(&mut g, p, "Agent of Atlas", Zone::Battlefield);
    let other = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let foreign = put(&mut g, p.other(), "Agent of Atlas", Zone::Battlefield);
    activate(&mut g, p, phil, 0, [0; 6], &[]);
    resolved(&mut g);
    assert!(g.object_view(phil).tapped);
    for (id, n) in [(phil, 0), (hero, 1), (other, 0), (foreign, 0)] {
        assert_eq!(
            g.counter_count(id, CounterKind::PlusOne),
            n,
            "Coulson recipient {id}"
        );
    }
}

#[test]
fn msh_scientists_connive_land_and_nonland_then_shared_basic_landcycling() {
    for discarded in ["Forest", "Shock"] {
        let p = Player::P0;
        let mut g = scenario(p);
        let draw = put(&mut g, p, discarded, Zone::Library);
        let scientist = spell(&mut g, p, "A.I.M. Scientists", [0, 1, 0, 0, 0, 3], None);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::Discard { card: draw })
        });
        act(&mut g, Action::Discard { card: draw });
        resolved(&mut g);
        assert_eq!(g.object_view(draw).zone, Zone::Graveyard);
        assert_eq!(
            stats(&g, scientist),
            if discarded == "Forest" {
                (3, 3)
            } else {
                (4, 4)
            }
        );
        assert_eq!(g.hand_count(p), 0);
    }
    for name in ["A.I.M. Scientists", "Borough Backup"] {
        let p = Player::P0;
        let mut g = scenario(p);
        let basic = put(&mut g, p, "Island", Zone::Library);
        let card = put(&mut g, p, name, Zone::Hand);
        activate(&mut g, p, card, 0, [0, 0, 0, 0, 0, 2], &[]);
        until(&mut g, |g| {
            g.legal_actions()
                .contains(&Action::ChooseSearch { card: basic })
        });
        act(&mut g, Action::ChooseSearch { card: basic });
        if g.legal_actions().contains(&Action::FinishSearch) {
            act(&mut g, Action::FinishSearch);
        }
        resolved(&mut g);
        assert_eq!(
            g.object_view(card).zone,
            Zone::Graveyard,
            "{name}: discard cost"
        );
        assert_eq!(
            g.object_view(basic).zone,
            Zone::Hand,
            "{name}: basic land in hand"
        );
        assert_eq!(g.state.players[p.index()].mana, [0; 6], "{name}: two mana");
    }
}

#[test]
fn msh_synthoids_surveil_two_and_ant_army_token_choice_and_borough_tokens() {
    let p = Player::P0;
    let mut g = scenario(p);
    let keep = put(&mut g, p, "Shock", Zone::Library);
    let mill = put(&mut g, p, "Island", Zone::Library);
    let synth = spell(&mut g, p, "A.I.M. Synthoids", [0, 0, 0, 0, 0, 2], None);
    until(&mut g, |g| {
        g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::ArrangeLibrary { .. }))
    });
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
    resolved(&mut g);
    assert_eq!(stats(&g, synth), (1, 3));
    assert_eq!(g.object_view(mill).zone, Zone::Graveyard);
    assert_eq!(g.state.players[p.index()].library.last(), Some(&keep));
    for subtype in ["Food", "Treasure"] {
        let mut g = scenario(p);
        let army = spell(&mut g, p, "Ant-Man's Army", [0, 0, 0, 0, 1, 2], None);
        until(&mut g, |g| {
            g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseToken { .. }))
        });
        choose(
            &mut g,
            |a| matches!(a,Action::ChooseToken{token}if token.has_subtype(subtype)),
        );
        resolved(&mut g);
        assert_eq!(stats(&g, army), (3, 2));
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| o.zone == Zone::Battlefield && o.is_token())
            .unwrap()
            .0 as CardId;
        assert!(g.object_view(token).has_subtype(subtype));
        if subtype == "Food" {
            activate(&mut g, p, token, 0, [0, 0, 0, 0, 0, 2], &[]);
            resolved(&mut g);
            assert_eq!(g.state.players[p.index()].life, 23);
        } else {
            g.state.players[p.index()].mana = [0; 6];
            choose(
                &mut g,
                |a| matches!(a,Action::ActivateMana{card,production:[1,0,0,0,0,0],..}if *card==token),
            );
            assert_eq!(g.state.players[p.index()].mana, [1, 0, 0, 0, 0, 0]);
        }
        assert_eq!(g.object_view(token).zone, Zone::Outside);
    }
    let mut g = scenario(p);
    spell(&mut g, p, "Borough Backup", [1, 0, 0, 0, 0, 4], None);
    resolved(&mut g);
    let tokens = creatures(&g, p);
    assert_eq!(tokens.len(), 2);
    for id in tokens {
        assert_eq!(stats(&g, id), (3, 2));
        assert!(g.keywords(id).contains(Keywords::VIGILANCE));
        assert!(g.object_view(id).has_subtype("Hero"));
    }
}

#[test]
fn msh_prowess_family_has_real_noncreature_trigger_and_cleanup() {
    for (name, mana, body) in [
        ("Agent of Atlas", [1, 0, 0, 0, 0, 1], (2, 2)),
        ("Crimson Operative", [0, 0, 0, 1, 0, 3], (3, 2)),
    ] {
        let p = Player::P0;
        let mut g = scenario(p);
        let exiled = put(&mut g, p, "Forest", Zone::Library);
        let source = spell(&mut g, p, name, mana, None);
        resolved(&mut g);
        if name == "Crimson Operative" {
            assert_eq!(g.object_view(exiled).zone, Zone::Exile);
            assert!(
                g.legal_actions()
                    .contains(&Action::PlayLand { card: exiled })
            );
            act(&mut g, Action::PlayLand { card: exiled });
            assert_eq!(g.object_view(exiled).zone, Zone::Battlefield);
        }
        spell(
            &mut g,
            p,
            "Shock",
            [0, 0, 0, 1, 0, 0],
            Some(Target::Player(p.other())),
        );
        resolved(&mut g);
        assert_eq!(stats(&g, source), (body.0 + 1, body.1 + 1), "{name}");
        g.finish_cleanup();
        assert_eq!(stats(&g, source), body, "{name}: duration");
    }
}

#[test]
fn msh_second_draw_and_attuma_tribal_draw_use_event_counts() {
    let p = Player::P0;
    let mut g = scenario(p);
    let cavalry = spell(&mut g, p, "Atlantean Cavalry", [0, 1, 0, 0, 0, 2], None);
    resolved(&mut g);
    for expected in [1, 1] {
        spell(&mut g, p, "Divination", [0, 1, 0, 0, 0, 2], None);
        resolved(&mut g);
        assert_eq!(g.counter_count(cavalry, CounterKind::PlusOne), expected);
    }
    let attuma = put(&mut g, p, "Attuma, Atlantean Warlord", Zone::Battlefield);
    let other = put(&mut g, p.other(), "Atlantean Cavalry", Zone::Battlefield);
    assert_eq!(stats(&g, cavalry), (5, 4));
    assert_eq!(stats(&g, attuma), (3, 4));
    assert_eq!(stats(&g, other), (3, 2));
    g.state.turn += 1;
    g.begin_turn();
    let hand = g.hand_count(p);
    attack(&mut g, &[cavalry, attuma]);
    resolved(&mut g);
    assert_eq!(
        g.hand_count(p),
        hand + 1,
        "one draw for multiple Merfolk attacking one player"
    );
    assert!(!g.object_view(cavalry).tapped, "vigilance");
}

#[test]
fn msh_baxter_modes_and_dark_fortress_entry_or_basic_guard() {
    let p = Player::P0;
    for witness in [false, true] {
        let mut g = scenario(p);
        let baxter = put(&mut g, p, "Baxter Building", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
        if witness {
            put(&mut g, p, "Agents of S.H.I.E.L.D.", Zone::Battlefield);
        }
        let a = Action::AnnounceActivate {
            card: baxter,
            ability: 2,
            counter_payment: Counters::default(),
        };
        assert_eq!(g.legal_actions().contains(&a), witness);
        if witness {
            let hand = g.hand_count(p);
            activate(&mut g, p, baxter, 2, [0, 0, 0, 0, 0, 4], &[]);
            resolved(&mut g);
            assert_eq!(g.hand_count(p), hand + 1);
            assert!(g.object_view(baxter).tapped);
        } else {
            choose(
                &mut g,
                |a| matches!(a,Action::ActivateMana{card,ability:1,payment:[0,0,0,0,0,4],production:[1,1,1,1,0,0],..}if *card==baxter),
            );
            assert_eq!(g.state.players[p.index()].mana, [1, 1, 1, 1, 0, 0]);
            assert!(g.object_view(baxter).tapped);
        }
    }
    for (entered, basic) in [(false, false), (true, false), (false, true)] {
        let mut g = scenario(p);
        g.state.players[p.index()].mana = [0; 6];
        let land = put(
            &mut g,
            p,
            "Dark Fortress",
            if entered {
                Zone::Hand
            } else {
                Zone::Battlefield
            },
        );
        if entered {
            act(&mut g, Action::PlayLand { card: land });
        } else {
            g.state.turn += 1;
            g.begin_turn();
            g.state.step = Step::Main1;
        }
        if basic {
            put(&mut g, p, "Forest", Zone::Battlefield);
        }
        let colors: Vec<_> = g
            .legal_actions()
            .into_iter()
            .filter_map(|a| match a {
                Action::ActivateMana {
                    card, production, ..
                } if card == land => Some(production),
                _ => None,
            })
            .collect();
        assert!(colors.contains(&[0, 0, 0, 0, 0, 1]));
        assert_eq!(colors.contains(&[0, 0, 1, 0, 0, 0]), entered || basic);
        assert_eq!(colors.contains(&[0, 0, 0, 1, 0, 0]), entered || basic);
    }
}

#[test]
fn msh_dark_deed_depower_and_blazing_crescendo_concrete_stats_and_durations() {
    let p = Player::P0;
    let mut g = scenario(p);
    let victim = put(&mut g, p.other(), "Aggressive Mammoth", Zone::Battlefield);
    spell(
        &mut g,
        p,
        "Dark Deed",
        [0, 0, 1, 0, 0, 1],
        Some(Target::Card(victim)),
    );
    resolved(&mut g);
    assert_eq!(
        stats(&g, victim),
        (4, 4),
        "Dark Deed changes an 8/8 by exactly -4/-4"
    );
    g.finish_cleanup();
    assert_eq!(stats(&g, victim), (8, 8), "Dark Deed expires at cleanup");
    for attacking in [false, true] {
        let mut g = scenario(p);
        let target = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        if attacking {
            attack(&mut g, &[target]);
        }
        let hand = g.hand_count(p);
        spell(
            &mut g,
            p,
            "Depower",
            if attacking {
                [0, 1, 0, 0, 0, 0]
            } else {
                [0, 1, 0, 0, 0, 2]
            },
            Some(Target::Card(target)),
        );
        resolved(&mut g);
        assert_eq!(stats(&g, target), (-1, 3));
        assert_eq!(g.hand_count(p), hand + 1);
        g.finish_cleanup();
        assert_eq!(stats(&g, target), (3, 3));
    }
    let mut g = scenario(p);
    let target = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let top = put(&mut g, p, "Shock", Zone::Library);
    spell(
        &mut g,
        p,
        "Blazing Crescendo",
        [0, 0, 0, 1, 0, 1],
        Some(Target::Card(target)),
    );
    resolved(&mut g);
    assert_eq!(stats(&g, target), (6, 4));
    assert_eq!(g.object_view(top).zone, Zone::Exile);
    let permission = g
        .observe(p)
        .exile_permissions
        .into_iter()
        .find(|r| r.object.card == top)
        .unwrap();
    assert!(!permission.cast_only);
    g.finish_cleanup();
    assert_eq!(stats(&g, target), (3, 3));
    // The permission survives the current cleanup and the intervening opponent
    // turn; it expires at cleanup of the controller's next actual turn.
    assert!(
        g.state
            .exile_permissions
            .iter()
            .any(|r| r.object.card == top)
    );
    g.state.active = p;
    g.state.turn += 2;
    g.begin_turn();
    g.state.step = Step::Main1;
    g.state.pending = Pending::Priority;
    g.state.priority = p;
    g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
    assert!(
        g.legal_actions()
            .contains(&Action::AnnounceExile { card: top })
    );
    g.finish_cleanup();
    assert!(
        !g.state
            .exile_permissions
            .iter()
            .any(|r| r.object.card == top)
    );
}

#[test]
fn msh_doc_samson_counter_replacement_power_mana_and_beast_permanent_history() {
    let p = Player::P0;
    let mut g = scenario(p);
    let doc = spell(
        &mut g,
        p,
        "Doc Samson, Super Psychiatrist",
        [0, 0, 0, 0, 1, 4],
        None,
    );
    resolved(&mut g);
    assert_eq!(stats(&g, doc), (3, 6));
    let bot = spell(&mut g, p, "Aerial Doombot", [0, 1, 0, 0, 0, 0], None);
    resolved(&mut g);
    activate(&mut g, p, bot, 0, [0, 0, 0, 0, 0, 5], &[]);
    resolved(&mut g);
    assert_eq!(
        g.counter_count(bot, CounterKind::PlusOne),
        4,
        "three counters plus one"
    );
    // A turn-start fixture ages Doc so the tap mana ability is ready.
    g.state.turn += 1;
    g.begin_turn();
    g.state.step = Step::Main1;
    g.state.players[p.index()].mana = [0; 6];
    choose(
        &mut g,
        |a| matches!(a,Action::ActivateMana{card,production:[0,0,0,0,3,0],..}if *card==doc),
    );
    assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 3, 0]);
    assert!(g.object_view(doc).tapped);
    let beast = spell(
        &mut g,
        p,
        "Beast, Erudite Aerialist",
        [0, 1, 0, 0, 0, 3],
        None,
    );
    resolved(&mut g);
    assert!(!g.keywords(beast).contains(Keywords::FLYING));
    spell(
        &mut g,
        p,
        "Snakeskin Veil",
        [0, 0, 0, 0, 1, 0],
        Some(Target::Card(bot)),
    );
    resolved(&mut g);
    assert!(
        !g.keywords(beast).contains(Keywords::FLYING),
        "another permanent's counters do not qualify"
    );
    spell(
        &mut g,
        p,
        "Snakeskin Veil",
        [0, 0, 0, 0, 1, 0],
        Some(Target::Card(beast)),
    );
    resolved(&mut g);
    assert_eq!(g.counter_count(beast, CounterKind::PlusOne), 2);
    assert!(g.keywords(beast).contains(Keywords::FLYING));
    g.state.turn += 1;
    g.begin_turn();
    g.state.step = Step::Main1;
    assert!(
        !g.keywords(beast).contains(Keywords::FLYING),
        "historical condition resets while counters remain"
    );
    let hand = g.hand_count(p);
    attack(&mut g, &[beast]);
    until(&mut g, |g| g.state.pending == Pending::Blockers);
    act(&mut g, Action::FinishBlockers);
    until(&mut g, |g| g.hand_count(p) == hand + 1);
    assert_eq!(
        g.state.players[p.other().index()].life,
        15,
        "five real combat damage"
    );
    let mut shield = scenario(p);
    put(
        &mut shield,
        p,
        "Doc Samson, Super Psychiatrist",
        Zone::Battlefield,
    );
    let captain = spell(
        &mut shield,
        p,
        "Captain America, Super-Soldier",
        [2, 0, 0, 0, 0, 1],
        None,
    );
    resolved(&mut shield);
    assert_eq!(
        shield.counter_count(captain, CounterKind::Shield),
        2,
        "the replacement also adds one to a non-plus-one counter kind"
    );
    for placer in [p, p.other()] {
        let mut scoped = scenario(placer);
        put(
            &mut scoped,
            p,
            "Doc Samson, Super Psychiatrist",
            Zone::Battlefield,
        );
        let ant = put(
            &mut scoped,
            placer,
            "Ant-Man, Colony Commander",
            Zone::Battlefield,
        );
        let recipient = put(&mut scoped, placer.other(), "Bear Cub", Zone::Battlefield);
        scoped.state.players[placer.index()].mana = [0, 0, 0, 0, 0, 1];
        attack(&mut scoped, &[ant]);
        until(&mut scoped, |g| {
            g.legal_actions().contains(&Action::DeclineResolutionMana)
        });
        act(
            &mut scoped,
            Action::PayResolutionMana {
                payment: [0, 0, 0, 0, 0, 1],
            },
        );
        until(&mut scoped, |g| {
            g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(recipient),
            })
        });
        target(&mut scoped, Target::Card(recipient));
        finish_targets(&mut scoped);
        resolved(&mut scoped);
        assert_eq!(
            scoped.counter_count(recipient, CounterKind::PlusOne),
            1,
            "Samson needs both own placer and own recipient, placer={placer:?}"
        );
    }
}

#[test]
fn msh_avengers_assemble_flash_hero_bonus_and_each_end_step_condition() {
    let p = Player::P0;
    for condition in 0..3 {
        let mut g = scenario(p);
        let hero = put(&mut g, p, "Agent of Atlas", Zone::Battlefield);
        let nonhero = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Agent of Atlas", Zone::Battlefield);
        g.state.step = Step::BeginCombat;
        spell(&mut g, p, "Avengers Assemble!", [1, 0, 0, 0, 0, 4], None);
        resolved(&mut g);
        assert_eq!(
            stats(&g, hero),
            (5, 5),
            "2/2 plus Assemble +2/+2 and prowess +1/+1"
        );
        assert_eq!(stats(&g, nonhero), (2, 2));
        assert_eq!(stats(&g, enemy), (2, 2));
        if condition == 1 {
            g.finish_cleanup();
            assert_eq!(g.state.active, p.other());
            g.state.step = Step::BeginCombat;
            g.state.priority = p;
            spell(
                &mut g,
                p,
                "Captain Marvel, Earth's Protector",
                [2, 0, 0, 0, 0, 3],
                None,
            );
            resolved(&mut g);
        }
        if condition == 2 {
            attack(&mut g, &[hero]);
            resolved(&mut g);
        }
        let hand = g.hand_count(p);
        g.begin_step(Step::End);
        resolved(&mut g);
        assert_eq!(
            g.hand_count(p),
            hand + usize::from(condition != 0),
            "either player's end step, intervening condition"
        );
    }
}

fn finish_targets(g: &mut Game) {
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    if g.legal_actions()
        .contains(&Action::PayCosts { payment: [0; 6] })
    {
        act(g, Action::PayCosts { payment: [0; 6] });
    }
}
fn target(g: &mut Game, target: Target) {
    if !g.legal_actions().contains(&Action::ChooseTarget { target })
        && g.legal_actions().contains(&Action::FinishTargets)
    {
        act(g, Action::FinishTargets);
    }
    act(g, Action::ChooseTarget { target });
}
fn select_mode(g: &mut Game, text: &str) {
    let modes = g.observe(g.actor().unwrap()).mode_choices;
    let mode = modes
        .iter()
        .find(|m| m.text.to_lowercase().contains(&text.to_lowercase()))
        .unwrap_or_else(|| panic!("missing mode {text}: {modes:?}"))
        .mode;
    act(g, Action::ChooseMode { mode });
}

#[test]
fn msh_alien_invasion_grows_next_token_and_requires_able_attack() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = spell(&mut g, p, "Alien Invasion", [0, 0, 0, 2, 1, 1], None);
    resolved(&mut g);
    for size in [1, 2] {
        g.begin_step(Step::BeginCombat);
        resolved(&mut g);
        let tokens = creatures(&g, p);
        assert_eq!(tokens.len(), size as usize);
        let id = *tokens.last().unwrap();
        assert_eq!(stats(&g, id), (size, size));
        assert!(g.keywords(id).contains(Keywords::HASTE));
        assert_eq!(g.counter_count(source, CounterKind::Invasion), size as u64);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        assert!(
            !g.legal_actions().contains(&Action::FinishAttackers),
            "new hasty Alien must attack"
        );
        for id in tokens {
            if !g.object_view(id).tapped {
                act(&mut g, Action::Attack { card: id });
            }
        }
        act(&mut g, Action::FinishAttackers);
        // Move to another bounded beginning-of-combat event, without simulating a game.
        g.state.pending = Pending::Priority;
        g.state.priority = p;
    }
}

#[test]
fn msh_ares_attacking_death_returns_owner_card_after_source_departs() {
    let p = Player::P0;
    let mut g = scenario(p);
    let ares = put(&mut g, p, "Ares, God of War", Zone::Battlefield);
    let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    assert!(!g.legal_actions().contains(&Action::FinishAttackers));
    act(&mut g, Action::Attack { card: ares });
    act(&mut g, Action::Attack { card: bear });
    act(&mut g, Action::FinishAttackers);
    spell(
        &mut g,
        p,
        "Shock",
        [0, 0, 0, 1, 0, 0],
        Some(Target::Card(bear)),
    );
    until(&mut g, |g| g.object_view(bear).zone == Zone::Graveyard);
    spell(
        &mut g,
        p,
        "Murder",
        [0, 0, 2, 0, 0, 1],
        Some(Target::Card(ares)),
    );
    resolved(&mut g);
    assert_eq!(
        g.object_view(bear).zone,
        Zone::Hand,
        "queued death trigger survives Ares leaving"
    );
    assert_eq!(
        g.object_view(ares).zone,
        Zone::Hand,
        "Ares was also attacking when he died"
    );
}

#[test]
fn msh_sagas_three_chapters_have_independent_draw_cost_damage_and_token_outcomes() {
    let p = Player::P0;
    for accept in [false, true] {
        let mut g = scenario(p);
        put(&mut g, p, "Arc Reactor", Zone::Battlefield);
        let hand = g.hand_count(p);
        let opposing_hand = g.hand_count(p.other());
        let saga = spell(&mut g, p, "Armor Wars", [0, 1, 0, 1, 0, 2], None);
        until(&mut g, |g| {
            g.legal_actions()
                .contains(&Action::ChooseOptional { accept })
        });
        act(&mut g, Action::ChooseOptional { accept });
        resolved(&mut g);
        assert_eq!(g.hand_count(p), hand + usize::from(accept));
        assert_eq!(g.hand_count(p.other()), opposing_hand + usize::from(accept));
        assert_eq!(g.counter_count(saga, CounterKind::Lore), 1);
        g.begin_step(Step::Main1);
        resolved(&mut g);
        assert_eq!(g.counter_count(saga, CounterKind::Lore), 2);
        let artifact = spell(
            &mut g,
            p,
            "Captain America's Shield",
            [0, 0, 0, 0, 0, 1],
            None,
        );
        resolved(&mut g);
        assert_eq!(
            g.object_view(artifact).zone,
            Zone::Battlefield,
            "artifact-only one generic discount"
        );
        g.begin_step(Step::Main1);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Player(p.other()),
            })
        });
        target(&mut g, Target::Player(p.other()));
        finish_targets(&mut g);
        resolved(&mut g);
        assert_eq!(
            g.state.players[p.other().index()].life,
            15,
            "greatest artifact MV is Reactor's five"
        );
        assert_eq!(g.object_view(saga).zone, Zone::Graveyard);
    }
    let mut g = scenario(p);
    let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let siege = spell(&mut g, p, "Avengers: Under Siege", [0, 0, 1, 1, 0, 2], None);
    resolved(&mut g);
    let villains = creatures(&g, p)
        .into_iter()
        .filter(|&id| id != bear)
        .collect::<Vec<_>>();
    assert_eq!(villains.len(), 2);
    for &id in &villains {
        assert_eq!(stats(&g, id), (2, 1));
        assert!(g.keywords(id).contains(Keywords::MENACE));
    }
    g.begin_step(Step::Main1);
    resolved(&mut g);
    assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
    assert_eq!(g.state.players[p.other().index()].life, 18);
    for &id in &villains {
        assert_eq!(g.object_view(id).zone, Zone::Battlefield);
    }
    g.begin_step(Step::Main1);
    resolved(&mut g);
    let treasures = g
        .state
        .objects
        .iter()
        .filter(|o| o.zone == Zone::Battlefield && o.card.has_subtype("Treasure"))
        .count();
    assert_eq!(treasures, 2);
    assert_eq!(g.object_view(siege).zone, Zone::Graveyard);
}

#[test]
fn msh_recovery_modes_recover_their_types_and_respect_two_mode_limit() {
    let p = Player::P0;
    for (name, mana, modes, types) in [
        (
            "Call Damage Control",
            [0, 0, 0, 0, 1, 1],
            "Artifact and land",
            ["Captain America's Shield", "Forest"],
        ),
        (
            "Call Damage Control",
            [0, 0, 0, 0, 1, 1],
            "Creature and enchantment",
            ["Bear Cub", "Avengers Assemble!"],
        ),
        (
            "Decoy Ploy",
            [0, 0, 1, 0, 0, 1],
            "Both",
            ["Agents of HYDRA", "Agent of Atlas"],
        ),
    ] {
        let mut g = scenario(p);
        let ids = types.map(|n| put(&mut g, p, n, Zone::Graveyard));
        let foreign = put(&mut g, p.other(), types[0], Zone::Graveyard);
        let card = put(&mut g, p, name, Zone::Hand);
        g.state.players[p.index()].mana = mana;
        act(&mut g, Action::AnnounceCast { card });
        select_mode(&mut g, modes);
        assert!(
            !g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(foreign)
            }),
            "{name}: own graveyard"
        );
        for id in ids {
            target(&mut g, Target::Card(id));
        }
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        resolved(&mut g);
        for id in ids {
            assert_eq!(g.object_view(id).zone, Zone::Hand, "{name}");
        }
        assert_eq!(g.object_view(foreign).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].mana, [0; 6], "{name}");
    }
}

pub(super) fn library(g: &mut Game, p: Player, names: &[&str]) -> Vec<CardId> {
    let old = g.state.players[p.index()].library.to_vec();
    for id in old {
        g.state.objects[id as usize].zone = Zone::Outside;
    }
    g.state.players[p.index()].library.clear();
    let mut ids: Vec<_> = names
        .iter()
        .rev()
        .map(|name| put(g, p, name, Zone::Library))
        .collect();
    ids.reverse();
    ids
}
fn teamwork(g: &mut Game, card: CardId, teammates: &[CardId]) {
    act(g, Action::AnnounceCast { card });
    act(
        g,
        Action::ChooseTeamwork {
            pay: !teammates.is_empty(),
        },
    );
}
fn teamwork_pay(g: &mut Game, teammates: &[CardId]) {
    pay(g);
    if !teammates.is_empty() {
        let costs = g.observe(g.actor().unwrap()).cast_costs.unwrap();
        let clause = costs
            .clauses
            .iter()
            .position(|row| matches!(row.clause, CastCostClause::Teamwork(_)))
            .unwrap() as u16;
        act(g, Action::ChooseCastCost { clause });
        for &id in teammates {
            let object = g.object_ref(id);
            act(g, Action::ChooseTeammate { object });
        }
        act(g, Action::FinishTeamwork);
    }
}

#[test]
fn msh_cruel_alliance_teamwork_changes_target_and_maria_triggers_only_for_cost() {
    let p = Player::P0;
    for paid in [false, true] {
        let mut g = scenario(p);
        let maria = put(&mut g, p, "Agent Maria Hill", Zone::Battlefield);
        let small = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let large = put(&mut g, p.other(), "Doctor Doom", Zone::Battlefield);
        let card = put(&mut g, p, "Cruel Alliance", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 2];
        teamwork(
            &mut g,
            card,
            if paid {
                std::slice::from_ref(&maria)
            } else {
                &[]
            },
        );
        assert_eq!(
            g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(large)
            }),
            paid
        );
        target(&mut g, Target::Card(if paid { large } else { small }));
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        teamwork_pay(
            &mut g,
            if paid {
                std::slice::from_ref(&maria)
            } else {
                &[]
            },
        );
        resolved(&mut g);
        assert_eq!(
            g.object_view(if paid { large } else { small }).zone,
            Zone::Exile
        );
        assert_eq!(g.state.players[p.index()].life, if paid { 23 } else { 20 });
        assert_eq!(
            g.counter_count(maria, CounterKind::PlusOne),
            u64::from(paid)
        );
        assert_eq!(g.hand_count(p), usize::from(paid));
        assert_eq!(g.object_view(maria).tapped, paid);
    }
}

#[test]
fn msh_atlantis_teamwork_requires_both_modes_and_earth_reveals_exact_eight() {
    let p = Player::P0;
    for paid in [false, true] {
        let mut g = scenario(p);
        let one = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let two = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let item = put(&mut g, p, "Captain America's Shield", Zone::Battlefield);
        let card = put(&mut g, p, "Atlantis Attacks", Zone::Hand);
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 5];
        let members = if paid { vec![one, two] } else { vec![] };
        teamwork(&mut g, card, &members);
        select_mode(&mut g, if paid { "Both" } else { "Target player" });
        target(&mut g, Target::Player(p.other()));
        if paid {
            target(&mut g, Target::Card(enemy));
            target(&mut g, Target::Card(item));
        }
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        teamwork_pay(&mut g, &members);
        resolved(&mut g);
        assert_eq!(
            g.object_view(enemy).zone,
            if paid { Zone::Hand } else { Zone::Battlefield }
        );
        assert_eq!(
            g.object_view(item).zone,
            if paid { Zone::Hand } else { Zone::Battlefield }
        );
        let leviathan = creatures(&g, p.other())
            .into_iter()
            .find(|&id| g.object_view(id).has_subtype("Leviathan"))
            .unwrap();
        assert_eq!(stats(&g, leviathan), (6, 5));
        assert!(g.keywords(leviathan).contains(Keywords::HEXPROOF));
    }
    for (paid, take) in [(false, 0), (false, 1), (true, 2)] {
        let mut g = scenario(p);
        let one = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let two = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let top = library(
            &mut g,
            p,
            &[
                "Bear Cub",
                "Hill Giant",
                "Forest",
                "Forest",
                "Forest",
                "Forest",
                "Forest",
                "Forest",
            ],
        );
        let card = put(&mut g, p, "Earth's Mightiest Heroes", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 2, 4];
        let members = if paid { vec![one, two] } else { vec![] };
        teamwork(&mut g, card, &members);
        teamwork_pay(&mut g, &members);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::FinishLibrarySelection)
        });
        for &id in top.iter().take(take) {
            act(&mut g, Action::SelectLibrary { card: id });
        }
        act(&mut g, Action::FinishLibrarySelection);
        resolved(&mut g);
        for (index, id) in top.into_iter().enumerate() {
            assert_eq!(
                g.object_view(id).zone,
                if index < take {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
        }
        assert!(g.state.players[p.index()].library.is_empty());
    }
}

#[test]
fn msh_arc_reactor_improvise_pays_generic_and_produces_three_after_untap() {
    let p = Player::P0;
    let mut g = scenario(p);
    let artifacts = [
        put(&mut g, p, "A.I.M. Synthoids", Zone::Battlefield),
        put(&mut g, p, "A.I.M. Synthoids", Zone::Battlefield),
    ];
    let card = put(&mut g, p, "Arc Reactor", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
    act(&mut g, Action::AnnounceCast { card });
    act(&mut g, Action::BeginImprovise);
    for id in artifacts {
        let object = g.object_ref(id);
        act(&mut g, Action::ChooseImproviser { object });
    }
    act(
        &mut g,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 0, 3],
        },
    );
    resolved(&mut g);
    assert!(g.object_view(card).tapped);
    for id in artifacts {
        assert!(g.object_view(id).tapped);
    }
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    g.state.turn += 1;
    g.begin_turn();
    choose(
        &mut g,
        |a| matches!(a,Action::ActivateMana{card:id,production:[0,0,0,0,0,3],..}if *id==card),
    );
    assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 0, 3]);
    assert!(g.object_view(card).tapped);
}

#[test]
fn msh_shield_equips_vigilant_attacker_and_crowd_requires_alone_then_living_legend_untaps_once() {
    let p = Player::P0;
    let mut g = scenario(p);
    let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    let crowd = put(&mut g, p, "Crowd of True Believers", Zone::Battlefield);
    let shield = spell(
        &mut g,
        p,
        "Captain America's Shield",
        [0, 0, 0, 0, 0, 2],
        None,
    );
    resolved(&mut g);
    activate(
        &mut g,
        p,
        shield,
        0,
        [0, 0, 0, 0, 0, 2],
        &[Target::Card(bear)],
    );
    resolved(&mut g);
    assert_eq!(stats(&g, bear), (2, 10));
    assert!(g.keywords(bear).contains(Keywords::VIGILANCE));
    attack(&mut g, &[bear]);
    until(&mut g, |g| {
        g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(enemy),
        })
    });
    target(&mut g, Target::Card(enemy));
    finish_targets(&mut g);
    resolved(&mut g);
    assert!(g.object_view(enemy).tapped);
    assert!(!g.object_view(bear).tapped);
    activate(&mut g, p, crowd, 0, [0; 6], &[Target::Card(bear)]);
    resolved(&mut g);
    assert_eq!(stats(&g, bear), (3, 10));
    assert_eq!(g.state.players[p.index()].life, 21);
    g.finish_cleanup();
    assert_eq!(stats(&g, bear), (2, 10));
    let mut g = scenario(p);
    put(
        &mut g,
        p,
        "Captain America, Living Legend",
        Zone::Battlefield,
    );
    let phil = put(&mut g, p, "Agent Phil Coulson", Zone::Battlefield);
    let hero = put(&mut g, p, "Agent of Atlas", Zone::Battlefield);
    for (n, tapped) in [(1, false), (2, true)] {
        activate(&mut g, p, phil, 0, [0; 6], &[]);
        resolved(&mut g);
        assert_eq!(g.object_view(phil).tapped, tapped);
        assert_eq!(g.counter_count(hero, CounterKind::PlusOne), n);
    }
}

#[test]
fn msh_absorbing_man_copies_three_allowed_types_preserves_exceptions_and_expires() {
    let p = Player::P0;
    for name in [
        "A.I.M. Labs",
        "Captain America's Shield",
        "Avengers Assemble!",
    ] {
        let mut g = scenario(p);
        let target_id = put(&mut g, p.other(), name, Zone::Battlefield);
        let wrong = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let aura = put(&mut g, p.other(), "Stab", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(wrong));
        let source = put(&mut g, p, "Absorbing Man", Zone::Battlefield);
        g.begin_step(Step::Main1);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(target_id),
            })
        });
        assert!(!g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(wrong)
        }));
        assert!(
            !g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(aura)
            }),
            "non-Aura restriction"
        );
        target(&mut g, Target::Card(target_id));
        finish_targets(&mut g);
        resolved(&mut g);
        let view = g.object_view(source);
        assert_eq!(view.name(), "Absorbing Man");
        assert_eq!(stats(&g, source), (4, 4));
        assert!(view.has_subtype("Human") && view.has_subtype("Villain"));
        assert!(g.keywords(source).contains(Keywords::VIGILANCE));
        assert_eq!(
            g.state.players[p.index()].life,
            20,
            "copying a land is not entering it"
        );
        if name == "A.I.M. Labs" {
            assert!(view.card.is_land());
            g.state.players[p.index()].mana = [0; 6];
            choose(
                &mut g,
                |a| matches!(a,Action::ActivateMana{card,production:[0,1,0,0,0,0],..}if *card==source),
            );
            assert_eq!(g.state.players[p.index()].mana, [0, 1, 0, 0, 0, 0]);
        }
        g.state.turn += 1;
        g.begin_turn();
        assert!(!g.object_view(source).card.is_land());
        assert_eq!(stats(&g, source), (4, 4));
    }
    let mut declined = scenario(p);
    put(&mut declined, p.other(), "A.I.M. Labs", Zone::Battlefield);
    let unchanged = put(&mut declined, p, "Absorbing Man", Zone::Battlefield);
    declined.begin_step(Step::Main1);
    until(&mut declined, |g| {
        g.legal_actions().contains(&Action::FinishTargets)
    });
    finish_targets(&mut declined);
    resolved(&mut declined);
    assert!(!declined.object_view(unchanged).card.is_land());
    assert_eq!(stats(&declined, unchanged), (4, 4));
    let mut g = scenario(p);
    let phil = put(&mut g, p, "Agent Phil Coulson", Zone::Battlefield);
    let echo = put(&mut g, p, "Echo, Perceptive Prodigy", Zone::Battlefield);
    let hero = put(&mut g, p, "Agent of Atlas", Zone::Battlefield);
    activate(&mut g, p, phil, 0, [0; 6], &[]);
    let ability = g.state.stack.last().unwrap().id;
    activate(
        &mut g,
        p,
        echo,
        0,
        [0, 0, 0, 0, 0, 1],
        &[Target::Stack(ability)],
    );
    resolved(&mut g);
    assert_eq!(
        g.counter_count(hero, CounterKind::PlusOne),
        2,
        "copy plus original creature-source ability"
    );
    assert_eq!(g.counter_count(phil, CounterKind::PlusOne), 0);
    assert!(g.object_view(echo).tapped);
}

#[test]
fn msh_bruce_paid_x_transform_and_real_attacking_enrage_adds_combat() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = spell(&mut g, p, "Bruce Banner", [0, 1, 0, 0, 0, 0], None);
    resolved(&mut g);
    g.state.step = Step::BeginCombat;
    for ability in [0, 1] {
        assert!(
            !g.legal_actions().contains(&Action::AnnounceActivate {
                card: source,
                ability,
                counter_payment: Counters::default()
            }),
            "Bruce activates only as a sorcery"
        );
    }
    g.state.step = Step::Main1;
    g.state.turn += 1;
    g.begin_turn();
    g.state.step = Step::Main1;
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
    act(
        &mut g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(&mut g, Action::ChooseX { value: 2 });
    pay(&mut g);
    let hand = g.hand_count(p);
    resolved(&mut g);
    assert_eq!(g.hand_count(p), hand + 2);
    assert!(g.object_view(source).tapped);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    activate(&mut g, p, source, 1, [0, 0, 0, 2, 2, 2], &[]);
    resolved(&mut g);
    assert_eq!(g.object_view(source).name(), "The Incredible Hulk");
    assert_eq!(stats(&g, source), (8, 8));
    assert!(
        g.keywords(source)
            .contains(Keywords::REACH.union(Keywords::TRAMPLE))
    );
    spell(
        &mut g,
        p,
        "Shock",
        [0, 0, 0, 1, 0, 0],
        Some(Target::Card(source)),
    );
    resolved(&mut g);
    assert_eq!(stats(&g, source), (9, 9));
    assert!(
        g.object_view(source).tapped,
        "nonattacking enrage does not untap"
    );
    g.finish_cleanup();
    g.state.active = p;
    g.state.priority = p;
    g.state.turn += 1;
    g.begin_turn();
    g.state.step = Step::Main1;
    attack(&mut g, &[source]);
    assert!(g.object_view(source).tapped);
    spell(
        &mut g,
        p,
        "Shock",
        [0, 0, 0, 1, 0, 0],
        Some(Target::Card(source)),
    );
    resolved(&mut g);
    assert_eq!(stats(&g, source), (10, 10));
    assert!(!g.object_view(source).tapped);
    until(&mut g, |g| g.state.pending == Pending::Blockers);
    act(&mut g, Action::FinishBlockers);
    until(&mut g, |g| g.state.step == Step::EndCombat);
    let turn = g.state.turn;
    act(&mut g, Action::Pass);
    act(&mut g, Action::Pass);
    assert_eq!(g.state.step, Step::BeginCombat);
    assert_eq!(g.state.turn, turn);
}

#[test]
fn msh_villain_observers_cost_discount_connive_and_first_entry_limit() {
    let p = Player::P0;
    let mut g = scenario(p);
    put(
        &mut g,
        p,
        "Baron Strucker, HYDRA Overlord",
        Zone::Battlefield,
    );
    let cross = put(
        &mut g,
        p,
        "Crossbones, Malicious Mercenary",
        Zone::Battlefield,
    );
    let discard = put(&mut g, p, "Shock", Zone::Library);
    let hydra = spell(&mut g, p, "Agents of HYDRA", [0, 0, 1, 0, 0, 0], None);
    until(&mut g, |g| {
        g.legal_actions()
            .contains(&Action::ChooseOptional { accept: true })
    });
    act(&mut g, Action::ChooseOptional { accept: true });
    until(&mut g, |g| {
        g.legal_actions()
            .contains(&Action::Discard { card: discard })
    });
    act(&mut g, Action::Discard { card: discard });
    resolved(&mut g);
    assert_eq!(stats(&g, hydra), (2, 2));
    assert_eq!(stats(&g, cross), (4, 4));
    assert_eq!(g.state.players[p.other().index()].life, 18);
    spell(&mut g, p, "Aerial Doombot", [0, 1, 0, 0, 0, 0], None);
    resolved(&mut g);
    assert_eq!(g.counter_count(cross, CounterKind::PlusOne), 1);
    assert_eq!(g.state.players[p.other().index()].life, 18);
    spell(&mut g, p, "Bear Cub", [0, 0, 0, 0, 1, 1], None);
    resolved(&mut g);
    assert_eq!(
        g.counter_count(cross, CounterKind::PlusOne),
        1,
        "non-Villain doesn't qualify"
    );
    for make_token in [false, true] {
        let mut g = scenario(p);
        let panther = put(&mut g, p, "Black Panther, Vanguard", Zone::Battlefield);
        let maria = spell(&mut g, p, "Agent Maria Hill", [1, 0, 0, 0, 0, 0], None);
        until(&mut g, |g| !g.observe(p).mode_choices.is_empty());
        select_mode(&mut g, if make_token { "Create" } else { "Creatures" });
        finish_targets(&mut g);
        resolved(&mut g);
        assert_eq!(stats(&g, panther), if make_token { (4, 4) } else { (5, 5) });
        assert_eq!(stats(&g, maria), if make_token { (2, 1) } else { (3, 2) });
        assert_eq!(creatures(&g, p).len(), if make_token { 3 } else { 2 });
        spell(&mut g, p, "Borough Backup", [1, 0, 0, 0, 0, 4], None);
        resolved(&mut g);
        assert_eq!(
            creatures(&g, p).len(),
            if make_token { 5 } else { 4 },
            "token Heroes do not trigger Panther"
        );
        g.finish_cleanup();
        assert_eq!(stats(&g, panther), (4, 4));
    }
}

#[test]
fn msh_doctor_doom_tokens_conditional_indestructibility_and_own_end_step() {
    let p = Player::P0;
    for plan in [false, true] {
        let mut g = scenario(p);
        let doom = spell(&mut g, p, "Doctor Doom", [0, 0, 2, 0, 0, 4], None);
        resolved(&mut g);
        let tokens = creatures(&g, p)
            .into_iter()
            .filter(|&id| id != doom)
            .collect::<Vec<_>>();
        assert_eq!(tokens.len(), 2);
        for &id in &tokens {
            assert_eq!(stats(&g, id), (3, 3));
            assert_eq!(g.object_view(id).name(), "Doombot");
            assert!(g.object_view(id).has_subtype("Robot"));
        }
        assert!(g.keywords(doom).contains(Keywords::INDESTRUCTIBLE));
        if plan {
            put(&mut g, p, "Claim the Kingdom", Zone::Battlefield);
        }
        for id in tokens {
            spell(
                &mut g,
                p,
                "Lightning Strike",
                [0, 0, 0, 1, 0, 1],
                Some(Target::Card(id)),
            );
            resolved(&mut g);
        }
        assert_eq!(g.keywords(doom).contains(Keywords::INDESTRUCTIBLE), plan);
        let hand = g.hand_count(p);
        g.begin_step(Step::End);
        resolved(&mut g);
        assert_eq!(g.hand_count(p), hand + 1);
        assert_eq!(g.state.players[p.index()].life, 19);
        g.state.active = p.other();
        g.begin_step(Step::End);
        resolved(&mut g);
        assert_eq!(g.hand_count(p), hand + 1);
    }
}

#[test]
fn msh_colleen_targeted_spell_scries_and_mar_vell_opponent_spell_opens_flash_window() {
    let p = Player::P0;
    let mut g = scenario(p);
    let colleen = spell(
        &mut g,
        p,
        "Colleen Wing, Street Samurai",
        [1, 0, 0, 0, 0, 1],
        None,
    );
    resolved(&mut g);
    let bear = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let top = put(&mut g, p, "Forest", Zone::Library);
    spell(
        &mut g,
        p,
        "Shock",
        [0, 0, 0, 1, 0, 0],
        Some(Target::Card(bear)),
    );
    until(&mut g, |g| {
        g.legal_actions().contains(&Action::ArrangeLibrary {
            card: top,
            destination: LibraryDestination::Bottom,
        })
    });
    act(
        &mut g,
        Action::ArrangeLibrary {
            card: top,
            destination: LibraryDestination::Bottom,
        },
    );
    resolved(&mut g);
    assert_eq!(g.counter_count(colleen, CounterKind::PlusOne), 1);
    assert_eq!(g.state.players[p.index()].library.first(), Some(&top));
    let mut g = scenario(p);
    put(&mut g, p, "Captain Mar-Vell, Space-Born", Zone::Battlefield);
    let bot = put(&mut g, p, "Aerial Doombot", Zone::Hand);
    g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
    g.state.step = Step::BeginCombat;
    assert!(
        !g.legal_actions()
            .contains(&Action::AnnounceCast { card: bot })
    );
    spell(
        &mut g,
        p.other(),
        "Shock",
        [0, 0, 0, 1, 0, 0],
        Some(Target::Player(p)),
    );
    resolved(&mut g);
    priority(&mut g, p);
    assert!(
        g.legal_actions()
            .contains(&Action::AnnounceCast { card: bot })
    );
    cast(&mut g, bot, None);
    resolved(&mut g);
    assert_eq!(g.object_view(bot).zone, Zone::Battlefield);
}

#[test]
fn msh_super_soldier_shields_and_wings_toughness_response_with_ward() {
    let p = Player::P0;
    for removal in ["Shock", "Murder"] {
        let mut g = scenario(p);
        let hero = put(&mut g, p, "Agent of Atlas", Zone::Battlefield);
        let cap = spell(
            &mut g,
            p,
            "Captain America, Super-Soldier",
            [2, 0, 0, 0, 0, 1],
            None,
        );
        resolved(&mut g);
        assert_eq!(g.counter_count(cap, CounterKind::Shield), 1);
        assert!(g.keywords(cap).contains(Keywords::FIRST_STRIKE));
        priority(&mut g, p.other());
        g.state.players[p.other().index()].mana = [0, 0, 2, 1, 0, 1];
        let shock = put(&mut g, p.other(), "Shock", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: shock });
        assert!(!g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Player(p)
        }));
        assert!(!g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(hero)
        }));
        target(&mut g, Target::Card(cap));
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        act(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 0, 1, 0, 0],
            },
        );
        resolved(&mut g);
        assert_eq!(g.counter_count(cap, CounterKind::Shield), 0);
        assert_eq!(g.object_view(cap).damage, 0);
        if removal == "Murder" {
            spell(
                &mut g,
                p.other(),
                "Murder",
                [0, 0, 2, 0, 0, 1],
                Some(Target::Card(cap)),
            );
            resolved(&mut g);
            assert_eq!(g.object_view(cap).zone, Zone::Graveyard);
        } else {
            spell(
                &mut g,
                p.other(),
                "Shock",
                [0, 0, 0, 1, 0, 0],
                Some(Target::Player(p)),
            );
            resolved(&mut g);
            assert_eq!(g.state.players[p.index()].life, 18);
        }
    }
    let mut g = scenario(p);
    let wing = put(
        &mut g,
        p,
        "Captain America, Wings of Freedom",
        Zone::Battlefield,
    );
    let hero = put(&mut g, p, "Agent Maria Hill", Zone::Battlefield);
    attack(&mut g, &[wing]);
    until(&mut g, |g| {
        g.state.pending == Pending::Priority && g.state.stack.iter().any(|s| s.source.is_some_and(|source| source.id == wing))
    });
    spell(
        &mut g,
        p,
        "Giant Growth",
        [0, 0, 0, 0, 1, 0],
        Some(Target::Card(wing)),
    );
    resolved(&mut g);
    assert_eq!(stats(&g, wing), (6, 4));
    assert_eq!(
        stats(&g, hero),
        (6, 5),
        "attack trigger reads toughness after response"
    );
    spell(
        &mut g,
        p.other(),
        "Shock",
        [0, 0, 0, 1, 0, 0],
        Some(Target::Card(wing)),
    );
    until(&mut g, |g| {
        g.legal_actions().contains(&Action::DeclineResolutionMana)
    });
    assert_eq!(
        g.observe(p.other())
            .resolution_payment
            .unwrap()
            .cost
            .generic,
        1
    );
    act(&mut g, Action::DeclineResolutionMana);
    resolved(&mut g);
    assert_eq!(g.object_view(wing).damage, 0);
    g.finish_cleanup();
    assert_eq!(stats(&g, hero), (2, 1));
    assert_eq!(stats(&g, wing), (3, 1));
}

#[test]
fn msh_cloak_hand_or_creature_exile_returns_to_original_zone_and_may_decline() {
    let p = Player::P0;
    for route in [0, 1, 2] {
        let mut g = scenario(p);
        let creature = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let secret = put(&mut g, p.other(), "Shock", Zone::Hand);
        let land = put(&mut g, p.other(), "Forest", Zone::Hand);
        let source = spell(
            &mut g,
            p,
            "Cloak and Dagger, Entwined",
            [1, 0, 1, 0, 0, 1],
            None,
        );
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Player(p.other()),
            })
        });
        target(&mut g, Target::Player(p.other()));
        target(&mut g, Target::Card(creature));
        finish_targets(&mut g);
        until(&mut g, |g| g.observe(p).hand_choice.is_some());
        assert!(g.observe(p).objects.iter().any(|o| o.id == secret));
        assert!(
            g.observe(p).objects.iter().any(|o| o.id == land),
            "whole hand is revealed"
        );
        let chosen = match route {
            0 => None,
            1 => Some(g.object_ref(secret)),
            _ => Some(g.object_ref(creature)),
        };
        act(&mut g, Action::ChooseHandCard { object: chosen });
        resolved(&mut g);
        if route > 0 {
            assert_eq!(
                g.object_view(if route == 1 { secret } else { creature })
                    .zone,
                Zone::Exile
            );
        }
        spell(
            &mut g,
            p,
            "Murder",
            [0, 0, 2, 0, 0, 1],
            Some(Target::Card(source)),
        );
        resolved(&mut g);
        assert_eq!(g.object_view(secret).zone, Zone::Hand);
        assert_eq!(g.object_view(creature).zone, Zone::Battlefield);
    }
}

#[test]
fn msh_black_widow_and_daredevil_exile_options_and_exact_play_permissions() {
    let p = Player::P0;
    for counter in [false, true] {
        let mut g = scenario(p);
        let widow = put(&mut g, p, "Black Widow, Super Spy", Zone::Battlefield);
        let top = library(&mut g, p.other(), &["Forest", "Shock", "Forest"]);
        attack(&mut g, &[widow]);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        act(&mut g, Action::FinishBlockers);
        until(&mut g, |g| {
            g.legal_actions()
                .contains(&Action::ChooseOptional { accept: counter })
        });
        act(&mut g, Action::ChooseOptional { accept: counter });
        resolved(&mut g);
        assert_eq!(g.object_view(top[0]).zone, Zone::Exile);
        assert_eq!(g.object_view(top[1]).zone, Zone::Exile);
        assert_eq!(g.object_view(top[2]).zone, Zone::Library);
        assert_eq!(
            g.counter_count(widow, CounterKind::PlusOne),
            u64::from(counter)
        );
        if !counter {
            g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
            act(&mut g, Action::AnnounceExile { card: top[1] });
            target(&mut g, Target::Player(p.other()));
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            act(
                &mut g,
                Action::PayCosts {
                    payment: [0, 0, 0, 0, 1, 0],
                },
            );
            resolved(&mut g);
            assert_eq!(
                g.state.players[p.other().index()].life,
                16,
                "combat two plus stolen Shock two"
            );
            assert_eq!(g.object_view(top[1]).owner, p.other());
            assert_eq!(g.object_view(top[1]).zone, Zone::Graveyard);
        } else {
            assert!(
                !g.state
                    .exile_permissions
                    .iter()
                    .any(|r| r.object.card == top[1])
            );
        }
    }
    for name in ["Agent Maria Hill", "Forest"] {
        let mut g = scenario(p);
        let source = spell(
            &mut g,
            p,
            "Daredevil, Man Without Fear",
            [1, 0, 0, 1, 0, 2],
            None,
        );
        resolved(&mut g);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let top = put(&mut g, p, name, Zone::Library);
        assert_eq!(
            g.observe(p).known_library_top[p.index()]
                .as_ref()
                .unwrap()
                .object
                .card,
            top
        );
        assert!(g.observe(p.other()).known_library_top[p.index()].is_none());
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        assert!(
            g.legal_actions().contains(&Action::Attack { card: source }),
            "printed haste works on entry turn"
        );
        act(&mut g, Action::Attack { card: bear });
        act(&mut g, Action::FinishAttackers);
        until(&mut g, |g| {
            g.legal_actions()
                .contains(&Action::ChooseOptional { accept: true })
        });
        act(&mut g, Action::ChooseOptional { accept: true });
        resolved(&mut g);
        assert_eq!(
            stats(&g, source),
            if name == "Forest" { (3, 4) } else { (5, 5) }
        );
        assert_eq!(g.object_view(top).zone, Zone::Exile);
        g.state.step = Step::Main2;
        g.state.priority = p;
        if name == "Forest" {
            act(&mut g, Action::PlayLand { card: top });
        } else {
            g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 0];
            act(&mut g, Action::AnnounceExile { card: top });
            pay(&mut g);
            resolved(&mut g);
        }
        assert_eq!(g.object_view(top).zone, Zone::Battlefield);
        g.finish_cleanup();
        assert_eq!(stats(&g, source), (3, 4));
    }
}

#[test]
fn msh_zemo_hand_connive_and_fifteen_symbols_offer_three_card_copies() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = spell(&mut g, p, "Baron Helmut Zemo", [0, 0, 3, 0, 0, 0], None);
    resolved(&mut g);
    let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let discard = put(&mut g, p, "Shock", Zone::Library);
    spell(
        &mut g,
        p,
        "Dark Deed",
        [0, 0, 1, 0, 0, 1],
        Some(Target::Card(bear)),
    );
    until(&mut g, |g| {
        g.legal_actions()
            .contains(&Action::Discard { card: discard })
    });
    act(&mut g, Action::Discard { card: discard });
    resolved(&mut g);
    assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
    let cards = [
        "Baron Helmut Zemo",
        "Baron Helmut Zemo",
        "Baron Helmut Zemo",
        "Baron Helmut Zemo",
        "Duress",
        "Duress",
        "Duress",
    ]
    .iter()
    .map(|name| put(&mut g, p, name, Zone::Graveyard))
    .collect::<Vec<_>>();
    let boast = Action::AnnounceActivate {
        card: source,
        ability: 0,
        counter_payment: Counters::default(),
    };
    assert!(
        !g.legal_actions().contains(&boast),
        "boast requires attacking this turn"
    );
    g.state.turn += 1;
    g.begin_turn();
    g.state.step = Step::Main1;
    attack(&mut g, &[source]);
    priority(&mut g, p);
    g.state.players[p.index()].mana = [0; 6];
    act(&mut g, boast);
    for (index, &id) in cards.iter().enumerate() {
        let object = g.object_ref(id);
        act(&mut g, Action::ChooseAdditionalCost { object });
        if index == 5 {
            assert!(
                !g.legal_actions()
                    .contains(&Action::FinishGraveyardSymbolCost)
            );
        }
    }
    act(&mut g, Action::FinishGraveyardSymbolCost);
    if g.state.announcing.is_some() {
        pay(&mut g);
    }
    until(&mut g, |g| {
        g.legal_actions().contains(&Action::FinishResolutionCasts)
    });
    for id in &cards {
        assert_eq!(g.object_view(*id).zone, Zone::Exile);
    }
    for _ in 0..3 {
        let copy = g.legal_actions().into_iter().find(|a| matches!(a,
            Action::CastExiledDuringResolution { object, .. }
                if !cards.contains(&object.card) && g.object_view(object.card).name() == "Duress"
        )).expect("a Duress copy remains within the three-spell quota");
        act(&mut g, copy);
        target(&mut g, Target::Player(p.other()));
        finish_targets(&mut g);
    }
    assert!(
        !g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::CastExiledDuringResolution { .. })),
        "three casts exhaust quota"
    );
    if g.legal_actions().contains(&Action::FinishResolutionCasts) {
        act(&mut g, Action::FinishResolutionCasts);
    }
    resolved(&mut g);
    assert_eq!(
        g.counter_count(source, CounterKind::PlusOne),
        1,
        "copies are cast from exile, not hand"
    );
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}

#[test]
fn msh_four_plans_fire_real_threshold_events_then_sacrifice_before_reflexive_targets() {
    let p = Player::P0;
    let mut g = scenario(p);
    let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let plan = spell(&mut g, p, "Claim the Kingdom", [0, 0, 0, 0, 1, 1], None);
    resolved(&mut g);
    g.add_counter(plan, CounterKind::Plan, 3);
    let land = put(&mut g, p, "Forest", Zone::Hand);
    act(&mut g, Action::PlayLand { card: land });
    until(&mut g, |g| {
        g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(bear),
        })
    });
    target(&mut g, Target::Card(bear));
    finish_targets(&mut g);
    until(&mut g, |g| {
        g.object_view(plan).zone == Zone::Graveyard
            && g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(bear),
            })
    });
    assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 1);
    target(&mut g, Target::Card(bear));
    finish_targets(&mut g);
    resolved(&mut g);
    assert_eq!(g.counter_count(bear, CounterKind::Indestructible), 1);
    let mut g = scenario(p);
    let bear = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    let plan = spell(&mut g, p, "Death to Our Enemies", [0, 0, 0, 1, 0, 2], None);
    resolved(&mut g);
    g.add_counter(plan, CounterKind::Plan, 3);
    spell(&mut g, p, "Mox Jasper", [0; 6], None);
    until(&mut g, |g| {
        g.object_view(plan).zone == Zone::Graveyard
            && g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Player(p.other()),
            })
    });
    target(&mut g, Target::Player(p.other()));
    target(&mut g, Target::Card(bear));
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(&mut g, Action::FinishTargets);
    }
    act(&mut g, Action::AllocateTargetAmount { amount: 4 });
    act(&mut g, Action::AllocateTargetAmount { amount: 3 });
    finish_targets(&mut g);
    resolved(&mut g);
    assert_eq!(g.state.players[p.other().index()].life, 16);
    assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
    let treasure = g
        .state
        .objects
        .iter()
        .find(|o| o.zone == Zone::Battlefield && o.card.has_subtype("Treasure"))
        .unwrap();
    assert!(treasure.tapped);
    let mut g = scenario(p);
    let hidden = put(&mut g, p.other(), "Shock", Zone::Hand);
    let plan = spell(
        &mut g,
        p,
        "Construct a Cosmic Cube",
        [0, 0, 1, 0, 0, 2],
        None,
    );
    resolved(&mut g);
    g.add_counter(plan, CounterKind::Plan, 6);
    assert!(!g.observe(p).objects.iter().any(|o| o.id == hidden));
    spell(&mut g, p, "Divination", [0, 1, 0, 0, 0, 2], None);
    until(&mut g, |g| {
        g.object_view(plan).zone == Zone::Graveyard
            && g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Player(p.other()),
            })
    });
    target(&mut g, Target::Player(p.other()));
    finish_targets(&mut g);
    resolved(&mut g);
    assert_eq!(g.hand_count(p), 2);
    let villains = creatures(&g, p);
    assert_eq!(villains.len(), 1);
    assert_eq!(stats(&g, villains[0]), (2, 1));
    assert!(g.keywords(villains[0]).contains(Keywords::MENACE));
    g.finish_cleanup();
    assert_eq!(g.state.active, p.other());
    assert_eq!(g.actor(), Some(p));
    assert_eq!(g.semantic_actor(), Some(p.other()));
    assert!(g.observe(p).objects.iter().any(|o| o.id == hidden));
    let mut g = scenario(p);
    let top = library(
        &mut g,
        p.other(),
        &[
            "Agent Maria Hill",
            "Agent of Atlas",
            "Borough Backup",
            "Forest",
            "Forest",
        ],
    );
    let plan = spell(&mut g, p, "Doom Reigns Supreme", [0, 0, 1, 0, 0, 1], None);
    resolved(&mut g);
    g.add_counter(plan, CounterKind::Plan, 4);
    spell(&mut g, p, "Agents of HYDRA", [0, 0, 1, 0, 0, 1], None);
    until(&mut g, |g| {
        g.object_view(plan).zone == Zone::Graveyard
            && g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Player(p.other()),
            })
    });
    target(&mut g, Target::Player(p.other()));
    finish_targets(&mut g);
    until(&mut g, |g| {
        g.legal_actions().contains(&Action::FinishResolutionCasts)
    });
    for &id in &top {
        assert_eq!(g.object_view(id).zone, Zone::Exile);
    }
    for &id in &top[..2] {
        let object = g.object_ref(id);
        act(
            &mut g,
            Action::CastExiledDuringResolution {
                object,
                face: CastFace::Normal,
            },
        );
        if g.state.announcing.is_some() {
            pay(&mut g);
        }
    }
    assert!(
        !g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::CastExiledDuringResolution { .. })),
        "third eligible spell exceeds quota two"
    );
    if g.legal_actions().contains(&Action::FinishResolutionCasts) {
        act(&mut g, Action::FinishResolutionCasts);
    }
    resolved(&mut g);
    for &id in &top[..2] {
        assert_eq!(g.object_view(id).zone, Zone::Battlefield);
        assert_eq!(g.object_view(id).controller, p);
        assert_eq!(g.object_view(id).owner, p.other());
    }
    assert_eq!(g.object_view(top[2]).zone, Zone::Exile);
    assert_eq!(g.state.players[p.index()].life, 21);
    assert_eq!(g.state.players[p.other().index()].life, 19);
}

#[test]
fn msh_cosmic_cube_private_six_card_offer_has_power_limit_and_bottoms_remainder() {
    let p = Player::P0;
    let mut g = scenario(p);
    let top = library(
        &mut g,
        p,
        &[
            "Agent Maria Hill",
            "Doctor Doom",
            "Forest",
            "Forest",
            "Forest",
            "Forest",
            "Mountain",
        ],
    );
    spell(&mut g, p, "Cosmic Cube", [0, 0, 0, 0, 0, 5], None);
    resolved(&mut g);
    let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    attack(&mut g, &[bear]);
    until(&mut g, |g| {
        g.legal_actions().contains(&Action::FinishResolutionCasts)
    });
    let allowed = Action::CastExiledDuringResolution {
        object: g.object_ref(top[0]),
        face: CastFace::Normal,
    };
    let forbidden = Action::CastExiledDuringResolution {
        object: g.object_ref(top[1]),
        face: CastFace::Normal,
    };
    assert!(g.legal_actions().contains(&allowed));
    assert!(!g.legal_actions().contains(&forbidden));
    let looked = g.observe(p).library_cast_look.unwrap();
    assert_eq!(looked.cards.len(), 6);
    for id in &top[..6] {
        assert!(looked.cards.iter().any(|o| o.id == *id));
    }
    assert!(g.observe(p.other()).library_cast_look.is_none());
    assert!(g.observe(p.other()).resolution_free_casts.is_none());
    let random_before = g.state.rng;
    act(&mut g, allowed);
    if g.state.announcing.is_some() {
        pay(&mut g);
    }
    if g.legal_actions().contains(&Action::FinishResolutionCasts) {
        act(&mut g, Action::FinishResolutionCasts);
    }
    resolved(&mut g);
    assert_eq!(g.object_view(top[0]).zone, Zone::Battlefield);
    assert_eq!(g.state.players[p.index()].library.last(), Some(&top[6]));
    let mut remaining = g.state.players[p.index()].library.to_vec();
    remaining.sort_unstable();
    let mut expected = top[1..].to_vec();
    expected.sort_unstable();
    assert_eq!(remaining, expected);
    assert_ne!(
        g.state.rng, random_before,
        "unselected five cards use random bottom order"
    );
    let mut ward = scenario(p);
    let cube = put(&mut ward, p, "Cosmic Cube", Zone::Battlefield);
    spell(
        &mut ward,
        p.other(),
        "Disenchant",
        [1, 0, 0, 0, 0, 1],
        Some(Target::Card(cube)),
    );
    until(&mut ward, |g| {
        g.legal_actions().contains(&Action::DeclineResolutionMana)
    });
    assert_eq!(
        ward.observe(p.other())
            .resolution_payment
            .unwrap()
            .cost
            .generic,
        2
    );
    act(&mut ward, Action::DeclineResolutionMana);
    resolved(&mut ward);
    assert_eq!(
        ward.object_view(cube).zone,
        Zone::Battlefield,
        "unpaid ward two counters removal"
    );
}

#[test]
fn msh_avengers_disassembled_modes_damage_land_and_optional_basic_search() {
    let p = Player::P0;
    for both in [false, true] {
        let mut g = scenario(p);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let giant = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let land = put(&mut g, p.other(), "Forest", Zone::Battlefield);
        let basic = put(&mut g, p.other(), "Island", Zone::Library);
        let card = put(&mut g, p, "Avengers Disassembled", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 2, 0, 1];
        act(&mut g, Action::AnnounceCast { card });
        select_mode(&mut g, if both { "both" } else { "deal 3" });
        if both {
            target(&mut g, Target::Card(land));
        }
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        if both {
            until(&mut g, |g| {
                g.legal_actions()
                    .contains(&Action::ChooseOptional { accept: true })
            });
            assert_eq!(g.actor(), Some(p.other()));
            act(&mut g, Action::ChooseOptional { accept: true });
            act(&mut g, Action::ChooseSearch { card: basic });
            if g.legal_actions().contains(&Action::FinishSearch) {
                act(&mut g, Action::FinishSearch);
            }
        }
        resolved(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
        assert_eq!(g.object_view(giant).zone, Zone::Graveyard);
        assert_eq!(
            g.object_view(land).zone,
            if both {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
        assert_eq!(
            g.object_view(basic).zone,
            if both {
                Zone::Battlefield
            } else {
                Zone::Library
            }
        );
        if both {
            assert!(g.object_view(basic).tapped);
        }
    }
}

#[test]
fn msh_tower_castle_mana_restrictions_private_pick_and_castle_sacrifice() {
    let p = Player::P0;
    for name in [
        "Baxter Building",
        "Avengers Tower",
        "Castle Doom",
        "Dark Fortress",
    ] {
        let mut g = scenario(p);
        let id = put(&mut g, p, name, Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        choose(
            &mut g,
            |a| matches!(a, Action::ActivateMana { card, ability: 0, production: [0,0,0,0,0,1], .. } if *card == id),
        );
        assert_eq!(
            g.state.players[p.index()].mana,
            [0, 0, 0, 0, 0, 1],
            "{name}"
        );
        assert!(g.object_view(id).tapped, "{name}");
    }
    for (name, qualified, unqualified, initial, production) in [
        (
            "Avengers Tower",
            "Agent Maria Hill",
            "Shock",
            [0; 6],
            [1, 0, 0, 0, 0, 0],
        ),
        (
            "Castle Doom",
            "Captain America's Shield",
            "Agent Maria Hill",
            [0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0],
        ),
    ] {
        let mut g = scenario(p);
        let land = put(&mut g, p, name, Zone::Battlefield);
        g.state.players[p.index()].mana = initial;
        let spell_id = put(&mut g, p, qualified, Zone::Hand);
        choose(
            &mut g,
            |a| matches!(a,Action::ActivateMana{card,ability:1,production:m,..}if *card==land&&*m==production),
        );
        cast(&mut g, spell_id, None);
        resolved(&mut g);
        assert_eq!(g.object_view(spell_id).zone, Zone::Battlefield, "{name}");
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        let mut g = scenario(p);
        let land = put(&mut g, p, name, Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        let id = put(&mut g, p, unqualified, Zone::Hand);
        let color = if name == "Avengers Tower" {
            [0, 0, 0, 1, 0, 0]
        } else {
            [1, 0, 0, 0, 0, 0]
        };
        choose(
            &mut g,
            |a| matches!(a,Action::ActivateMana{card,ability:1,production:m,..}if *card==land&&*m==color),
        );
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: id }),
            "{name}: restricted mana cannot cast {unqualified}"
        );
    }
    let mut g = scenario(p);
    let tower = put(&mut g, p, "Avengers Tower", Zone::Battlefield);
    let brawler = put(&mut g, p, "Brave Brawler", Zone::Battlefield);
    g.state.turn += 1;
    g.begin_turn();
    g.state.step = Step::Main1;
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
    choose(
        &mut g,
        |a| matches!(a,Action::ActivateMana{card,ability:1,production:[1,0,0,0,0,0],..}if *card==tower),
    );
    act(
        &mut g,
        Action::AnnounceActivate {
            card: brawler,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(&mut g);
    resolved(&mut g);
    assert_eq!(
        g.counter_count(brawler, CounterKind::PlusOne),
        2,
        "Tower also pays Hero activations"
    );
    let mut g = scenario(p);
    let tower = put(&mut g, p, "Avengers Tower", Zone::Battlefield);
    let top = library(
        &mut g,
        p,
        &["Agent Maria Hill", "Shock", "Forest", "Mountain"],
    );
    activate(&mut g, p, tower, 2, [0, 0, 0, 0, 0, 4], &[]);
    until(&mut g, |g| {
        g.legal_actions()
            .contains(&Action::SelectLibrary { card: top[0] })
    });
    assert!(g.observe(p.other()).library_selection.is_none());
    act(&mut g, Action::SelectLibrary { card: top[0] });
    if g.legal_actions().contains(&Action::FinishLibrarySelection) {
        act(&mut g, Action::FinishLibrarySelection);
    }
    for &id in &top[1..3] {
        if g.legal_actions()
            .contains(&Action::OrderLibraryBottom { card: id })
        {
            act(&mut g, Action::OrderLibraryBottom { card: id });
        }
    }
    resolved(&mut g);
    assert_eq!(g.object_view(top[0]).zone, Zone::Hand);
    assert!(
        g.observe(p.other()).objects.iter().any(|o| o.id == top[0]),
        "chosen Hero is revealed"
    );
    assert_eq!(g.state.players[p.index()].library.last(), Some(&top[3]));
    let mut g = scenario(p);
    let castle = put(&mut g, p, "Castle Doom", Zone::Battlefield);
    let artifact = put(&mut g, p, "A.I.M. Synthoids", Zone::Battlefield);
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
    g.state.step = Step::BeginCombat;
    let announce = Action::AnnounceActivate {
        card: castle,
        ability: 2,
        counter_payment: Counters::default(),
    };
    assert!(!g.legal_actions().contains(&announce));
    g.state.step = Step::Main1;
    act(&mut g, announce);
    let object = g.object_ref(artifact);
    act(&mut g, Action::ChooseAdditionalCost { object });
    pay(&mut g);
    resolved(&mut g);
    assert_eq!(g.object_view(artifact).zone, Zone::Graveyard);
    let token = creatures(&g, p)[0];
    assert_eq!(stats(&g, token), (3, 3));
    assert_eq!(g.object_view(token).name(), "Doombot");
    assert!(g.object_view(castle).tapped);
}

#[test]
fn msh_quinjet_any_color_and_crew_threshold_animate_only_until_cleanup() {
    let p = Player::P0;
    let mut g = scenario(p);
    let jet = spell(&mut g, p, "Dependable Quinjet", [0, 0, 0, 0, 0, 3], None);
    resolved(&mut g);
    assert!(!g.is_creature(jet));
    choose(
        &mut g,
        |a| matches!(a,Action::ActivateMana{card,production:[0,0,0,0,1,0],..}if *card==jet),
    );
    assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 1, 0]);
    g.state.turn += 1;
    g.begin_turn();
    g.state.step = Step::Main1;
    let giant = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    act(
        &mut g,
        Action::AnnounceActivate {
            card: jet,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    let object = g.object_ref(giant);
    act(&mut g, Action::ChooseCrewer { object });
    assert!(!g.legal_actions().contains(&Action::FinishCrew));
    let object = g.object_ref(bear);
    act(&mut g, Action::ChooseCrewer { object });
    act(&mut g, Action::FinishCrew);
    if g.state.announcing.is_some() {
        pay(&mut g);
    }
    resolved(&mut g);
    assert!(g.is_creature(jet));
    assert_eq!(stats(&g, jet), (3, 3));
    assert!(g.keywords(jet).contains(Keywords::FLYING));
    assert!(g.object_view(giant).tapped && g.object_view(bear).tapped);
    g.finish_cleanup();
    assert!(!g.is_creature(jet));
}

#[test]
fn msh_ant_man_optional_payment_reflexive_counter_and_once_each_turn_insect() {
    let p = Player::P0;
    for accept in [false, true] {
        let mut g = scenario(p);
        let ant = put(&mut g, p, "Ant-Man, Colony Commander", Zone::Battlefield);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        attack(&mut g, &[ant]);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::DeclineResolutionMana)
        });
        act(
            &mut g,
            if accept {
                Action::PayResolutionMana {
                    payment: [0, 0, 0, 0, 0, 1],
                }
            } else {
                Action::DeclineResolutionMana
            },
        );
        if accept {
            until(&mut g, |g| {
                g.legal_actions().contains(&Action::ChooseTarget {
                    target: Target::Card(enemy),
                })
            });
            target(&mut g, Target::Card(enemy));
            finish_targets(&mut g);
        }
        resolved(&mut g);
        assert_eq!(
            g.counter_count(enemy, CounterKind::PlusOne),
            u64::from(accept)
        );
        let insects = |g: &Game| {
            g.state
                .objects
                .iter()
                .filter(|o| {
                    o.zone == Zone::Battlefield && o.is_token() && o.card.has_subtype("Insect")
                })
                .count()
        };
        assert_eq!(insects(&g), usize::from(accept));
        spell(
            &mut g,
            p,
            "Snakeskin Veil",
            [0, 0, 0, 0, 1, 0],
            Some(Target::Card(own)),
        );
        resolved(&mut g);
        assert_eq!(insects(&g), 1);
        g.state.turn += 1;
        g.begin_turn();
        g.state.step = Step::Main1;
        spell(
            &mut g,
            p,
            "Snakeskin Veil",
            [0, 0, 0, 0, 1, 0],
            Some(Target::Card(own)),
        );
        resolved(&mut g);
        assert_eq!(insects(&g), 2);
        for id in creatures(&g, p)
            .into_iter()
            .filter(|&id| g.state.objects[id as usize].is_token())
        {
            assert_eq!(stats(&g, id), (1, 1));
        }
    }
}

#[test]
fn msh_bullseye_optional_entry_and_both_activated_alternative_costs() {
    let p = Player::P0;
    for route in [0, 1, 2] {
        let mut g = scenario(p);
        let artifact = put(&mut g, p, "A.I.M. Synthoids", Zone::Battlefield);
        let discard = put(&mut g, p, "Shock", Zone::Hand);
        let source = spell(
            &mut g,
            p,
            "Bullseye, Death Dealer",
            [0, 0, 1, 0, 0, 2],
            None,
        );
        until(&mut g, |g| {
            g.legal_actions().iter().any(|a| {
                matches!(
                    a,
                    Action::ChooseOptional { .. }
                        | Action::DeclineEffectPayment
                        | Action::Discard { .. }
                        | Action::Sacrifice { .. }
                )
            })
        });
        if g.legal_actions()
            .contains(&Action::ChooseOptional { accept: route != 0 })
        {
            act(&mut g, Action::ChooseOptional { accept: route != 0 });
        }
        if route == 0 {
            if g.legal_actions().contains(&Action::DeclineEffectPayment) {
                act(&mut g, Action::DeclineEffectPayment);
            }
        } else {
            if route == 1 {
                let object = g.object_ref(artifact);
                act(&mut g, Action::Sacrifice { object });
            } else {
                act(&mut g, Action::Discard { card: discard });
            }
            until(&mut g, |g| {
                g.legal_actions().contains(&Action::ChooseTarget {
                    target: Target::Player(p.other()),
                })
            });
            target(&mut g, Target::Player(p.other()));
            finish_targets(&mut g);
        }
        resolved(&mut g);
        assert_eq!(stats(&g, source), (2, 3));
        assert_eq!(
            g.state.players[p.other().index()].life,
            if route == 0 { 20 } else { 18 }
        );
        assert_eq!(
            g.object_view(artifact).zone,
            if route == 1 {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
        assert_eq!(
            g.object_view(discard).zone,
            if route == 2 {
                Zone::Graveyard
            } else {
                Zone::Hand
            }
        );
    }
    for discard in [false, true] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Bullseye, Death Dealer", Zone::Battlefield);
        let cost = put(
            &mut g,
            p,
            if discard { "Shock" } else { "A.I.M. Synthoids" },
            if discard {
                Zone::Hand
            } else {
                Zone::Battlefield
            },
        );
        let land = put(&mut g, p, "Forest", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        act(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        if g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::ChooseActivationCost { .. }))
        {
            choose(&mut g, |a| matches!(a, Action::ChooseActivationCost { .. }));
        }
        target(&mut g, Target::Player(p.other()));
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        assert!(!g.legal_actions().contains(&Action::ChooseAdditionalCost {
            object: g.object_ref(land)
        }));
        let object = g.object_ref(cost);
        act(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        resolved(&mut g);
        assert_eq!(g.object_view(cost).zone, Zone::Graveyard);
        assert!(g.object_view(source).tapped);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}

#[test]
fn msh_echo_copies_creature_ability_and_changes_only_the_copys_target() {
    let p = Player::P1;
    let mut g = scenario(p);
    let bullseye = put(&mut g, p, "Bullseye, Death Dealer", Zone::Battlefield);
    let echo = put(&mut g, p, "Echo, Perceptive Prodigy", Zone::Battlefield);
    let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let discard = put(&mut g, p, "Shock", Zone::Hand);
    let shield = put(&mut g, p, "Captain America's Shield", Zone::Battlefield);
    activate(
        &mut g,
        p,
        shield,
        0,
        [0, 0, 0, 0, 0, 2],
        &[Target::Card(bear)],
    );
    let noncreature_ability = g.state.stack.last().unwrap().id;
    let foreign = put(&mut g, p.other(), "Agent Phil Coulson", Zone::Battlefield);
    activate(&mut g, p.other(), foreign, 0, [0; 6], &[]);
    let opponents_ability = g.state.stack.last().unwrap().id;
    spell(
        &mut g,
        p,
        "Shock",
        [0, 0, 0, 1, 0, 0],
        Some(Target::Player(p.other())),
    );
    let spell_on_stack = g.state.stack.last().unwrap().id;
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
    act(
        &mut g,
        Action::AnnounceActivate {
            card: bullseye,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    if g.legal_actions()
        .iter()
        .any(|a| matches!(a, Action::ChooseActivationCost { .. }))
    {
        choose(&mut g, |a| matches!(a, Action::ChooseActivationCost { .. }));
    }
    target(&mut g, Target::Player(p.other()));
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(&mut g, Action::FinishTargets);
    }
    let object = g.object_ref(discard);
    act(&mut g, Action::ChooseAdditionalCost { object });
    pay(&mut g);
    let original = g.state.stack.last().unwrap().id;
    priority(&mut g, p);
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
    act(
        &mut g,
        Action::AnnounceActivate {
            card: echo,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    for forbidden in [noncreature_ability, opponents_ability, spell_on_stack] {
        assert!(
            !g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Stack(forbidden)
            }),
            "Echo requires a controlled ability from a creature source"
        );
    }
    target(&mut g, Target::Stack(original));
    finish_targets(&mut g);
    act(
        &mut g,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 0, 1],
        },
    );
    until(&mut g, |g| g.observe(p).retarget.is_some());
    act(
        &mut g,
        Action::SetStackTarget {
            index: 0,
            target: Target::Card(bear),
        },
    );
    act(&mut g, Action::FinishStackTargets);
    resolved(&mut g);
    assert_eq!(
        g.object_view(bear).zone,
        Zone::Graveyard,
        "copy dealt two to new target"
    );
    assert_eq!(
        g.state.players[p.other().index()].life,
        16,
        "original two-damage target plus original Shock were retained"
    );
    assert_eq!(g.object_view(discard).zone, Zone::Graveyard);
    assert!(g.object_view(echo).tapped && g.object_view(bullseye).tapped);
}

#[test]
fn msh_printed_keyword_bindings_use_shared_combat_and_replacement_proofs() {
    // These bindings supplement each card's independent ability scenarios. Actual
    // primitive outcomes are covered in assurance::interactions and msh_b combat
    // families; a matching flag alone is never the card's complete assurance.
    for (name, expected) in [
        ("Abomination, Terrifying Titan", Keywords::TRAMPLE),
        ("Absorbing Man", Keywords::VIGILANCE),
        ("Aerial Doombot", Keywords::FLYING),
        ("Agent Phil Coulson", Keywords::VIGILANCE),
        ("Atlantean Cavalry", Keywords::VIGILANCE),
        ("Black Widow, Double Agent", Keywords::DEATHTOUCH),
        ("Black Widow, Super Spy", Keywords::MENACE),
        ("Brave Brawler", Keywords::LIFELINK),
        ("Captain America's Shield", Keywords::INDESTRUCTIBLE),
        ("Captain America, Living Legend", Keywords::VIGILANCE),
        ("Captain America, Super-Soldier", Keywords::FIRST_STRIKE),
        (
            "Captain America, Wings of Freedom",
            Keywords::FLYING.union(Keywords::FIRST_STRIKE),
        ),
        (
            "Captain Mar-Vell, Space-Born",
            Keywords::FLYING.union(Keywords::VIGILANCE),
        ),
        (
            "Captain Marvel, Earth's Protector",
            Keywords::FLYING.union(Keywords::LIFELINK),
        ),
        (
            "Cloak and Dagger, Entwined",
            Keywords::DEATHTOUCH.union(Keywords::LIFELINK),
        ),
        ("Crossbones, Malicious Mercenary", Keywords::DEATHTOUCH),
        (
            "Daredevil, Man Without Fear",
            Keywords::VIGILANCE.union(Keywords::HASTE),
        ),
        ("Echo, Perceptive Prodigy", Keywords::VIGILANCE),
    ] {
        let mut g = scenario(Player::P0);
        let id = put(&mut g, Player::P0, name, Zone::Battlefield);
        assert!(
            g.keywords(id).contains(expected),
            "{name}: literal printed keyword binding"
        );
    }
}

#[test]
fn msh_a_literal_creature_costs_and_bodies_supplement_full_ability_scenarios() {
    // Literal values transcribed from the pinned Oracle faces. This table checks
    // paid casting and base bodies; the separate families prove printed abilities.
    for (name, mana, power, toughness) in [
        ("A.I.M. Scientists", [0, 1, 0, 0, 0, 3], 3, 3),
        ("A.I.M. Synthoids", [0, 0, 0, 0, 0, 2], 1, 3),
        ("Abomination, Terrifying Titan", [0, 0, 0, 1, 0, 3], 4, 4),
        ("Absorbing Man", [0, 1, 0, 0, 1, 1], 4, 4),
        ("Aerial Doombot", [0, 1, 0, 0, 0, 0], 1, 1),
        ("Agent 13, Sharon Carter", [1, 0, 0, 0, 0, 2], 3, 2),
        ("Agent Maria Hill", [1, 0, 0, 0, 0, 0], 2, 1),
        ("Agent Phil Coulson", [1, 0, 0, 0, 0, 1], 2, 2),
        ("Agent of Atlas", [1, 0, 0, 0, 0, 1], 2, 2),
        ("Agents of HYDRA", [0, 0, 1, 0, 0, 1], 1, 1),
        ("Agents of S.H.I.E.L.D.", [1, 0, 0, 0, 0, 2], 2, 4),
        ("Ant-Man's Army", [0, 0, 0, 0, 1, 2], 3, 2),
        ("Ant-Man, Colony Commander", [0, 1, 0, 0, 1, 1], 2, 2),
        ("Ares, God of War", [0, 0, 1, 1, 0, 1], 4, 3),
        ("Arnim Zola, Bio-Fanatic", [0, 0, 1, 0, 0, 2], 2, 3),
        ("Atlantean Cavalry", [0, 1, 0, 0, 0, 2], 3, 2),
        ("Attuma, Atlantean Warlord", [0, 2, 0, 0, 0, 2], 3, 4),
        ("Baron Helmut Zemo", [0, 0, 3, 0, 0, 0], 3, 3),
        ("Baron Strucker, HYDRA Overlord", [0, 0, 1, 0, 0, 2], 2, 2),
        ("Beast, Erudite Aerialist", [0, 0, 0, 0, 1, 3], 3, 3),
        ("Black Panther, Vanguard", [1, 0, 0, 0, 1, 2], 4, 4),
        ("Black Widow, Double Agent", [1, 0, 1, 0, 0, 1], 3, 2),
        ("Black Widow, Super Spy", [0, 0, 1, 0, 0, 1], 2, 1),
        ("Bold Biochemist", [0, 1, 0, 0, 0, 1], 1, 3),
        ("Brave Brawler", [1, 0, 0, 0, 0, 1], 2, 1),
        ("Bruce Banner", [0, 1, 0, 0, 0, 0], 1, 1),
        ("Bullseye, Death Dealer", [0, 0, 1, 0, 0, 2], 2, 3),
        ("Captain America, Living Legend", [1, 1, 0, 0, 0, 1], 3, 4),
        ("Captain America, Super-Soldier", [2, 0, 0, 0, 0, 1], 3, 2),
        (
            "Captain America, Wings of Freedom",
            [1, 0, 0, 0, 0, 2],
            3,
            1,
        ),
        ("Captain Mar-Vell, Space-Born", [1, 0, 0, 0, 0, 4], 4, 4),
        (
            "Captain Marvel, Earth's Protector",
            [2, 0, 0, 0, 0, 3],
            5,
            4,
        ),
        ("Cloak and Dagger, Entwined", [1, 0, 1, 0, 0, 1], 2, 2),
        ("Colleen Wing, Street Samurai", [1, 0, 0, 0, 0, 1], 2, 2),
        ("Crimson Operative", [0, 0, 0, 1, 0, 3], 3, 2),
        ("Crossbones, Malicious Mercenary", [0, 0, 1, 0, 0, 3], 3, 3),
        ("Crowd of True Believers", [1, 0, 0, 0, 0, 0], 1, 2),
        ("Daredevil, Man Without Fear", [1, 0, 0, 1, 0, 2], 3, 4),
        ("Doc Samson, Super Psychiatrist", [0, 0, 0, 0, 1, 4], 3, 6),
        ("Doctor Doom", [0, 0, 2, 0, 0, 4], 3, 3),
        ("Echo, Perceptive Prodigy", [0, 1, 0, 0, 0, 2], 1, 4),
    ] {
        let mut g = scenario(Player::P0);
        let id = spell(&mut g, Player::P0, name, mana, None);
        until(&mut g, |g| g.object_view(id).zone == Zone::Battlefield);
        assert_eq!(
            stats(&g, id),
            (power, toughness),
            "{name}: independent base body"
        );
        assert!(g.is_creature(id), "{name}");
    }
}
