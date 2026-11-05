mod support;
use rustmage as _;
use rustmage_core::{
    cards::{CounterKind, Counters},
    engine::*,
};
use support::*;

#[test]
fn jace_plot_should_allow_casting_the_back_face_on_a_later_turn() {
    let p = Player::P0;
    let mut g = scenario(p);
    let jace = put(&mut g, p, "Jace Reawakened", Zone::Battlefield);
    g.add_counter(jace, CounterKind::Loyalty, 3);
    let tony = put(
        &mut g,
        p,
        "Tony Stark // The Invincible Iron Man",
        Zone::Hand,
    );
    let back = Action::AnnouncePlottedFace {
        card: tony,
        face: CastFace::Back,
    };
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
    let object = g.object_ref(tony);
    act(&mut g, Action::SelectResolutionExile { object });
    assert_eq!(g.object_view(tony).zone, Zone::Exile);
    reject(&mut g, Action::AnnouncePlotted { card: tony });
    reject(&mut g, back.clone());

    for _ in 0..80 {
        if g.state.active == p && g.state.step == Step::Main1 && g.state.turn > plotted_turn {
            break;
        }
        match g.state.pending {
            Pending::Attackers => act(&mut g, Action::FinishAttackers),
            Pending::Blockers => act(&mut g, Action::FinishBlockers),
            _ => act(&mut g, Action::Pass),
        }
    }
    assert!(g.state.turn > plotted_turn);
    assert_eq!(g.state.active, p);
    assert_eq!(g.state.step, Step::Main1);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    let front = Action::AnnouncePlotted { card: tony };
    let mut front_branch = g.clone();
    act(&mut front_branch, front);
    pay(&mut front_branch);
    assert_eq!(front_branch.spell_characteristics()[0].name, "Tony Stark");

    let index = g
        .legal_actions()
        .iter()
        .position(|a| *a == back)
        .expect("plot allows the selected modal spell face");
    let checkpoint = g.snapshot();
    let mut indexed = g.clone();
    indexed.apply_index(p, g.revision(), index).unwrap();
    act(&mut g, back.clone());
    assert_eq!(indexed.snapshot(), g.snapshot());
    pay(&mut g);
    assert_eq!(g.spell_characteristics()[0].face, CastFace::Back);
    settle(&mut g);
    assert_eq!(g.object_view(tony).name, "The Invincible Iron Man");
    indexed.restore(&checkpoint).unwrap();
    assert!(indexed.legal_actions().contains(&back));
}

#[test]
fn plot_allows_adventure_spells_and_excludes_transforming_back_faces() {
    let p = Player::P0;
    let mut g = scenario(p);
    let adventurer = put(
        &mut g,
        p,
        "Callous Sell-Sword // Burn Together",
        Zone::Exile,
    );
    let transformer = put(
        &mut g,
        p,
        "Aang, Swift Savior // Aang and La, Ocean's Fury",
        Zone::Exile,
    );
    let creature = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    g.remember_plotted(p, g.object_ref(adventurer));
    g.remember_plotted(p, g.object_ref(transformer));
    g.state.turn += 1;
    g.state.players[p.index()].mana = [0; 6];
    reject(
        &mut g,
        Action::AnnouncePlottedFace {
            card: transformer,
            face: CastFace::Back,
        },
    );
    act(
        &mut g,
        Action::AnnouncePlottedFace {
            card: adventurer,
            face: CastFace::Adventure,
        },
    );
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(creature),
        },
    );
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(&mut g, Action::FinishTargets);
    }
    pay(&mut g);
    assert_eq!(g.spell_characteristics()[0].name, "Burn Together");
    settle(&mut g);
    assert_eq!(g.object_view(adventurer).zone, Zone::Exile);
    assert_eq!(g.state.players[p.other().index()].life, 18);
    reject(&mut g, Action::AnnouncePlotted { card: adventurer });
}
