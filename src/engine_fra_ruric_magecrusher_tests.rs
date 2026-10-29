use super::*;

fn hit(g: &mut Game, source: DamageSource, target: Player, combat: bool, amount: i128) {
    g.apply_damage_batch(&[DamageEvent {
        source, target: Target::Player(target), amount, combat, preventable: true,
    }]);
}

#[test]
fn ruric_hexproof_tracks_actual_combat_damage_for_exact_incarnation() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let ruric = put(&mut g, player, "Ruric Thar, Magecrusher", Zone::Battlefield);
        assert!(g.keywords(ruric).contains(Keywords::HEXPROOF));
        let initial = g.snapshot();
        let source = g.damage_source(ruric);
        hit(&mut g, source, player.other(), false, 2);
        assert!(g.keywords(ruric).contains(Keywords::HEXPROOF));
        hit(&mut g, source, player.other(), true, 0);
        assert!(g.keywords(ruric).contains(Keywords::HEXPROOF));
        let mut prevented = source;
        prevented.keywords = prevented.keywords.union(Keywords::PREVENT_COMBAT_BY);
        hit(&mut g, prevented, player.other(), true, 2);
        assert!(g.keywords(ruric).contains(Keywords::HEXPROOF));
        hit(&mut g, source, player.other(), true, 2);
        assert!(!g.keywords(ruric).contains(Keywords::HEXPROOF));
        for viewer in [player, player.other()] {
            assert!(g.observe(viewer).objects.iter().find(|o| o.id == ruric).unwrap().dealt_combat_damage_this_incarnation);
        }
        g.begin_turn();
        assert!(!g.keywords(ruric).contains(Keywords::HEXPROOF));
        let current = g.object_ref(ruric);
        g.change_controller(current, player.other());
        assert!(!g.keywords(ruric).contains(Keywords::HEXPROOF));
        g.move_object(ruric, Zone::Exile);
        g.move_object(ruric, Zone::Battlefield);
        assert!(g.keywords(ruric).contains(Keywords::HEXPROOF));
        // Delayed damage from the old object cannot mark the new incarnation.
        hit(&mut g, source, player.other(), true, 2);
        assert!(g.keywords(ruric).contains(Keywords::HEXPROOF));
        g.restore(&initial).unwrap();
        assert!(g.keywords(ruric).contains(Keywords::HEXPROOF));
    }
}
