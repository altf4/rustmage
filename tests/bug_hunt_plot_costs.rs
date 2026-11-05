mod support;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

#[test]
fn plotting_dragons_prey_must_still_check_its_target_cost_increase() {
    let p = Player::P0;
    let mut g = scenario(p);
    let jace = put(&mut g, p, "Jace Reawakened", Zone::Battlefield);
    g.add_counter(jace, CounterKind::Loyalty, 3);
    let prey = put(&mut g, p, "Dragon's Prey", Zone::Hand);
    let dragon = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
    let plotted_turn = g.state.turn;
    act(
        &mut g,
        Action::AnnounceActivate {
            card: jace,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    pay(&mut g);
    for _ in 0..8 {
        if g.state.pending != Pending::Priority {
            break;
        }
        act(&mut g, Action::Pass);
    }
    let object = g.object_ref(prey);
    act(&mut g, Action::SelectResolutionExile { object });
    for _ in 0..80 {
        if g.state.active == p && g.state.step == Step::Main1 && g.state.turn > plotted_turn {
            break;
        }
        let action = match g.state.pending {
            Pending::Attackers => Action::FinishAttackers,
            Pending::Blockers => Action::FinishBlockers,
            Pending::Priority => Action::Pass,
            ref pending => panic!("unexpected pending choice {pending:?}"),
        };
        act(&mut g, action);
    }
    assert!(g.state.turn > plotted_turn);
    assert_eq!(g.state.active, p);
    assert_eq!(g.state.step, Step::Main1);
    assert_eq!(g.object_view(prey).zone, Zone::Exile);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);

    let cast_plotted = Action::AnnouncePlotted { card: prey };
    let choose_dragon = Action::ChooseTarget {
        target: Target::Card(dragon),
    };
    let mut funded = g.clone();
    funded.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
    act(&mut funded, cast_plotted.clone());
    act(&mut funded, choose_dragon);
    if funded.legal_actions().contains(&Action::FinishTargets) {
        act(&mut funded, Action::FinishTargets);
    }
    pay(&mut funded);
    settle(&mut funded);
    assert_eq!(funded.object_view(dragon).zone, Zone::Graveyard);

    reject(&mut g, cast_plotted.clone());

    // A free completion exists with a different target, but choosing the
    // Dragon still requires its two mana increase.
    let bear = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    let index = g
        .legal_actions()
        .iter()
        .position(|action| *action == cast_plotted)
        .expect("a plot cast with an affordable target");
    let checkpoint = g.snapshot();
    let mut indexed = g.clone();
    indexed.apply_index(p, g.revision(), index).unwrap();
    act(&mut g, cast_plotted.clone());
    assert_eq!(indexed.snapshot(), g.snapshot());
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(dragon),
        },
    );
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(&mut g, Action::FinishTargets);
    }
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
    assert_eq!(g.object_view(dragon).zone, Zone::Battlefield);
    indexed.restore(&checkpoint).unwrap();
    assert!(indexed.legal_actions().contains(&cast_plotted));
}
