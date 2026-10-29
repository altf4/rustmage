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
        let copy = g.allocate_card_copy(Card::from_name("Bear Cub").unwrap(), player);
        g.state.objects[copy.card as usize].zone = Zone::Graveyard;
        assert!(!g.item_target_matches(&item, EACH_TYPE, Target::Card(copy.card)));
        let mut measured = item.clone();
        let before = measured.owned_bytes();
        measured.target_type_slots = Some(Box::new(vec![1, 16]));
        assert_eq!(
            measured.owned_bytes() - before,
            std::mem::size_of::<Vec<u16>>() + 2 * std::mem::size_of::<u16>()
        );
        measured.target_type_slots = None;
        measured.target_kinds_lki = Some(Box::new(vec![(g.object_ref(dual), 17)]));
        assert_eq!(
            measured.owned_bytes() - before,
            std::mem::size_of::<Vec<(ObjectRef, u16)>>() + std::mem::size_of::<(ObjectRef, u16)>()
        );
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

#[test]
fn uldaros_actual_target_actions_freeze_explicit_slots_and_restore() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let echo = put(
            &mut g,
            player,
            "Echo, Perceptive Prodigy",
            Zone::Battlefield,
        );
        let dual = put(&mut g, player, "Solemn Simulacrum", Zone::Graveyard);
        let creature = put(&mut g, player, "Bear Cub", Zone::Graveyard);
        let artifact = put(&mut g, player, "Gilded Lotus", Zone::Graveyard);
        let replacement = put(&mut g, player, "Bear Cub", Zone::Graveyard);
        let foreign = put(&mut g, player.other(), "Shock", Zone::Graveyard);
        let land = put(&mut g, player, "Forest", Zone::Graveyard);
        let source = put(&mut g, player, "Shock", Zone::Hand);
        g.state.objects[source as usize].card = Card::test_index(4739);
        act(&mut g, Action::AnnounceCast { card: source });
        pay(&mut g);
        for _ in 0..40 {
            if matches!(g.state.pending, Pending::Targets { .. }) {
                break;
            }
            act(&mut g, Action::Pass);
        }
        let typed = |id, type_mask| Action::ChooseTargetCardType {
            target: Target::Card(id),
            type_mask,
        };
        assert!(g.legal_actions().contains(&typed(dual, 1)));
        assert!(g.legal_actions().contains(&typed(dual, 16)));
        assert!(!g.legal_actions().contains(&typed(land, 2)));
        assert!(!g.legal_actions().contains(&typed(foreign, 8)));
        let saved = g.snapshot();
        for _ in 0..2 {
            act(&mut g, typed(dual, 16));
            assert!(!g.legal_actions().contains(&typed(dual, 1)));
            assert!(!g.legal_actions().contains(&typed(artifact, 16)));
            assert!(g.legal_actions().contains(&typed(creature, 1)));
            act(&mut g, typed(creature, 1));
            act(&mut g, Action::FinishTargets);
            let item = g.state.stack.last().unwrap();
            let slots = item.target_type_slots.as_ref().unwrap();
            assert_eq!(slots.len(), 2);
            for (target, slot) in item.targets.iter().zip(slots.iter()) {
                match target.target {
                    Target::Card(id) if id == dual => assert_eq!(*slot, 16),
                    Target::Card(id) if id == creature => assert_eq!(*slot, 1),
                    _ => panic!("unexpected target"),
                }
            }
            let template = item.clone();
            let creature_index = template
                .targets
                .iter()
                .position(|target| target.target == Target::Card(creature))
                .unwrap();
            g.move_object(dual, Zone::Exile);
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: echo,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Stack(template.id),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            pay(&mut g);
            act(&mut g, Action::Pass);
            act(&mut g, Action::Pass);
            assert!(!g.legal_actions().contains(&Action::SetStackTarget {
                index: creature_index as u32,
                target: Target::Card(artifact),
            }));
            let changed = Action::SetStackTarget {
                index: creature_index as u32,
                target: Target::Card(replacement),
            };
            assert!(g.legal_actions().contains(&changed));
            act(&mut g, changed);
            assert!(g.legal_actions().contains(&Action::FinishStackTargets));
            act(&mut g, Action::FinishStackTargets);
            let copy = g.state.stack.last().unwrap();
            assert_eq!(copy.target_type_slots, template.target_type_slots);
            assert!(
                copy.targets
                    .iter()
                    .any(|target| target.target == Target::Card(dual))
            );
            assert!(
                copy.targets
                    .iter()
                    .any(|target| target.target == Target::Card(replacement))
            );
            g.restore(&saved).unwrap();
        }
    }
}
