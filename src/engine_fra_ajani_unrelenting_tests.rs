use super::*;

fn cadets(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && o.card == Card::test_index(5298))
            .then_some(id as CardId)
        })
        .collect()
}

fn activate(g: &mut Game, card: CardId, ability: u16) {
    act(
        g,
        Action::AnnounceActivate {
            card,
            ability,
            counter_payment: Counters::default(),
        },
    );
    pay(g);
}

#[test]
fn ajani_loyalty_activations_create_cadets_and_plus_one_grants_haste() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let ajani = put(&mut g, p, "Ajani Unrelenting", Zone::Battlefield);
        g.state.objects[ajani as usize].counters.loyalty = 5;
        let ally = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        activate(&mut g, ajani, 0);
        assert_eq!(g.counter_count(ajani, CounterKind::Loyalty), 6);
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            settle(&mut g);
            let made = cadets(&g, p);
            assert_eq!(made.len(), 1);
            assert_eq!(stats(&g, ally), (3, 2));
            assert_eq!(stats(&g, made[0]), (3, 2));
            assert!(g.keywords(ally).contains(Keywords::HASTE));
            assert!(g.keywords(made[0]).contains(Keywords::HASTE));
            assert!(!g.keywords(enemy).contains(Keywords::HASTE));
        }
    }
}

#[test]
fn ajani_minus_two_discards_hand_then_counts_creatures_at_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let ajani = put(&mut g, p, "Ajani Unrelenting", Zone::Battlefield);
        g.state.objects[ajani as usize].counters.loyalty = 5;
        put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let first = put(&mut g, p, "Forest", Zone::Hand);
        let second = put(&mut g, p, "Hill Giant", Zone::Hand);
        let before = g.state.players[p.index()].cards_drawn_this_turn;
        activate(&mut g, ajani, 1);
        for _ in 0..48 {
            if g.state.stack.is_empty()
                && g.state.resolving.is_none()
                && g.state.pending == Pending::Priority
            {
                break;
            }
            choose(&mut g, |action| {
                matches!(
                    action,
                    Action::Pass | Action::PutTrigger { .. } | Action::Discard { .. }
                )
            });
        }
        settle(&mut g);
        assert_eq!(g.object_view(first).zone, Zone::Graveyard);
        assert_eq!(g.object_view(second).zone, Zone::Graveyard);
        assert_eq!(cadets(&g, p).len(), 1);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, before + 2);
    }
}

#[test]
fn ajani_minus_three_damages_all_creatures_except_its_controllers_tokens() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let ajani = put(&mut g, p, "Ajani Unrelenting", Zone::Battlefield);
        g.state.objects[ajani as usize].counters.loyalty = 5;
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.create_tokens(p, Card::test_index(5298), 1, false);
        let own_token = cadets(&g, p)[0];
        g.create_tokens(p.other(), Card::test_index(5298), 1, false);
        let enemy_token = cadets(&g, p.other())[0];
        activate(&mut g, ajani, 2);
        g.move_object(ajani, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(own).zone, Zone::Graveyard);
        assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
        assert_ne!(g.object_view(enemy_token).zone, Zone::Battlefield);
        assert_eq!(g.object_view(own_token).zone, Zone::Battlefield);
        assert_eq!(cadets(&g, p).len(), 2);
    }
}

#[test]
fn ajani_observes_another_controlled_planeswalkers_loyalty_activation() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let ajani = put(&mut g, p, "Ajani Unrelenting", Zone::Battlefield);
        g.state.objects[ajani as usize].counters.loyalty = 5;
        g.create_tokens(p, Card::JacePlaneswalkerToken, 1, false);
        let jace = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, o)| {
                (o.zone == Zone::Battlefield && o.card == Card::JacePlaneswalkerToken)
                    .then_some(id as CardId)
            })
            .unwrap();
        g.state.objects[jace as usize].counters.loyalty = 4;
        activate(&mut g, jace, 1);
        settle(&mut g);
        assert_eq!(cadets(&g, p).len(), 1);
    }
}
