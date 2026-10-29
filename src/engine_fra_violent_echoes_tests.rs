use super::*;

fn controlled_jaces(g: &Game, p: Player) -> Vec<CardId> {
    g.state.objects.iter().enumerate().filter_map(|(id, o)|
        (o.zone == Zone::Battlefield && o.controller == p && o.is_token() && o.card == Card::JacePlaneswalkerToken).then_some(id as CardId)).collect()
}

#[test]
fn violent_echoes_empowers_by_actual_excess_on_creatures_and_planeswalkers() {
    for p in [Player::P0, Player::P1] {
        for (name, marked, loyalty, excess) in [("Bear Cub", 0, 0, 4), ("Bear Cub", 1, 0, 5), ("Ajani, Caller of the Pride", 0, 4, 2), ("Ajani, Caller of the Pride", 0, 6, 0)] {
            let mut g = scenario(p);
            let target = put(&mut g, p.other(), name, Zone::Battlefield);
            g.state.objects[target as usize].damage = marked;
            if loyalty > 0 { g.state.objects[target as usize].counters.loyalty = loyalty; }
            let spell = put(&mut g, p, "Violent Echoes", Zone::Hand);
            cast(&mut g, spell, Some(Target::Card(target)));
            let saved = g.snapshot();
            super::super::contracts::branch_contract(g.clone());
            for replay in 0..2 {
                if replay == 1 { g.restore(&saved).unwrap(); }
                settle(&mut g);
                let jaces = controlled_jaces(&g, p);
                if excess == 0 { assert!(jaces.is_empty()); }
                else {
                    assert_eq!(jaces.len(), 1);
                    assert_eq!(g.counter_count(jaces[0], CounterKind::Loyalty), excess);
                }
            }
        }
    }
}

#[test]
fn violent_echoes_no_excess_after_prevention_and_fizzles_when_target_leaves() {
    for p in [Player::P0, Player::P1] {
        for leaves in [false, true] {
            let mut g = scenario(p);
            let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            if !leaves { g.add_counter(target, CounterKind::Shield, 1); }
            let spell = put(&mut g, p, "Violent Echoes", Zone::Hand);
            cast(&mut g, spell, Some(Target::Card(target)));
            if leaves { g.move_object(target, Zone::Graveyard); }
            settle(&mut g);
            assert!(controlled_jaces(&g, p).is_empty());
            if !leaves {
                assert_eq!(g.object_view(target).zone, Zone::Battlefield);
                assert_eq!(g.object_view(target).damage, 0);
            }
        }
    }
}

#[test]
fn violent_echoes_uses_existing_jace_and_does_not_create_another() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        g.create_tokens(p, Card::JacePlaneswalkerToken, 1, false);
        let jace = controlled_jaces(&g, p)[0];
        g.add_counter(jace, CounterKind::Loyalty, 1);
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Violent Echoes", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(target)));
        settle(&mut g);
        assert_eq!(controlled_jaces(&g, p), vec![jace]);
        assert_eq!(g.counter_count(jace, CounterKind::Loyalty), 5);
    }
}

#[test]
fn violent_echoes_uses_greatest_excess_for_creature_planeswalker() {
    for p in [Player::P0, Player::P1] {
        for (loyalty, toughness) in [(2, 5), (5, 2)] {
            let mut g = scenario(p);
            let target = put(&mut g, p.other(), "Ajani, Caller of the Pride", Zone::Battlefield);
            g.state.objects[target as usize].counters.loyalty = loyalty;
            g.change_object_form(g.object_ref(target), AnimationSpec {
                artifact: false,
                creature_types: AnimationCreatureTypes::Unchanged,
                base: Some((2, toughness)),
                keywords: Keywords::NONE,
                duration: AnimationDuration::EndOfTurn,
            }, characteristic_layers::AnimationTypeMode::AddCreature);
            let spell = put(&mut g, p, "Violent Echoes", Zone::Hand);
            cast(&mut g, spell, Some(Target::Card(target)));
            settle(&mut g);
            let jaces = controlled_jaces(&g, p);
            assert_eq!(jaces.len(), 1);
            assert_eq!(g.counter_count(jaces[0], CounterKind::Loyalty), 4);
        }
    }
}
