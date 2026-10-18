use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::tests::spawn;
use super::*;
fn fixture(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    assert_eq!(card.is_supported(), name != "Boggart Mischief");
    spawn(g, p, card, zone)
}
fn activate(g: &mut Game, id: CardId, index: u16, target: Option<Target>) {
    let action = g.legal_actions().into_iter().find(|a| matches!(a, Action::Activate {card, ability, target: t, ..} if *card==id && *ability==index && *t==target)).unwrap_or_else(|| panic!("missing activation {id} {index} {:?}", g.state.pending));
    checked(g, action);
}
fn damage(g: &Game, id: CardId, target: Target, amount: i128, combat: bool) -> DamageEvent {
    DamageEvent {
        source: g.damage_source(id),
        target,
        amount,
        combat,
    }
}
#[test]
fn event_counter_core_tome_pays_counter_before_resolution_latches_and_no_intervening_recheck() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = fixture(&mut g, p, "Mazemind Tome", Zone::Hand);
        cast(&mut g, id, None, &[]);
        settle(&mut g);
        for n in 1..=4 {
            g.state.priority = p;
            g.state.objects[id as usize].tapped = false;
            activate(&mut g, id, 1, None);
            assert_eq!(g.counter_count(id, CounterKind::Page), n);
            if n < 4 {
                settle(&mut g);
                continue;
            }
            assert_eq!(g.state.stack.len(), 2, "state trigger above activation");
            for _ in 0..3 {
                g.prepare_triggers();
            }
            assert_eq!(g.state.stack.len(), 2, "latched exact packet");
            g.set_counters(id, Counters::default());
            let life = g.state.players[p.index()].life;
            settle(&mut g);
            assert_eq!(g.state.objects[id as usize].zone, Zone::Exile);
            assert_eq!(g.state.players[p.index()].life, life + 4);
        }
    }
}
#[test]
fn event_counter_core_tome_retriggers_if_countered_but_departure_does_not_pay() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = fixture(&mut g, p, "Mazemind Tome", Zone::Battlefield);
        g.add_counter(id, CounterKind::Page, 4);
        g.prepare_triggers();
        assert_eq!(g.state.stack.len(), 1);
        let packet = g.state.stack.last().unwrap().clone();
        assert!(g.counter_stack_item(packet.id, false));
        g.prepare_triggers();
        assert_eq!(g.state.stack.len(), 1);
        assert_ne!(g.state.stack[0].id, packet.id);
        let life = g.state.players[p.index()].life;
        g.move_object(id, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life);
    }
}
#[test]
fn event_counter_core_bomber_grants_independent_any_recipient_batch_triggers() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bomber = fixture(&mut g, p, "Dropkick Bomber", Zone::Hand);
        cast(&mut g, bomber, None, &[]);
        settle(&mut g);
        let goblin = named(&mut g, p, "Goblin Piker", Zone::Battlefield);
        let blocker = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        assert_eq!(g.stats(goblin).power, 3);
        for _ in 0..2 {
            g.state.priority = p;
            activate(&mut g, bomber, 0, Some(Target::Card(goblin)));
            settle(&mut g);
        }
        assert!(g.keywords(goblin).contains(Keywords::FLYING));
        assert_eq!(g.state.temporary_abilities.as_ref().unwrap().len(), 2);
        let event = damage(&g, goblin, Target::Card(blocker), 1, true);
        let events = [
            event,
            DamageEvent {
                target: Target::Player(p.other()),
                ..event
            },
        ];
        g.apply_damage_batch(&events);
        assert_eq!(
            g.state.pending_triggers.len(),
            2,
            "one per granted instance, not damaged recipient"
        );
        g.move_object(bomber, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.state.objects[goblin as usize].zone, Zone::Graveyard);
    }
}
#[test]
fn event_counter_core_ooze_records_nonzero_exact_damage_during_ability_removal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ooze = fixture(&mut g, p, "Predator Ooze", Zone::Hand);
        cast(&mut g, ooze, None, &[]);
        settle(&mut g);
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let stop = named(&mut g, p.other(), "Stop Cold", Zone::Hand);
        g.state.priority = p.other();
        cast(&mut g, stop, None, &[Target::Card(ooze)]);
        settle(&mut g);
        let event = damage(&g, ooze, Target::Card(bear), 1, false);
        g.apply_damage_batch(&[event, DamageEvent { amount: 0, ..event }]);
        assert_eq!(g.observe(p).creature_damage_history.len(), 1);
        let saved = g.snapshot();
        let mut owned = {
            let parent = g.clone();
            parent.clone()
        };
        owned.move_object(stop, Zone::Graveyard);
        owned.move_object(bear, Zone::Graveyard);
        settle(&mut owned);
        assert_eq!(owned.counter_count(ooze, CounterKind::PlusOne), 1);
        g.move_object(bear, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(
            g.counter_count(ooze, CounterKind::PlusOne),
            0,
            "removed ability cannot trigger"
        );
        g.restore(&saved).unwrap();
        g.move_object(stop, Zone::Graveyard);
        g.move_object(bear, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.snapshot(), owned.snapshot());
    }
}

