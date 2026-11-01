use super::*;

#[test]
fn moon_changes_creature_to_abilityless_land_across_copy_and_restore() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let victim = put(&mut g, p, "Serra Angel", Zone::Battlefield);
        let aura = put(&mut g, p, "Imprisoned in the Moon", Zone::Hand);
        cast(&mut g, aura, Some(Target::Card(victim)));
        settle(&mut g);
        assert_eq!(g.current_kinds(victim), 2);
        assert!(!g.keywords(victim).contains(Keywords::FLYING));
        assert!(!g.printed_abilities_available(victim));
        assert!(g.legal_actions().iter().any(|a| matches!(a,
            Action::TapGrantedMana { card, production, .. }
                if *card == victim && *production == [0,0,0,0,0,1])));
        let before = g.snapshot();
        let reference = g.object_ref(victim);
        g.apply_temporary_copy(
            reference,
            CopyRecipe::new(Card::from_name("Bear Cub").unwrap()),
            temporary_copy::CopyDeadline::EndOfTurn,
        );
        assert_eq!(g.current_kinds(victim), 2);
        assert!(!g.printed_abilities_available(victim));
        g.move_object(aura, Zone::Graveyard);
        assert_eq!(g.current_kinds(victim), 1);
        assert_eq!(stats(&g, victim), (2, 2));
        g.restore(&before).unwrap();
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.current_kinds(victim), 2);
        g.move_object(aura, Zone::Graveyard);
        assert!(g.is_creature(victim));
        assert!(g.keywords(victim).contains(Keywords::FLYING));
    }
}
