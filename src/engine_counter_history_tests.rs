//! Exact-object/player history and placer-qualified triggers, through paid actions.
use super::ability_world::CharacteristicWorld;
use super::assurance::big_helpers::{checked, named, next_turn, settle, setup};
use super::*;

fn cast(g: &mut Game, id: CardId, target: Option<Target>) {
    checked(g, Action::AnnounceCast { card: id });
    for _ in 0..12 {
        if g.state
            .announcing
            .as_ref()
            .is_none_or(|a| a.item.card != id || a.item.kind != StackKind::Spell)
        {
            return;
        }
        if let Some(target) = target.filter(|t| {
            g.legal_actions()
                .contains(&Action::ChooseTarget { target: *t })
        }) {
            checked(g, Action::ChooseTarget { target });
            continue;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::ChooseSpellCost { .. }
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                )
            })
            .unwrap();
        checked(g, action);
    }
    panic!("unfinished paid cast");
}
fn paid(g: &mut Game, p: Player, name: &str, target: Option<Target>) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, target);
    settle(g);
    id
}
fn placement(
    g: &Game,
    placer: Player,
    id: CardId,
    kind: CounterKind,
    amount: u64,
) -> replacements::CounterPlacement {
    let view = g.object_view(id);
    replacements::CounterPlacement {
        object: view.reference,
        card: view.card,
        owner: view.owner,
        controller: view.controller,
        placed_by: placer,
        subtypes: view.granted_subtypes,
        kind: view.kinds,
        amount: Counters::of(kind, amount),
        effect: true,
        entry_index: None,
        private: false,
    }
}
fn place(g: &mut Game, placer: Player, id: CardId, kind: CounterKind, amount: u64) {
    let row = placement(g, placer, id, kind, amount);
    g.stage_counter_placements(vec![row], replacements::ResumeMode::Pending);
    while g.advance_replacements(false) {
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseReplacement { .. }))
            .unwrap();
        checked(g, action);
    }
}
fn flying(g: &Game, id: CardId) -> bool {
    g.object_view(id).keywords.contains(Keywords::FLYING)
}

#[test]
fn counter_history_beast_paid_hybrid_both_halves_removed_counters_and_turn_expiry() {
    for p in [Player::P0, Player::P1] {
        for color in [1, 4] {
            let mut g = setup(p);
            g.state.players[p.index()].mana = [0; 6];
            g.state.players[p.index()].mana[color] = 4;
            let beast = paid(&mut g, p, "Beast, Erudite Aerialist", None);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert!(!flying(&g, beast));
            g.state.players[p.index()].mana = [30; 6];
            paid(&mut g, p, "Snakeskin Veil", Some(Target::Card(beast)));
            assert_eq!(g.counter_count(beast, CounterKind::PlusOne), 1);
            assert!(flying(&g, beast));
            let object = g.object_ref(beast);
            assert!(g.plus_one_placed_this_turn(object, p));
            g.set_counters(beast, Counters::default());
            assert!(flying(&g, beast));
            assert_eq!(
                g.observe(p).counter_placement_history,
                g.observe(p.other()).counter_placement_history
            );
            next_turn(&mut g);
            assert!(!flying(&g, beast));
            assert!(g.state.counter_placement_history.is_none());
        }
    }
}

#[test]
fn counter_history_kinds_placer_current_controller_and_exact_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let beast = paid(&mut g, p, "Beast, Erudite Aerialist", None);
        let old = g.object_ref(beast);
        place(&mut g, p, beast, CounterKind::Stun, 1);
        assert!(g.put_counter_on_source_this_turn(old, p));
        assert!(!g.plus_one_placed_this_turn(old, p));
        assert!(!flying(&g, beast));
        place(&mut g, p.other(), beast, CounterKind::PlusOne, 1);
        assert!(g.plus_one_placed_this_turn(old, p.other()));
        assert!(!flying(&g, beast));
        g.change_controller(old, p.other());
        assert!(flying(&g, beast));
        g.change_controller(old, p);
        assert!(!flying(&g, beast));
        place(&mut g, p, beast, CounterKind::PlusOne, 1);
        assert!(flying(&g, beast));
        assert_eq!(g.state.counter_placement_history.as_ref().unwrap().len(), 2);
        // A later nonqualifying placement cannot downgrade the existing row.
        place(&mut g, p, beast, CounterKind::Plan, 1);
        assert!(g.plus_one_placed_this_turn(old, p));
        g.move_object(beast, Zone::Exile);
        g.move_object(beast, Zone::Battlefield);
        assert_ne!(g.object_ref(beast), old);
        assert!(!flying(&g, beast));
        assert!(!g.put_counter_on_source_this_turn(g.object_ref(beast), p));
        let clone = g.clone();
        assert!(
            g.state
                .counter_placement_history
                .as_ref()
                .unwrap()
                .ptr_eq(clone.state.counter_placement_history.as_ref().unwrap())
        );
        let snapshot = g.snapshot();
        place(&mut g, p, beast, CounterKind::PlusOne, 1);
        assert!(flying(&g, beast));
        assert!(!flying(&clone, beast));
        g.restore(&snapshot).unwrap();
        assert!(!flying(&g, beast));
    }
}

