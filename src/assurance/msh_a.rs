//! Independent bounded Oracle scenarios for A.I.M. Labs through Echo.
use super::*;

// These advance only mandatory priority/trigger plumbing. Card choices remain
// explicit in each scenario; no policy, search or per-action branch checks run.
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..32 {
        if stop(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("32-action priority budget exhausted: {:?}", g.state.pending);
}
fn resolved(g: &mut Game) {
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
    while g
        .legal_actions()
        .iter()
        .any(|a| matches!(a, Action::ChooseActivationCost { .. }))
    {
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
fn attack(g: &mut Game, cards: &[CardId]) {
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
    let victim = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    spell(
        &mut g,
        p,
        "Dark Deed",
        [0, 0, 1, 0, 0, 1],
        Some(Target::Card(victim)),
    );
    resolved(&mut g);
    assert_eq!(
        g.object_view(victim).zone,
        Zone::Graveyard,
        "-4 toughness kills a 3/3"
    );
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
}

#[test]
fn msh_avengers_assemble_flash_hero_bonus_and_each_end_step_condition() {
    let p = Player::P0;
    for hero_entered in [false, true] {
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
        if hero_entered {
            g.state.step = Step::Main1;
            spell(&mut g, p, "Agent Maria Hill", [1, 0, 0, 0, 0, 0], None);
            resolved(&mut g);
        }
        let hand = g.hand_count(p);
        g.state.active = p.other();
        g.begin_step(Step::End);
        resolved(&mut g);
        assert_eq!(
            g.hand_count(p),
            hand + usize::from(hero_entered),
            "opponent's end step, intervening condition"
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
        .find(|m| m.text.eq_ignore_ascii_case(text) || m.text.starts_with(text))
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