#[test]
fn event_counter_core_tome_scry_private_and_counter_cost_not_an_effect() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = fixture(&mut g, p, "Mazemind Tome", Zone::Battlefield);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        let top = *g.state.players[p.index()].library.last().unwrap();
        activate(&mut g, id, 0, None);
        assert_eq!(
            g.counter_count(id, CounterKind::Page),
            1,
            "cost is not doubled by effect-only replacement"
        );
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert!(matches!(g.state.pending,Pending::EffectLibrary {player} if player==p));
        let own = serde_json::to_value(g.observe(p)).unwrap();
        let other = serde_json::to_value(g.observe(p.other())).unwrap();
        assert_ne!(own["library_choice"], other["library_choice"]);
        let a=g.legal_actions().into_iter().find(|a| matches!(a,Action::ArrangeLibrary {card,destination: LibraryDestination::Bottom} if *card==top)).unwrap();
        checked(&mut g, a);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library[0], top);
    }
}

#[test]
fn event_counter_core_ooze_exact_incarnations_simultaneous_death_and_turn_lifetime() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ooze = fixture(&mut g, p, "Predator Ooze", Zone::Battlefield);
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let event = damage(&g, ooze, Target::Card(bear), 1, false);
        g.apply_damage_batch(&[event]);
        g.move_object(bear, Zone::Exile);
        g.move_object(bear, Zone::Battlefield);
        g.move_object(bear, Zone::Graveyard);
        assert!(
            g.state.pending_triggers.is_empty(),
            "recipient new incarnation is unmarked"
        );
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let event = damage(&g, ooze, Target::Card(bear), 1, false);
        g.apply_damage_batch(&[event]);
        let sources = g.death_trigger_sources();
        let events = g.move_objects_raw(&[ooze, bear], Zone::Graveyard);
        g.finish_zone_events(&events, &sources);
        assert_eq!(
            g.state.pending_triggers.len(),
            1,
            "simultaneous departure looks back at source"
        );
        g.move_object(ooze, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(
            g.counter_count(ooze, CounterKind::PlusOne),
            0,
            "old packet cannot grow new incarnation"
        );
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let event = damage(&g, ooze, Target::Card(bear), 1, false);
        g.apply_damage_batch(&[event]);
        ability_timeline_tests::next_turn(&mut g);
        assert!(g.state.creature_damage_history.is_none());
        g.move_object(bear, Zone::Graveyard);
        assert!(
            g.state.pending_triggers.is_empty(),
            "old-turn damage expired"
        );
    }
}

