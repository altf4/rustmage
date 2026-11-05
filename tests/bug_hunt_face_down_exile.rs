mod support;
use rustmage as _;
use rustmage_core::{
    cards::{CountFilter, CountZone, StatValue},
    engine::*,
};
use support::*;

#[test]
fn huskburster_swarm_ignores_creature_types_of_face_down_exiled_cards() {
    let p = Player::P0;
    let mut g = scenario(p);
    let cage = put(&mut g, p, "Collector's Cage", Zone::Hand);
    let bear = put(&mut g, p, "Bear Cub", Zone::Library);
    let swarm = put(&mut g, p, "Huskburster Swarm", Zone::Hand);
    cast(&mut g, cage, None);
    for _ in 0..12 {
        if matches!(g.state.pending, Pending::EffectLibrarySelection { .. }) {
            break;
        }
        choose(&mut g, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    act(&mut g, Action::SelectLibrary { card: bear });
    if g.legal_actions().contains(&Action::FinishLibrarySelection) {
        act(&mut g, Action::FinishLibrarySelection);
    }
    settle(&mut g);

    assert_eq!(g.object_view(bear).zone, Zone::Exile);
    assert!(g.can_view_hideaway(p, g.object_ref(bear)));
    assert!(!g.can_view_hideaway(p.other(), g.object_ref(bear)));
    assert_eq!(
        g.observe(p)
            .objects
            .iter()
            .find(|object| object.id == bear)
            .unwrap()
            .name,
        "Bear Cub"
    );
    assert!(
        !g.observe(p.other())
            .objects
            .iter()
            .any(|object| object.id == bear)
    );

    let hidden = g.object_view(bear);
    assert_eq!(hidden.kinds, 0);
    assert!(!hidden.has_subtype("Bear"));
    assert_eq!(g.object_mana_value(bear), 0);
    let filter = CountFilter {
        zone: CountZone::Exile,
        relation: 0,
        kind: 0,
        exclude: 0,
        subtype: None,
        basic: false,
        other: false,
        cards_only: true,
    };
    let count = StatValue::Count(filter);
    assert_eq!(g.stat_value(count, g.object_ref(swarm), p), 1);
    assert!(Game::count_object_matches(
        filter,
        g.object_ref(swarm),
        p,
        bear,
        &g.state.objects[bear as usize],
        &|_| panic!("an untyped count must not query subtypes"),
        &|_, _| panic!("an untyped count must not query card types"),
    ));

    g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 6];
    let cast_swarm = Action::AnnounceCast { card: swarm };
    reject(&mut g, cast_swarm.clone());
    assert!(
        !g.legal_actions()
            .iter()
            .any(|a| { matches!(a, Action::Cast { card, .. } if *card == swarm) })
    );

    g.reveal_hideaway_object(g.object_ref(bear));
    let revealed = g.object_view(bear);
    assert_eq!(revealed.kinds, 1);
    assert!(revealed.has_subtype("Bear"));
    assert_eq!(g.object_mana_value(bear), 2);
    assert_eq!(g.stat_value(count, g.object_ref(swarm), p), 1);
    act(&mut g, cast_swarm);
    pay(&mut g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert_eq!(g.object_view(swarm).zone, Zone::Stack);
}

#[test]
fn an_exiled_face_down_spell_uses_its_cast_face_to_check_protection_and_funding() {
    let player = Player::P0;
    let mut game = scenario(player);
    let robbery = put(&mut game, player, "Outrageous Robbery", Zone::Hand);
    let spell = put(
        &mut game,
        player.other(),
        "Seized from Slumber",
        Zone::Library,
    );
    let elenda = put(
        &mut game,
        player.other(),
        "Elenda, Saint of Dusk",
        Zone::Battlefield,
    );
    let eligible = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    game.state.objects[elenda as usize].tapped = true;
    game.state.players[player.index()].mana = [0, 0, 3, 0, 0, 0];
    act(&mut game, Action::AnnounceCast { card: robbery });
    act(&mut game, Action::ChooseX { value: 1 });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(player.other()),
        },
    );
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Exile);
    assert_eq!(game.object_view(spell).kinds, 0);

    game.state.players[player.index()].mana = [1, 0, 0, 0, 0, 1];
    let announce = Action::AnnounceExile { card: spell };
    reject(&mut game, announce.clone());
    game.state.objects[eligible as usize].tapped = true;
    act(&mut game, announce);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(elenda),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(eligible),
        },
    );
    pay(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    settle(&mut game);
    assert_eq!(game.object_view(eligible).zone, Zone::Graveyard);
}
