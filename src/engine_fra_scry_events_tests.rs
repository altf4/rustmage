//! Scry and surveil event behavior, including gated Reality Fracture cards.
use super::*;

fn wait_for_choice(g: &mut Game) {
    for _ in 0..32 {
        if matches!(g.state.pending, Pending::EffectLibrary { .. }) {
            return;
        }
        assert_eq!(g.state.pending, Pending::Priority);
        act(g, Action::Pass);
    }
    panic!("expected a library choice");
}

fn scry(g: &mut Game, p: Player) {
    let spell = put(g, p, "Plunge into Winter", Zone::Hand);
    announce(g, spell, &[]);
}

fn surveil(g: &mut Game, p: Player) {
    let spell = put(g, p, "Prismari Charm", Zone::Hand);
    act(g, Action::AnnounceCast { card: spell });
    act(g, Action::ChooseMode { mode: 0 });
    pay(g);
}

fn arrange(g: &mut Game, destination: LibraryDestination) {
    while matches!(g.state.pending, Pending::EffectLibrary { .. }) {
        choose(
            g,
            |action| matches!(action, Action::ArrangeLibrary { destination: d, .. } if *d == destination),
        );
    }
}

fn thopters(g: &Game, p: Player) -> usize {
    g.state
        .objects
        .iter()
        .filter(|o| o.zone == Zone::Battlefield && o.controller == p && o.card == Card(5266))
        .count()
}

#[test]
fn fra_scry_events_resolve_after_arrangement_and_observe_each_player() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let denz = put(&mut g, p, "Denzilore Fatehold", Zone::Battlefield);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Denzilore Fatehold", Zone::Battlefield);
        scry(&mut g, p);
        wait_for_choice(&mut g);
        assert!(g.state.pending_triggers.is_empty());
        assert!(g.observe(p.other()).library_choice.is_none());
        super::super::contracts::branch_contract(g.clone());
        arrange(&mut g, LibraryDestination::Bottom);
        settle(&mut g);
        assert_eq!(g.counter_count(denz, CounterKind::PlusOne), 1);
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 1);
        assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 0);
        surveil(&mut g, p);
        wait_for_choice(&mut g);
        arrange(&mut g, LibraryDestination::Graveyard);
        settle(&mut g);
        assert_eq!(g.counter_count(denz, CounterKind::PlusOne), 2);
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 2);
        assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 0);
    }
}

#[test]
fn fra_saheeli_scry_and_surveil_trigger_only_once_per_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Saheeli, Consul of Oversight", Zone::Battlefield);
        scry(&mut g, p);
        wait_for_choice(&mut g);
        arrange(&mut g, LibraryDestination::Top);
        settle(&mut g);
        assert_eq!(thopters(&g, p), 1);
        surveil(&mut g, p);
        wait_for_choice(&mut g);
        arrange(&mut g, LibraryDestination::Top);
        settle(&mut g);
        assert_eq!(thopters(&g, p), 1);
        g.finish_cleanup();
        g.state.turn += 1;
        g.begin_turn();
        scry(&mut g, p);
        wait_for_choice(&mut g);
        arrange(&mut g, LibraryDestination::Top);
        settle(&mut g);
        assert_eq!(thopters(&g, p), 2);
    }
}

#[test]
fn fra_zero_scry_and_surveil_do_not_emit_an_event() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Denzilore Fatehold", Zone::Battlefield);
        scry(&mut g, p);
        wait_for_choice(&mut g);
        // Drive the existing interpreter with a zero-valued test program.
        let resolution = g.state.resolving.as_mut().unwrap();
        resolution.library = None;
        resolution.program = &[Instruction::Scry(0), Instruction::Surveil(0)];
        resolution.pc = 0;
        g.state.pending = Pending::Priority;
        g.run_program();
        assert!(g.state.pending_triggers.is_empty());
        assert!(
            g.state
                .stack
                .iter()
                .all(|item| item.source_card != Card::from_name("Denzilore Fatehold").unwrap())
        );
    }
}

#[test]
fn fra_scry_or_surveil_positive_on_empty_library_still_triggers() {
    for p in [Player::P0, Player::P1] {
        for program in [&[Instruction::Scry(1)][..], &[Instruction::Surveil(2)][..]] {
            let mut g = scenario(p);
            let denz = put(&mut g, p, "Denzilore Fatehold", Zone::Battlefield);
            scry(&mut g, p);
            wait_for_choice(&mut g);
            let resolution = g.state.resolving.as_mut().unwrap();
            resolution.library = None;
            resolution.program = program;
            resolution.pc = 0;
            g.state.players[p.index()].library.clear();
            g.state.pending = Pending::Priority;
            g.run_program();
            settle(&mut g);
            assert_eq!(g.counter_count(denz, CounterKind::PlusOne), 1);
        }
    }
}

#[test]
fn fra_proft_payment_is_optional_and_draws_after_scry() {
    for p in [Player::P0, Player::P1] {
        for paid in [false, true] {
            let mut g = scenario(p);
            let proft = put(&mut g, p, "Proft, Consulting Detective", Zone::Battlefield);
            let top = put(&mut g, p, "Island", Zone::Library);
            let hand = g.hand_count(p);
            scry(&mut g, p);
            wait_for_choice(&mut g);
            arrange(&mut g, LibraryDestination::Top);
            for _ in 0..32 {
                if matches!(g.state.pending, Pending::ResolutionMana { .. }) {
                    break;
                }
                act(&mut g, Action::Pass);
            }
            assert!(matches!(g.state.pending, Pending::ResolutionMana { .. }));
            super::super::contracts::branch_contract(g.clone());
            if paid {
                choose(&mut g, |a| matches!(a, Action::PayResolutionMana { .. }));
            } else {
                act(&mut g, Action::DeclineResolutionMana);
            }
            settle(&mut g);
            assert_eq!(
                g.counter_count(proft, CounterKind::PlusOne),
                u64::from(paid)
            );
            // The spell draws the top card before Proft's ability resolves.
            assert_eq!(g.state.objects[top as usize].zone, Zone::Hand);
            assert_eq!(g.hand_count(p), hand + 1 + usize::from(paid));
        }
    }
}
