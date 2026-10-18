use super::ability_timeline_tests::{checked, setup};
use super::tests::spawn;
use super::*;

fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    spawn(g, p, Card::from_name(name).unwrap(), zone)
}
fn settle(g: &mut Game, mode: u16, target: Option<CardId>) {
    g.prepare_triggers();
    for _ in 0..150 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.announcing.is_none()
            && g.state.resolving.is_none()
        {
            return;
        }
        let actions = g.legal_actions();
        let chosen = target.map(|card| Action::ChooseTarget {
            target: Target::Card(card),
        });
        let action = if actions.contains(&Action::ChooseMode { mode }) {
            Action::ChooseMode { mode }
        } else if let Some(a) = chosen.filter(|a| actions.contains(a)) {
            a
        } else if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            actions
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .or_else(|| {
                    g.legal_actions()
                        .into_iter()
                        .find(|a| matches!(a, Action::FinishTargets))
                })
                .or_else(|| {
                    g.legal_actions()
                        .into_iter()
                        .find(|a| !matches!(a, Action::Concede))
                })
                .unwrap()
        };
        checked(g, action);
    }
    panic!("unsettled {:?}", g.state.pending);
}
fn cast(g: &mut Game, id: CardId, _mode: Option<u16>, targets: &[Target]) {
    checked(g, Action::AnnounceCast { card: id });
    for &target in targets {
        checked(g, Action::ChooseTarget { target });
    }
    for _ in 0..10 {
        if g.state
            .announcing
            .as_ref()
            .is_none_or(|a| a.item.kind != StackKind::Spell || a.item.card != id)
        {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets))
            .or_else(|| {
                g.legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::PayCosts { .. }))
            })
            .unwrap();
        checked(g, a);
    }
    panic!("spell announcement did not complete");
}
fn paid(g: &mut Game, id: CardId) {
    cast(g, id, None, &[]);
    settle(g, 0, None);
}
fn end(g: &mut Game) {
    g.begin_step(Step::End);
    settle(g, 0, None);
}
fn kill(g: &mut Game, p: Player, id: CardId) {
    let murder = named(g, p, "Murder", Zone::Hand);
    cast(g, murder, None, &[Target::Card(id)]);
    settle(g, 0, None);
}
#[test]
fn return_links_vessel_paid_death_counter_replacement_owner_and_no_second_return() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        let vessel = named(&mut g, p, "Infernal Vessel", Zone::Hand);
        paid(&mut g, vessel);
        let old = g.object_ref(vessel);
        kill(&mut g, p, vessel);
        assert_ne!(g.object_ref(vessel), old);
        assert!(g.on_battlefield(vessel));
        assert_eq!(g.counter_count(vessel, CounterKind::PlusOne), 4);
        assert!(g.has_subtype(vessel, "Demon"));
        assert!(g.has_subtype(vessel, "Human"));
        assert!(g.has_subtype(vessel, "Cleric"));
        kill(&mut g, p, vessel);
        assert_eq!(g.state.objects[vessel as usize].zone, Zone::Graveyard);
        assert!(!g.has_subtype(vessel, "Demon"));
    }
}
#[test]
fn return_links_familiar_cast_only_counters_delayed_decrement_and_captured_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let familiar = named(&mut g, p, "Nine-Lives Familiar", Zone::Hand);
        paid(&mut g, familiar);
        assert_eq!(g.counter_count(familiar, CounterKind::Revival), 8);
        g.state.objects[familiar as usize].controller = p.other();
        kill(&mut g, p, familiar);
        assert_eq!(g.state.objects[familiar as usize].zone, Zone::Graveyard);
        let captured = g.observe(p).delayed_returns;
        assert_eq!(captured.len(), 1);
        assert_eq!(captured[0].returning.controller, p.other());
        let saved = g.snapshot();
        let mut owned = {
            let parent = g.clone();
            parent.clone()
        };
        end(&mut g);
        end(&mut owned);
        assert_eq!(g.snapshot(), owned.snapshot());
        assert!(g.on_battlefield(familiar));
        assert_eq!(g.state.objects[familiar as usize].controller, p.other());
        assert_eq!(g.counter_count(familiar, CounterKind::Revival), 7);
        let after = g.snapshot();
        g.restore(&saved).unwrap();
        end(&mut g);
        assert_eq!(after, g.snapshot());
        g.move_object(familiar, Zone::Hand);
        let movement = g.entry_move(familiar, Zone::Battlefield, false);
        g.stage_entries(vec![movement], None);
        settle(&mut g, 0, None);
        assert_eq!(g.counter_count(familiar, CounterKind::Revival), 0);
        kill(&mut g, p, familiar);
        assert!(g.observe(p).delayed_returns.is_empty());
    }
}
#[test]
fn return_links_familiar_stale_graveyard_and_end_step_created_delay_waits() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let cat = named(&mut g, p, "Nine-Lives Familiar", Zone::Hand);
        paid(&mut g, cat);
        g.begin_step(Step::End);
        kill(&mut g, p, cat);
        assert_eq!(g.observe(p).delayed_returns.len(), 1);
        let dead = g.object_ref(cat);
        g.move_object(cat, Zone::Hand);
        g.move_object(cat, Zone::Graveyard);
        assert_ne!(dead, g.object_ref(cat));
        end(&mut g);
        assert_eq!(g.state.objects[cat as usize].zone, Zone::Graveyard);
        assert!(g.observe(p).delayed_returns.is_empty());
    }
}
#[test]
fn return_links_call_entry_angel_visible_to_giada_and_flying_is_ordered_persistent() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let call = named(&mut g, p, "Valkyrie's Call", Zone::Hand);
        paid(&mut g, call);
        named(&mut g, p, "Giada, Font of Hope", Zone::Battlefield);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        let bear = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        kill(&mut g, p, bear);
        assert!(g.on_battlefield(bear));
        assert!(g.has_subtype(bear, "Angel"));
        assert!(g.has_subtype(bear, "Bear"));
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 4);
        assert!(g.keywords(bear).contains(Keywords::FLYING));
        g.finish_cleanup();
        assert!(g.keywords(bear).contains(Keywords::FLYING));
        g.state.players[p.index()].mana = [30; 6];
        g.state.step = Step::Main1;
        g.state.active = p;
        g.state.priority = p;
        g.state.pending = Pending::Priority;
        let aura = named(&mut g, p, "Witness Protection", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(bear)]);
        settle(&mut g, 0, None);
        assert!(!g.keywords(bear).contains(Keywords::FLYING));
        assert!(!g.has_subtype(bear, "Angel"));
        g.move_object(aura, Zone::Graveyard);
        assert!(g.keywords(bear).contains(Keywords::FLYING));
        assert!(g.has_subtype(bear, "Angel"));
        kill(&mut g, p, bear);
        assert_eq!(g.state.objects[bear as usize].zone, Zone::Graveyard);
    }
}
#[test]
fn return_links_call_simultaneous_source_departure_and_owner_return() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let call = named(&mut g, p, "Valkyrie's Call", Zone::Battlefield);
        let bear = spawn(&mut g, p.other(), Card::GrizzlyBears, Zone::Battlefield);
        g.state.objects[bear as usize].controller = p;
        g.sacrifice_batch(&[(p, call), (p, bear)]);
        settle(&mut g, 0, None);
        assert!(g.on_battlefield(bear));
        assert_eq!(g.state.objects[bear as usize].controller, p.other());
        assert!(g.has_subtype(bear, "Angel"));
    }
}
#[test]
fn return_links_prince_all_modes_owned_target_and_source_independent_delay() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = setup(p);
            for _ in 0..3 {
                spawn(&mut g, p, Card::Forest, Zone::Library);
            }
            let bear = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
            g.state.objects[bear as usize].controller = p.other();
            let prince = named(&mut g, p, "Charming Prince", Zone::Hand);
            cast(&mut g, prince, None, &[]);
            settle(&mut g, mode, Some(bear));
            if mode == 1 {
                assert_eq!(g.state.players[p.index()].life, 23);
            }
            if mode == 2 {
                assert_eq!(g.state.objects[bear as usize].zone, Zone::Exile);
                kill(&mut g, p, prince);
                end(&mut g);
                assert!(g.on_battlefield(bear));
                assert_eq!(g.state.objects[bear as usize].controller, p);
            }
        }
    }
}
#[test]
fn return_links_kykar_noncreature_modes_flying_spirit_and_owner_return() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..2 {
            let mut g = setup(p);
            let kykar = named(&mut g, p, "Kykar, Zephyr Awakener", Zone::Hand);
            paid(&mut g, kykar);
            assert!(g.keywords(kykar).contains(Keywords::FLYING));
            let bear = spawn(&mut g, p.other(), Card::GrizzlyBears, Zone::Battlefield);
            g.state.objects[bear as usize].controller = p;
            let shock = named(&mut g, p, "Shock", Zone::Hand);
            cast(&mut g, shock, None, &[Target::Player(p.other())]);
            settle(&mut g, mode, Some(bear));
            if mode == 0 {
                assert_eq!(g.state.objects[bear as usize].zone, Zone::Exile);
                end(&mut g);
                assert_eq!(g.state.objects[bear as usize].controller, p.other());
            } else {
                let tokens: Vec<_> = g
                    .observe(p)
                    .objects
                    .into_iter()
                    .filter(|o| o.is_token() && o.has_subtype("Spirit"))
                    .collect();
                assert_eq!(tokens.len(), 1);
                assert!(tokens[0].keywords.contains(Keywords::FLYING));
                assert_eq!(tokens[0].power, 1);
            }
        }
    }
}

fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
#[test]
fn return_links_prince_rejects_self_and_foreign_owned_then_ignores_stale_exile() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        g.state.objects[own as usize].controller = p.other();
        let foreign = spawn(&mut g, p.other(), Card::GrizzlyBears, Zone::Battlefield);
        g.state.objects[foreign as usize].controller = p;
        let prince = named(&mut g, p, "Charming Prince", Zone::Hand);
        cast(&mut g, prince, None, &[]);
        for _ in 0..8 {
            if g.legal_actions().contains(&Action::ChooseMode { mode: 2 }) {
                break;
            }
            checked(&mut g, Action::Pass);
        }
        checked(&mut g, Action::ChooseMode { mode: 2 });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(prince),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(foreign),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        settle(&mut g, 2, None);
        assert_eq!(g.observe(p).delayed_returns.len(), 1);
        let exiled = g.object_ref(own);
        g.move_object(own, Zone::Graveyard);
        g.move_object(own, Zone::Exile);
        assert_ne!(g.object_ref(own), exiled);
        end(&mut g);
        assert_eq!(g.state.objects[own as usize].zone, Zone::Exile);
    }
}
#[test]
fn return_links_familiar_counter_replacements_apply_again_and_vessel_owner_is_distinct() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        let cat = named(&mut g, p, "Nine-Lives Familiar", Zone::Hand);
        paid(&mut g, cat);
        assert_eq!(g.counter_count(cat, CounterKind::Revival), 16);
        kill(&mut g, p, cat);
        end(&mut g);
        assert_eq!(g.counter_count(cat, CounterKind::Revival), 30);
        g.state.step = Step::Main1;
        g.state.priority = p;
        let vessel = named(&mut g, p, "Infernal Vessel", Zone::Hand);
        paid(&mut g, vessel);
        g.state.objects[vessel as usize].controller = p.other();
        kill(&mut g, p, vessel);
        assert!(g.on_battlefield(vessel));
        assert_eq!(g.state.objects[vessel as usize].controller, p);
    }
}
#[test]
fn return_links_death_effective_types_ability_removal_and_call_filters() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let vessel = named(&mut g, p, "Infernal Vessel", Zone::Hand);
        paid(&mut g, vessel);
        let aura = named(&mut g, p, "Witness Protection", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(vessel)]);
        settle(&mut g, 0, None);
        kill(&mut g, p, vessel);
        assert_eq!(g.state.objects[vessel as usize].zone, Zone::Graveyard);
        named(&mut g, p, "Valkyrie's Call", Zone::Battlefield);
        let angel = named(&mut g, p, "Giada, Font of Hope", Zone::Battlefield);
        kill(&mut g, p, angel);
        assert_eq!(g.state.objects[angel as usize].zone, Zone::Graveyard);
        let token = named(
            &mut g,
            p,
            "@token/spirit_1_1_white_flying",
            Zone::Battlefield,
        );
        kill(&mut g, p, token);
        assert!(!g.on_battlefield(token));
        let bear = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        g.sacrifice_batch(&[(p, bear)]);
        g.move_object(bear, Zone::Hand);
        g.move_object(bear, Zone::Graveyard);
        settle(&mut g, 0, None);
        assert_eq!(g.state.objects[bear as usize].zone, Zone::Graveyard);
    }
}
#[test]
fn return_links_blinked_token_still_queues_delay_but_cannot_return_or_bind_reused_slot() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let token = named(
            &mut g,
            p,
            "@token/spirit_1_1_white_flying",
            Zone::Battlefield,
        );
        let prince = named(&mut g, p, "Charming Prince", Zone::Hand);
        cast(&mut g, prince, None, &[]);
        settle(&mut g, 2, Some(token));
        assert!(!g.on_battlefield(token));
        assert_eq!(g.observe(p).delayed_returns.len(), 1);
        let delayed = g.observe(p).delayed_returns[0].returning.object;
        let mut owned = {
            let parent = g.clone();
            parent.clone()
        };
        end(&mut g);
        end(&mut owned);
        assert_eq!(g.snapshot(), owned.snapshot());
        assert!(!g.on_battlefield(token));
        assert_ne!(g.object_ref(token), delayed);
    }
}
#[test]
fn return_links_kykar_queued_mode_survives_departure_and_creature_cast_is_not_a_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let kykar = named(&mut g, p, "Kykar, Zephyr Awakener", Zone::Hand);
        paid(&mut g, kykar);
        let bear = named(&mut g, p, "Bear Cub", Zone::Hand);
        paid(&mut g, bear);
        assert!(g.observe(p).objects.iter().all(|o| !o.is_token()));
        let spell = named(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, spell, None, &[Target::Player(p.other())]);
        g.move_object(kykar, Zone::Graveyard);
        settle(&mut g, 1, None);
        let tokens: Vec<_> = g
            .observe(p)
            .objects
            .into_iter()
            .filter(|o| o.is_token())
            .collect();
        assert_eq!(tokens.len(), 1);
        assert!(tokens[0].has_subtype("Spirit"));
    }
}

