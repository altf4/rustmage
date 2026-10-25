//! Whole-card paid producer scenarios using ordinary admitted fixture objects.
//! No program, cost, target, provider, or rules definition is substituted.
//! Explicit primitive identity/characteristic seams are labeled at their use.
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::*;
const ORIGIN: &str = "Origin of the Avengers";
const NICK: &str = "Nick Fury, Agent of S.H.I.E.L.D.";
fn quiet(g: &Game) -> bool {
    g.state.resolving.is_none()
        && g.state.stack.is_empty()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
        && g.state.pending == Pending::Priority
}
fn views(g: &Game) -> [serde_json::Value; 2] {
    [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap())
}
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    let seen = views(g);
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
    assert_eq!(views(g), seen);
}
fn orphan(g: Game, action: Action) -> Game {
    let actor = g.actor().unwrap();
    let revision = g.revision();
    let before = g.snapshot();
    let seen = views(&g);
    let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
    let (mut raw, mut indexed) = (g.clone(), g.clone());
    drop(g);
    raw.apply(actor, action.clone()).unwrap();
    indexed.apply_index(actor, revision, index).unwrap();
    let after = raw.snapshot();
    assert_eq!(after, indexed.snapshot());
    assert_eq!(views(&raw), views(&indexed));
    raw.restore(&before).unwrap();
    let mut restored = seen;
    for view in &mut restored {
        view["revision"] = serde_json::json!(raw.revision());
    }
    assert_eq!(views(&raw), restored);
    assert_eq!(
        raw.apply_index(actor, revision, index),
        Err(Error::StaleRevision)
    );
    raw.apply(actor, action).unwrap();
    assert_eq!(raw.snapshot(), after);
    indexed
}

fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..120 {
        if stop(g) {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                        | Action::Discard { .. }
                )
            })
            .unwrap_or_else(|| panic!("unexpected {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("stalled {:?}", g.state.pending);
}
fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    priority(g, p);
    let c = Card::from_name(name).unwrap();
    let id = named(g, p, name, Zone::Hand);
    let (colored, generic) = c.cost();
    g.state.players[p.index()].mana = colored;
    g.state.players[p.index()].mana[5] += generic;
    cast(g, id, None, &[]);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    if name != ORIGIN {
        settle(g);
    }
    id
}
fn origin(g: &mut Game, p: Player) -> CardId {
    let id = paid(g, p, ORIGIN);
    until(g, |g| g.observe(p).library_choice.is_some());
    while let Some(v) = g.observe(p).library_choice {
        assert!(g.observe(p.other()).library_choice.is_none());
        checked(
            g,
            Action::ArrangeLibrary {
                card: v.remaining[0].id,
                destination: LibraryDestination::Top,
            },
        );
    }
    settle(g);
    id
}
fn second(g: &mut Game, p: Player) {
    next_turn(g);
    next_turn(g);
    until(g, |g| g.observe(p).effect_choice.is_some());
}
fn announce(id: CardId) -> Action {
    Action::AnnounceActivate {
        card: id,
        ability: 0,
        counter_payment: Counters::default(),
    }
}
fn pay(g: &mut Game) {
    for _ in 0..12 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::ChooseActivationCost { .. }
                        | Action::PayCosts { .. }
                        | Action::FinishTargets
                )
            })
            .unwrap();
        checked(g, a);
    }
    panic!("unfinished payment");
}
fn activate(g: &mut Game, p: Player, id: CardId) {
    priority(g, p);
    checked(g, announce(id));
    pay(g);
}
fn look(g: &mut Game, p: Player) {
    until(g, |g| g.observe(p).library_selection.is_some() || quiet(g));
}
fn select(g: &mut Game, id: CardId) {
    checked(g, Action::SelectLibrary { card: id });
    checked(g, Action::FinishLibrarySelection);
}
fn empty_library(g: &mut Game, p: Player) {
    while let Some(&id) = g.state.players[p.index()].library.last() {
        g.move_object(id, Zone::Outside);
    }
}
#[test]
fn msh_origin_nick_cards_origin_paid_scry_two_private_permutations_owned() {
    for p in [Player::P0, Player::P1] {
        for bottom in [false, true] {
            let mut g = setup(p);
            let a = named(&mut g, p, "Island", Zone::Library);
            let b = named(&mut g, p, "Mountain", Zone::Library);
            let source = paid(&mut g, p, ORIGIN);
            until(&mut g, |g| g.observe(p).library_choice.is_some());
            assert_eq!(g.counter_count(source, CounterKind::Lore), 1);
            let v = g.observe(p).library_choice.unwrap();
            assert_eq!(v.remaining.len(), 2);
            assert!(g.observe(p.other()).library_choice.is_none());
            reject(
                &mut g,
                Action::ArrangeLibrary {
                    card: CardId::MAX,
                    destination: LibraryDestination::Top,
                },
            );
            g = orphan(
                g,
                Action::ArrangeLibrary {
                    card: b,
                    destination: if bottom {
                        LibraryDestination::Bottom
                    } else {
                        LibraryDestination::Top
                    },
                },
            );
            checked(
                &mut g,
                Action::ArrangeLibrary {
                    card: a,
                    destination: LibraryDestination::Top,
                },
            );
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].library.last(),
                Some(if bottom { &a } else { &b })
            );
            if bottom {
                assert_eq!(g.state.players[p.index()].library.first(), Some(&b));
            }
            assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        }
    }
}
#[test]
fn msh_origin_nick_cards_origin_natural_chapters_put_and_all_current_creatures_final_sacrifice() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = origin(&mut g, p);
        let own = paid(&mut g, p, "Bear Cub");
        let foreign = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let hero = named(&mut g, p, "King T'Challa", Zone::Hand);
        second(&mut g, p);
        assert_eq!(g.counter_count(source, CounterKind::Lore), 2);
        let n = g.hand_count(p);
        let object = g.object_ref(hero);
        g = orphan(g, Action::PutHandPermanent { object });
        settle(&mut g);
        assert_eq!(g.hand_count(p), n - 1);
        assert_eq!(g.object_view(hero).zone, Zone::Battlefield);
        let late = paid(&mut g, p, "Borough Backup");
        assert_eq!(g.object_view(late).zone, Zone::Graveyard);
        let creatures: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(i, o)| {
                (o.zone == Zone::Battlefield && o.controller == p && g.is_creature(i as CardId))
                    .then_some(i as CardId)
            })
            .collect();
        assert!(creatures.len() >= 3);
        next_turn(&mut g);
        next_turn(&mut g);
        until(&mut g, |g| g.object_view(source).zone == Zone::Graveyard);
        settle(&mut g);
        for id in creatures {
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1, "{id}");
        }
        assert_eq!(g.counter_count(own, CounterKind::PlusOne), 1);
        assert_eq!(g.counter_count(foreign, CounterKind::PlusOne), 0);
    }
}
#[test]
fn msh_origin_nick_cards_origin_filter_intersection_changeling_and_decline_draw() {
    for p in [Player::P0, Player::P1] {
        for chosen in [
            "Jennifer Walters",
            "King T'Challa",
            "Three Tree Mascot",
            "decline",
        ] {
            let mut g = setup(p);
            origin(&mut g, p);
            let ids = [
                "Jennifer Walters",
                "King T'Challa",
                "Three Tree Mascot",
                "Pet Avengers",
                "Namor the Sub-Mariner",
                "Hill Giant",
            ]
            .map(|name| named(&mut g, p, name, Zone::Hand));
            second(&mut g, p);
            let v = g.observe(p).effect_choice.unwrap();
            assert_eq!(
                v.candidates,
                ids[..3]
                    .iter()
                    .map(|&id| g.object_ref(id))
                    .collect::<Vec<_>>()
            );
            for &id in &ids[3..] {
                let object = g.object_ref(id);
                reject(&mut g, Action::PutHandPermanent { object });
            }
            let n = g.hand_count(p);
            if chosen == "decline" {
                checked(&mut g, Action::DeclineHandEntry);
                assert_eq!(g.hand_count(p), n + 1);
            } else {
                let id = ids
                    .iter()
                    .copied()
                    .find(|&id| g.object_view(id).card.name() == chosen)
                    .unwrap();
                let object = g.object_ref(id);
                checked(&mut g, Action::PutHandPermanent { object });
                assert_eq!(g.hand_count(p), n - 1);
                assert_eq!(g.object_view(id).zone, Zone::Battlefield);
            }
            settle(&mut g);
        }
    }
}
#[test]
fn msh_origin_nick_cards_origin_hidden_eligible_and_empty_hand_same_nonactor_observation() {
    for p in [Player::P0, Player::P1] {
        let games = ["Jennifer Walters", "Hill Giant"].map(|name| {
            let mut g = setup(p);
            origin(&mut g, p);
            named(&mut g, p, name, Zone::Hand);
            second(&mut g, p);
            g
        });
        assert_eq!(
            views(&games[0])[p.other().index()],
            views(&games[1])[p.other().index()]
        );
        assert_eq!(
            games[0].observe(p).effect_choice.unwrap().candidates.len(),
            1
        );
        assert!(
            games[1]
                .observe(p)
                .effect_choice
                .unwrap()
                .candidates
                .is_empty()
        );
    }
}
#[test]
fn msh_origin_nick_cards_origin_queued_chapter_survives_actual_paid_disenchant() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = origin(&mut g, p);
        let hero = named(&mut g, p, "King T'Challa", Zone::Hand);
        next_turn(&mut g);
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state
                .stack
                .iter()
                .any(|s| s.card == source && s.kind == StackKind::TriggeredAbility { ability: 1 })
        });
        priority(&mut g, p.other());
        let removal = named(&mut g, p.other(), "Disenchant", Zone::Hand);
        g.state.players[p.other().index()].mana = [1, 0, 0, 0, 0, 1];
        cast(&mut g, removal, None, &[Target::Card(source)]);
        until(&mut g, |g| g.observe(p).effect_choice.is_some());
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        let object = g.object_ref(hero);
        checked(&mut g, Action::PutHandPermanent { object });
        settle(&mut g);
        assert_eq!(g.object_view(hero).zone, Zone::Battlefield);
    }
}
#[test]
fn msh_origin_nick_cards_origin_decline_draw_resumes_actual_lorehold_miracle_pause() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        origin(&mut g, p);
        second(&mut g, p);
        // The normal draw step already happened. A fresh supported Lorehold only
        // grants the first-draw replacement on a later turn, so this explicit
        // turn-history boundary isolates that existing owned pause.
        named(&mut g, p, "Lorehold, the Historian", Zone::Battlefield);
        g.state.players[p.index()].cards_drawn_this_turn = 0;
        let n = g.hand_count(p);
        checked(&mut g, Action::DeclineHandEntry);
        assert!(matches!(g.state.pending, Pending::MiracleReveal { .. }));
        assert!(g.observe(p.other()).miracle.is_none());
        g = orphan(g, Action::DeclineMiracle);
        settle(&mut g);
        assert_eq!(g.hand_count(p), n + 1);
    }
}
#[test]
fn msh_origin_nick_cards_origin_doubling_season_chapter_crossing_and_counter_replacement() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let creature = paid(&mut g, p, "Bear Cub");
        let source = origin(&mut g, p);
        paid(&mut g, p, "Doubling Season");
        next_turn(&mut g);
        next_turn(&mut g);
        until(&mut g, |g| g.observe(p).effect_choice.is_some());
        assert_eq!(g.counter_count(source, CounterKind::Lore), 2);
        // Turn-based lore is not an effect, so Doubling Season does not double it.
        checked(&mut g, Action::DeclineHandEntry);
        settle(&mut g);
        next_turn(&mut g);
        next_turn(&mut g);
        until(&mut g, |g| g.object_view(source).zone == Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.counter_count(creature, CounterKind::PlusOne), 2);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    }
}
#[test]
fn msh_origin_nick_cards_nick_paid_white_body_entry_discount_older_full_cost_atomicity() {
    for p in [Player::P0, Player::P1] {
        for older in [false, true] {
            if !older {
                let mut deck = vec![Card::from_name(NICK).unwrap(); 4];
                deck.extend(vec![Card::from_name(ORIGIN).unwrap(); 4]);
                deck.extend(vec![Card::Plains; 20]);
                for name in ["Island", "Swamp", "Mountain", "Forest"] {
                    deck.extend(vec![Card::from_name(name).unwrap(); 8]);
                }
                assert_eq!(deck.len(), 60);
                assert!(
                    Game::new(Config {
                        decks: [deck.clone(), deck],
                        seed: 20261025,
                        starting_player: p,
                        deck_rules: DeckRules::Standard
                    })
                    .is_ok()
                );
            }
            let mut g = setup(p);
            let source = paid(&mut g, p, NICK);
            assert_eq!(
                (g.object_view(source).power, g.object_view(source).toughness),
                (2, 1)
            );
            if older {
                next_turn(&mut g);
                next_turn(&mut g);
                until(&mut g, |g| g.state.step == Step::Main1 && quiet(g));
            }
            g.state.players[p.index()].mana = [0, 1, 1, 1, 0, 0];
            reject(&mut g, announce(source));
            g.state.players[p.index()].mana = if older {
                [1, 1, 1, 1, 1, 0]
            } else {
                [0, 1, 1, 1, 1, 0]
            };
            activate(&mut g, p, source);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            look(&mut g, p);
            assert_eq!(g.counter_count(source, CounterKind::PlusOne), 2);
            assert_eq!(
                (g.object_view(source).power, g.object_view(source).toughness),
                (4, 3)
            );
            checked(&mut g, Action::FinishLibrarySelection);
            settle(&mut g);
            g.state.players[p.index()].mana = [30; 6];
            reject(&mut g, announce(source));
            assert_eq!(g.object_view(source).power_up_uses[0].count, 1);
        }
    }
}
#[test]
fn msh_origin_nick_cards_nick_hero_equipment_vehicle_or_filter_without_targeting() {
    for p in [Player::P0, Player::P1] {
        for name in [
            "King T'Challa",
            "Captain America's Shield",
            "S.H.I.E.L.D. Helicarrier",
        ] {
            let mut g = setup(p);
            let source = paid(&mut g, p, NICK);
            let selected = named(&mut g, p, name, Zone::Library);
            let wrong = named(&mut g, p, "Hill Giant", Zone::Library);
            g.state.players[p.index()].mana = [0, 1, 1, 1, 1, 0];
            activate(&mut g, p, source);
            look(&mut g, p);
            assert!(g.observe(p.other()).library_selection.is_none());
            reject(&mut g, Action::SelectLibrary { card: wrong });
            g = orphan(g, Action::SelectLibrary { card: selected });
            checked(&mut g, Action::FinishLibrarySelection);
            if g.observe(p).entry_transform.is_some() {
                checked(&mut g, Action::ChooseOptional { accept: false });
            }
            settle(&mut g);
            assert_eq!(g.object_view(selected).zone, Zone::Battlefield);
            assert_eq!(g.object_view(selected).controller, p);
        }
    }
}
#[test]
fn msh_origin_nick_cards_nick_paid_mentor_front_entry_transform_before_remainder_and_trigger_stack()
{
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, NICK);
        let a = paid(&mut g, p, "Mentor of the Meek");
        let b = paid(&mut g, p, "Mentor of the Meek");
        let uprising = paid(&mut g, p, "Garruk's Uprising");
        let eight = named(&mut g, p, "Island", Zone::Library);
        for _ in 0..6 {
            named(&mut g, p, "Forest", Zone::Library);
        }
        let hero = named(&mut g, p, "Jennifer Walters", Zone::Library);
        g.state.players[p.index()].mana = [0, 1, 1, 1, 1, 0];
        activate(&mut g, p, source);
        look(&mut g, p);
        select(&mut g, hero);
        assert_eq!(g.object_view(hero).power, 2);
        assert!(g.state.stack.is_empty());
        assert_eq!(
            g.state
                .pending_triggers
                .iter()
                .filter(|s| s.card == a || s.card == b)
                .count(),
            2
        );
        assert!(!g.state.pending_triggers.iter().any(|s| s.card == uprising));
        let n = g.hand_count(p);
        g = orphan(g, Action::ChooseOptional { accept: true });
        assert_eq!(g.object_view(hero).power, 6);
        assert_eq!(g.state.players[p.index()].library.last(), Some(&eight));
        assert!(
            !g.state
                .pending_triggers
                .iter()
                .chain(&g.state.stack)
                .any(|s| s.card == uprising)
        );
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        for _ in 0..30 {
            if quiet(&g) {
                break;
            }
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::Pass
                            | Action::PutTrigger { .. }
                            | Action::PayResolutionMana { .. }
                            | Action::FinishTargets
                    )
                })
                .unwrap();
            checked(&mut g, action);
        }
        assert!(quiet(&g));
        assert_eq!(g.hand_count(p), n + 2);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn msh_origin_nick_cards_nick_real_legend_menu_after_transform_decline_or_accept() {
    for p in [Player::P0, Player::P1] {
        for accept in [false, true] {
            let mut g = setup(p);
            let source = paid(&mut g, p, NICK);
            let old = paid(&mut g, p, "Jennifer Walters");
            let new = named(&mut g, p, "Jennifer Walters", Zone::Library);
            g.state.players[p.index()].mana = [0, 1, 1, 1, 1, 0];
            activate(&mut g, p, source);
            look(&mut g, p);
            select(&mut g, new);
            assert_eq!(g.object_view(old).zone, Zone::Battlefield);
            assert_eq!(g.object_view(new).zone, Zone::Battlefield);
            checked(&mut g, Action::ChooseOptional { accept });
            if !accept {
                let action = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::KeepLegend { .. }))
                    .unwrap();
                g = orphan(g, action);
            }
            settle(&mut g);
            if accept {
                assert_eq!(g.object_view(old).zone, Zone::Battlefield);
                assert_eq!(g.object_view(new).card.name(), "The Sensational She-Hulk");
            } else {
                assert_ne!(g.object_view(old).zone, g.object_view(new).zone);
            }
        }
    }
}
#[test]
fn msh_origin_nick_cards_nick_monica_optional_faces_and_provider_refresh() {
    for p in [Player::P0, Player::P1] {
        for accept in [false, true] {
            let mut g = setup(p);
            let source = paid(&mut g, p, NICK);
            let hero = named(&mut g, p, "Monica Rambeau", Zone::Library);
            g.state.players[p.index()].mana = [0, 1, 1, 1, 1, 0];
            activate(&mut g, p, source);
            look(&mut g, p);
            select(&mut g, hero);
            g = orphan(g, Action::ChooseOptional { accept });
            settle(&mut g);
            assert_eq!(
                g.object_view(hero).card.name(),
                if accept {
                    "Photon, Living Light"
                } else {
                    "Monica Rambeau"
                }
            );
            assert_eq!(
                g.object_view(hero).face.unwrap().transforms,
                u64::from(accept)
            );
        }
    }
}
#[test]
fn msh_origin_nick_cards_nick_actual_paid_murder_source_gone_keeps_library_ability() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, NICK);
        let hero = named(&mut g, p, "Three Tree Mascot", Zone::Library);
        g.state.players[p.index()].mana = [0, 1, 1, 1, 1, 0];
        activate(&mut g, p, source);
        priority(&mut g, p.other());
        let removal = named(&mut g, p.other(), "Murder", Zone::Hand);
        g.state.players[p.other().index()].mana = [0, 0, 2, 0, 0, 1];
        cast(&mut g, removal, None, &[Target::Card(source)]);
        look(&mut g, p);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        select(&mut g, hero);
        settle(&mut g);
        assert_eq!(g.object_view(hero).zone, Zone::Battlefield);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 0);
    }
}
#[test]
fn msh_origin_nick_cards_nick_empty_short_top_seven_optional_and_owned_random_replay() {
    for p in [Player::P0, Player::P1] {
        for count in [0, 1, 6, 7, 8] {
            let mut g = setup(p);
            let source = paid(&mut g, p, NICK);
            empty_library(&mut g, p);
            let mut ids = Vec::new();
            for _ in 0..count {
                ids.push(named(&mut g, p, "Island", Zone::Library));
            }
            g.state.players[p.index()].mana = [0, 1, 1, 1, 1, 0];
            activate(&mut g, p, source);
            look(&mut g, p);
            if count > 0 {
                assert_eq!(
                    g.observe(p).library_selection.unwrap().remaining.len(),
                    count.min(7)
                );
                assert!(g.observe(p.other()).library_selection.is_none());
                reject(
                    &mut g,
                    Action::SelectLibrary {
                        card: ids[count - 1],
                    },
                );
                g = orphan(g, Action::FinishLibrarySelection);
            }
            assert!(quiet(&g));
            assert_eq!(g.state.players[p.index()].library.len(), count);
            if count == 8 {
                assert_eq!(g.state.players[p.index()].library.last(), Some(&ids[0]));
            }
            assert_eq!(g.counter_count(source, CounterKind::PlusOne), 2);
        }
    }
}
#[test]
fn msh_origin_nick_cards_nick_instant_opponent_turn_tapped_sick_and_once_survives_control() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, NICK);
        g.state.objects[source as usize].tapped = true;
        assert!(g.state.objects[source as usize].summoning_sick);
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.step == Step::Upkeep && g.actor() == Some(p)
        });
        g.state.players[p.index()].mana = [1, 1, 1, 1, 1, 0];
        activate(&mut g, p, source);
        look(&mut g, p);
        checked(&mut g, Action::FinishLibrarySelection);
        settle(&mut g);
        // Explicit control boundary: the admitted pool has no instant control
        // transfer for this exact window. Once history belongs to the object.
        g.state.objects[source as usize].controller = p.other();
        g.state.priority = p.other();
        g.state.players[p.other().index()].mana = [30; 6];
        reject(&mut g, announce(source));
        g.state.objects[source as usize].controller = p;
        g.state.priority = p;
        g.move_object(source, Zone::Exile);
        g.move_object(source, Zone::Battlefield);
        settle(&mut g);
        assert!(g.object_view(source).power_up_uses.is_empty());
        g.state.players[p.index()].mana = [0, 1, 1, 1, 1, 0];
        activate(&mut g, p, source);
        look(&mut g, p);
        checked(&mut g, Action::FinishLibrarySelection);
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 2);
    }
}
#[test]
fn msh_origin_nick_cards_nick_actual_restricted_hero_lots_and_prospective_funding() {
    for p in [Player::P0, Player::P1] {
        for good in [false, true] {
            let mut g = setup(p);
            let source = paid(&mut g, p, NICK);
            let land = named(
                &mut g,
                p,
                if good {
                    "Avengers Tower"
                } else {
                    "Villainous Hideout"
                },
                Zone::Battlefield,
            );
            g.state.players[p.index()].mana = [0, 0, 1, 1, 1, 0];
            assert_eq!(
                g.legal_actions().contains(&announce(source)),
                good,
                "prospective restricted blue"
            );
            checked(
                &mut g,
                Action::ActivateMana {
                    card: land,
                    ability: 1,
                    payment: [0; 6],
                    counter_payment: Counters::default(),
                    production: [0, 1, 0, 0, 0, 0],
                },
            );
            assert_eq!(
                g.legal_actions().contains(&announce(source)),
                good,
                "actual restricted blue"
            );
            if good {
                activate(&mut g, p, source);
                look(&mut g, p);
                checked(&mut g, Action::FinishLibrarySelection);
                settle(&mut g);
                assert_eq!(g.counter_count(source, CounterKind::PlusOne), 2);
            } else {
                reject(&mut g, announce(source));
            }
        }
    }
}

