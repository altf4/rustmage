//! Focused printed outcomes; shared engine invariants live in core tests.
//! Case of the Uneaten Feast and exact-incarnation Case state.
use super::*;

#[test]
fn case_sacrifice_grants_only_current_creature_cards_until_turn_end() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let case = put(&mut g, p, "Case of the Uneaten Feast", Zone::Battlefield);
        let creature = put(&mut g, p, "Llanowar Elves", Zone::Graveyard);
        let land = put(&mut g, p, "Forest", Zone::Graveyard);
        let opponent = put(&mut g, p.other(), "Llanowar Elves", Zone::Graveyard);
        g.solve_case(g.object_ref(case));
        big_helpers::checked(
            &mut g,
            Action::AnnounceActivate {
                card: case,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        assert_eq!(g.object_view(case).zone, Zone::Graveyard);
        big_helpers::settle(&mut g);
        assert!(g.has_graveyard_cast(p, creature, GraveyardCastKind::Cast));
        assert!(!g.has_graveyard_cast(p, land, GraveyardCastKind::Cast));
        assert!(!g.has_graveyard_cast(p, opponent, GraveyardCastKind::Cast));
        let late = put(&mut g, p, "Llanowar Elves", Zone::Graveyard);
        assert!(!g.has_graveyard_cast(p, late, GraveyardCastKind::Cast));
        g.move_object(creature, Zone::Exile);
        g.move_object(creature, Zone::Graveyard);
        assert!(!g.has_graveyard_cast(p, creature, GraveyardCastKind::Cast));
    }
}

#[test]
fn case_triggers_for_controlled_creature_entries_even_after_solving() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let case = put(&mut g, p, "Case of the Uneaten Feast", Zone::Battlefield);
        g.solve_case(g.object_ref(case));
        let own = put(&mut g, p, "Llanowar Elves", Zone::Hand);
        let enemy = put(&mut g, p.other(), "Llanowar Elves", Zone::Hand);
        let life = g.state.players[p.index()].life;
        g.move_object(enemy, Zone::Battlefield);
        big_helpers::settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life);
        g.move_object(own, Zone::Battlefield);
        big_helpers::settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life + 1);
    }
}
