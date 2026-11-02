//! Actual cast, contribution, attack and graveyard events for four whole creatures.
mod support;
use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;
fn activation(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}
fn drain(g: &mut Game) {
    for _ in 0..64 {
        g.prepare_triggers();
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
        {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("bounded resolution");
}
fn reach(g: &mut Game, wanted: impl Fn(&Action) -> bool) {
    for _ in 0..24 {
        g.prepare_triggers();
        if g.legal_actions().iter().any(&wanted) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("choice missing {:?}", g.state.pending);
}
fn attack(g: &mut Game, ids: &[CardId]) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    for &card in ids {
        act(g, Action::Attack { card });
    }
    act(g, Action::FinishAttackers);
}
#[test]
fn adrenaline_jockey_damages_the_actual_offturn_caster_regardless_of_observer_controller() {
    for owner in [Player::P0, Player::P1] {
        for caster in [Player::P0, Player::P1] {
            for offturn in [false, true] {
                let mut g = scenario(caster);
                let source = put(&mut g, owner, "Adrenaline Jockey", Zone::Battlefield);
                assert_eq!(stats(&g, source), (3, 3));
                let turtle = put(&mut g, caster.other(), "Aegis Turtle", Zone::Battlefield);
                g.state.active = if offturn { caster.other() } else { caster };
                g.state.priority = caster;
                let spell = put(&mut g, caster, "Shock", Zone::Hand);
                cast(&mut g, spell, Some(Target::Card(turtle)));
                drain(&mut g);
                assert_eq!(
                    g.state.players[caster.index()].life,
                    if offturn { 16 } else { 20 }
                );
                assert_eq!(g.state.players[caster.other().index()].life, 20);
                assert_eq!(g.object_view(turtle).damage, 2);
                assert_eq!(g.object_view(source).counters.plus_one, 0);
            }
        }
    }
}
#[test]
fn adrenaline_jockey_counts_committed_own_exhaust_even_if_the_ability_is_countered() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Adrenaline Jockey", Zone::Battlefield);
        let engineer = put(&mut g, p, "Draconautics Engineer", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, activation(engineer, 0));
        assert_eq!(g.object_view(source).counters.plus_one, 0);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
        act(&mut g, activation(engineer, 0));
        pay(&mut g);
        g.prepare_triggers();
        assert_eq!(g.object_view(source).counters.plus_one, 0);
        let packet = g
            .state
            .stack
            .iter()
            .find(|s| matches!(s.kind, StackKind::ActivatedAbility { ability: 0 }))
            .unwrap()
            .id;
        assert!(g.counter_stack_item(packet, false));
        drain(&mut g);
        assert_eq!(g.object_view(engineer).counters.plus_one, 0);
        assert_eq!(g.object_view(source).counters.plus_one, 1);
        assert_eq!(stats(&g, source), (4, 4));
        let enemy = put(
            &mut g,
            p.other(),
            "Draconautics Engineer",
            Zone::Battlefield,
        );
        g.state.priority = p.other();
        act(&mut g, activation(enemy, 0));
        pay(&mut g);
        drain(&mut g);
        assert_eq!(g.object_view(source).counters.plus_one, 1);
    }
}
#[test]
fn grim_javelineer_attack_buff_and_delayed_surveil_keep_original_controller_after_source_departure()
{
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Grim Javelineer", Zone::Battlefield);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let nonattacker = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let top = put(&mut g, p, "Shock", Zone::Library);
        attack(&mut g, &[bear]);
        reach(&mut g, |a| matches!(a, Action::ChooseTarget { .. }));
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(nonattacker),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        drain(&mut g);
        assert_eq!(stats(&g, bear), (3, 2));
        assert_eq!(stats(&g, source), (3, 2));
        g.move_object(source, Zone::Graveyard);
        g.state.objects[bear as usize].controller = p.other();
        g.move_object(bear, Zone::Graveyard);
        reach(&mut g, |a| matches!(a, Action::ArrangeLibrary { .. }));
        assert_eq!(g.actor(), Some(p));
        assert!(g.observe(p).library_choice.is_some());
        assert!(g.observe(p.other()).library_choice.is_none());
        let saved = g.snapshot();
        let object = g.object_ref(top);
        let a = Action::ArrangeLibrary {
            card: top,
            destination: LibraryDestination::Graveyard,
        };
        let mut raw = g.clone();
        let mut indexed = g.clone();
        let at = indexed
            .legal_actions()
            .iter()
            .position(|x| *x == a)
            .unwrap();
        act(&mut raw, a);
        indexed.apply_index(p, indexed.revision(), at).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        indexed.restore(&saved).unwrap();
        drop(g);
        act(
            &mut indexed,
            Action::ArrangeLibrary {
                card: top,
                destination: LibraryDestination::Graveyard,
            },
        );
        drain(&mut indexed);
        assert_eq!(indexed.object_view(top).zone, Zone::Graveyard);
        assert_ne!(indexed.object_ref(top), object);
    }
}
#[test]
fn grim_javelineer_delayed_death_rejects_a_new_incarnation_and_expires_at_cleanup() {
    for stale in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        put(&mut g, p, "Grim Javelineer", Zone::Battlefield);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        attack(&mut g, &[bear]);
        reach(&mut g, |a| matches!(a, Action::ChooseTarget { .. }));
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        drain(&mut g);
        if stale {
            g.move_object(bear, Zone::Hand);
            g.move_object(bear, Zone::Battlefield);
        } else {
            g.finish_cleanup();
            assert_eq!(stats(&g, bear), (2, 2));
        }
        g.move_object(bear, Zone::Graveyard);
        drain(&mut g);
        assert!(g.observe(p).library_choice.is_none());
    }
}
#[test]
fn reckless_velocitaur_buffs_only_its_exact_crew_or_saddle_recipient_during_own_main() {
    for p in [Player::P0, Player::P1] {
        for (step, own, expected) in [
            (Step::Main1, true, true),
            (Step::Main2, true, true),
            (Step::Upkeep, true, false),
            (Step::Main1, false, false),
        ] {
            for crew in [false, true] {
                if !crew && step != Step::Main1 {
                    continue;
                }
                let mut g = scenario(p);
                g.state.step = step;
                g.state.active = if own { p } else { p.other() };
                let source = put(&mut g, p, "Reckless Velocitaur", Zone::Battlefield);
                let recipient = put(
                    &mut g,
                    p,
                    if crew {
                        "Clamorous Ironclad"
                    } else {
                        "Gilded Ghoda"
                    },
                    Zone::Battlefield,
                );
                let base = if crew { (6, 3) } else { (2, 2) };
                if !crew && !own {
                    continue;
                }
                act(&mut g, activation(recipient, 0));
                if matches!(g.state.pending, Pending::PayCosts { .. }) {
                    pay(&mut g);
                }
                let object = g.object_ref(source);
                act(
                    &mut g,
                    if crew {
                        Action::ChooseCrewer { object }
                    } else {
                        Action::ChooseSaddler { object }
                    },
                );
                act(
                    &mut g,
                    if crew {
                        Action::FinishCrew
                    } else {
                        Action::FinishSaddle
                    },
                );
                drain(&mut g);
                assert!(g.object_view(source).tapped);
                assert_eq!(stats(&g, source), (3, 3));
                assert_eq!(
                    stats(&g, recipient),
                    (base.0 + if expected { 2 } else { 0 }, base.1)
                );
                assert_eq!(g.keywords(recipient).contains(Keywords::TRAMPLE), expected);
                g.finish_cleanup();
                assert_eq!(stats(&g, recipient), if crew { (0, 0) } else { base });
                assert_eq!(g.is_creature(recipient), !crew);
                assert!(!g.keywords(recipient).contains(Keywords::TRAMPLE));
            }
        }
    }
}
#[test]
fn reckless_velocitaur_queued_buff_survives_pilot_departure_but_not_recipient_reentry() {
    for stale in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let pilot = put(&mut g, p, "Reckless Velocitaur", Zone::Battlefield);
        let vehicle = put(&mut g, p, "Clamorous Ironclad", Zone::Battlefield);
        act(&mut g, activation(vehicle, 0));
        if matches!(g.state.pending, Pending::PayCosts { .. }) {
            pay(&mut g);
        }
        let object = g.object_ref(pilot);
        act(&mut g, Action::ChooseCrewer { object });
        act(&mut g, Action::FinishCrew);
        g.move_object(pilot, Zone::Graveyard);
        if stale {
            g.move_object(vehicle, Zone::Hand);
            g.move_object(vehicle, Zone::Battlefield);
        }
        drain(&mut g);
        assert_eq!(g.keywords(vehicle).contains(Keywords::TRAMPLE), !stale);
        assert_eq!(g.is_creature(vehicle), !stale);
        assert_eq!(stats(&g, vehicle), if stale { (0, 0) } else { (8, 3) });
    }
}
#[test]
fn bloodghast_actual_landfall_optional_return_and_live_haste_enable_attack_but_never_block() {
    for p in [Player::P0, Player::P1] {
        for accept in [false, true] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Bloodghast", Zone::Graveyard);
            let land = put(&mut g, p, "Forest", Zone::Hand);
            act(&mut g, Action::PlayLand { card: land });
            reach(&mut g, |a| matches!(a, Action::ChooseOptional { .. }));
            act(&mut g, Action::ChooseOptional { accept });
            drain(&mut g);
            assert_eq!(
                g.object_view(source).zone,
                if accept {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
            if !accept {
                continue;
            }
            assert_eq!(stats(&g, source), (2, 1));
            assert!(!g.object_view(source).tapped);
            g.state.players[p.other().index()].life = 11;
            assert!(!g.keywords(source).contains(Keywords::HASTE));
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            reject(&mut g, Action::Attack { card: source });
            g.state.players[p.other().index()].life = 10;
            assert!(g.keywords(source).contains(Keywords::HASTE));
            act(&mut g, Action::Attack { card: source });
            act(&mut g, Action::FinishAttackers);
            drain(&mut g);
            g.finish_cleanup();
            g.state.objects[source as usize].tapped = false;
            g.state.active = p.other();
            g.state.priority = p.other();
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            attack(&mut g, &[enemy]);
            drain(&mut g);
            g.state.step = Step::Blockers;
            g.state.pending = Pending::Blockers;
            reject(
                &mut g,
                Action::Block {
                    blocker: source,
                    attacker: enemy,
                },
            );
            assert!(g.keywords(source).contains(Keywords::CANT_BLOCK));
        }
    }
}
#[test]
fn bloodghast_foreign_land_or_stale_graveyard_incarnation_does_not_return() {
    for p in [Player::P0, Player::P1] {
        for stale in [false, true] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Bloodghast", Zone::Graveyard);
            let who = if stale { p } else { p.other() };
            let land = put(&mut g, who, "Forest", Zone::Hand);
            g.state.active = who;
            g.state.priority = who;
            act(&mut g, Action::PlayLand { card: land });
            if stale {
                g.move_object(source, Zone::Exile);
                g.move_object(source, Zone::Graveyard);
                // The queued optional trigger remains, but accepting it cannot
                // return a different graveyard incarnation.
                reach(&mut g, |a| matches!(a, Action::ChooseOptional { .. }));
                act(&mut g, Action::ChooseOptional { accept: true });
            }
            drain(&mut g);
            assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        }
    }
}
