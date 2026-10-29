use super::*;

fn artifact_animation(g: &mut Game, id: CardId, base: Option<(i128, i128)>) {
    let object = g.object_ref(id);
    let timestamp = g.next_role_timestamp();
    g.state.roles.get_or_insert_with(Default::default).animations.push(AnimationView {
        object, timestamp, artifact: true, land: false, preserve_card_types: true,
        food_only: false, artifact_noncreature: false, colors: None,
        creature_types: None, all_creature_types: false, base,
        keywords: Keywords::NONE, expires_turn: None,
    });
}

#[test]
fn puppet_targets_artifact_aura_and_both_fall_off_after_it_becomes_creature() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let target = put(&mut g, p, "Witness Protection", Zone::Battlefield);
        g.state.objects[target as usize].attached_to = Some(g.object_ref(bear));
        artifact_animation(&mut g, target, None);
        assert_ne!(g.current_kinds(target) & 16, 0);
        assert!(ability_world::CharacteristicWorld::is_aura(&g, target));
        let puppet = put(&mut g, p, "Puppet Crafting", Zone::Hand);
        let saved = g.snapshot();
        super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 { g.restore(&saved).unwrap(); }
            cast(&mut g, puppet, Some(Target::Card(target)));
            settle(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Graveyard);
            assert_eq!(g.object_view(puppet).zone, Zone::Graveyard);
            assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
        }
    }
}

#[test]
fn puppet_falls_off_when_enchanted_permanent_loses_all_permitted_types() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let target = put(&mut g, p, "Hedron Archive", Zone::Battlefield);
        let puppet = put(&mut g, p, "Puppet Crafting", Zone::Hand);
        cast(&mut g, puppet, Some(Target::Card(target)));
        settle(&mut g);
        assert_eq!(stats(&g, target), (5, 5));
        let saved = g.snapshot();
        super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 { g.restore(&saved).unwrap(); }
            g.apply_temporary_copy(g.object_ref(target), CopyRecipe::new(Card::from_name("Bear Cub").unwrap()), CopyDeadline::EndOfTurn);
            assert!(!g.aura_can_enchant(g.state.objects[puppet as usize].card, p, g.object_ref(target)));
            g.state_based_actions();
            assert_eq!(g.object_view(puppet).zone, Zone::Graveyard);
            assert_eq!(g.object_view(target).zone, Zone::Battlefield);
        }
    }
}

#[test]
fn puppet_reattachment_restarts_its_layer_timestamp() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let first = put(&mut g, p, "Hedron Archive", Zone::Battlefield);
        let second = put(&mut g, p, "Hedron Archive", Zone::Battlefield);
        let puppet = put(&mut g, p, "Puppet Crafting", Zone::Hand);
        cast(&mut g, puppet, Some(Target::Card(first)));
        settle(&mut g);
        let second_ref = g.object_ref(second);
        // This independent, later 3/3 effect precedes the synthetic reattachment.
        let timestamp = g.next_role_timestamp();
        g.state.roles.get_or_insert_with(Default::default).animations.push(AnimationView {
            object: second_ref, timestamp, artifact: false, land: false, preserve_card_types: false,
            food_only: false, artifact_noncreature: false, colors: None,
            creature_types: None, all_creature_types: false, base: Some((3, 3)),
            keywords: Keywords::NONE, expires_turn: None,
        });
        assert_eq!(stats(&g, second), (3, 3));
        assert!(g.aura_can_enchant(g.state.objects[puppet as usize].card, p, second_ref));
        let saved = g.snapshot();
        super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 { g.restore(&saved).unwrap(); }
            let aura_ref = g.object_ref(puppet);
            // Model an Aura-moving effect; preserve its incarnation and retimestamp.
            g.state.objects[puppet as usize].attached_to = Some(second_ref);
            g.retimestamp_live_static_source(aura_ref);
            assert!(!g.is_creature(first));
            assert_eq!(stats(&g, second), (5, 5));
            assert!(g.has_subtype(second, "Construct"));
        }
    }
}
