use super::*;

#[test]
fn land_subtype_loss_and_regain_changes_funding_and_live_mana_together() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let land = put(&mut g, player, "Island", Zone::Battlefield);
        let source = g.object_ref(land);
        g.state.players[player.index()].mana = [0; 6];
        let blue = FundingGoal {
            actor: player,
            required: [0, 1, 0, 0, 0, 0],
            generic: 0,
            reserve: None,
            source: None,
        };
        let green = FundingGoal {
            actor: player,
            required: [0, 0, 0, 0, 1, 0],
            generic: 0,
            reserve: None,
            source: None,
        };
        assert!(g.can_fund(blue.clone()));
        let first = g.next_role_timestamp();
        g.state
            .roles
            .get_or_insert_with(Default::default)
            .animations
            .push(characteristic_layers::food_form(source, first));
        let second = g.next_role_timestamp();
        g.state
            .roles
            .get_or_insert_with(Default::default)
            .animations
            .push(AnimationView {
                object: source,
                timestamp: second,
                artifact: false,
                land: true,
                preserve_card_types: false,
                food_only: false,
                artifact_noncreature: false,
                enchantment_noncreature: false,
                colors: None,
                creature_types: None,
                all_creature_types: false,
                base: Some((1, 1)),
                keywords: Keywords::NONE,
                expires_turn: None,
            });
        assert!(!g.can_fund(blue));
        assert!(!g.can_fund(green.clone()));
        let item = g.cast_preview(player, land, Zone::Battlefield, source, 0, [0; 6]);
        g.add_basic_land_type(&item, Subject::Source, crate::cards::BasicLandType::Forest);
        assert!(g.can_fund(green));
        assert!(g.legal_actions().contains(&Action::TapMana { card: land }));
        let before = g.snapshot();
        g.apply(player, Action::TapMana { card: land }).unwrap();
        assert_eq!(g.state.players[player.index()].mana, [0, 0, 0, 0, 1, 0]);
        g.restore(&before).unwrap();
        assert_eq!(g.snapshot(), before);
    }
}
