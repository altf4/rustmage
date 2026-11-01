//! Whole printed quest/search/cycling outcomes; hidden-zone choices remain owned.
use super::*;
const FIXTURES: &[&str] = &[
    "Last Light of Durin's Day",
    "Mountain",
    "Forest",
    "Shivan Dragon",
    "Barkform Harvester",
    "Doubling Season",
    "Rest in Peace",
    "Shock",
    "Bear Cub",
    "Secluded Courtyard",
    "Prophetic Prism",
];
fn fixture(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    assert!(FIXTURES.contains(&name), "unlisted fixture {name}");
    put(g, p, name, zone)
}
fn quiet(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.pending_triggers.is_empty()
        && g.state.resolving.is_none()
        && g.state.ordering_triggers.is_empty()
}
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..48 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("last-light action budget: {:?}", g.state.pending);
}
fn finish(g: &mut Game) {
    until(g, quiet);
}
fn search(g: &mut Game) {
    until(g, |g| {
        matches!(g.state.pending, Pending::HandLibrarySearch { .. })
    });
}
fn paid(g: &mut Game, p: Player) -> CardId {
    let id = fixture(g, p, "Last Light of Durin's Day", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 1];
    cast(g, id, None);
    finish(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert_eq!(g.object_view(id).zone, Zone::Battlefield);
    id
}
fn land(g: &mut Game, owner: Player, controller: Player, name: &str) -> CardId {
    let id = fixture(g, owner, name, Zone::Hand);
    let mut movement = g.entry_move(id, Zone::Battlefield, false);
    movement.controller = controller;
    g.stage_entries(vec![movement], None);
    g.prepare_triggers();
    id
}
fn ready(p: Player) -> (Game, CardId) {
    let mut g = scenario(p);
    let source = paid(&mut g, p);
    g.add_counter(source, CounterKind::Quest, 5);
    land(&mut g, p, p, "Mountain");
    search(&mut g);
    (g, source)
}
#[test]
fn last_light_paid_quest_trigger_uses_actual_subtype_control_and_resolution_threshold() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = paid(&mut g, p);
        land(&mut g, p, p, "Forest");
        finish(&mut g);
        land(&mut g, p, p.other(), "Mountain");
        finish(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::Quest), 0);
        land(&mut g, p.other(), p, "Mountain");
        assert_eq!(g.counter_count(source, CounterKind::Quest), 0); // event does not put counter
        finish(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::Quest), 1);
        g.add_counter(source, CounterKind::Quest, 4);
        land(&mut g, p, p, "Mountain");
        g.pay_counters(source, &Counters::of(CounterKind::Quest, 1));
        finish(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::Quest), 5);
        land(&mut g, p, p, "Mountain");
        search(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        act(
            &mut g,
            Action::ChooseHandLibrarySearchZones {
                hand: true,
                library: false,
            },
        );
        act(&mut g, Action::FinishSearch);
        finish(&mut g);
    }
}
#[test]
fn last_light_hidden_zone_modes_find_zero_or_dragon_and_shuffle_only_searched_library() {
    for p in [Player::P0, Player::P1] {
        for (hand, library) in [(true, false), (false, true), (true, true)] {
            for find in [false, true] {
                let (mut g, source) = ready(p);
                let h = fixture(&mut g, p, "Shivan Dragon", Zone::Hand);
                let l = fixture(&mut g, p, "Shivan Dragon", Zone::Library);
                let other = fixture(&mut g, p.other(), "Shivan Dragon", Zone::Hand);
                let bear = fixture(&mut g, p, "Bear Cub", Zone::Hand);
                let old_rng = format!("{:?}", g.state.rng);
                let old_library = g.state.players[p.index()].library.clone();
                act(
                    &mut g,
                    Action::ChooseHandLibrarySearchZones { hand, library },
                );
                reject(&mut g, Action::ChooseSearch { card: other });
                reject(&mut g, Action::ChooseSearch { card: bear });
                assert_eq!(
                    g.legal_actions()
                        .contains(&Action::ChooseSearch { card: h }),
                    hand
                );
                assert_eq!(
                    g.legal_actions()
                        .contains(&Action::ChooseSearch { card: l }),
                    library
                );
                assert!(
                    g.observe(p.other())
                        .hand_library_search
                        .as_ref()
                        .unwrap()
                        .cards
                        .is_none()
                );
                assert!(
                    g.observe(p.other())
                        .objects
                        .iter()
                        .all(|o| o.id != h && o.id != l)
                );
                let chosen = if hand { h } else { l };
                if find {
                    act(&mut g, Action::ChooseSearch { card: chosen });
                }
                act(&mut g, Action::FinishSearch);
                finish(&mut g);
                assert_eq!(g.object_view(source).zone, Zone::Graveyard);
                assert_eq!(
                    g.object_view(chosen).zone,
                    if find {
                        Zone::Battlefield
                    } else if hand {
                        Zone::Hand
                    } else {
                        Zone::Library
                    }
                );
                assert_eq!(old_rng == format!("{:?}", g.state.rng), !library);
                if !library {
                    assert_eq!(old_library, g.state.players[p.index()].library);
                }
            }
        }
    }
}
#[test]
fn last_light_changeling_is_dragon_card_and_search_is_not_cast_or_creature_only() {
    let (mut g, _) = ready(Player::P0);
    let changeling = fixture(&mut g, Player::P0, "Barkform Harvester", Zone::Library);
    let program = Card::from_name("Last Light of Durin's Day")
        .unwrap()
        .triggers()[0]
        .program;
    assert!(matches!(
        program.last(),
        Some(Instruction::SearchHandAndOrLibrary(
            crate::SearchFilter::Subtype("Dragon")
        ))
    ));
    act(
        &mut g,
        Action::ChooseHandLibrarySearchZones {
            hand: false,
            library: true,
        },
    );
    act(&mut g, Action::ChooseSearch { card: changeling });
    act(&mut g, Action::FinishSearch);
    finish(&mut g);
    assert_eq!(g.object_view(changeling).zone, Zone::Battlefield);
    assert_eq!(stats(&g, changeling), (2, 3));
    assert_eq!(g.state.players[0].mana, [0; 6]);
}
#[test]
fn last_light_counter_replacement_and_sacrifice_replacement_still_search() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = paid(&mut g, p);
        fixture(&mut g, p, "Doubling Season", Zone::Battlefield);
        fixture(&mut g, p, "Rest in Peace", Zone::Battlefield);
        g.add_counter(source, CounterKind::Quest, 4);
        land(&mut g, p, p, "Mountain");
        search(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Exile);
        act(
            &mut g,
            Action::ChooseHandLibrarySearchZones {
                hand: true,
                library: false,
            },
        );
        act(&mut g, Action::FinishSearch);
        finish(&mut g);
    }
}
#[test]
fn last_light_source_departure_reentry_and_control_loss_prevent_sacrifice_payoff() {
    for mode in 0..3 {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = paid(&mut g, p);
        g.add_counter(source, CounterKind::Quest, 5);
        land(&mut g, p, p, "Mountain");
        if mode == 0 {
            g.move_object(source, Zone::Graveyard);
        } else if mode == 1 {
            g.move_object(source, Zone::Hand);
            g.move_object(source, Zone::Battlefield);
            g.add_counter(source, CounterKind::Quest, 6);
        } else {
            g.record_control(g.object_ref(source), p.other(), None);
            g.refresh_controllers();
        }
        finish(&mut g);
        assert!(g.observe(p).hand_library_search.is_none());
        if mode == 1 {
            assert_eq!(g.counter_count(source, CounterKind::Quest), 6);
        }
        if mode == 2 {
            assert_eq!(g.object_view(source).zone, Zone::Battlefield);
            assert_eq!(g.counter_count(source, CounterKind::Quest), 6);
        }
    }
}
#[test]
fn last_light_owned_zone_and_selection_branches_privacy_stale_and_restore() {
    let (mut g, _) = ready(Player::P1);
    let h = fixture(&mut g, Player::P1, "Shivan Dragon", Zone::Hand);
    let l = fixture(&mut g, Player::P1, "Shivan Dragon", Zone::Library);
    contracts::branch_contract(g.clone());
    reject(
        &mut g,
        Action::ChooseHandLibrarySearchZones {
            hand: false,
            library: false,
        },
    );
    act(
        &mut g,
        Action::ChooseHandLibrarySearchZones {
            hand: true,
            library: true,
        },
    );
    contracts::branch_contract(g.clone());
    act(&mut g, Action::ChooseSearch { card: h });
    assert!(
        g.observe(Player::P0)
            .hand_library_search
            .as_ref()
            .unwrap()
            .selected
            .is_none()
    );
    contracts::branch_contract(g.clone());
    let saved = g.snapshot();
    let mut child = g.clone();
    drop(g);
    act(&mut child, Action::ChooseSearch { card: l });
    act(&mut child, Action::FinishSearch);
    finish(&mut child);
    assert_eq!(child.object_view(l).zone, Zone::Battlefield);
    assert_eq!(child.object_view(h).zone, Zone::Hand);
    child.restore(&saved).unwrap();
    child.move_object(h, Zone::Graveyard);
    child.move_object(h, Zone::Hand);
    act(&mut child, Action::FinishSearch);
    finish(&mut child);
    assert_eq!(child.object_view(h).zone, Zone::Hand);
    assert_eq!(child.object_view(l).zone, Zone::Library);
}
#[test]
fn last_light_mountaincycling_paid_discard_private_search_reveal_shuffle_and_no_quest() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = fixture(&mut g, p, "Last Light of Durin's Day", Zone::Hand);
        let mountain = fixture(&mut g, p, "Mountain", Zone::Library);
        let forest = fixture(&mut g, p, "Forest", Zone::Library);
        g.state.players[p.index()].mana = [0; 6];
        let action = Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        };
        reject(&mut g, action.clone());
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        act(&mut g, action);
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectSearch { .. })
        });
        assert!(g.observe(p.other()).search_choice.is_none());
        reject(&mut g, Action::ChooseSearch { card: forest });
        let old = format!("{:?}", g.state.rng);
        act(&mut g, Action::ChooseSearch { card: mountain });
        act(&mut g, Action::FinishSearch);
        finish(&mut g);
        assert_eq!(g.object_view(mountain).zone, Zone::Hand);
        assert!(
            g.observe(p.other())
                .objects
                .iter()
                .any(|o| o.id == mountain)
        );
        assert_ne!(old, format!("{:?}", g.state.rng));
        assert_eq!(g.counter_count(source, CounterKind::Quest), 0);
    }
}
#[test]
fn last_light_simultaneous_token_mountains_make_distinct_triggers_without_second_search() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = paid(&mut g, p);
    g.add_counter(source, CounterKind::Quest, 5);
    let tokens = g.create_copy_groups_inner(
        p,
        &[TokenGroupView {
            token: Card::Mountain,
            amount: 2,
            tapped: false,
        }],
        &[Some(CopyRecipe::new(Card::Mountain))],
        None,
    );
    assert_eq!(tokens.len(), 2);
    g.prepare_triggers();
    search(&mut g);
    act(
        &mut g,
        Action::ChooseHandLibrarySearchZones {
            hand: true,
            library: false,
        },
    );
    act(&mut g, Action::FinishSearch);
    finish(&mut g);
    assert!(tokens.iter().all(|r| g.object_view(r.card).is_token()));
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    assert!(g.observe(p).hand_library_search.is_none());
}
#[test]
fn last_light_counter_capacity_failure_is_transactional_and_named_wire_is_canonical() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = paid(&mut g, p);
    g.set_counters(source, Counters::of(CounterKind::Quest, u64::MAX));
    land(&mut g, p, p, "Mountain");
    until(&mut g, |g| {
        g.state.stack.len() == 1 && g.state.pending == Pending::Priority
    });
    act(&mut g, Action::Pass);
    let saved = g.snapshot();
    let revision = g.revision();
    let actor = g.actor().unwrap();
    let views = [
        serde_json::to_value(g.observe(p)).unwrap(),
        serde_json::to_value(g.observe(p.other())).unwrap(),
    ];
    assert!(matches!(
        g.apply(actor, Action::Pass),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), saved);
    assert_eq!(g.revision(), revision);
    assert_eq!(serde_json::to_value(g.observe(p)).unwrap(), views[0]);
    assert_eq!(
        serde_json::to_value(g.observe(p.other())).unwrap(),
        views[1]
    );
    assert_eq!(g.counter_count(source, CounterKind::Quest), u64::MAX);
    let wire = serde_json::to_value(Counters::of(CounterKind::Quest, 6)).unwrap();
    assert_eq!(wire["named"], serde_json::json!([["Quest", 6]]));
    assert_eq!(
        serde_json::from_value::<Counters>(wire)
            .unwrap()
            .get(CounterKind::Quest),
        6
    );
}
#[test]
fn last_light_shared_hidden_search_preserves_noncreature_cards_and_suspended_entry_shuffle() {
    // The frozen admitted pool has no noncreature Dragon. This generic helper
    // root proves the continuation has no hidden creature restriction; the card's
    // immutable program separately proves its exact Dragon-only predicate.
    for p in [Player::P0, Player::P1] {
        for name in ["Prophetic Prism", "Shock"] {
            let (mut g, _) = ready(p);
            let chosen = fixture(&mut g, p, name, Zone::Hand);
            g.start_hand_library_search(p, crate::SearchFilter::Any);
            act(
                &mut g,
                Action::ChooseHandLibrarySearchZones {
                    hand: true,
                    library: false,
                },
            );
            act(&mut g, Action::ChooseSearch { card: chosen });
            act(&mut g, Action::FinishSearch);
            finish(&mut g);
            assert_eq!(
                g.object_view(chosen).zone,
                if name == "Shock" {
                    Zone::Hand
                } else {
                    Zone::Battlefield
                }
            );
            if name == "Shock" {
                assert!(g.observe(p.other()).objects.iter().all(|o| o.id != chosen));
            }
        }
        let (mut g, _) = ready(p);
        let land = fixture(&mut g, p, "Secluded Courtyard", Zone::Library);
        g.start_hand_library_search(p, crate::SearchFilter::Any);
        act(
            &mut g,
            Action::ChooseHandLibrarySearchZones {
                hand: false,
                library: true,
            },
        );
        act(&mut g, Action::ChooseSearch { card: land });
        let rng = g.state.rng;
        act(&mut g, Action::FinishSearch);
        assert!(g.observe(p).entry_value.is_some());
        assert_eq!(g.state.rng, rng);
        contracts::branch_contract(g.clone());
        let saved = g.snapshot();
        let mut child = g.clone();
        drop(g);
        let action = Action::ChooseEntryValue {
            value: EntryChoiceValue::CreatureType(
                crate::cards::CreatureType::from_name("Dragon").unwrap(),
            ),
        };
        act(&mut child, action.clone());
        finish(&mut child);
        assert_eq!(child.object_view(land).zone, Zone::Battlefield);
        assert_ne!(child.state.rng, rng);
        let expected = child.snapshot();
        child.restore(&saved).unwrap();
        act(&mut child, action);
        finish(&mut child);
        assert_eq!(child.snapshot(), expected);
    }
}
#[test]
fn last_light_cycling_in_opponents_combat_can_fail_find_and_discard_replacement_is_paid() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        fixture(&mut g, p, "Rest in Peace", Zone::Battlefield);
        let source = fixture(&mut g, p, "Last Light of Durin's Day", Zone::Hand);
        fixture(&mut g, p, "Mountain", Zone::Library);
        g.state.active = p.other();
        g.state.priority = p;
        g.state.step = Step::BeginCombat;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        act(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Exile);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectSearch { .. })
        });
        let rng = g.state.rng;
        let count = g.hand_count(p);
        act(&mut g, Action::FinishSearch);
        finish(&mut g);
        assert_ne!(g.state.rng, rng);
        assert_eq!(g.hand_count(p), count);
    }
}
