//! Additional timing and replacement interactions for FRA scry consumers.
use super::*;
use crate::cards::StaticAbilityFragment;

fn wait_scry_choice(g: &mut Game) {
    for _ in 0..48 {
        if matches!(g.state.pending, Pending::EffectLibrary { .. }) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("expected scry or surveil choice");
}

fn keep_top(g: &mut Game) {
    while matches!(g.state.pending, Pending::EffectLibrary { .. }) {
        choose(g, |a| {
            matches!(
                a,
                Action::ArrangeLibrary {
                    destination: LibraryDestination::Top,
                    ..
                }
            )
        });
    }
}

fn cast_opt(g: &mut Game, p: Player) {
    let opt = put(g, p, "Opt", Zone::Hand);
    cast(g, opt, None);
}

#[test]
fn futurescribe_checks_scry_history_when_its_trigger_resolves() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Desperate Futurescribe", Zone::Battlefield);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.begin_step(Step::BeginCombat);
        g.prepare_triggers();
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        assert!(!g.state.players[p.index()].scried_or_surveilled_this_turn);
        g.state.priority = p;
        cast_opt(&mut g, p);
        wait_scry_choice(&mut g);
        keep_top(&mut g);
        assert!(g.state.players[p.index()].scried_or_surveilled_this_turn);
        super::super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 1);
        g.finish_cleanup();
        assert_eq!(stats(&g, bear), (3, 3));
    }
}

#[test]
fn phantasm_loses_and_recovers_its_conditional_attack_permission() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let phantasm = put(&mut g, p, "Surveillance Phantasm", Zone::Battlefield);
        cast_opt(&mut g, p);
        wait_scry_choice(&mut g);
        keep_top(&mut g);
        settle(&mut g);
        let rule = |f: &StaticAbilityFragment| {
            matches!(f, StaticAbilityFragment::AttackWithoutDefenderIfScried)
        };
        assert!(g.continuous_fragment_available(phantasm, rule));
        assert!(g.can_declare_attacker(p, phantasm));
        let aura = put(&mut g, p.other(), "Fresh Start", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(phantasm));
        assert!(!g.continuous_fragment_available(phantasm, rule));
        let ability = Action::AnnounceActivate {
            card: phantasm,
            ability: 0,
            counter_payment: Counters::default(),
        };
        reject(&mut g, ability.clone());
        g.move_object(aura, Zone::Graveyard);
        assert!(g.continuous_fragment_available(phantasm, rule));
        assert!(g.legal_actions().contains(&ability));
        assert!(g.can_declare_attacker(p, phantasm));
    }
}

#[test]
fn proctor_observes_own_and_other_entries_but_not_opposing_entries() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let proctor = put(&mut g, p, "Proctor of Potential", Zone::Hand);
        cast(&mut g, proctor, None);
        wait_scry_choice(&mut g);
        keep_top(&mut g);
        settle(&mut g);
        let own = put(&mut g, p, "Bear Cub", Zone::Hand);
        cast(&mut g, own, None);
        wait_scry_choice(&mut g);
        keep_top(&mut g);
        settle(&mut g);
        g.state.active = p.other();
        g.state.priority = p.other();
        g.state.step = Step::Main1;
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Hand);
        cast(&mut g, enemy, None);
        settle(&mut g);
        assert!(g.state.pending_triggers.is_empty());
        assert!(!matches!(g.state.pending, Pending::EffectLibrary { .. }));
    }
}

#[test]
fn proctor_returned_with_finality_is_exiled_when_it_would_die() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let proctor = put(&mut g, p, "Proctor of Potential", Zone::Graveyard);
        cast_opt(&mut g, p);
        wait_scry_choice(&mut g);
        keep_top(&mut g);
        settle(&mut g);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: proctor,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        wait_scry_choice(&mut g);
        assert_eq!(g.counter_count(proctor, CounterKind::Finality), 1);
        keep_top(&mut g);
        settle(&mut g);
        g.move_object(proctor, Zone::Graveyard);
        assert_eq!(g.object_view(proctor).zone, Zone::Exile);
    }
}

#[test]
fn denzilore_counts_creatures_controlled_at_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Denzilore Fatehold", Zone::Battlefield);
        let departed = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        cast_opt(&mut g, p);
        wait_scry_choice(&mut g);
        keep_top(&mut g);
        g.move_object(departed, Zone::Exile);
        let late = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let opponent = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        super::super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
        assert_eq!(g.counter_count(late, CounterKind::PlusOne), 1);
        assert_eq!(g.counter_count(departed, CounterKind::PlusOne), 0);
        assert_eq!(g.counter_count(opponent, CounterKind::PlusOne), 0);
    }
}