#[test]
fn event_counter_core_bomber_target_legality_controller_capture_and_expiration() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bomber = fixture(&mut g, p, "Dropkick Bomber", Zone::Battlefield);
        let goblin = named(&mut g, p, "Goblin Piker", Zone::Battlefield);
        let foreign = named(&mut g, p.other(), "Goblin Piker", Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        for invalid in [bomber, foreign, bear] {
            assert!(!g.legal_actions().iter().any(|a| matches!(a, Action::Activate {card, target: Some(Target::Card(target)), ..} if *card==bomber && *target==invalid)));
        }
        activate(&mut g, bomber, 0, Some(Target::Card(goblin)));
        settle(&mut g);
        let mut control = g.activation_item(p, bomber, 0);
        control.targets.push(g.bind_target(Target::Card(goblin)));
        g.gain_control(&control, Subject::Targets, PlayerSet::Opponent);
        let event = damage(&g, goblin, Target::Card(bear), 1, false);
        g.apply_damage_batch(&[event]);
        assert!(
            g.state.pending_triggers.is_empty(),
            "noncombat damage does not trigger"
        );
        g.apply_damage_batch(&[DamageEvent {
            combat: true,
            ..event
        }]);
        assert_eq!(g.state.pending_triggers[0].controller, p.other());
        control.controller = p.other();
        g.gain_control(&control, Subject::Targets, PlayerSet::Opponent);
        settle(&mut g);
        assert!(
            g.on_battlefield(goblin),
            "captured controller cannot sacrifice a stolen source"
        );
        g.finish_cleanup();
        assert!(g.state.temporary_abilities.is_none());
        assert!(!g.keywords(goblin).contains(Keywords::FLYING));
    }
}

#[test]
fn event_counter_core_bomber_later_removal_suppresses_granted_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bomber = fixture(&mut g, p, "Dropkick Bomber", Zone::Battlefield);
        let goblin = named(&mut g, p, "Goblin Piker", Zone::Battlefield);
        activate(&mut g, bomber, 0, Some(Target::Card(goblin)));
        settle(&mut g);
        let stop = named(&mut g, p.other(), "Stop Cold", Zone::Hand);
        g.state.priority = p.other();
        cast(&mut g, stop, None, &[Target::Card(goblin)]);
        settle(&mut g);
        let event = damage(&g, goblin, Target::Player(p.other()), 1, true);
        g.apply_damage_batch(&[event]);
        assert!(g.state.pending_triggers.is_empty());
        g.move_object(stop, Zone::Graveyard);
        g.apply_damage_batch(&[event]);
        assert_eq!(g.state.pending_triggers.len(), 1);
        settle(&mut g);
        assert!(!g.on_battlefield(goblin));
    }
}

#[test]
fn event_counter_core_ooze_paid_attack_and_indestructible() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ooze = fixture(&mut g, p, "Predator Ooze", Zone::Hand);
        cast(&mut g, ooze, None, &[]);
        settle(&mut g);
        g.state.objects[ooze as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: ooze });
        checked(&mut g, Action::FinishAttackers);
        settle(&mut g);
        assert_eq!(g.counter_count(ooze, CounterKind::PlusOne), 1);
        let dealer = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let event = damage(&g, dealer, Target::Card(ooze), 20, false);
        g.apply_damage_batch(&[event]);
        g.state_based_actions();
        assert!(g.on_battlefield(ooze));
        assert!(g.keywords(ooze).contains(Keywords::INDESTRUCTIBLE));
    }
}

#[test]
fn event_counter_core_bomber_oracle_allows_noncreature_goblin_permanent() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bomber = fixture(&mut g, p, "Dropkick Bomber", Zone::Battlefield);
        // Metadata-only gated fixture: its unrelated card behavior remains absent.
        // Oracle says Goblin, whereas the old Java filter wrongly says creature.
        let kindred = fixture(&mut g, p, "Boggart Mischief", Zone::Battlefield);
        assert!(!g.is_creature(kindred));
        assert!(g.has_subtype(kindred, "Goblin"));
        activate(&mut g, bomber, 0, Some(Target::Card(kindred)));
        settle(&mut g);
        assert!(g.keywords(kindred).contains(Keywords::FLYING));
        assert_eq!(
            g.state.temporary_abilities.as_ref().unwrap()[0].recipient,
            g.object_ref(kindred)
        );
    }
}

#[test]
fn event_counter_core_tome_counter_cost_capacity_error_is_atomic() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let tome = fixture(&mut g, p, "Mazemind Tome", Zone::Battlefield);
        g.set_counters(tome, Counters::of(CounterKind::Page, u64::MAX));
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a,Action::Activate{card,ability:1,..} if *card==tome))
            .unwrap();
        let snapshot = g.snapshot();
        let revision = g.revision();
        assert!(matches!(
            g.apply(p, action),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), snapshot);
        assert_eq!(g.revision(), revision);
    }
}
