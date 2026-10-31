//! Exact source identity for a craft ability's exile-cost return.
use super::*;

fn exile_packet(g: &mut Game, source: CardId, controller: Player) -> SpellView {
    let mut packet = g.activation_item(controller, source, 0);
    let event = g.move_object(source, Zone::Exile);
    assert!(event.moved);
    packet.source_move = Some(Box::new(TriggerSourceMoveView {
        before: event.before.reference,
        after: event.after,
        origin: event.before.zone,
        destination: event.destination,
    }));
    packet
}

#[test]
fn craft_return_uses_the_exact_exiled_source_and_owner() {
    for owner in [Player::P0, Player::P1] {
        let mut g = scenario(owner);
        let source = put(&mut g, owner, "Oko, Lorwyn Liege", Zone::Battlefield);
        g.state.objects[source as usize].controller = owner.other();
        let packet = exile_packet(&mut g, source, owner.other());
        let checkpoint = g.snapshot();
        let mut branch = g.clone();
        g.return_crafted_source_transformed(&packet);
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        assert_eq!(g.object_view(source).controller, owner);
        assert_eq!(g.object_view(source).card.name(), "Oko, Shadowmoor Scion");
        branch.return_crafted_source_transformed(&packet);
        assert_eq!(g.snapshot(), branch.snapshot());
        g.restore(&checkpoint).unwrap();
        g.return_crafted_source_transformed(&packet);
        assert_eq!(g.snapshot(), branch.snapshot());
    }
}

#[test]
fn craft_return_rejects_a_later_exile_incarnation() {
    for owner in [Player::P0, Player::P1] {
        let mut g = scenario(owner);
        let source = put(&mut g, owner, "Oko, Lorwyn Liege", Zone::Battlefield);
        let packet = exile_packet(&mut g, source, owner);
        g.move_object(source, Zone::Graveyard);
        g.move_object(source, Zone::Exile);
        let before = g.snapshot();
        g.return_crafted_source_transformed(&packet);
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.object_view(source).zone, Zone::Exile);
    }
}

#[test]
fn craft_artifact_cost_uses_control_on_battlefield_and_ownership_in_graveyard() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Short Bow", Zone::Battlefield);
        let own = put(&mut g, player, "Short Bow", Zone::Battlefield);
        let grave = put(&mut g, player, "Short Bow", Zone::Graveyard);
        let theirs = put(&mut g, player.other(), "Short Bow", Zone::Battlefield);
        let their_grave = put(&mut g, player.other(), "Short Bow", Zone::Graveyard);
        let creature = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let stolen = put(&mut g, player.other(), "Short Bow", Zone::Battlefield);
        g.state.objects[stolen as usize].controller = player;
        let cost = AdditionalCost {
            kind: EffectChoiceKind::ExileControlledOrGraveyard,
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 16,
                nonland: false,
                other: true,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: None,
            },
        };
        let src = g.object_ref(source);
        for valid in [own, grave, stolen] {
            assert!(g.additional_candidate(cost, player, src, g.object_ref(valid)));
        }
        for invalid in [source, theirs, their_grave, creature] {
            assert!(!g.additional_candidate(cost, player, src, g.object_ref(invalid)));
        }
        let old = g.object_ref(own);
        g.move_object(own, Zone::Exile);
        g.move_object(own, Zone::Battlefield);
        assert!(!g.additional_candidate(cost, player, src, old));
    }
}

#[test]
fn craft_graveyard_material_cannot_be_exiled_through_a_prohibition() {
    use crate::engine::ability_world::TEST_EXILE_PROHIBITOR;
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            TEST_EXILE_PROHIBITOR.with(|p| p.set(None));
        }
    }
    let _reset = Reset;
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Short Bow", Zone::Battlefield);
        let grave = put(&mut g, player, "Short Bow", Zone::Graveyard);
        let provider = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        let cost = AdditionalCost {
            kind: EffectChoiceKind::ExileControlledOrGraveyard,
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 16,
                nonland: false,
                other: true,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: None,
            },
        };
        let grave_ref = g.object_ref(grave);
        let goal = FundingGoal {
            actor: player,
            required: [0; 6],
            generic: 1,
            reserve: Some((grave_ref, cost)),
            source: None,
        };
        assert!(g.can_fund(goal.clone()));
        TEST_EXILE_PROHIBITOR.with(|p| p.set(Some(g.object_ref(provider))));
        assert!(!g.additional_candidate(cost, player, g.object_ref(source), grave_ref));
        assert!(!g.can_fund(goal));
        TEST_EXILE_PROHIBITOR.with(|p| p.set(None));
    }
}
