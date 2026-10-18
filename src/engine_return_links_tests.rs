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
        if g.state.objects[id as usize].zone == Zone::Stack {
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
