//! Fixed layer-five colors in actual and borrowed speculative worlds.
use super::super::ability_timeline_tests::{cast, named, settle, setup};
use super::super::ability_world::CharacteristicWorld;
use super::*;
#[test]
fn ecl_green_fixed_color_timeline_matches_funding_aura_order_expiry_and_incarnations() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.set_color_effects(&[bear], 31, crate::cards::AnimationDuration::Permanent);
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 0,
            source: None,
            reserve: Some((
                g.object_ref(bear),
                AdditionalCost {
                    kind: EffectChoiceKind::Sacrifice,
                    filter: evidence::cost(0).filter,
                },
            )),
        };
        let world = FundingState::from_game(&g, &goal);
        assert_eq!(world.colors(bear), 31);
        assert!(world.guard_available(ActivationGuard::ControlColorsAtLeast(5), p));
        let aura = named(&mut g, p, "Witness Protection", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!(g.object_view(bear).colors, 17);
        g.set_color_effects(&[bear], 2, crate::cards::AnimationDuration::Permanent);
        assert_eq!(g.object_view(bear).colors, 2);
        let mut world = FundingState::from_game(&g, &goal);
        assert_eq!(world.colors(bear), 2);
        assert!(!world.guard_available(ActivationGuard::ControlColorsAtLeast(5), p));
        let index = world.resources.iter().position(|r| r.id == bear).unwrap();
        world.resources[index].object.zone = Zone::Graveyard;
        assert_eq!(
            world.colors(bear),
            Card::from_name("Bear Cub").unwrap().colors()
        );
        g.set_color_effects(&[bear], 31, crate::cards::AnimationDuration::EndOfTurn);
        assert_eq!(g.object_view(bear).colors, 31);
        super::super::ability_timeline_tests::next_turn(&mut g);
        assert_eq!(g.object_view(bear).colors, 2);
        let original = g.object_ref(bear);
        g.move_object(bear, Zone::Exile);
        g.move_object(bear, Zone::Battlefield);
        assert_ne!(g.object_ref(bear), original);
        assert_eq!(g.object_view(bear).colors, 16);
        assert!(
            !g.observe(p)
                .color_effects
                .iter()
                .any(|row| row.object == original)
        );
        // Mutating a branch's fixed color leaves its parent and restored state intact.
        let snapshot = g.snapshot();
        let mut child = g.clone();
        child.set_color_effects(&[bear], 1, crate::cards::AnimationDuration::Permanent);
        assert_eq!(child.object_view(bear).colors, 1);
        assert_eq!(g.object_view(bear).colors, 16);
        child.restore(&snapshot).unwrap();
        assert_eq!(child.snapshot(), g.snapshot());
    }
}

#[test]
fn ecl_green_fixed_colors_distinguish_otherwise_identical_funding_resource_keys() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let first = named(&mut g, p, "Forest", Zone::Battlefield);
        let second = named(&mut g, p, "Forest", Zone::Battlefield);
        named(&mut g, p, "Bloom Tender", Zone::Battlefield);
        g.set_color_effects(&[first], 2, crate::cards::AnimationDuration::Permanent);
        g.set_color_effects(&[second], 8, crate::cards::AnimationDuration::Permanent);
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 0,
            source: None,
            reserve: None,
        };
        let mut left = FundingState::from_game(&g, &goal);
        let mut right = left.clone();
        left.resources
            .iter_mut()
            .find(|r| r.id == first)
            .unwrap()
            .object
            .zone = Zone::Graveyard;
        right
            .resources
            .iter_mut()
            .find(|r| r.id == second)
            .unwrap()
            .object
            .zone = Zone::Graveyard;
        assert_ne!(left.controlled_colors(p), right.controlled_colors(p));
        assert!(left.key(&goal) != right.key(&goal));
    }
}
