mod support;
use rustmage as _;
use rustmage_core::engine::*;
use support::*;

#[test]
fn airbent_adventure_cannot_use_its_printed_mana_cost() {
    let p = Player::P0;
    let mut g = scenario(p);
    let sword = put(
        &mut g,
        p,
        "Callous Sell-Sword // Burn Together",
        Zone::Battlefield,
    );
    let appa = put(&mut g, p, "Appa, Steadfast Guardian", Zone::Hand);

    cast(&mut g, appa, None);
    for _ in 0..8 {
        if matches!(g.state.pending, Pending::Targets { .. }) {
            break;
        }
        act(&mut g, Action::Pass);
    }
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(sword),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(&mut g, Action::FinishTargets);
    }
    settle(&mut g);
    assert_eq!(g.object_view(sword).zone, Zone::Exile);

    let airbend_adventure = Action::AnnounceAirbend {
        card: sword,
        face: CastFace::Adventure,
        flash_surcharge: false,
    };
    // The Adventure is a legal face if the actual airbend cost is affordable.
    let mut two_mana = g.clone();
    two_mana.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
    let index = two_mana
        .legal_actions()
        .iter()
        .position(|action| *action == airbend_adventure)
        .expect("Adventure may be cast for the airbend cost");
    let checkpoint = two_mana.snapshot();
    let mut indexed = two_mana.clone();
    indexed.apply_index(p, two_mana.revision(), index).unwrap();
    act(&mut two_mana, airbend_adventure.clone());
    assert_eq!(indexed.snapshot(), two_mana.snapshot());
    act(
        &mut two_mana,
        Action::ChooseTarget {
            target: Target::Card(appa),
        },
    );
    act(
        &mut two_mana,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    if two_mana.legal_actions().contains(&Action::FinishTargets) {
        act(&mut two_mana, Action::FinishTargets);
    }
    pay(&mut two_mana);
    assert_eq!(two_mana.object_view(sword).zone, Zone::Stack);
    assert_eq!(two_mana.spell_characteristics()[0].name, "Burn Together");
    assert_eq!(two_mana.state.players[p.index()].mana, [0; 6]);

    indexed.restore(&checkpoint).unwrap();
    assert!(indexed.legal_actions().contains(&airbend_adventure));

    g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
    let printed_cost = Action::AnnounceAdventure { card: sword };
    reject(&mut g, airbend_adventure);
    reject(&mut g, printed_cost.clone());

    // A second, ordinary exile permission independently authorizes the same
    // spell face at its printed cost.
    g.grant_exile_play_permission(p, g.object_ref(sword), false);
    assert!(g.legal_actions().contains(&printed_cost));
}
