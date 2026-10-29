//! Independent target-selector tests. Uldaros remains gated.
use super::*;

const OWN_NONLAND: TargetRule = TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 0,
    relation: 0,
    other: false,
    nonland: true,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const EACH_TYPE: TargetRule = TargetRule::Grouped {
    base: &OWN_NONLAND,
    constraint: TargetGroupConstraint::DistinctCardTypeSlots,
};

#[test]
fn graveyard_type_slots_allow_reassignment_but_reject_duplicates_and_collisions() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let dual = put(&mut g, player, "Solemn Simulacrum", Zone::Graveyard);
        let creature = put(&mut g, player, "Bear Cub", Zone::Graveyard);
        let another_creature = put(&mut g, player, "Bear Cub", Zone::Graveyard);
        let artifact = put(&mut g, player, "Gilded Lotus", Zone::Graveyard);
        let land = put(&mut g, player, "Forest", Zone::Graveyard);
        let enemy = put(&mut g, player.other(), "Shock", Zone::Graveyard);
        let item = g.activation_item(player, source, 0);
        assert!(g.item_target_matches(&item, EACH_TYPE, Target::Card(dual)));
        assert!(!g.item_target_matches(&item, EACH_TYPE, Target::Card(land)));
        assert!(!g.item_target_matches(&item, EACH_TYPE, Target::Card(enemy)));
        let chosen = vec![g.bind_target(Target::Card(dual))];
        assert!(!g.target_group_compatible(EACH_TYPE, 0, &chosen, Target::Card(dual)));
        assert!(g.target_group_compatible(EACH_TYPE, 0, &chosen, Target::Card(creature)));
        assert!(g.target_group_compatible(EACH_TYPE, 0, &chosen, Target::Card(artifact)));
        let chosen = vec![
            g.bind_target(Target::Card(dual)),
            g.bind_target(Target::Card(creature)),
        ];
        assert!(!g.target_group_compatible(EACH_TYPE, 0, &chosen, Target::Card(another_creature)));
        assert!(!g.target_group_compatible(EACH_TYPE, 0, &chosen, Target::Card(artifact)));
        let snapshot = g.snapshot();
        g.move_object(dual, Zone::Exile);
        assert!(!g.target_group_compatible(EACH_TYPE, 0, &chosen, Target::Card(artifact)));
        g.restore(&snapshot).unwrap();
        assert!(g.item_target_matches(&item, EACH_TYPE, Target::Card(dual)));
        assert!(g.target_group_compatible(
            EACH_TYPE,
            0,
            &[g.bind_target(Target::Card(dual))],
            Target::Card(creature)
        ));
    }
}