#[test]
fn msh_origin_nick_cards_nick_selection_exact_looked_refs_one_pick_and_outside_top_seven() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, NICK);
        empty_library(&mut g, p);
        let outside = named(&mut g, p, "King T'Challa", Zone::Library);
        let a = named(&mut g, p, "Captain America's Shield", Zone::Library);
        let b = named(&mut g, p, "Three Tree Mascot", Zone::Library);
        for name in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
            named(&mut g, p, name, Zone::Library);
        }
        let hand = named(&mut g, p, "King T'Challa", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 1, 1, 1, 0];
        activate(&mut g, p, source);
        look(&mut g, p);
        reject(&mut g, Action::SelectLibrary { card: outside });
        reject(&mut g, Action::SelectLibrary { card: hand });
        checked(&mut g, Action::SelectLibrary { card: a });
        reject(&mut g, Action::SelectLibrary { card: b });
        g = orphan(g, Action::FinishLibrarySelection);
        settle(&mut g);
        assert_eq!(g.object_view(a).zone, Zone::Battlefield);
        assert_eq!(g.object_view(b).zone, Zone::Library);
        assert_eq!(g.state.players[p.index()].library.last(), Some(&outside));
    }
}
#[test]
fn msh_origin_nick_cards_nick_captured_player_and_old_source_reference_survive_control_and_blink() {
    for p in [Player::P0, Player::P1] {
        for blink in [false, true] {
            let mut g = setup(p);
            let source = paid(&mut g, p, NICK);
            let entrant = named(&mut g, p, "Three Tree Mascot", Zone::Library);
            g.state.players[p.index()].mana = [0, 1, 1, 1, 1, 0];
            activate(&mut g, p, source);
            // Explicit no-priority identity/control fixture after a real paid packet.
            // The shared core separately proves the same exact-reference boundary.
            if blink {
                g.move_object(source, Zone::Exile);
                g.move_object(source, Zone::Battlefield);
            } else {
                g.state.objects[source as usize].controller = p.other();
            }
            look(&mut g, p);
            assert_eq!(g.actor(), Some(p));
            assert!(g.observe(p.other()).library_selection.is_none());
            select(&mut g, entrant);
            settle(&mut g);
            assert_eq!(g.object_view(entrant).controller, p);
            assert_eq!(
                g.counter_count(source, CounterKind::PlusOne),
                if blink { 0 } else { 2 }
            );
        }
    }
}
#[test]
fn msh_origin_nick_cards_nick_suppression_disables_actual_and_funding_activation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, NICK);
        priority(&mut g, p.other());
        let aura = named(&mut g, p.other(), "Witness Protection", Zone::Hand);
        // A sorcery-speed Aura is cast on its controller's precombat main in this
        // ordinary scenario setup; the recipient was genuinely paid previously.
        g.state.active = p.other();
        g.state.step = Step::Main1;
        g.state.players[p.other().index()].mana = [0, 1, 0, 0, 0, 0];
        cast(&mut g, aura, None, &[Target::Card(source)]);
        settle(&mut g);
        priority(&mut g, p);
        g.state.players[p.index()].mana = [30; 6];
        reject(&mut g, announce(source));
        assert_eq!(
            (g.object_view(source).power, g.object_view(source).toughness),
            (1, 1)
        );
    }
}
#[test]
fn msh_origin_nick_cards_kindred_hero_is_not_a_creature_in_origin_filter_but_nick_accepts_it() {
    // Bounded immutable-characteristic fixture only: Firdoch Core remains gated
    // and is never cast or put onto the battlefield by this test.
    let kindred = Card::from_name("Firdoch Core").unwrap();
    assert!(!kindred.is_supported());
    assert!(kindred.has_subtype("Hero"));
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        origin(&mut g, p);
        let id = named(&mut g, p, "Plains", Zone::Hand);
        g.state.objects[id as usize].card = kindred;
        second(&mut g, p);
        let object = g.object_ref(id);
        reject(&mut g, Action::PutHandPermanent { object });
        checked(&mut g, Action::DeclineHandEntry);
        settle(&mut g);
        let mut g = setup(p);
        let source = paid(&mut g, p, NICK);
        let id = named(&mut g, p, "Plains", Zone::Library);
        g.state.objects[id as usize].card = kindred;
        g.state.players[p.index()].mana = [0, 1, 1, 1, 1, 0];
        activate(&mut g, p, source);
        look(&mut g, p);
        assert!(
            g.legal_actions()
                .contains(&Action::SelectLibrary { card: id })
        );
        checked(&mut g, Action::FinishLibrarySelection);
        settle(&mut g);
        assert_eq!(g.object_view(id).zone, Zone::Library);
    }
}

