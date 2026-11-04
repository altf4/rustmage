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