#[test]
fn return_links_combined_simultaneous_control_aura_departure_preserves_death_controller() {
    for p in [Player::P0, Player::P1] {
        for reverse in [false, true] {
            let mut g = setup(p);
            let cat = named(&mut g, p.other(), "Nine-Lives Familiar", Zone::Battlefield);
            g.set_counters(cat, Counters::of(CounterKind::Revival, 8));
            let aura = named(&mut g, p, "Confiscate", Zone::Battlefield);
            g.state.objects[aura as usize].attached_to = Some(g.object_ref(cat));
            let reference = g.object_ref(aura);
            g.retimestamp_live_static_source(reference);
            g.refresh_controllers();
            assert_eq!(g.object_view(cat).controller, p);
            let ids = if reverse { [cat, aura] } else { [aura, cat] };
            let bounds: Vec<_> = ids
                .into_iter()
                .map(|id| g.bind_target(Target::Card(id)))
                .collect();
            let mut owned = {
                let parent = g.clone();
                parent.clone()
            };
            g.move_target_batch(&bounds, Zone::Graveyard);
            owned.move_target_batch(&bounds, Zone::Graveyard);
            assert_eq!(g.snapshot(), owned.snapshot());
            let trigger = g
                .state
                .pending_triggers
                .iter()
                .find(|item| item.source_card.name() == "Nine-Lives Familiar")
                .unwrap();
            assert_eq!(
                trigger.controller, p,
                "death uses the pre-event control world"
            );
            settle(&mut g, 0, None);
            settle(&mut owned, 0, None);
            let pending = g.snapshot();
            end(&mut g);
            end(&mut owned);
            assert_eq!(g.snapshot(), owned.snapshot());
            assert_eq!(g.object_view(cat).owner, p.other());
            assert_eq!(g.object_view(cat).controller, p);
            assert_eq!(g.counter_count(cat, CounterKind::Revival), 7);
            let final_state = g.snapshot();
            g.restore(&pending).unwrap();
            end(&mut g);
            assert_eq!(g.snapshot(), final_state);
            assert_eq!(g.object_view(aura).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn return_links_animated_land_keeps_owned_effect_but_projects_subtypes_only_when_eligible() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Valkyrie's Call", Zone::Battlefield);
        named(&mut g, p, "Giada, Font of Hope", Zone::Battlefield);
        let land = spawn(&mut g, p, Card::Forest, Zone::Battlefield);
        // Reuse the same noncopiable crew animation overlay as copy-core tests.
        g.state.crew = Some(Box::new(crew::CrewState {
            animated: vec![g.object_ref(land)],
            ..Default::default()
        }));
        g.state.objects[land as usize].growth = Stats {
            power: 3,
            toughness: 3,
        };
        assert!(g.is_creature(land));
        kill(&mut g, p, land);
        assert!(g.on_battlefield(land));
        assert!(!g.is_creature(land));
        assert!(!g.has_subtype(land, "Angel"));
        assert_eq!(
            g.counter_count(land, CounterKind::PlusOne),
            1,
            "Giada doesn't count a land as an entering Angel"
        );
        assert!(g.keywords(land).contains(Keywords::FLYING));
        let before = g.snapshot();
        let mut owned = {
            let parent = g.clone();
            parent.clone()
        };
        for game in [&mut g, &mut owned] {
            game.state.crew = Some(Box::new(crew::CrewState {
                animated: vec![game.object_ref(land)],
                ..Default::default()
            }));
            assert!(game.is_creature(land));
            assert!(game.has_subtype(land, "Angel"));
            assert!(game.keywords(land).contains(Keywords::FLYING));
        }
        assert_eq!(g.snapshot(), owned.snapshot());
        g.restore(&before).unwrap();
        assert!(!g.has_subtype(land, "Angel"));
        assert_eq!(g.counter_count(land, CounterKind::PlusOne), 1);
    }
}