#[test]
fn msh_origin_nick_cards_origin_empty_and_single_card_scry_owned() {
    for p in [Player::P0, Player::P1] {
        for count in [0, 1] {
            let mut g = setup(p);
            empty_library(&mut g, p);
            let card = (count == 1).then(|| named(&mut g, p, "Island", Zone::Library));
            let source = paid(&mut g, p, ORIGIN);
            until(&mut g, |g| {
                quiet(g) || g.observe(p).library_choice.is_some()
            });
            if let Some(card) = card {
                assert_eq!(g.observe(p).library_choice.unwrap().remaining.len(), 1);
                assert!(g.observe(p.other()).library_choice.is_none());
                g = orphan(
                    g,
                    Action::ArrangeLibrary {
                        card,
                        destination: LibraryDestination::Bottom,
                    },
                );
                settle(&mut g);
            }
            assert!(quiet(&g));
            assert_eq!(g.counter_count(source, CounterKind::Lore), 1);
            assert_eq!(g.state.players[p.index()].library.len(), count);
        }
    }
}
#[test]
fn msh_origin_nick_cards_origin_queued_final_chapter_uses_resolution_creatures_after_source_destruction()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = origin(&mut g, p);
        let old = paid(&mut g, p, "Bear Cub");
        second(&mut g, p);
        checked(&mut g, Action::DeclineHandEntry);
        settle(&mut g);
        next_turn(&mut g);
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state
                .stack
                .iter()
                .any(|s| s.card == source && s.kind == StackKind::TriggeredAbility { ability: 2 })
        });
        priority(&mut g, p);
        let removal = named(&mut g, p, "Disenchant", Zone::Hand);
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 1];
        cast(&mut g, removal, None, &[Target::Card(source)]);
        until(&mut g, |g| g.object_view(source).zone == Zone::Graveyard);
        // This genuinely paid instant puts new creatures onto the battlefield
        // after chapter III was queued, before it resolves.
        paid(&mut g, p, "Hire a Crew");
        settle(&mut g);
        let ours: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, o)| {
                (o.zone == Zone::Battlefield && o.controller == p && g.is_creature(id as CardId))
                    .then_some(id as CardId)
            })
            .collect();
        assert_eq!(ours.len(), 2);
        for id in ours {
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1);
        }
        assert_eq!(g.counter_count(old, CounterKind::PlusOne), 1);
    }
}
#[test]
fn msh_origin_nick_cards_nick_counters_use_actual_doubling_season_before_private_look() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, NICK);
        paid(&mut g, p, "Doubling Season");
        g.state.players[p.index()].mana = [0, 1, 1, 1, 1, 0];
        activate(&mut g, p, source);
        look(&mut g, p);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 4);
        assert_eq!(
            (g.object_view(source).power, g.object_view(source).toughness),
            (6, 5)
        );
        checked(&mut g, Action::FinishLibrarySelection);
        settle(&mut g);
    }
}
