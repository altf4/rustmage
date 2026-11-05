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
    let count = StatValue::Count(CountFilter {
        zone: CountZone::Exile,
        relation: 0,
        kind: 0,
        exclude: 0,
        subtype: None,
        basic: false,
        other: false,
        cards_only: true,
    });
    assert_eq!(g.stat_value(count, g.object_ref(swarm), p), 1);

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
