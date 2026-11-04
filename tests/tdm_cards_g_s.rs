//! Actual TDM G–S definitions: printed outcomes and representative owned contracts.
mod support;
use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

fn idle(g: &Game) -> bool {
    g.outcome().is_some()
        || (g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty())
}
fn advance(g: &mut Game, targets: &[Target], done: impl Fn(&Game) -> bool) {
    for _ in 0..96 {
        if done(g) {
            return;
        }
        let actions = g.legal_actions();
        if let Some(a) = targets
            .iter()
            .map(|&target| Action::ChooseTarget { target })
            .find(|a| actions.contains(a))
        {
            act(g, a);
            continue;
        }
        let a = actions
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::FinishStackTargets
                        | Action::ChooseReplacement { .. }
                )
            })
            .unwrap_or_else(|| panic!("explicit choice needed: {:?}", g.state.pending));
        act(g, a);
    }
    panic!("96-action card scenario budget");
}
fn drain(g: &mut Game) {
    advance(g, &[], idle);
}
fn drain_to(g: &mut Game, ids: &[CardId]) {
    advance(
        g,
        &ids.iter().copied().map(Target::Card).collect::<Vec<_>>(),
        idle,
    );
}
fn activate(g: &mut Game, source: CardId, ability: u16, targets: &[CardId]) {
    act(
        g,
        Action::AnnounceActivate {
            card: source,
            ability,
            counter_payment: Counters::default(),
        },
    );
    for &target in targets {
        act(
            g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
}
fn battlefield(g: &Game, name: &str) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield && o.is_token() && o.card.has_subtype(name))
                .then_some(id as CardId)
        })
        .collect()
}

