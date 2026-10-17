use super::super::ability_world::CharacteristicWorld;
use super::super::tests::{scenario, spawn};
use super::*;

#[test]
fn subtype_layers_funding_uses_virtual_counters_exact_incarnations_and_replacement_membership() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario();
        let squire = spawn(
            &mut g,
            p,
            Card::from_name("Skyknight Squire").unwrap(),
            Zone::Battlefield,
        );
        g.state.objects[squire as usize].counters.plus_one = 3;
        let spec = |subtype| AdditionalCost {
            kind: EffectChoiceKind::Sacrifice,
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 1,
                nonland: false,
                other: false,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: Some(subtype),
            },
        };
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 0,
            source: None,
            reserve: Some((g.object_ref(squire), spec("Knight"))),
        };
        let original = g.snapshot();
        let mut world = FundingState::from_game(&g, &goal);
        assert!(world.obligations(&goal));
        let index = world.resources.iter().position(|r| r.id == squire).unwrap();
        assert!(
            world
                .keywords(&world.resources[index])
                .contains(Keywords::FLYING)
        );
        world.resources[index].object.counters.plus_one = 2;
        assert!(!world.obligations(&goal));
        assert!(
            !world
                .keywords(&world.resources[index])
                .contains(Keywords::FLYING)
        );
        assert_eq!(g.snapshot(), original);
        drop(world);

        let source = spawn(&mut g, p.other(), Card::Shock, Zone::Hand);
        let mut item = g.cast_item(p, source, [0; 6]);
        item.targets.push(BoundTarget {
            group: 0,
            target: Target::Card(squire),
            object: Some(g.object_ref(squire)),
        });
        g.set_creature_types(&item, Subject::Targets, &["Werewolf"]);
        g.move_object(source, Zone::Graveyard);
        assert!(
            g.has_subtype(squire, "Werewolf"),
            "source departure does not revoke a resolving effect"
        );
        let goal = FundingGoal {
            reserve: Some((g.object_ref(squire), spec("Werewolf"))),
            ..goal
        };
        let original = g.snapshot();
        let mut world = FundingState::from_game(&g, &goal);
        assert!(world.obligations(&goal));
        assert!(
            !world
                .granted_subtypes(squire)
                .has(world.object(squire).card, "Cat")
        );
        assert!(
            !world
                .granted_subtypes(squire)
                .has(world.object(squire).card, "Knight")
        );
        let index = world.resources.iter().position(|r| r.id == squire).unwrap();
        world.resources[index].object.generation += 1;
        assert!(
            !world
                .granted_subtypes(squire)
                .has(world.object(squire).card, "Werewolf")
        );
        world.resources[index].object.generation -= 1;
        world.resources[index].object.zone = Zone::Graveyard;
        assert!(!world.obligations(&goal));
        assert!(
            !world
                .granted_subtypes(squire)
                .has(world.object(squire).card, "Werewolf")
        );
        assert_eq!(g.snapshot(), original);
    }
}
