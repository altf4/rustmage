use super::*;

#[test]
fn schema186_cross_feature_rollback_keeps_published_pyre_state_representable() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let hidden = put(&mut g, player, "Bear Cub", Zone::Hand);
        g.state.objects[hidden as usize].card = Card::from_name("Pyre Rhymer // Molten Tide").unwrap();
        g.state.players[player.index()].mountain_mana_bonus = 1;
        let quality = mana_lots::Quality {
            restriction: ManaRestrictionView::NoncreatureSpell,
            spell_bonus: ManaSpellBonusView::None,
            retention: crate::cards::ManaRetention::Step,
            copy_source: None,
            treasure_source: false,
        };
        g.add_quality_mana(player, [0, 1, 0, 0, 0, 0], Some(quality));
        let recipient = g.object_ref(source);
        g.state.roles.get_or_insert_default().until_next_turn_stats.push(UntilNextTurnStatsView {
            recipient, controller: player, power: -4, toughness: -1,
        });
        let mut inner = g.activation_item(player, source, 0);
        inner.loyalty_x = Some(u64::from(u32::MAX) + 17);
        let mut outer = g.activation_item(player, source, 0);
        outer.copy_event = Some(Box::new(stack_copies::CopyEvent { spell: inner, amount: 1 }));
        g.state.pending_triggers.push(outer);
        assert!(!prior_schema186_state_supported(&g.state));
        for viewer in [player, player.other()] {
            let view = serde_json::to_string(&g.observe(viewer)).unwrap();
            assert!(project_prior_schema186(&view).is_err());
        }
        g.begin_attack_rollback(player);
        g.state.pending_triggers.clear();
        g.state.roles.as_mut().unwrap().until_next_turn_stats.clear();
        g.state.mana_lots = None;
        // The checkpoint still owns all three schema-186 features.
        assert!(!prior_schema186_state_supported(&g.state));
        let saved = g.snapshot();
        g.state.cast_rollback = None;
        assert!(prior_schema186_state_supported(&g.state));
        assert!(!prior_schema185_state_supported(&g.state));
        for viewer in [player, player.other()] {
            let view = serde_json::to_string(&g.observe(viewer)).unwrap();
            assert_eq!(project_prior_schema186(&view).unwrap(), view);
            assert!(project_prior_schema185(&view).is_err());
        }
        g.restore(&saved).unwrap();
        assert!(!prior_schema186_state_supported(&g.state));
    }
}