#[test]
fn gurmag_nightwatch_private_optional_top_choice_and_owned_branches() {
    for p in [Player::P0, Player::P1] {
        let mut parent = scenario(p);
        let a = put(&mut parent, p, "Island", Zone::Library);
        let b = put(&mut parent, p, "Swamp", Zone::Library);
        let c = put(&mut parent, p, "Mountain", Zone::Library);
        let source = put(&mut parent, p, "Gurmag Nightwatch", Zone::Hand);
        cast(&mut parent, source, None);
        advance(&mut parent, &[], |g| {
            matches!(g.state.pending, Pending::EffectLibrarySelection { .. })
        });
        assert!(parent.observe(p).library_selection.is_some());
        assert!(parent.observe(p.other()).library_selection.is_none());
        let action = Action::SelectLibrary { card: b };
        let revision = parent.revision();
        let index = parent
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        let saved = parent.snapshot();
        let mut raw = parent.clone();
        let mut indexed = parent.clone();
        let mut none = parent.clone();
        drop(parent);
        reject(&mut raw, Action::SelectLibrary { card: source });
        assert_eq!(raw.apply(p.other(), action.clone()), Err(Error::WrongActor));
        assert_eq!(raw.snapshot(), saved);
        act(&mut raw, action.clone());
        indexed.apply_index(p, revision, index).unwrap();
        for child in [&mut raw, &mut indexed] {
            act(child, Action::FinishLibrarySelection);
            drain(child);
        }
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.top_library_card(p), Some(b));
        assert_eq!(raw.object_view(a).zone, Zone::Graveyard);
        assert_eq!(raw.object_view(c).zone, Zone::Graveyard);
        act(&mut none, Action::FinishLibrarySelection);
        drain(&mut none);
        for id in [a, b, c] {
            assert_eq!(none.object_view(id).zone, Zone::Graveyard);
        }
        indexed.restore(&saved).unwrap();
        assert_eq!(
            indexed.apply_index(p, revision, index),
            Err(Error::StaleRevision)
        );
        act(&mut indexed, action);
        act(&mut indexed, Action::FinishLibrarySelection);
        drain(&mut indexed);
        assert_eq!(indexed.snapshot(), raw.snapshot());
    }
}
#[test]
fn gurmag_rakshasa_opposite_target_groups_survive_partial_invalidation() {
    let p = Player::P0;
    let mut g = scenario(p);
    let own = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    let rak = put(&mut g, p, "Gurmag Rakshasa", Zone::Hand);
    cast(&mut g, rak, None);
    advance(&mut g, &[Target::Card(enemy), Target::Card(own)], |g| {
        g.object_view(rak).zone == Zone::Battlefield
            && g.state.pending == Pending::Priority
            && !g.state.stack.is_empty()
    });
    assert!(g.keywords(rak).contains(Keywords::MENACE));
    g.move_object(enemy, Zone::Hand);
    drain(&mut g);
    assert_eq!(stats(&g, own), (4, 4));
    assert_eq!(g.object_view(enemy).zone, Zone::Hand);
    let mut g = scenario(p);
    let enemy = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    let rak = put(&mut g, p, "Gurmag Rakshasa", Zone::Hand);
    cast(&mut g, rak, None);
    drain_to(&mut g, &[enemy, rak]);
    assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
    assert_eq!(stats(&g, rak), (7, 7));
}
#[test]
fn host_enters_with_two_and_transfers_independent_dying_creatures_counters() {
    let p = Player::P1;
    let mut g = scenario(p);
    let host = put(&mut g, p, "Host of the Hereafter", Zone::Hand);
    cast(&mut g, host, None);
    drain(&mut g);
    assert_eq!(stats(&g, host), (4, 4));
    assert_eq!(g.counter_count(host, CounterKind::PlusOne), 2);
    let dead = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let target = put(&mut g, p, "Shivan Dragon", Zone::Battlefield);
    g.add_counter(dead, CounterKind::Flying, 1);
    g.add_counter(dead, CounterKind::Soul, 3);
    g.move_object(dead, Zone::Graveyard);
    drain_to(&mut g, &[target]);
    assert_eq!(g.counter_count(target, CounterKind::Soul), 3);
    assert_eq!(g.counter_count(target, CounterKind::Flying), 1);
    assert_eq!(g.counter_count(target, CounterKind::PlusOne), 0);
    g.move_object(host, Zone::Graveyard);
    drain_to(&mut g, &[target]);
    assert_eq!(g.counter_count(target, CounterKind::PlusOne), 2);
}
#[test]
fn iridescent_tiger_cast_entry_adds_each_color_but_uncast_entry_does_not() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let tiger = put(&mut g, p, "Iridescent Tiger", Zone::Hand);
        cast(&mut g, tiger, None);
        advance(&mut g, &[], |g| {
            g.object_view(tiger).zone == Zone::Battlefield
                && g.state.pending == Pending::Priority
                && !g.state.stack.is_empty()
        });
        let before = g.state.players[p.index()].mana;
        drain(&mut g);
        let after = g.state.players[p.index()].mana;
        for i in 0..6 {
            assert_eq!(after[i], before[i] + u32::from(i < 5));
        }
        let before = after;
        g.move_object(tiger, Zone::Hand);
        g.move_object(tiger, Zone::Battlefield);
        drain(&mut g);
        assert_eq!(g.state.players[p.index()].mana, before);
    }
}
#[test]
fn renew_keyword_counter_batches_and_sage_add_before_double() {
    for (name, expected) in [
        (
            "Qarsi Revenant",
            vec![
                (CounterKind::Flying, 1),
                (CounterKind::Deathtouch, 1),
                (CounterKind::Lifelink, 1),
            ],
        ),
        (
            "Kheru Goldkeeper",
            vec![(CounterKind::PlusOne, 2), (CounterKind::Flying, 1)],
        ),
        ("Sage of the Fang", vec![(CounterKind::PlusOne, 8)]),
    ] {
        let p = Player::P0;
        let mut g = scenario(p);
        let recipient = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        if name == "Sage of the Fang" {
            g.add_counter(recipient, CounterKind::PlusOne, 3);
        }
        let source = put(&mut g, p, name, Zone::Graveyard);
        g.state.players[p.index()].mana = [0; 6];
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        g.state.players[p.index()].mana = match name {
            "Qarsi Revenant" => [0, 0, 1, 0, 0, 2],
            "Kheru Goldkeeper" => [0, 1, 1, 0, 1, 2],
            "Sage of the Fang" => [0, 0, 0, 0, 1, 3],
            _ => unreachable!(),
        };
        activate(&mut g, source, 0, &[recipient]);
        assert_eq!(g.object_view(source).zone, Zone::Exile);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        drain(&mut g);
        for (kind, count) in expected {
            assert_eq!(g.counter_count(recipient, kind), count, "{name}");
        }
        if name == "Qarsi Revenant" {
            assert!(
                g.keywords(recipient).contains(
                    Keywords::FLYING
                        .union(Keywords::DEATHTOUCH)
                        .union(Keywords::LIFELINK)
                )
            );
        }
    }
    let p = Player::P1;
    let mut g = scenario(p);
    let target = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    let sage = put(&mut g, p, "Sage of the Fang", Zone::Hand);
    cast(&mut g, sage, None);
    drain_to(&mut g, &[target]);
    assert_eq!(g.counter_count(target, CounterKind::PlusOne), 1);
}
#[test]
fn graveyard_departure_observers_use_own_turn_and_skimmer_once_per_turn() {
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Kheru Goldkeeper", Zone::Battlefield);
    put(&mut g, p, "Kishla Skimmer", Zone::Battlefield);
    let recipient = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let hand = g.hand_count(p);
    for turn in [p, p, p.other()] {
        g.state.active = turn;
        g.state.priority = p;
        let source = put(&mut g, p, "Qarsi Revenant", Zone::Graveyard);
        // Renew is a sorcery. An instant-speed Sentinel activation supplies the opponent-turn departure.
        if turn == p {
            activate(&mut g, source, 0, &[recipient]);
        } else {
            let sentinel = put(&mut g, p, "Jade-Cast Sentinel", Zone::Battlefield);
            activate(&mut g, sentinel, 0, &[source]);
        }
        drain(&mut g);
    }
    assert_eq!(g.hand_count(p), hand + 1);
    assert_eq!(battlefield(&g, "Treasure").len(), 2);
    g.state.active = p;
    g.state.priority = p;
    g.begin_turn();
    g.state.step = Step::Main1;
    g.state.pending = Pending::Priority;
    g.state.priority = p;
    let source = put(&mut g, p, "Qarsi Revenant", Zone::Graveyard);
    activate(&mut g, source, 0, &[recipient]);
    drain(&mut g);
    assert_eq!(g.hand_count(p), hand + 2);
    assert_eq!(battlefield(&g, "Treasure").len(), 3);
}
#[test]
fn rainveil_mill_optional_and_mana_uses_current_power() {
    for accept in [false, true] {
        let p = Player::P1;
        let mut g = scenario(p);
        let before = g.library_card_count(p);
        let source = put(&mut g, p, "Rainveil Rejuvenator", Zone::Hand);
        cast(&mut g, source, None);
        advance(&mut g, &[], |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        act(&mut g, Action::ChooseOptional { accept });
        drain(&mut g);
        assert_eq!(g.library_card_count(p), before - if accept { 3 } else { 0 });
        g.begin_turn();
        g.add_counter(source, CounterKind::PlusOne, 3);
        let before = g.state.players[p.index()].mana[4];
        choose(
            &mut g,
            |a| matches!(a,Action::ActivateMana { card, production, .. } if *card==source && *production==[0,0,0,0,5,0]),
        );
        drain(&mut g);
        assert_eq!(g.state.players[p.index()].mana[4], before + 5);
        assert!(g.object_view(source).tapped);
    }
}

#[test]
fn first_family_intrinsic_headers_and_printed_keywords() {
    for (name, power, toughness, keywords) in [
        ("Gurmag Nightwatch", 3, 3, Keywords::NONE),
        ("Gurmag Rakshasa", 5, 5, Keywords::MENACE),
        ("Host of the Hereafter", 2, 2, Keywords::NONE),
        ("Iridescent Tiger", 3, 4, Keywords::NONE),
        ("Kheru Goldkeeper", 3, 3, Keywords::FLYING),
        ("Kishla Skimmer", 2, 2, Keywords::FLYING),
        (
            "Qarsi Revenant",
            3,
            3,
            Keywords::FLYING
                .union(Keywords::DEATHTOUCH)
                .union(Keywords::LIFELINK),
        ),
        ("Rainveil Rejuvenator", 2, 4, Keywords::NONE),
        ("Sage of the Fang", 2, 2, Keywords::NONE),
    ] {
        let c = Card::from_name(name).unwrap();
        assert_eq!(c.effect(), Effect::Creature(power, toughness), "{name}");
        assert_eq!(c.keywords(), keywords, "{name}");
    }
}

// A supported shell is replaced in hand, before source registration, by the real
// authored definition. This allows outcome verification while admission is gated.
fn candidate(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    assert!(matches!(zone, Zone::Hand | Zone::Graveyard));
    let card = Card::from_name(name).unwrap();
    if card.is_supported() {
        return put(g, p, name, zone);
    }
    let id = put(g, p, "Grizzly Bears", Zone::Hand);
    g.state.objects[id as usize].card = card;
    if zone == Zone::Graveyard {
        g.state.objects[id as usize].zone = Zone::Graveyard;
        g.register_graveyard_sources(id);
    }
    id
}
fn cast_candidate(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = candidate(g, p, name, Zone::Hand);
    cast(g, id, None);
    drain(g);
    id
}
fn search_choose(g: &mut Game, id: CardId) {
    act(g, Action::ChooseSearch { card: id });
    if g.legal_actions().contains(&Action::FinishSearch) {
        act(g, Action::FinishSearch);
    }
}
fn reset_main(g: &mut Game, p: Player) {
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Main1;
    g.state.pending = Pending::Priority;
}
fn attack_one(g: &mut Game, p: Player, id: CardId) {
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    act(g, Action::Attack { card: id });
    act(g, Action::FinishAttackers);
}
fn combat_until(g: &mut Game, p: Player, done: impl Fn(&Game) -> bool) {
    for _ in 0..96 {
        if done(g) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass | Action::FinishBlockers | Action::PutTrigger { .. }
            ) || matches!(a,Action::AssignDamage {target:Target::Player(target),..} if *target==p.other())
        });
    }
    panic!("combat action budget");
}
#[test]
fn monuments_search_their_basic_subtypes_then_pay_sacrifice_and_create_exact_tokens() {
    for (name, found_name, token_type, count, mana, temporary) in [
        (
            "Jeskai Monument",
            "Island",
            "Bird",
            2,
            [1, 1, 0, 1, 0, 1],
            false,
        ),
        (
            "Mardu Monument",
            "Swamp",
            "Warrior",
            3,
            [1, 0, 1, 1, 0, 2],
            true,
        ),
    ] {
        let p = Player::P0;
        let mut g = scenario(p);
        let found = put(&mut g, p, found_name, Zone::Library);
        let excluded = put(&mut g, p, "Forest", Zone::Library);
        let source = candidate(&mut g, p, name, Zone::Hand);
        cast(&mut g, source, None);
        advance(&mut g, &[], |g| {
            matches!(g.state.pending, Pending::EffectSearch { .. })
        });
        reject(&mut g, Action::ChooseSearch { card: excluded });
        search_choose(&mut g, found);
        drain(&mut g);
        assert_eq!(g.object_view(found).zone, Zone::Hand);
        g.state.players[p.index()].mana = mana;
        activate(&mut g, source, 0, &[]);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        drain(&mut g);
        let tokens = battlefield(&g, token_type);
        assert_eq!(tokens.len(), count, "{name}");
        for id in &tokens {
            assert_eq!(stats(&g, *id), (1, 1));
            assert_eq!(g.object_view(*id).colors, if temporary { 8 } else { 1 });
            assert!(g.keywords(*id).contains(if temporary {
                Keywords::HASTE.union(Keywords::MENACE)
            } else {
                Keywords::FLYING
            }));
        }
        if temporary {
            g.state.step = Step::Cleanup;
            g.finish_cleanup();
            for id in tokens {
                assert!(
                    !g.keywords(id)
                        .intersects(Keywords::HASTE.union(Keywords::MENACE))
                );
            }
        }
    }
}
#[test]
fn harmonize_mammoth_and_routine_use_printed_alternates_and_exile() {
    for (name, mana) in [
        ("Mammoth Bellow", [0, 1, 0, 1, 1, 1]),
        ("Roamer's Routine", [0, 0, 0, 0, 1, 0]),
    ] {
        let p = Player::P1;
        let mut g = scenario(p);
        let helper = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        g.add_counter(helper, CounterKind::PlusOne, 1); // four power reduces generic mana only
        let land = put(&mut g, p, "Island", Zone::Library);
        let source = candidate(&mut g, p, name, Zone::Graveyard);
        g.state.players[p.index()].mana = mana;
        act(&mut g, Action::AnnounceHarmonize { card: source });
        let object = g.object_ref(helper);
        act(
            &mut g,
            Action::ChooseHarmonizer {
                object: Some(object),
            },
        );
        pay(&mut g);
        if matches!(g.state.pending, Pending::CastCosts { .. }) {
            let slot = g
                .state
                .announcing
                .as_ref()
                .unwrap()
                .cast_costs
                .as_ref()
                .unwrap()
                .clauses
                .iter()
                .position(|r| !r.paid)
                .unwrap();
            act(
                &mut g,
                Action::ChooseCastCost {
                    clause: slot as u16,
                },
            );
        }
        assert!(g.object_view(helper).tapped);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        if name == "Roamer's Routine" {
            advance(&mut g, &[], |g| {
                matches!(g.state.pending, Pending::EffectSearch { .. })
            });
            search_choose(&mut g, land);
        }
        drain(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Exile);
        if name == "Mammoth Bellow" {
            let tokens = battlefield(&g, "Elephant");
            assert_eq!(tokens.len(), 1);
            assert_eq!(stats(&g, tokens[0]), (5, 5));
            assert_eq!(g.object_view(tokens[0]).colors, 16);
        } else {
            assert_eq!(g.object_view(land).zone, Zone::Battlefield);
            assert!(g.object_view(land).tapped);
        }
    }
    for name in ["Mammoth Bellow", "Roamer's Routine"] {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = candidate(&mut g, p, name, Zone::Hand);
        cast(&mut g, source, None);
        if name == "Roamer's Routine" {
            advance(&mut g, &[], |g| {
                matches!(g.state.pending, Pending::EffectSearch { .. })
            });
            act(&mut g, Action::FinishSearch);
        }
        drain(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    }
}
#[test]
fn villages_entry_conditions_mana_and_activated_effects() {
    for (name, qualifier, color) in [
        ("Kishla Village", "Island", 4),
        ("Mistrise Village", "Forest", 1),
    ] {
        for have in [false, true] {
            let p = Player::P0;
            let mut g = scenario(p);
            if have {
                put(&mut g, p, qualifier, Zone::Battlefield);
            }
            let source = candidate(&mut g, p, name, Zone::Hand);
            act(&mut g, Action::PlayLand { card: source });
            drain(&mut g);
            assert_eq!(g.object_view(source).tapped, !have);
            if have {
                let before = g.state.players[p.index()].mana[color];
                choose(
                    &mut g,
                    |a| matches!(a,Action::ActivateMana {card,..}if *card==source),
                );
                assert_eq!(g.state.players[p.index()].mana[color], before + 1);
            }
        }
        let p = Player::P1;
        let mut g = scenario(p);
        put(&mut g, p, qualifier, Zone::Battlefield);
        let source = candidate(&mut g, p, name, Zone::Hand);
        act(&mut g, Action::PlayLand { card: source });
        drain(&mut g);
        g.state.players[p.index()].mana = if color == 4 {
            [0, 0, 0, 0, 1, 3]
        } else {
            [0, 1, 0, 0, 0, 0]
        };
        let top = put(&mut g, p, "Island", Zone::Library);
        let second = put(&mut g, p, "Swamp", Zone::Library);
        activate(&mut g, source, 1, &[]);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        if color == 4 {
            advance(&mut g, &[], |g| {
                g.legal_actions()
                    .iter()
                    .any(|a| matches!(a, Action::ArrangeLibrary { .. }))
            });
            act(
                &mut g,
                Action::ArrangeLibrary {
                    card: second,
                    destination: LibraryDestination::Graveyard,
                },
            );
            act(
                &mut g,
                Action::ArrangeLibrary {
                    card: top,
                    destination: LibraryDestination::Top,
                },
            );
            drain(&mut g);
            assert_eq!(g.object_view(second).zone, Zone::Graveyard);
            assert_eq!(g.top_library_card(p), Some(top));
        } else {
            drain(&mut g);
            g.state.players[p.index()].mana = [8; 6];
            let spell = put(&mut g, p, "Grizzly Bears", Zone::Hand);
            cast(&mut g, spell, None);
            assert!(
                g.observe(p)
                    .uncounterable_spell_ids
                    .contains(&g.state.stack.last().unwrap().id)
            );
            drain(&mut g);
            let later = put(&mut g, p, "Grizzly Bears", Zone::Hand);
            cast(&mut g, later, None);
            assert!(
                !g.observe(p)
                    .uncounterable_spell_ids
                    .contains(&g.state.stack.last().unwrap().id)
            );
            drain(&mut g);
        }
    }
}
#[test]
fn perennation_returns_permanent_with_both_protection_counters() {
    for name in ["Grizzly Bears", "Forest", "Short Bow"] {
        let p = Player::P0;
        let mut g = scenario(p);
        let target = put(&mut g, p, name, Zone::Graveyard);
        let source = candidate(&mut g, p, "Perennation", Zone::Hand);
        cast(&mut g, source, Some(Target::Card(target)));
        drain(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Battlefield);
        assert_eq!(g.counter_count(target, CounterKind::Hexproof), 1);
        assert_eq!(g.counter_count(target, CounterKind::Indestructible), 1);
        if name == "Grizzly Bears" {
            assert!(
                g.keywords(target)
                    .contains(Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE))
            );
        }
    }
}
#[test]
fn lie_in_wait_uses_graveyard_creatures_power_after_return_and_independent_targets() {
    for invalidate_return in [false, true] {
        let p = Player::P1;
        let mut g = scenario(p);
        let returned = put(&mut g, p, "Hill Giant", Zone::Graveyard);
        let target = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
        let source = candidate(&mut g, p, "Lie in Wait", Zone::Hand);
        announce(
            &mut g,
            source,
            &[Target::Card(returned), Target::Card(target)],
        );
        if invalidate_return {
            g.move_object(returned, Zone::Exile);
        }
        drain(&mut g);
        assert_eq!(
            g.object_view(returned).zone,
            if invalidate_return {
                Zone::Exile
            } else {
                Zone::Hand
            }
        );
        assert_eq!(
            g.object_view(target).damage,
            if invalidate_return { 0 } else { 3 }
        );
    }
}
#[test]
fn monastery_messenger_optional_recovery_excludes_creatures_and_lands() {
    let p = Player::P0;
    let mut g = scenario(p);
    let instant = put(&mut g, p, "Rebellious Strike", Zone::Graveyard);
    let creature = put(&mut g, p, "Grizzly Bears", Zone::Graveyard);
    let land = put(&mut g, p, "Forest", Zone::Graveyard);
    let source = candidate(&mut g, p, "Monastery Messenger", Zone::Hand);
    cast(&mut g, source, None);
    advance(&mut g, &[], |g| {
        matches!(g.state.pending, Pending::Targets { .. })
    });
    for target in [creature, land] {
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
    }
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(instant),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(&mut g, Action::FinishTargets);
    }
    drain(&mut g);
    assert_eq!(g.top_library_card(p), Some(instant));
    assert!(
        g.keywords(source)
            .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
    );
}
#[test]
fn roiling_dragonstorm_loots_and_returns_only_for_own_dragon_entry() {
    let p = Player::P1;
    let mut g = scenario(p);
    let discard = put(&mut g, p, "Forest", Zone::Hand);
    let hand = g.hand_count(p);
    let source = candidate(&mut g, p, "Roiling Dragonstorm", Zone::Hand);
    cast(&mut g, source, None);
    advance(&mut g, &[], |g| {
        matches!(g.state.pending, Pending::EffectDiscard { .. })
    });
    act(&mut g, Action::Discard { card: discard });
    drain(&mut g);
    assert_eq!(g.hand_count(p), hand + 1);
    assert_eq!(g.object_view(discard).zone, Zone::Graveyard);
    let dragon = put(&mut g, p.other(), "Shivan Dragon", Zone::Hand);
    g.move_object(dragon, Zone::Battlefield);
    drain(&mut g);
    assert_eq!(g.object_view(source).zone, Zone::Battlefield);
    let own = put(&mut g, p, "Shivan Dragon", Zone::Hand);
    cast(&mut g, own, None);
    drain(&mut g);
    assert_eq!(g.object_view(source).zone, Zone::Hand);
}
#[test]
fn hundred_battle_veteran_counts_distinct_kinds_and_graveyard_cast_enters_finality() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = candidate(&mut g, p, "Hundred-Battle Veteran", Zone::Graveyard);
    g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 3];
    cast(&mut g, source, None);
    drain(&mut g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert_eq!(g.counter_count(source, CounterKind::Finality), 1);
    assert_eq!(stats(&g, source), (4, 2));
    let a = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let b = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    g.add_counter(a, CounterKind::PlusOne, 1);
    g.add_counter(b, CounterKind::PlusOne, 1);
    assert_eq!(stats(&g, source), (4, 2));
    g.add_counter(b, CounterKind::Soul, 1);
    assert_eq!(stats(&g, source), (6, 6));
    g.pay_counters(b, &Counters::of(CounterKind::Soul, 1));
    assert_eq!(stats(&g, source), (4, 2));
    g.state.players[p.index()].mana = [8; 6];
    // A nontargeted fixture move exercises the shared finality replacement after a real graveyard cast.
    g.move_object(source, Zone::Graveyard);
    assert_eq!(g.object_view(source).zone, Zone::Exile);
}
#[test]
fn karakyk_hexproof_tracks_all_damage_in_exact_incarnation_across_turns() {
    let p = Player::P1;
    let mut g = scenario(p);
    let guardian = cast_candidate(&mut g, p, "Karakyk Guardian");
    assert_eq!(stats(&g, guardian), (6, 5));
    assert!(
        g.keywords(guardian).contains(
            Keywords::FLYING
                .union(Keywords::VIGILANCE)
                .union(Keywords::TRAMPLE)
                .union(Keywords::HEXPROOF)
        )
    );
    let enemy = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
    let spell = put(&mut g, p, "Knockout Maneuver", Zone::Hand);
    announce(
        &mut g,
        spell,
        &[Target::Card(guardian), Target::Card(enemy)],
    );
    drain(&mut g);
    assert!(!g.keywords(guardian).contains(Keywords::HEXPROOF));
    g.begin_turn();
    assert!(!g.keywords(guardian).contains(Keywords::HEXPROOF));
    g.move_object(guardian, Zone::Hand);
    g.move_object(guardian, Zone::Battlefield);
    drain(&mut g);
    assert!(g.keywords(guardian).contains(Keywords::HEXPROOF));
}
#[test]
fn krotiq_paid_permission_expires_and_does_not_remove_defender() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = cast_candidate(&mut g, p, "Krotiq Nestguard");
    g.begin_turn();
    reset_main(&mut g, p);
    assert!(!g.can_declare_attacker(p, source));
    g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 2];
    activate(&mut g, source, 0, &[]);
    drain(&mut g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert!(g.can_declare_attacker(p, source));
    assert!(g.keywords(source).contains(Keywords::DEFENDER));
    g.state.step = Step::Cleanup;
    g.finish_cleanup();
    assert!(!g.can_declare_attacker(p, source));
}
#[test]
fn krumar_chosen_x_pays_life_mana_and_tap_then_chooses_counters_or_sized_spirit() {
    for counters in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = cast_candidate(&mut g, p, "Krumar Initiate");
        g.begin_turn();
        reset_main(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 3];
        g.state.players[p.index()].life = 10;
        act(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        reject(&mut g, Action::ChooseX { value: 4 });
        act(&mut g, Action::ChooseX { value: 3 });
        pay(&mut g);
        if matches!(g.state.pending, Pending::CastCosts { .. }) {
            let slot = g
                .state
                .announcing
                .as_ref()
                .unwrap()
                .cast_costs
                .as_ref()
                .unwrap()
                .clauses
                .iter()
                .position(|r| !r.paid)
                .unwrap();
            act(
                &mut g,
                Action::ChooseCastCost {
                    clause: slot as u16,
                },
            );
        }
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.state.players[p.index()].life, 7);
        assert!(g.object_view(source).tapped);
        advance(&mut g, &[], |g| {
            matches!(g.state.pending, Pending::Endure { .. })
        });
        act(
            &mut g,
            Action::ChooseEndure {
                put_counters: counters,
            },
        );
        drain(&mut g);
        if counters {
            assert_eq!(stats(&g, source), (5, 5));
            assert_eq!(g.counter_count(source, CounterKind::PlusOne), 3);
        } else {
            let spirits = battlefield(&g, "Spirit");
            assert_eq!(spirits.len(), 1);
            assert_eq!(stats(&g, spirits[0]), (3, 3));
            assert_eq!(g.object_view(spirits[0]).colors, 1);
        }
    }
}
#[test]
fn lasyd_optional_mill_counts_lands_and_renew_recounts_current_graveyard_lands() {
    for accept in [false, true] {
        let p = Player::P1;
        let mut g = scenario(p);
        put(&mut g, p, "Forest", Zone::Battlefield);
        put(&mut g, p, "Island", Zone::Battlefield);
        let before = g.library_card_count(p);
        let source = candidate(&mut g, p, "Lasyd Prowler", Zone::Hand);
        cast(&mut g, source, None);
        advance(&mut g, &[], |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        act(&mut g, Action::ChooseOptional { accept });
        drain(&mut g);
        assert_eq!(g.library_card_count(p), before - if accept { 2 } else { 0 });
        g.move_object(source, Zone::Graveyard);
        let target = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let extra = put(&mut g, p, "Swamp", Zone::Graveyard);
        let count = g
            .state
            .objects
            .iter()
            .filter(|o| o.owner == p && o.zone == Zone::Graveyard && o.card.is_land())
            .count() as u64;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
        activate(&mut g, source, 0, &[target]);
        g.move_object(extra, Zone::Hand);
        drain(&mut g);
        assert_eq!(g.counter_count(target, CounterKind::PlusOne), count - 1);
    }
}
#[test]
fn herd_heirloom_creature_restricted_mana_and_temporary_trample_draw_ability() {
    let p = Player::P0;
    let mut g = scenario(p);
    let heirloom = cast_candidate(&mut g, p, "Herd Heirloom");
    g.state.players[p.index()].mana = [0; 6];
    choose(
        &mut g,
        |a| matches!(a,Action::ActivateMana {card,production,..}if *card==heirloom&&*production==[0,1,0,0,0,0]),
    );
    let instant = put(&mut g, p, "Humbling Elder", Zone::Hand); // actual creature flash spell can spend this blue mana
    cast(&mut g, instant, None);
    drain(&mut g);
    assert_eq!(g.object_view(instant).zone, Zone::Battlefield);
    g.state.objects[heirloom as usize].tapped = false;
    choose(
        &mut g,
        |a| matches!(a,Action::ActivateMana {card,production,..}if *card==heirloom&&*production==[0,1,0,0,0,0]),
    );
    let noncreature = put(&mut g, p, "Opt", Zone::Hand);
    reject(&mut g, Action::AnnounceCast { card: noncreature });
    g.state.objects[heirloom as usize].tapped = false;
    let large = put(&mut g, p, "Shivan Dragon", Zone::Battlefield);
    activate(&mut g, heirloom, 1, &[large]);
    drain(&mut g);
    assert!(g.keywords(large).contains(Keywords::TRAMPLE));
    g.state.objects[large as usize].growth.power = -4;
    let hand = g.hand_count(p);
    attack_one(&mut g, p, large);
    combat_until(&mut g, p, |g| g.hand_count(p) == hand + 1);
    assert_eq!(g.state.players[p.other().index()].life, 19);
    g.state.step = Step::Cleanup;
    g.finish_cleanup();
    assert!(!g.keywords(large).contains(Keywords::TRAMPLE));
}

#[test]
fn second_family_intrinsic_creature_headers() {
    for (name, p, t, k) in [
        ("Hundred-Battle Veteran", 4, 2, Keywords::NONE),
        (
            "Karakyk Guardian",
            6,
            5,
            Keywords::FLYING
                .union(Keywords::VIGILANCE)
                .union(Keywords::TRAMPLE),
        ),
        ("Krotiq Nestguard", 4, 4, Keywords::DEFENDER),
        ("Krumar Initiate", 2, 2, Keywords::NONE),
        ("Lasyd Prowler", 5, 5, Keywords::NONE),
        (
            "Monastery Messenger",
            2,
            3,
            Keywords::FLYING.union(Keywords::VIGILANCE),
        ),
    ] {
        let c = Card::from_name(name).unwrap();
        assert_eq!(c.effect(), Effect::Creature(p, t), "{name}");
        assert_eq!(c.keywords(), k, "{name}");
    }
}
#[test]
fn monastery_messenger_may_decline_and_pays_colored_twobrid_route() {
    let p = Player::P1;
    let mut g = scenario(p);
    let grave = put(&mut g, p, "Rebellious Strike", Zone::Graveyard);
    let source = candidate(&mut g, p, "Monastery Messenger", Zone::Hand);
    g.state.players[p.index()].mana = [1, 1, 0, 1, 0, 0];
    cast(&mut g, source, None);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    drain(&mut g);
    assert_eq!(g.object_view(grave).zone, Zone::Graveyard);
    assert_eq!(stats(&g, source), (2, 3));
}
#[test]
fn villages_accept_either_owned_qualifier_and_ignore_opposing_qualifiers() {
    for (name, qualifiers) in [
        ("Kishla Village", ["Island", "Swamp"]),
        ("Mistrise Village", ["Mountain", "Forest"]),
    ] {
        for qualifier in qualifiers {
            let p = Player::P0;
            let mut g = scenario(p);
            put(&mut g, p, qualifier, Zone::Battlefield);
            let source = candidate(&mut g, p, name, Zone::Hand);
            act(&mut g, Action::PlayLand { card: source });
            drain(&mut g);
            assert!(!g.object_view(source).tapped, "{name}/{qualifier}");
        }
        let p = Player::P1;
        let mut g = scenario(p);
        for qualifier in qualifiers {
            put(&mut g, p.other(), qualifier, Zone::Battlefield);
        }
        let source = candidate(&mut g, p, name, Zone::Hand);
        act(&mut g, Action::PlayLand { card: source });
        drain(&mut g);
        assert!(g.object_view(source).tapped, "{name}: opposing qualifier");
    }
}
#[test]
fn monuments_each_basic_subtype_and_activation_timing_are_wired() {
    for (name, basics) in [
        ("Jeskai Monument", ["Island", "Mountain", "Plains"]),
        ("Mardu Monument", ["Mountain", "Plains", "Swamp"]),
    ] {
        for basic in basics {
            let p = Player::P0;
            let mut g = scenario(p);
            let found = put(&mut g, p, basic, Zone::Library);
            let source = candidate(&mut g, p, name, Zone::Hand);
            cast(&mut g, source, None);
            advance(&mut g, &[], |g| {
                matches!(g.state.pending, Pending::EffectSearch { .. })
            });
            search_choose(&mut g, found);
            drain(&mut g);
            assert_eq!(g.object_view(found).zone, Zone::Hand, "{name}/{basic}");
            let ability = &Card::from_name(name).unwrap().activations()[0];
            assert!(ability.sorcery && ability.cost.tap && ability.cost.sacrifice);
            g.state.step = Step::Upkeep;
            reject(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            g.state.step = Step::Main1;
            g.state.objects[source as usize].tapped = true;
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
}

fn finish_ordered_costs(g: &mut Game) {
    for _ in 0..12 {
        if g.state.announcing.is_none() {
            return;
        }
        match g.state.pending {
            Pending::PayCosts { .. } => pay(g),
            Pending::CastCosts { .. } => {
                let slot = g
                    .state
                    .announcing
                    .as_ref()
                    .unwrap()
                    .cast_costs
                    .as_ref()
                    .unwrap()
                    .clauses
                    .iter()
                    .position(|r| !r.paid)
                    .unwrap();
                act(
                    g,
                    Action::ChooseCastCost {
                        clause: slot as u16,
                    },
                );
            }
            _ => panic!("unexpected ordered cost {:?}", g.state.pending),
        }
    }
    panic!("ordered cost budget");
}
fn begin_step(g: &mut Game, step: Step) {
    g.begin_step(step);
    g.give_priority();
}
fn choose_behold(g: &mut Game, source: CardId, targets: &[CardId], witness: Option<CardId>) {
    act(g, Action::AnnounceCast { card: source });
    act(
        g,
        Action::ChooseCastBehold {
            pay: witness.is_some(),
        },
    );
    for &id in targets {
        act(
            g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
    if let Some(id) = witness {
        let slot = g
            .state
            .announcing
            .as_ref()
            .unwrap()
            .cast_costs
            .as_ref()
            .unwrap()
            .clauses
            .iter()
            .position(|r| !r.paid)
            .unwrap();
        act(
            g,
            Action::ChooseCastCost {
                clause: slot as u16,
            },
        );
        let object = g.object_ref(id);
        act(g, Action::ChooseBeholdCost { object });
    }
}
#[test]
fn hollowmurk_sultai_draws_once_per_turn_and_abzan_pumps_an_attacker_only() {
    for mode in [0, 1] {
        let p = Player::P0;
        let mut g = scenario(p);
        let bear = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let siege = candidate(&mut g, p, "Hollowmurk Siege", Zone::Hand);
        cast(&mut g, siege, None);
        advance(&mut g, &[], |g| {
            matches!(g.state.pending, Pending::EntryValue { .. })
        });
        act(
            &mut g,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::NamedMode(mode),
            },
        );
        drain(&mut g);
        let hand = g.hand_count(p);
        for _ in 0..2 {
            let counter = put(&mut g, p, "Lightfoot Technique", Zone::Hand);
            cast(&mut g, counter, Some(Target::Card(bear)));
            drain(&mut g);
        }
        assert_eq!(g.hand_count(p), hand + usize::from(mode == 0));
        if mode == 0 {
            g.begin_turn();
            reset_main(&mut g, p);
            g.state.players[p.index()].mana = [8; 6];
            let counter = put(&mut g, p, "Lightfoot Technique", Zone::Hand);
            cast(&mut g, counter, Some(Target::Card(bear)));
            drain(&mut g);
            assert_eq!(g.hand_count(p), hand + 2);
        } else {
            attack_one(&mut g, p, bear);
            drain_to(&mut g, &[bear]);
            assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 3);
            assert!(g.keywords(bear).contains(Keywords::MENACE));
        }
    }
}
#[test]
fn kishla_trawlers_optional_creature_exile_precedes_reflexive_spell_target() {
    for pay_cost in [false, true] {
        let p = Player::P1;
        let mut g = scenario(p);
        let creature = put(&mut g, p, "Grizzly Bears", Zone::Graveyard);
        let instant = put(&mut g, p, "Rebellious Strike", Zone::Graveyard);
        let source = candidate(&mut g, p, "Kishla Trawlers", Zone::Hand);
        cast(&mut g, source, None);
        advance(&mut g, &[], |g| {
            matches!(g.state.pending, Pending::EffectChoice { .. })
        });
        assert!(!g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(instant)
        }));
        if pay_cost {
            let object = g.object_ref(creature);
            act(&mut g, Action::Sacrifice { object });
            drain_to(&mut g, &[instant]);
        } else {
            act(&mut g, Action::DeclineEffectPayment);
            drain(&mut g);
        }
        assert_eq!(
            g.object_view(creature).zone,
            if pay_cost {
                Zone::Exile
            } else {
                Zone::Graveyard
            }
        );
        assert_eq!(
            g.object_view(instant).zone,
            if pay_cost {
                Zone::Hand
            } else {
                Zone::Graveyard
            }
        );
    }
}
#[test]
fn kotis_actual_combat_exiles_damage_count_and_offers_each_spell_at_its_own_limit() {
    let p = Player::P0;
    let mut g = scenario(p);
    let kotis = cast_candidate(&mut g, p, "Kotis, the Fangkeeper");
    g.begin_turn();
    reset_main(&mut g, p);
    let bear = put(&mut g, p.other(), "Grizzly Bears", Zone::Library);
    let bow = put(&mut g, p.other(), "Short Bow", Zone::Library);
    g.state.players[p.index()].mana = [0; 6];
    attack_one(&mut g, p, kotis);
    combat_until(&mut g, p, |g| {
        matches!(g.state.pending, Pending::ResolutionCasts { .. })
    });
    for id in [bow, bear] {
        let object = g.object_ref(id);
        act(
            &mut g,
            Action::CastExiledDuringResolution {
                object,
                face: CastFace::Normal,
            },
        );
        pay(&mut g);
    }
    act(&mut g, Action::FinishResolutionCasts);
    drain(&mut g);
    for id in [bow, bear] {
        assert_eq!(g.object_view(id).zone, Zone::Battlefield);
        assert_eq!(g.object_view(id).controller, p);
        assert_eq!(g.object_view(id).owner, p.other());
    }
    assert_eq!(g.state.players[p.other().index()].life, 18);
    assert!(g.keywords(kotis).contains(Keywords::INDESTRUCTIBLE));
}
#[test]
fn lotuslight_search_assigns_distinct_black_green_blue_roles() {
    let p = Player::P1;
    let mut g = scenario(p);
    let black = put(&mut g, p, "Kin-Tree Nurturer", Zone::Library);
    let green = put(&mut g, p, "Llanowar Elves", Zone::Library);
    let blue = put(&mut g, p, "Humbling Elder", Zone::Library);
    let duplicate = put(&mut g, p, "Grizzly Bears", Zone::Library);
    let red = put(&mut g, p, "Shock", Zone::Library);
    let source = candidate(&mut g, p, "Lotuslight Dancers", Zone::Hand);
    cast(&mut g, source, None);
    advance(&mut g, &[], |g| {
        matches!(g.state.pending, Pending::EffectSearch { .. })
    });
    reject(&mut g, Action::ChooseSearch { card: red });
    act(&mut g, Action::ChooseSearch { card: green });
    reject(&mut g, Action::ChooseSearch { card: duplicate });
    act(&mut g, Action::ChooseSearch { card: black });
    act(&mut g, Action::ChooseSearch { card: blue });
    act(&mut g, Action::FinishSearch);
    drain(&mut g);
    for id in [black, green, blue] {
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
    }
    assert!(g.keywords(source).contains(Keywords::LIFELINK));
}
#[test]
fn maelstrom_colorless_restricted_dragon_omen_mana_and_dragon_search() {
    let p = Player::P0;
    let mut g = scenario(p);
    let land = candidate(&mut g, p, "Maelstrom of the Spirit Dragon", Zone::Hand);
    act(&mut g, Action::PlayLand { card: land });
    g.state.players[p.index()].mana = [0; 6];
    choose(
        &mut g,
        |a| matches!(a,Action::ActivateMana {card,ability:0,..}if*card==land),
    );
    assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 0, 1]);
    g.state.objects[land as usize].tapped = false;
    choose(
        &mut g,
        |a| matches!(a,Action::ActivateMana {card,ability:1,production,..}if *card==land&&*production==[1,0,0,0,0,0]),
    );
    let ordinary = put(&mut g, p, "Rebellious Strike", Zone::Hand);
    reject(&mut g, Action::AnnounceCast { card: ordinary });
    let omen = put(&mut g, p, "Riling Dawnbreaker", Zone::Hand);
    act(&mut g, Action::AnnounceOmen { card: omen });
    pay(&mut g);
    drain(&mut g);
    assert_eq!(battlefield(&g, "Soldier").len(), 1);
    g.state.objects[land as usize].tapped = false;
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
    let dragon = put(&mut g, p, "Shivan Dragon", Zone::Library);
    activate(&mut g, land, 2, &[]);
    assert_eq!(g.object_view(land).zone, Zone::Graveyard);
    advance(&mut g, &[], |g| {
        matches!(g.state.pending, Pending::EffectSearch { .. })
    });
    search_choose(&mut g, dragon);
    drain(&mut g);
    assert_eq!(g.object_view(dragon).zone, Zone::Hand);
}
#[test]
fn magmatic_hellkite_search_belongs_to_destroyed_lands_controller_and_enters_stunned() {
    let p = Player::P0;
    let mut g = scenario(p);
    let land = put(&mut g, p.other(), "Mystic Monastery", Zone::Battlefield);
    let basic = put(&mut g, p.other(), "Island", Zone::Library);
    let source = candidate(&mut g, p, "Magmatic Hellkite", Zone::Hand);
    cast(&mut g, source, None);
    advance(&mut g, &[Target::Card(land)], |g| {
        matches!(g.state.pending, Pending::EffectSearch { .. })
    });
    assert_eq!(g.actor(), Some(p.other()));
    search_choose(&mut g, basic);
    drain(&mut g);
    assert_eq!(g.object_view(land).zone, Zone::Graveyard);
    assert_eq!(g.object_view(basic).controller, p.other());
    assert!(g.object_view(basic).tapped);
    assert_eq!(g.counter_count(basic, CounterKind::Stun), 1);
    assert!(g.keywords(source).contains(Keywords::FLYING));
}
#[test]
fn mardu_linked_copy_uses_printed_blueprint_and_waits_for_controllers_end_step() {
    let p = Player::P1;
    let mut g = scenario(p);
    let blueprint = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    g.add_counter(blueprint, CounterKind::PlusOne, 3);
    let source = candidate(&mut g, p, "Mardu Siegebreaker", Zone::Hand);
    cast(&mut g, source, None);
    drain_to(&mut g, &[blueprint]);
    assert_eq!(g.object_view(blueprint).zone, Zone::Exile);
    attack_one(&mut g, p, source);
    drain(&mut g);
    let copies = g
        .state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| o.is_token() && o.zone == Zone::Battlefield)
        .map(|(id, _)| id as CardId)
        .collect::<Vec<_>>();
    assert_eq!(copies.len(), 1);
    let copy = copies[0];
    assert_eq!(stats(&g, copy), (2, 2));
    assert!(g.object_view(copy).tapped);
    assert!(g.state.attackers.iter().any(|a| a.card == copy));
    g.state.active = p.other();
    begin_step(&mut g, Step::End);
    drain(&mut g);
    assert_eq!(g.object_view(copy).zone, Zone::Battlefield);
    g.state.active = p;
    begin_step(&mut g, Step::End);
    drain(&mut g);
    assert_ne!(g.object_view(copy).zone, Zone::Battlefield);
    g.move_object(source, Zone::Graveyard);
    drain(&mut g);
    assert_eq!(g.object_view(blueprint).zone, Zone::Battlefield);
    assert_eq!(g.counter_count(blueprint, CounterKind::PlusOne), 0);
}
#[test]
fn molten_exhale_requires_behold_for_flash_and_deals_four() {
    for flash in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let target = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
        let source = candidate(&mut g, p, "Molten Exhale", Zone::Hand);
        let witness = if flash {
            g.state.active = p.other();
            reject(&mut g, Action::AnnounceCast { card: source });
            Some(put(&mut g, p, "Shivan Dragon", Zone::Hand))
        } else {
            None
        };
        choose_behold(&mut g, source, &[target], witness);
        if let Some(id) = witness {
            g.move_object(id, Zone::Graveyard);
        }
        drain(&mut g);
        assert_eq!(g.object_view(target).damage, 4);
    }
}
#[test]
fn osseous_and_piercing_use_committed_behold_and_their_printed_target_effects() {
    for beheld in [false, true] {
        let p = Player::P1;
        let mut g = scenario(p);
        let attacker = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
        attack_one(&mut g, p.other(), attacker);
        drain(&mut g);
        g.state.priority = p;
        let witness = if beheld {
            Some(put(&mut g, p, "Shivan Dragon", Zone::Hand))
        } else {
            None
        };
        let source = candidate(&mut g, p, "Osseous Exhale", Zone::Hand);
        choose_behold(&mut g, source, &[attacker], witness);
        if let Some(id) = witness {
            g.move_object(id, Zone::Graveyard);
        }
        drain(&mut g);
        assert_eq!(g.object_view(attacker).zone, Zone::Graveyard);
        assert_eq!(
            g.state.players[p.index()].life,
            20 + if beheld { 2 } else { 0 }
        );
        let mut g = scenario(p);
        let shooter = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let target = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
        let witness = if beheld {
            Some(put(&mut g, p, "Shivan Dragon", Zone::Hand))
        } else {
            None
        };
        let source = candidate(&mut g, p, "Piercing Exhale", Zone::Hand);
        choose_behold(&mut g, source, &[shooter, target], witness);
        if beheld {
            advance(&mut g, &[], |g| {
                g.legal_actions()
                    .iter()
                    .any(|a| matches!(a, Action::ArrangeLibrary { .. }))
            });
            for _ in 0..2 {
                choose(&mut g, |a| {
                    matches!(
                        a,
                        Action::ArrangeLibrary {
                            destination: LibraryDestination::Graveyard,
                            ..
                        }
                    )
                });
            }
        }
        drain(&mut g);
        assert_eq!(g.object_view(target).damage, 2);
        assert_eq!(g.library_card_count(p), if beheld { 6 } else { 8 });
    }
}
#[test]
fn naga_enters_as_opposing_creature_and_renew_copies_others_without_counters() {
    let p = Player::P0;
    let mut g = scenario(p);
    let blueprint = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
    let source = candidate(&mut g, p, "Naga Fleshcrafter", Zone::Hand);
    cast(&mut g, source, None);
    advance(&mut g, &[], |g| {
        matches!(g.state.pending, Pending::EntryCopy { .. })
    });
    let object = g.object_ref(blueprint);
    act(
        &mut g,
        Action::ChooseEntryCopy {
            object: Some(object),
        },
    );
    drain(&mut g);
    assert_eq!(stats(&g, source), (5, 5));
    assert!(g.keywords(source).contains(Keywords::FLYING));
    g.move_object(source, Zone::Graveyard);
    let target = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let other = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
    g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 2];
    activate(&mut g, source, 0, &[target]);
    drain(&mut g);
    assert_eq!(g.counter_count(target, CounterKind::PlusOne), 1);
    assert_eq!(stats(&g, target), (3, 3));
    assert_eq!(stats(&g, other), (2, 2));
    assert_eq!(g.counter_count(other, CounterKind::PlusOne), 0);
    g.state.step = Step::Cleanup;
    g.finish_cleanup();
    assert_eq!(stats(&g, other), (1, 1));
}
#[test]
fn narset_own_end_step_optional_hand_payment_draws_current_spell_count_even_when_empty() {
    for accept in [false, true] {
        for empty in [false, true] {
            let p = Player::P0;
            let mut g = scenario(p);
            cast_candidate(&mut g, p, "Narset, Jeskai Waymaster");
            let second = put(&mut g, p, "Short Bow", Zone::Hand);
            cast(&mut g, second, None);
            drain(&mut g);
            let discarded = if empty {
                None
            } else {
                Some(put(&mut g, p, "Forest", Zone::Hand))
            };
            let before = g.hand_count(p);
            begin_step(&mut g, Step::End);
            advance(&mut g, &[], |g| {
                matches!(g.state.pending, Pending::EffectOptional { .. })
            });
            act(&mut g, Action::ChooseOptional { accept });
            if accept {
                if let Some(card) = discarded {
                    act(&mut g, Action::Discard { card });
                }
            }
            drain(&mut g);
            assert_eq!(g.hand_count(p), if accept { 2 } else { before });
            if let Some(id) = discarded {
                assert_eq!(
                    g.object_view(id).zone,
                    if accept { Zone::Graveyard } else { Zone::Hand }
                );
            }
        }
    }
}
#[test]
fn natures_rhythm_harmonize_preserves_chosen_x_as_mana_value_bound() {
    for harmonize in [false, true] {
        let p = Player::P1;
        let mut g = scenario(p);
        let helper = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let eligible = put(&mut g, p, "Grizzly Bears", Zone::Library);
        let too_big = put(&mut g, p, "Shivan Dragon", Zone::Library);
        let source = candidate(
            &mut g,
            p,
            "Nature's Rhythm",
            if harmonize {
                Zone::Graveyard
            } else {
                Zone::Hand
            },
        );
        if !harmonize {
            // X can be funded by generic mana; both printed green symbols
            // remain mandatory even when four total mana are available.
            for greens in [0, 1] {
                g.state.players[p.index()].mana = [0, 0, 0, 0, greens, 4 - greens];
                reject(&mut g, Action::AnnounceCast { card: source });
            }
        }
        g.state.players[p.index()].mana = if harmonize {
            [0, 0, 0, 0, 4, 0]
        } else {
            [0, 0, 0, 0, 2, 2]
        };
        act(
            &mut g,
            if harmonize {
                Action::AnnounceHarmonize { card: source }
            } else {
                Action::AnnounceCast { card: source }
            },
        );
        act(&mut g, Action::ChooseX { value: 2 });
        if harmonize {
            let object = g.object_ref(helper);
            act(
                &mut g,
                Action::ChooseHarmonizer {
                    object: Some(object),
                },
            );
        }
        finish_ordered_costs(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.object_view(helper).tapped, harmonize);
        advance(&mut g, &[], |g| {
            matches!(g.state.pending, Pending::EffectSearch { .. })
        });
        reject(&mut g, Action::ChooseSearch { card: too_big });
        search_choose(&mut g, eligible);
        drain(&mut g);
        assert_eq!(g.object_view(eligible).zone, Zone::Battlefield);
        assert_eq!(
            g.object_view(source).zone,
            if harmonize {
                Zone::Exile
            } else {
                Zone::Graveyard
            }
        );
    }
}
#[test]
fn neriv_doubles_new_creatures_damage_but_excludes_old_and_opposing_sources() {
    let p = Player::P0;
    let mut g = scenario(p);
    let old = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    // The fixture registers battlefield puts as this-turn entries. Cross a
    // real turn boundary before casting Neriv to make this source old.
    g.begin_turn();
    reset_main(&mut g, p);
    g.state.players[p.index()].mana = [8; 6];
    assert!(!g.entered_this_turn(g.object_ref(old)));
    let neriv = cast_candidate(&mut g, p, "Neriv, Heart of the Storm");
    let new = put(&mut g, p, "Grizzly Bears", Zone::Hand);
    cast(&mut g, new, None);
    drain(&mut g);
    let enemy = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    assert!(g.entered_this_turn(g.object_ref(new)));
    assert!(g.entered_this_turn(g.object_ref(enemy)));
    for (source, expected) in [(new, 4), (old, 2), (enemy, 2)] {
        let before = g.state.players[p.other().index()].life;
        let event = DamageEvent {
            preventable: true,
            source: g.damage_source(source),
            target: Target::Player(p.other()),
            amount: 2,
            combat: false,
        };
        g.apply_damage_batch(&[event]);
        g.advance_replacements(true);
        g.give_priority();
        drain(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, before - expected);
    }
    assert!(g.keywords(neriv).contains(Keywords::FLYING));
}
#[test]
fn new_way_forward_prevents_chosen_source_once_then_reflects_and_draws_exact_amount() {
    let p = Player::P1;
    let mut g = scenario(p);
    let chosen = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    let source = candidate(&mut g, p, "New Way Forward", Zone::Hand);
    g.state.players[p.index()].mana = [1, 1, 0, 1, 0, 2];
    cast(&mut g, source, None);
    advance(&mut g, &[], |g| {
        matches!(g.state.pending, Pending::DamageSource { .. })
    });
    let old = g.object_ref(chosen);
    act(&mut g, Action::ChooseDamageSource { source: old });
    drain(&mut g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    let hand = g.hand_count(p);
    let event = DamageEvent {
        preventable: true,
        source: g.damage_source(chosen),
        target: Target::Player(p),
        amount: 3,
        combat: false,
    };
    g.apply_damage_batch(&[event.clone()]);
    g.advance_replacements(true);
    g.give_priority();
    drain(&mut g);
    assert_eq!(g.state.players[p.index()].life, 20);
    assert_eq!(g.state.players[p.other().index()].life, 17);
    assert_eq!(g.hand_count(p), hand + 3);
    g.apply_damage_batch(&[event]);
    g.advance_replacements(true);
    g.give_priority();
    drain(&mut g);
    assert_eq!(g.state.players[p.index()].life, 17);
    assert_eq!(g.hand_count(p), hand + 3);
}
#[test]
fn rally_all_three_modes_cost_two_less_after_an_actual_cast_and_monks_have_prowess() {
    for mode in 0..3 {
        let p = Player::P0;
        let mut g = scenario(p);
        let first = put(&mut g, p, "Short Bow", Zone::Hand);
        cast(&mut g, first, None);
        drain(&mut g);
        let a = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let b = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let large = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
        let source = candidate(&mut g, p, "Rally the Monastery", Zone::Hand);
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 1];
        act(&mut g, Action::AnnounceCast { card: source });
        act(&mut g, Action::ChooseMode { mode });
        if mode == 1 {
            for target in [a, b] {
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(target),
                    },
                );
            }
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
        }
        if mode == 2 {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(a),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(large),
                },
            );
        }
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        drain(&mut g);
        match mode {
            0 => {
                let monks = battlefield(&g, "Monk");
                assert_eq!(monks.len(), 2);
                for id in &monks {
                    assert_eq!(stats(&g, *id), (1, 1));
                    assert_eq!(g.object_view(*id).colors, 1);
                }
                g.state.players[p.index()].mana = [8; 6];
                let spell = put(&mut g, p, "Short Bow", Zone::Hand);
                cast(&mut g, spell, None);
                drain(&mut g);
                for id in monks {
                    assert_eq!(stats(&g, id), (2, 2));
                }
            }
            1 => {
                assert_eq!(stats(&g, a), (4, 4));
                assert_eq!(stats(&g, b), (5, 5));
            }
            2 => assert_eq!(g.object_view(large).zone, Zone::Graveyard),
            _ => unreachable!(),
        }
    }
}
fn finish_one_of_three(g: &mut Game) -> CardId {
    advance(g, &[], |g| {
        matches!(g.state.pending, Pending::EffectLibrarySelection { .. })
    });
    let id = g
        .legal_actions()
        .iter()
        .find_map(|a| {
            if let Action::SelectLibrary { card } = a {
                Some(*card)
            } else {
                None
            }
        })
        .unwrap();
    act(g, Action::SelectLibrary { card: id });
    if g.legal_actions().contains(&Action::FinishLibrarySelection) {
        act(g, Action::FinishLibrarySelection);
    }
    for _ in 0..4 {
        if g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::OrderLibraryBottom { .. }))
        {
            choose(g, |a| matches!(a, Action::OrderLibraryBottom { .. }));
        } else {
            break;
        }
    }
    drain(g);
    id
}
fn next_saga_chapter(g: &mut Game, p: Player) {
    g.begin_turn();
    reset_main(g, p);
    begin_step(g, Step::Main1);
}
#[test]
fn rediscover_chapters_choose_cards_then_install_repeated_source_independent_noncreature_callback()
{
    let p = Player::P1;
    let mut g = scenario(p);
    let source = candidate(&mut g, p, "Rediscover the Way", Zone::Hand);
    cast(&mut g, source, None);
    let first = finish_one_of_three(&mut g);
    assert_eq!(g.object_view(first).zone, Zone::Hand);
    assert_eq!(g.counter_count(source, CounterKind::Lore), 1);
    next_saga_chapter(&mut g, p);
    let second = finish_one_of_three(&mut g);
    assert_eq!(g.object_view(second).zone, Zone::Hand);
    assert_eq!(g.counter_count(source, CounterKind::Lore), 2);
    next_saga_chapter(&mut g, p);
    drain(&mut g);
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    let a = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let b = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    for target in [a, b] {
        g.state.players[p.index()].mana = [8; 6];
        let spell = put(&mut g, p, "Short Bow", Zone::Hand);
        cast(&mut g, spell, None);
        drain_to(&mut g, &[target]);
        assert!(g.keywords(target).contains(Keywords::DOUBLE_STRIKE));
    }
    g.state.step = Step::Cleanup;
    g.finish_cleanup();
    for id in [a, b] {
        assert!(!g.keywords(id).contains(Keywords::DOUBLE_STRIKE));
    }
}
#[test]
fn revival_saga_makes_spirits_distributes_three_then_grants_current_creatures_keywords() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = candidate(&mut g, p, "Revival of the Ancestors", Zone::Hand);
    cast(&mut g, source, None);
    drain(&mut g);
    let spirits = battlefield(&g, "Spirit");
    assert_eq!(spirits.len(), 3);
    for &id in &spirits {
        assert_eq!(stats(&g, id), (1, 1));
        assert_eq!(g.object_view(id).colors, 1);
        assert!(!g.keywords(id).contains(Keywords::FLYING));
    }
    next_saga_chapter(&mut g, p);
    advance(&mut g, &[], |g| {
        matches!(g.state.pending, Pending::Targets { .. })
    });
    for &id in &spirits {
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
    for _ in 0..3 {
        act(&mut g, Action::AllocateTargetAmount { amount: 1 });
    }
    drain(&mut g);
    for &id in &spirits {
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1);
    }
    next_saga_chapter(&mut g, p);
    drain(&mut g);
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    for &id in &spirits {
        assert!(
            g.keywords(id)
                .contains(Keywords::TRAMPLE.union(Keywords::LIFELINK))
        );
    }
    let later = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    assert!(
        !g.keywords(later)
            .intersects(Keywords::TRAMPLE.union(Keywords::LIFELINK))
    );
}
#[test]
fn reverberating_summons_animates_each_combat_then_pays_entire_hand_and_sacrifice() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = cast_candidate(&mut g, p, "Reverberating Summons");
    begin_step(&mut g, Step::BeginCombat);
    drain(&mut g);
    assert!(!g.is_creature(source));
    reset_main(&mut g, p);
    let spell = put(&mut g, p, "Short Bow", Zone::Hand);
    cast(&mut g, spell, None);
    drain(&mut g);
    g.state.active = p.other();
    begin_step(&mut g, Step::BeginCombat);
    drain(&mut g);
    assert!(g.is_creature(source));
    assert_eq!(stats(&g, source), (3, 3));
    assert!(g.keywords(source).contains(Keywords::HASTE));
    assert!(g.object_view(source).card.kind_mask() & 32 != 0);
    reset_main(&mut g, p);
    let a = put(&mut g, p, "Forest", Zone::Hand);
    let b = put(&mut g, p, "Island", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 1];
    act(
        &mut g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    finish_ordered_costs(&mut g);
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    assert_eq!(g.object_view(a).zone, Zone::Graveyard);
    assert_eq!(g.object_view(b).zone, Zone::Graveyard);
    drain(&mut g);
    assert_eq!(g.hand_count(p), 2);
}
#[test]
fn ringing_strike_taps_stops_natural_untap_and_grants_paid_untap_to_enchanted_controller() {
    let p = Player::P0;
    let mut g = scenario(p);
    let creature = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    let aura = candidate(&mut g, p, "Ringing Strike Mastery", Zone::Hand);
    cast(&mut g, aura, Some(Target::Card(creature)));
    drain(&mut g);
    assert!(g.object_view(creature).tapped);
    g.untap_objects(
        &[creature],
        untap_rules::UntapCause::ControllerStep(p.other()),
    );
    assert!(g.object_view(creature).tapped);
    // The recipient's controller owns this activation. Its Aura need not untap.
    g.state.objects[aura as usize].tapped = true;
    let grantor = g.object_ref(aura);
    reject(
        &mut g,
        Action::AnnounceAttachedAbility {
            grantor,
            ability: 0,
        },
    );
    g.state.priority = p.other();
    g.state.players[p.other().index()].mana = [0, 0, 0, 0, 0, 5];
    act(
        &mut g,
        Action::AnnounceAttachedAbility {
            grantor,
            ability: 0,
        },
    );
    pay(&mut g);
    drain(&mut g);
    assert!(!g.object_view(creature).tapped);
    assert!(g.object_view(aura).tapped);
    assert_eq!(g.state.players[p.other().index()].mana, [0; 6]);
}
#[test]
fn rite_returns_own_permanents_and_shuffles_only_selected_players_graveyard_then_exiles() {
    let p = Player::P1;
    let mut g = scenario(p);
    let a = put(&mut g, p, "Grizzly Bears", Zone::Graveyard);
    let b = put(&mut g, p, "Forest", Zone::Graveyard);
    let shuffled = put(&mut g, p.other(), "Rebellious Strike", Zone::Graveyard);
    let wrong = put(&mut g, p, "Rebellious Strike", Zone::Graveyard);
    let before = g.library_card_count(p.other());
    let source = candidate(&mut g, p, "Rite of Renewal", Zone::Hand);
    act(&mut g, Action::AnnounceCast { card: source });
    for target in [a, b] {
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(&mut g, Action::FinishTargets);
    }
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(wrong),
        },
    );
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(shuffled),
        },
    );
    act(&mut g, Action::FinishTargets);
    pay(&mut g);
    drain(&mut g);
    for id in [a, b] {
        assert_eq!(g.object_view(id).zone, Zone::Hand);
    }
    assert_eq!(g.object_view(shuffled).zone, Zone::Library);
    assert_eq!(g.library_card_count(p.other()), before + 1);
    assert_eq!(g.object_view(wrong).zone, Zone::Graveyard);
    assert_eq!(g.object_view(source).zone, Zone::Exile);
}
#[test]
fn riverwheel_taps_adds_three_stun_and_grants_only_chosen_exile_card_through_next_turn() {
    let p = Player::P0;
    let mut g = scenario(p);
    let target = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    let unchosen = put(&mut g, p, "Island", Zone::Library);
    let chosen = put(&mut g, p, "Grizzly Bears", Zone::Library);
    let source = candidate(&mut g, p, "Riverwheel Sweep", Zone::Hand);
    g.state.players[p.index()].mana = [1, 1, 0, 1, 0, 0];
    cast(&mut g, source, Some(Target::Card(target)));
    advance(&mut g, &[], |g| {
        matches!(g.state.pending, Pending::MilledSelection { .. })
    });
    let object = g.object_ref(chosen);
    act(&mut g, Action::SelectMilled { object });
    if g.legal_actions().contains(&Action::FinishMilledSelection) {
        act(&mut g, Action::FinishMilledSelection);
    }
    drain(&mut g);
    assert!(g.object_view(target).tapped);
    assert_eq!(g.counter_count(target, CounterKind::Stun), 3);
    assert_eq!(g.object_view(unchosen).zone, Zone::Exile);
    reject(&mut g, Action::PlayLand { card: unchosen });
    let mut expiry = g.clone();
    g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
    act(&mut g, Action::AnnounceExile { card: chosen });
    pay(&mut g);
    drain(&mut g);
    assert_eq!(g.object_view(chosen).zone, Zone::Battlefield);
    expiry.state.step = Step::Cleanup;
    expiry.finish_cleanup();
    expiry.state.active = p;
    expiry.begin_turn();
    reset_main(&mut expiry, p);
    expiry.state.players[p.index()].mana = [8; 6];
    assert!(
        expiry
            .legal_actions()
            .iter()
            .any(|a| matches!(a,Action::AnnounceExile {card}if*card==chosen))
    );
    expiry.state.step = Step::Cleanup;
    expiry.finish_cleanup();
    reset_main(&mut expiry, p);
    reject(&mut expiry, Action::AnnounceExile { card: chosen });
}
#[test]
fn rot_curse_decayed_attack_sacrifices_and_renew_requires_exactly_x_creature_targets() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = cast_candidate(&mut g, p, "Rot-Curse Rakshasa");
    g.begin_turn();
    reset_main(&mut g, p);
    assert!(!g.can_be_blocker(source));
    assert!(
        g.keywords(source)
            .contains(Keywords::TRAMPLE.union(Keywords::DECAYED))
    );
    attack_one(&mut g, p, source);
    drain(&mut g);
    begin_step(&mut g, Step::EndCombat);
    drain(&mut g);
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    reset_main(&mut g, p);
    let a = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let b = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    g.state.players[p.index()].mana = [0, 0, 2, 0, 0, 2];
    act(
        &mut g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(&mut g, Action::ChooseX { value: 2 });
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(a),
        },
    );
    reject(&mut g, Action::FinishTargets);
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(b),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(&mut g, Action::FinishTargets);
    }
    pay(&mut g);
    drain(&mut g);
    assert_eq!(g.object_view(source).zone, Zone::Exile);
    for id in [a, b] {
        assert_eq!(g.counter_count(id, CounterKind::Decayed), 1);
        assert!(!g.can_be_blocker(id));
    }
}
#[test]
fn third_family_intrinsic_creature_headers_and_keywords() {
    for (name, p, t, k) in [
        ("Kishla Trawlers", 3, 2, Keywords::NONE),
        ("Kotis, the Fangkeeper", 2, 1, Keywords::INDESTRUCTIBLE),
        ("Lotuslight Dancers", 3, 6, Keywords::LIFELINK),
        ("Magmatic Hellkite", 4, 5, Keywords::FLYING),
        (
            "Mardu Siegebreaker",
            4,
            4,
            Keywords::HASTE.union(Keywords::DEATHTOUCH),
        ),
        ("Naga Fleshcrafter", 0, 0, Keywords::NONE),
        ("Narset, Jeskai Waymaster", 3, 4, Keywords::NONE),
        ("Neriv, Heart of the Storm", 4, 5, Keywords::FLYING),
        (
            "Rot-Curse Rakshasa",
            5,
            5,
            Keywords::TRAMPLE.union(Keywords::DECAYED),
        ),
    ] {
        let c = Card::from_name(name).unwrap();
        assert_eq!(c.effect(), Effect::Creature(p, t), "{name}");
        assert_eq!(c.keywords(), k, "{name}");
    }
}

#[test]
fn naga_may_decline_entry_copy_and_zero_toughness_original_dies() {
    let p = Player::P1;
    let mut g = scenario(p);
    put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
    let source = candidate(&mut g, p, "Naga Fleshcrafter", Zone::Hand);
    cast(&mut g, source, None);
    advance(&mut g, &[], |g| {
        matches!(g.state.pending, Pending::EntryCopy { .. })
    });
    act(&mut g, Action::ChooseEntryCopy { object: None });
    drain(&mut g);
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
}