#[test]
fn counter_history_beast_combat_damage_draws_with_flying_and_noncombat_does_not() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let beast = paid(&mut g, p, "Beast, Erudite Aerialist", None);
        paid(&mut g, p, "Snakeskin Veil", Some(Target::Card(beast)));
        let blocker = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        g.state.objects[beast as usize].summoning_sick = false;
        let secret = named(&mut g, p, "Bear Cub", Zone::Library);
        let before = g.hand_count(p);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: beast });
        checked(&mut g, Action::FinishAttackers);
        for _ in 0..20 {
            if g.state.pending == Pending::Blockers {
                break;
            }
            checked(&mut g, Action::Pass);
        }
        let snapshot = g.snapshot();
        assert_eq!(
            g.apply(
                g.actor().unwrap(),
                Action::Block {
                    blocker,
                    attacker: beast
                }
            ),
            Err(Error::IllegalAction)
        );
        assert_eq!(g.snapshot(), snapshot);
        checked(&mut g, Action::FinishBlockers);
        for _ in 0..30 {
            if g.hand_count(p) == before + 1 {
                break;
            }
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::AssignDamage { .. }))
                .unwrap();
            checked(&mut g, action);
        }
        assert_eq!(g.hand_count(p), before + 1);
        assert_eq!(g.object_view(secret).zone, Zone::Hand);
        assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == secret));
        let before = g.hand_count(p);
        let source = g.damage_source(beast);
        g.apply_damage_batch(&[DamageEvent {
            source,
            target: Target::Player(p.other()),
            amount: 1,
            combat: false,
            preventable: true,
        }]);
        settle(&mut g);
        assert_eq!(g.hand_count(p), before);
    }
}

#[test]
fn counter_history_beast_layer_six_suppression_restores_history_qualified_flying() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let beast = paid(&mut g, p, "Beast, Erudite Aerialist", None);
        place(&mut g, p, beast, CounterKind::PlusOne, 1);
        let aura = paid(&mut g, p, "Witness Protection", Some(Target::Card(beast)));
        assert!(!flying(&g, beast));
        g.set_counters(beast, Counters::default());
        g.move_object(aura, Zone::Graveyard);
        assert!(flying(&g, beast));
    }
}

#[test]
fn counter_history_knight_paid_opponent_recipient_own_placer_once_turn_and_no_self_loop() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let knight = paid(&mut g, p, "Knight of Wundagore", None);
        assert!(g.object_view(knight).keywords.contains(Keywords::TRAMPLE));
        let own = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        // A different player putting counters on our creature does not qualify.
        checked(&mut g, Action::Pass);
        paid(
            &mut g,
            p.other(),
            "Fleeting Flight",
            Some(Target::Card(own)),
        );
        assert_eq!(g.counter_count(knight, CounterKind::PlusOne), 0);
        // Putting a counter on Knight itself does not consume its trigger limit.
        paid(&mut g, p, "Fleeting Flight", Some(Target::Card(knight)));
        assert_eq!(g.counter_count(knight, CounterKind::PlusOne), 1);
        paid(&mut g, p, "Fleeting Flight", Some(Target::Card(enemy)));
        assert_eq!(g.counter_count(knight, CounterKind::PlusOne), 2);
        paid(&mut g, p, "Fleeting Flight", Some(Target::Card(own)));
        assert_eq!(g.counter_count(knight, CounterKind::PlusOne), 2);
        next_turn(&mut g);
        g.state.players[p.index()].mana = [30; 6];
        checked(&mut g, Action::Pass);
        paid(&mut g, p, "Fleeting Flight", Some(Target::Card(enemy)));
        assert_eq!(g.counter_count(knight, CounterKind::PlusOne), 3);
    }
}

#[test]
fn counter_history_knight_replacement_batch_keeps_placer_and_triggers_once_per_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = paid(&mut g, p, "Knight of Wundagore", None);
        let b = paid(&mut g, p, "Knight of Wundagore", None);
        let enemy = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let another = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        place(&mut g, p, land, CounterKind::PlusOne, 1);
        place(&mut g, p, enemy, CounterKind::Stun, 1);
        settle(&mut g);
        assert_eq!(
            (
                g.counter_count(a, CounterKind::PlusOne),
                g.counter_count(b, CounterKind::PlusOne)
            ),
            (0, 0)
        );
        named(&mut g, p.other(), "Doubling Season", Zone::Battlefield);
        let rows = [enemy, another]
            .into_iter()
            .map(|id| placement(&g, p, id, CounterKind::PlusOne, 1))
            .collect();
        g.stage_counter_placements(rows, replacements::ResumeMode::Pending);
        g.advance_replacements(false);
        settle(&mut g);
        assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 2);
        assert_eq!(g.counter_count(another, CounterKind::PlusOne), 2);
        assert_eq!(
            (
                g.counter_count(a, CounterKind::PlusOne),
                g.counter_count(b, CounterKind::PlusOne)
            ),
            (1, 1)
        );
        assert!(g.plus_one_placed_this_turn(g.object_ref(enemy), p));
        assert!(!g.plus_one_placed_this_turn(g.object_ref(enemy), p.other()));
    }
}

