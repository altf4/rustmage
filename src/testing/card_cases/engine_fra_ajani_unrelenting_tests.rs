//! Focused printed outcomes; shared engine invariants live in core tests.
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
                && o.card == Card::from_name("@token/adcb48b581ad80b8654a11b9").unwrap())
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
fn ajani_minus_two_discards_hand_then_counts_creatures_at_resolution() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let ajani = put(&mut g, p, "Ajani Unrelenting", Zone::Battlefield);
        g.state.objects[ajani as usize].counters.loyalty = 5;
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.create_tokens(
            p,
            Card::from_name("@token/adcb48b581ad80b8654a11b9").unwrap(),
            1,
            false,
        );
        let own_token = cadets(&g, p)[0];
        g.create_tokens(
            p.other(),
            Card::from_name("@token/adcb48b581ad80b8654a11b9").unwrap(),
            1,
            false,
        );
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
