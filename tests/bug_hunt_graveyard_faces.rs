mod support;
use rustmage as _;
use rustmage_core::PermanentType;
use rustmage_core::engine::*;
use support::*;

#[test]
fn muldrotha_can_cast_modal_back_as_artifact_after_creature_slot_is_used() {
    let p = Player::P0;
    let mut g = scenario(p);
    let muldrotha = put(&mut g, p, "Muldrotha, the Gravetide", Zone::Battlefield);
    let bear = put(&mut g, p, "Bear Cub", Zone::Graveyard);
    let collar = put(&mut g, p, "Basilisk Collar", Zone::Graveyard);
    let tony = put(
        &mut g,
        p,
        "Tony Stark // The Invincible Iron Man",
        Zone::Graveyard,
    );
    let permission = g.object_ref(muldrotha);
    let cast_creature = |card| Action::PlayGraveyardPermanent {
        card,
        permission,
        permanent_type: PermanentType::Creature,
    };
    let cast_artifact = Action::AnnounceGraveyardBack {
        card: tony,
        permission,
        permanent_type: PermanentType::Artifact,
    };

    act(&mut g, cast_creature(bear));
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
    assert!(!g.legal_actions().contains(&cast_creature(tony)));

    g.state.players[p.index()].mana = [0, 1, 0, 1, 0, 3];
    reject(&mut g, cast_artifact.clone());
    g.state.players[p.index()].mana[5] = 4;
    let index = g
        .legal_actions()
        .iter()
        .position(|action| *action == cast_artifact)
        .expect("the selected back face qualifies for the artifact allowance");
    let checkpoint = g.snapshot();
    let mut indexed = g.clone();
    indexed.apply_index(p, g.revision(), index).unwrap();
    act(&mut g, cast_artifact);
    assert_eq!(indexed.snapshot(), g.snapshot());
    pay(&mut g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    settle(&mut g);
    assert_eq!(g.object_view(tony).zone, Zone::Battlefield);
    assert_eq!(g.object_view(tony).card.name(), "The Invincible Iron Man");

    let cast_collar = Action::PlayGraveyardPermanent {
        card: collar,
        permission,
        permanent_type: PermanentType::Artifact,
    };
    g.state.players[p.index()].mana[5] = 1;
    reject(&mut g, cast_collar.clone());
    indexed.restore(&checkpoint).unwrap();
    assert!(indexed.legal_actions().contains(&cast_collar));
}