#[test]
fn counter_history_entering_with_counters_counts_before_characteristics_and_knight_callback() {
    for p in [Player::P0, Player::P1] {
        for kind in [CounterKind::PlusOne, CounterKind::Stun] {
            let mut g = setup(p);
            let knight = paid(&mut g, p, "Knight of Wundagore", None);
            let beast = named(&mut g, p, "Beast, Erudite Aerialist", Zone::Hand);
            let mut movement = g.entry_move(beast, Zone::Battlefield, false);
            movement.effect_counter = Some((kind, 2));
            g.stage_entries(vec![movement], None);
            g.advance_entries();
            assert_eq!(flying(&g, beast), kind == CounterKind::PlusOne);
            assert!(g.put_counter_on_source_this_turn(g.object_ref(beast), p));
            settle(&mut g);
            assert_eq!(
                g.counter_count(knight, CounterKind::PlusOne),
                u64::from(kind == CounterKind::PlusOne)
            );
        }
    }
}

#[test]
fn counter_history_knight_departure_never_rebinds_and_failed_resolution_is_atomic() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let knight = paid(&mut g, p, "Knight of Wundagore", None);
        let recipient = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        place(&mut g, p, recipient, CounterKind::PlusOne, 1);
        g.prepare_triggers();
        g.move_object(knight, Zone::Exile);
        g.move_object(knight, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.counter_count(knight, CounterKind::PlusOne), 0);
        g.set_counters(knight, Counters::of(CounterKind::PlusOne, u64::MAX));
        place(&mut g, p, recipient, CounterKind::PlusOne, 1);
        g.prepare_triggers();
        let mut rejected = false;
        for _ in 0..8 {
            let snapshot = g.snapshot();
            let revision = g.revision();
            match g.apply(g.actor().unwrap(), Action::Pass) {
                Ok(()) => (),
                Err(Error::CapacityExceeded(_)) => {
                    assert_eq!(g.snapshot(), snapshot);
                    assert_eq!(g.revision(), revision);
                    rejected = true;
                    break;
                }
                other => panic!("unexpected {other:?}"),
            }
        }
        assert!(rejected);
    }
}

#[test]
fn counter_history_legacy_recipient_controller_trigger_remains_distinct_from_placer_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let knight = paid(&mut g, p, "Knight of Wundagore", None);
        let legacy = named(&mut g, p, "Wildwood Scourge", Zone::Battlefield);
        g.set_counters(legacy, Counters::of(CounterKind::PlusOne, 1));
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        place(&mut g, p.other(), own, CounterKind::PlusOne, 1);
        assert!(
            g.state
                .pending_triggers
                .iter()
                .any(|item| item.card == legacy)
        );
        assert!(
            !g.state
                .pending_triggers
                .iter()
                .any(|item| item.card == knight)
        );
        g.move_object(legacy, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.counter_count(knight, CounterKind::PlusOne), 0);
    }
}

#[test]
fn counter_history_knight_paid_trample_assigns_lethal_and_excess_to_player() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let knight = paid(&mut g, p, "Knight of Wundagore", None);
        paid(&mut g, p, "Snakeskin Veil", Some(Target::Card(knight)));
        let blocker = named(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
        g.state.objects[knight as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: knight });
        checked(&mut g, Action::FinishAttackers);
        for _ in 0..10 {
            if g.state.pending == Pending::Blockers {
                break;
            }
            checked(&mut g, Action::Pass);
        }
        checked(
            &mut g,
            Action::Block {
                blocker,
                attacker: knight,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        for _ in 0..10 {
            if g.state.pending == Pending::Damage {
                break;
            }
            checked(&mut g, Action::Pass);
        }
        let snapshot = g.snapshot();
        assert_eq!(
            g.apply(
                g.actor().unwrap(),
                Action::AssignDamage {
                    target: Target::Card(blocker),
                    amount: 0
                }
            ),
            Err(Error::IllegalAction)
        );
        assert_eq!(g.snapshot(), snapshot);
        checked(
            &mut g,
            Action::AssignDamage {
                target: Target::Card(blocker),
                amount: 1,
            },
        );
        assert_eq!(g.object_view(blocker).zone, Zone::Graveyard);
        assert_eq!(g.object_view(knight).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}
