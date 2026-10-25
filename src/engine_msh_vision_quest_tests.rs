//! Individual paid Vision Quest scenarios; bounded entry and incarnation probes are labelled.
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::*;
use crate::CreatureType;
const QUEST: &str = "Vision Quest";
const BOT: &str = "A.I.M. Synthoids";
fn views(g: &Game) -> [serde_json::Value; 2] {
    [Player::P0, Player::P1].map(|p| {
        let mut v = g.observe(p);
        v.revision = 0;
        serde_json::to_value(v).unwrap()
    })
}
fn reject(g: &mut Game, a: Action) {
    let before = (g.snapshot(), g.revision(), g.state_bytes(), views(g));
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(
        (g.snapshot(), g.revision(), g.state_bytes(), views(g)),
        before
    );
}
fn orphan(g: Game, a: Action) -> Game {
    let actor = g.actor().unwrap();
    let rev = g.revision();
    let ix = g.legal_actions().iter().position(|v| *v == a).unwrap();
    let before = g.snapshot();
    let oldviews = views(&g);
    let (mut raw, mut indexed) = (g.clone(), g.clone());
    drop(g);
    raw.apply(actor, a.clone()).unwrap();
    indexed.apply_index(actor, rev, ix).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(views(&raw), views(&indexed));
    let after = raw.snapshot();
    raw.restore(&before).unwrap();
    assert_eq!(views(&raw), oldviews);
    assert_eq!(raw.apply_index(actor, rev, ix), Err(Error::StaleRevision));
    raw.apply(actor, a).unwrap();
    assert_eq!(raw.snapshot(), after);
    indexed
}
fn main(g: &mut Game, p: Player) {
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Main1;
    g.state.pending = Pending::Priority;
    g.state.passes = 0;
}
fn moved(g: &mut Game, ids: &[CardId], to: Zone) {
    let before = g.death_trigger_sources();
    let events = g.move_objects_raw(ids, to);
    g.finish_zone_events(&events, &before);
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..160 {
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
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::PutTrigger { .. }
                        | Action::ArrangeLibrary {
                            destination: LibraryDestination::Top,
                            ..
                        }
                )
            })
            .unwrap_or_else(|| panic!("stalled {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("hand state stalled {:?}", g.state.pending)
}

fn mana(g: &mut Game, p: Player, c: Card) {
    let (mut m, n) = c.cost();
    m[5] += n;
    g.state.players[p.index()].mana = m;
}

fn card(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let c = Card::from_name(name).unwrap();
    if c.is_supported() {
        return named(g, p, name, zone);
    } // Only the explicitly labelled Marketback metadata fixture remains gated.
    assert_eq!(name, "Marketback Walker");
    let id = named(g, p, "Hill Giant", Zone::Outside);
    g.state.objects[id as usize].card = c;
    g.move_object(id, zone);
    id
}
fn library(g: &mut Game, p: Player, names: &[&str]) -> Vec<CardId> {
    let ids = g.state.players[p.index()].library.to_vec();
    moved(g, &ids, Zone::Outside);
    names
        .iter()
        .rev()
        .map(|n| card(g, p, n, Zone::Library))
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}
fn pay(g: &mut Game) {
    for _ in 0..20 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
            .unwrap();
        checked(g, a);
    }
    panic!("payment")
}
fn quest(g: &mut Game, p: Player, x: u32) -> CardId {
    main(g, p);
    let id = card(g, p, QUEST, Zone::Hand);
    g.state.players[p.index()].mana = [0, 1, 0, 1, 0, x];
    checked(g, Action::AnnounceCast { card: id });
    checked(g, Action::ChooseX { value: x });
    pay(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn zones(g: &mut Game, library: bool, graveyard: bool) {
    until(g, |g| {
        matches!(g.state.pending, Pending::LibraryGraveyardSearch { .. })
    });
    checked(g, Action::ChooseSearchZones { library, graveyard });
}
fn take(g: &mut Game, id: CardId) {
    checked(g, Action::ChooseSearch { card: id });
    checked(g, Action::FinishSearch);
    settle(g);
}
fn selected(g: &Game, id: CardId) -> bool {
    g.legal_actions()
        .contains(&Action::ChooseSearch { card: id })
}
#[test]
fn msh_vision_quest_paid_x_domain_and_actual_colored_costs() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = card(&mut g, p, QUEST, Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 8];
        reject(&mut g, Action::AnnounceCast { card: id });
        g.state.players[p.index()].mana = [0, 1, 0, 1, 0, 4];
        checked(&mut g, Action::AnnounceCast { card: id });
        reject(&mut g, Action::ChooseX { value: 5 });
        g = orphan(g, Action::ChooseX { value: 4 });
        pay(&mut g);
        zones(&mut g, true, false);
        let v = g.observe(p).zone_search.unwrap();
        assert_eq!(
            (v.maximum_mana_value, v.counter_amount, v.haste),
            (4, 4, true)
        );
        checked(&mut g, Action::FinishSearch);
        settle(&mut g);
    }
}
#[test]
fn msh_vision_quest_library_entry_x_counters_haste_boundary_and_attack() {
    for p in [Player::P0, Player::P1] {
        for x in [2, 3, 4, 6] {
            let mut g = setup(p);
            let b = library(&mut g, p, &[BOT])[0];
            quest(&mut g, p, x);
            zones(&mut g, true, false);
            take(&mut g, b);
            assert_eq!(g.counter_count(b, CounterKind::PlusOne), u64::from(x));
            assert_eq!(g.object_view(b).power, 1 + i128::from(x));
            assert_eq!(g.keywords(b).contains(Keywords::HASTE), x >= 4);
            assert!(g.state.objects[b as usize].summoning_sick);
            g.begin_step(Step::BeginCombat);
            until(&mut g, |g| g.state.pending == Pending::Attackers);
            if x >= 4 {
                checked(&mut g, Action::Attack { card: b });
                checked(&mut g, Action::FinishAttackers);
                until(&mut g, |g| g.state.pending == Pending::Blockers);
                checked(&mut g, Action::FinishBlockers);
                until(&mut g, |g| g.state.step == Step::EndCombat);
                assert_eq!(
                    g.state.players[p.other().index()].life,
                    20 - (1 + i128::from(x))
                );
            } else {
                reject(&mut g, Action::Attack { card: b });
                checked(&mut g, Action::FinishAttackers);
            }
            next_turn(&mut g);
            assert!(!g.keywords(b).contains(Keywords::HASTE));
        }
    }
}
#[test]
fn msh_vision_quest_artifact_and_creature_both_required_and_mana_value_inclusive() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ids = library(&mut g, p, &[BOT, "Bear Cub", "Arc Reactor", "The Vision"]);
        quest(&mut g, p, 2);
        zones(&mut g, true, false);
        assert!(selected(&g, ids[0]));
        for &id in &ids[1..] {
            reject(&mut g, Action::ChooseSearch { card: id });
        }
        take(&mut g, ids[0]);
    }
}
#[test]
fn msh_vision_quest_graveyard_only_keeps_entire_library_rng_and_known_top() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ids = library(&mut g, p, &["Forest", "Shock", "Mountain"]);
        g.remember_library_top(p, g.object_ref(ids[0]));
        let before_rng = g.state.rng;
        let b = named(&mut g, p, BOT, Zone::Graveyard);
        quest(&mut g, p, 4);
        zones(&mut g, false, true);
        let lib = g.state.players[p.index()].library.to_vec();
        checked(&mut g, Action::ChooseSearch { card: b });
        checked(&mut g, Action::FinishSearch);
        // Before the separate Synthoids surveil trigger resolves.
        assert_eq!(g.state.players[p.index()].library.to_vec(), lib);
        assert_eq!(g.state.rng, before_rng);
        assert_eq!(g.public_library_top(p), Some(g.object_ref(ids[0])));
        assert_eq!(g.counter_count(b, CounterKind::PlusOne), 4);
        settle(&mut g);
    }
}
#[test]
fn msh_vision_quest_three_nonempty_zone_subsets_and_public_grave_required() {
    for p in [Player::P0, Player::P1] {
        for (library, graveyard) in [(true, false), (false, true), (true, true)] {
            let mut g = setup(p);
            let b = named(&mut g, p, BOT, Zone::Graveyard);
            let l = library_fixture(&mut g, p);
            quest(&mut g, p, 2);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::LibraryGraveyardSearch { .. })
            });
            reject(
                &mut g,
                Action::ChooseSearchZones {
                    library: false,
                    graveyard: false,
                },
            );
            checked(&mut g, Action::ChooseSearchZones { library, graveyard });
            assert_eq!(selected(&g, b), graveyard);
            assert_eq!(selected(&g, l), library);
            if graveyard {
                reject(&mut g, Action::FinishSearch);
                take(&mut g, b);
            } else {
                checked(&mut g, Action::FinishSearch);
                settle(&mut g);
                assert_eq!(g.object_view(b).zone, Zone::Graveyard);
            }
        }
    }
}
fn library_fixture(g: &mut Game, p: Player) -> CardId {
    library(g, p, &[BOT, "Forest"])[0]
}
#[test]
fn msh_vision_quest_zero_and_empty_grave_search_no_illegal_no_zone_decline() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let b = named(&mut g, p, BOT, Zone::Graveyard);
        let lib = g.state.players[p.index()].library.to_vec();
        let rng = g.state.rng;
        quest(&mut g, p, 0);
        zones(&mut g, false, true);
        assert!(!selected(&g, b));
        checked(&mut g, Action::FinishSearch);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.to_vec(), lib);
        assert_eq!(g.state.rng, rng);
        assert_eq!(g.object_view(b).zone, Zone::Graveyard);
    }
}
#[test]
fn msh_vision_quest_library_fail_to_find_still_shuffles_when_grave_has_card() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ids = library(&mut g, p, &["Forest", "Shock", "Mountain", "Island"]);
        let b = named(&mut g, p, BOT, Zone::Graveyard);
        g.remember_library_top(p, g.object_ref(ids[0]));
        let rng = g.state.rng;
        quest(&mut g, p, 2);
        zones(&mut g, true, false);
        checked(&mut g, Action::FinishSearch);
        settle(&mut g);
        assert_ne!(g.state.rng, rng);
        assert_eq!(g.public_library_top(p), None);
        assert_eq!(g.object_view(b).zone, Zone::Graveyard);
    }
}
#[test]
fn msh_vision_quest_private_library_both_views_counterfactual_and_orphan_choices() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ids = library(&mut g, p, &[BOT, "Shock"]);
        quest(&mut g, p, 4);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::LibraryGraveyardSearch { .. })
        });
        g = orphan(
            g,
            Action::ChooseSearchZones {
                library: true,
                graveyard: true,
            },
        );
        assert!(g.observe(p.other()).zone_search.unwrap().cards.is_none());
        let mut hidden = g.clone();
        hidden.state.objects[ids[1] as usize].card = Card::from_name("Giant Growth").unwrap();
        assert_eq!(
            serde_json::to_value(g.observe(p.other())).unwrap(),
            serde_json::to_value(hidden.observe(p.other())).unwrap()
        );
        g = orphan(g, Action::ChooseSearch { card: ids[0] });
        assert!(g.observe(p.other()).zone_search.unwrap().selected.is_none());
        g = orphan(g, Action::FinishSearch);
        settle(&mut g);
        assert!(g.on_battlefield(ids[0]));
    }
}
#[test]
fn msh_vision_quest_wrong_owner_wrong_zone_and_stale_selected_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let b = named(&mut g, p, BOT, Zone::Graveyard);
        let wrong = named(&mut g, p.other(), BOT, Zone::Graveyard);
        let hand = named(&mut g, p, BOT, Zone::Hand);
        quest(&mut g, p, 4);
        zones(&mut g, false, true);
        for id in [wrong, hand] {
            reject(&mut g, Action::ChooseSearch { card: id });
        }
        checked(&mut g, Action::ChooseSearch { card: b });
        let old = g.object_ref(b);
        moved(&mut g, &[b], Zone::Exile);
        moved(&mut g, &[b], Zone::Graveyard);
        assert_ne!(g.object_ref(b), old);
        reject(&mut g, Action::FinishSearch);
        take(&mut g, b);
        assert!(g.on_battlefield(b));
    }
}
#[test]
fn msh_vision_quest_entry_choice_is_owned_private_then_counters_and_haste() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let b = library(&mut g, p, &["Adaptive Automaton"])[0];
        quest(&mut g, p, 4);
        zones(&mut g, true, false);
        checked(&mut g, Action::ChooseSearch { card: b });
        checked(&mut g, Action::FinishSearch);
        assert!(matches!(g.state.pending, Pending::EntryValue { .. }));
        assert_eq!(g.object_view(b).zone, Zone::Library);
        assert!(g.observe(p).entry_value.is_some());
        assert!(g.observe(p.other()).entry_value.is_none());
        g = orphan(
            g,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::CreatureType(CreatureType::from_name("Hero").unwrap()),
            },
        );
        settle(&mut g);
        assert_eq!(g.counter_count(b, CounterKind::PlusOne), 4);
        assert_eq!((g.stats(b).power, g.stats(b).toughness), (6, 6));
        assert!(g.keywords(b).contains(Keywords::HASTE));
    }
}
#[test]
fn msh_vision_quest_entry_counter_replacements_do_not_change_haste_x_threshold() {
    for p in [Player::P0, Player::P1] {
        for x in [3, 4] {
            let mut g = setup(p);
            named(&mut g, p, "Doubling Season", Zone::Battlefield);
            let b = named(&mut g, p, BOT, Zone::Graveyard);
            quest(&mut g, p, x);
            zones(&mut g, false, true);
            take(&mut g, b);
            assert_eq!(g.counter_count(b, CounterKind::PlusOne), 2 * u64::from(x));
            assert_eq!(g.keywords(b).contains(Keywords::HASTE), x == 4);
        }
    }
}
#[test]
fn msh_vision_quest_entry_counter_replacement_pause_and_orphan_restore() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        named(
            &mut g,
            p,
            "Doc Samson, Super Psychiatrist",
            Zone::Battlefield,
        );
        let b = named(&mut g, p, BOT, Zone::Graveyard);
        quest(&mut g, p, 4);
        zones(&mut g, false, true);
        checked(&mut g, Action::ChooseSearch { card: b });
        checked(&mut g, Action::FinishSearch);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Replacement { .. })
        });
        assert_eq!(g.object_view(b).zone, Zone::Graveyard);
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseReplacement { .. }))
            .unwrap();
        g = orphan(g, a);
        settle(&mut g);
        assert!([9, 10].contains(&g.counter_count(b, CounterKind::PlusOne)));
        assert!(g.keywords(b).contains(Keywords::HASTE));
    }
}
#[test]
fn msh_vision_quest_haste_grant_timestamp_loses_to_later_ability_removal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let b = named(&mut g, p, BOT, Zone::Graveyard);
        quest(&mut g, p, 4);
        zones(&mut g, false, true);
        take(&mut g, b);
        assert!(g.keywords(b).contains(Keywords::HASTE));
        main(&mut g, p);
        let witness = named(&mut g, p, "Witness Protection", Zone::Hand);
        mana(&mut g, p, Card::from_name("Witness Protection").unwrap());
        cast(&mut g, witness, None, &[Target::Card(b)]);
        settle(&mut g);
        assert!(!g.keywords(b).contains(Keywords::HASTE));
    }
}
#[test]
fn msh_vision_quest_exact_entry_haste_does_not_follow_blink() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let b = named(&mut g, p, BOT, Zone::Graveyard);
        quest(&mut g, p, 4);
        zones(&mut g, false, true);
        take(&mut g, b);
        let old = g.object_ref(b);
        moved(&mut g, &[b], Zone::Exile);
        moved(&mut g, &[b], Zone::Battlefield);
        assert_ne!(g.object_ref(b), old);
        assert_eq!(g.counter_count(b, CounterKind::PlusOne), 0);
        assert!(!g.keywords(b).contains(Keywords::HASTE));
    }
}
#[test]
fn msh_vision_quest_opponent_entry_tapped_replacement_and_counters() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(
            &mut g,
            p.other(),
            "Authority of the Consuls",
            Zone::Battlefield,
        );
        let b = named(&mut g, p, BOT, Zone::Graveyard);
        quest(&mut g, p, 4);
        zones(&mut g, false, true);
        take(&mut g, b);
        assert!(g.object_view(b).tapped);
        assert_eq!(g.counter_count(b, CounterKind::PlusOne), 4);
        assert!(g.keywords(b).contains(Keywords::HASTE));
        assert_eq!(g.state.players[p.other().index()].life, 21);
    }
}
#[test]
fn msh_vision_quest_actual_paid_spell_copy_keeps_x_and_independent_search() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = named(&mut g, p, BOT, Zone::Graveyard);
        let b = named(&mut g, p, BOT, Zone::Graveyard);
        quest(&mut g, p, 4);
        let packet = g.state.stack.last().unwrap().card;
        let copy = named(&mut g, p, "Choreographed Sparks", Zone::Hand);
        mana(&mut g, p, Card::from_name("Choreographed Sparks").unwrap());
        cast(&mut g, copy, Some(0), &[Target::Spell(packet)]);
        zones(&mut g, false, true);
        take_until_next(&mut g, a);
        zones(&mut g, false, true);
        take(&mut g, b);
        for id in [a, b] {
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), 4);
            assert!(g.keywords(id).contains(Keywords::HASTE));
        }
    }
}
fn take_until_next(g: &mut Game, id: CardId) {
    checked(g, Action::ChooseSearch { card: id });
    checked(g, Action::FinishSearch);
}
#[test]
fn msh_vision_quest_metadata_x_card_mana_value_zero_without_admitting_its_behavior() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let b = card(&mut g, p, "Marketback Walker", Zone::Graveyard);
        assert!(!g.object_view(b).card.is_supported());
        quest(&mut g, p, 0);
        zones(&mut g, true, false);
        assert!(!selected(&g, b)); // GY was not chosen.
        // Metadata-only presence in the library: inspect eligibility, never resolve this gated card.
        moved(&mut g, &[b], Zone::Library);
        assert!(selected(&g, b));
        checked(&mut g, Action::FinishSearch);
        settle(&mut g);
        assert_eq!(g.object_view(b).zone, Zone::Library);
    }
}
#[test]
fn msh_vision_quest_controlled_player_search_entitlement_and_semantic_owner() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.schedule_player_control(p.other(), p);
        g.state.active = p.other();
        g.begin_turn();
        main(&mut g, p.other());
        let b = library(&mut g, p.other(), &[BOT])[0];
        quest(&mut g, p.other(), 4);
        zones(&mut g, true, false);
        assert_eq!(g.actor(), Some(p));
        for v in [p, p.other()] {
            assert!(g.observe(v).zone_search.unwrap().cards.is_some());
        }
        take(&mut g, b);
        assert_eq!(g.object_view(b).controller, p.other());
        assert_eq!(g.counter_count(b, CounterKind::PlusOne), 4);
    }
}
#[test]
fn msh_vision_quest_failed_entry_completion_never_grants_haste_or_counters() {
    for p in [Player::P0, Player::P1] {
        for skipped in [false, true] {
            let mut g = setup(p);
            let b = named(&mut g, p, BOT, Zone::Hand);
            let source = named(&mut g, p, "Shock", Zone::Graveyard);
            let mut m = g.entry_move(
                b,
                if skipped {
                    Zone::Battlefield
                } else {
                    Zone::Exile
                },
                false,
            );
            m.skipped = skipped;
            m.effect_counter = Some((CounterKind::PlusOne, 4));
            m.keywords_after = Some((g.object_ref(source), Keywords::HASTE));
            g.stage_entries(vec![m], None);
            assert!(!g.advance_entries());
            assert!(!g.on_battlefield(b));
            assert_eq!(g.counter_count(b, CounterKind::PlusOne), 0);
            assert!(!g.keywords(b).contains(Keywords::HASTE));
        }
    }
}
