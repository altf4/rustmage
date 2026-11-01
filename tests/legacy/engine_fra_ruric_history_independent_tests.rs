use super::*;

fn combat_hit(g: &mut Game, creature: CardId, defender: Player) {
    let source = g.damage_source(creature);
    g.apply_damage_batch(&[DamageEvent {
        source,
        target: Target::Player(defender),
        amount: 1,
        combat: true,
        preventable: true,
    }]);
}

#[test]
fn ruric_copy_preserves_incarnation_history_through_copy_changes_and_restore() {
    for p in [Player::P0, Player::P1] {
        for damaged in [false, true] {
            let mut g = scenario(p);
            let creature = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            if damaged {
                combat_hit(&mut g, creature, p.other());
            }
            let ref_before = g.object_ref(creature);
            g.apply_temporary_copy(
                ref_before,
                CopyRecipe::new(Card::from_name("Ruric Thar, Magecrusher").unwrap()),
                CopyDeadline::EndOfTurn,
            );
            assert_eq!(g.keywords(creature).contains(Keywords::HEXPROOF), !damaged);
            let saved = g.snapshot();
            super::super::contracts::branch_contract(g.clone());
            g.apply_temporary_copy(
                ref_before,
                CopyRecipe::new(Card::from_name("Bear Cub").unwrap()),
                CopyDeadline::EndOfTurn,
            );
            if !damaged {
                combat_hit(&mut g, creature, p.other());
            }
            g.apply_temporary_copy(
                ref_before,
                CopyRecipe::new(Card::from_name("Ruric Thar, Magecrusher").unwrap()),
                CopyDeadline::EndOfTurn,
            );
            assert!(!g.keywords(creature).contains(Keywords::HEXPROOF));
            g.restore(&saved).unwrap();
            assert_eq!(g.keywords(creature).contains(Keywords::HEXPROOF), !damaged);
        }
    }
}

#[test]
fn ruric_records_damage_while_its_abilities_are_suppressed() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Ruric Thar, Magecrusher", Zone::Battlefield);
        let aura = put(&mut g, p.other(), "Witness Protection", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(source));
        assert!(!g.keywords(source).contains(Keywords::HEXPROOF));
        combat_hit(&mut g, source, p.other());
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        g.move_object(aura, Zone::Graveyard);
        assert!(!g.keywords(source).contains(Keywords::HEXPROOF));
        g.restore(&saved).unwrap();
        g.move_object(aura, Zone::Graveyard);
        assert!(!g.keywords(source).contains(Keywords::HEXPROOF));
    }
}
