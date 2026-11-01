//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

fn curse_fixture(g: &mut Game, player: Player) -> CardId {
    let id = put(g, player, "Bear Cub", Zone::Battlefield);
    g.state.objects[id as usize].card = Card::from_name("Garruk, Curse Breaker").unwrap();
    g.state.objects[id as usize].counters.loyalty = 5;
    g.register_battlefield_sources(id);
    id
}
fn curse_activate(g: &mut Game, source: CardId, ability: u16) {
    act(
        g,
        Action::AnnounceActivate {
            card: source,
            ability,
            counter_payment: Counters::default(),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
}

#[test]
fn garruk_beast_identity_is_visible_to_both_players() {
    {
        let player = Player::P0;
        let mut g = scenario(player);
        let source = curse_fixture(&mut g, player);
        curse_activate(&mut g, source, 1);
        settle(&mut g);
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, object)| {
                (object.zone == Zone::Battlefield
                    && object.card == Card::from_name("@token/0bdfcac57503bc474764a02e").unwrap())
                .then_some(id as CardId)
            })
            .unwrap();
        for viewer in [player, player.other()] {
            let view = g
                .observe(viewer)
                .objects
                .into_iter()
                .find(|view| view.id == token)
                .unwrap();
            assert_eq!(view.name(), "Beast Token");
            assert_eq!(view.colors, 16);
            assert!(view.has_subtype("Beast"));
            assert!(view.keywords.contains(Keywords::TRAMPLE));
        }
    }
}

#[test]
fn garruk_duration_ignores_skipped_turn_and_ends_on_actual_extra_turn() {
    {
        let player = Player::P0;
        let mut g = scenario(player);
        let source = curse_fixture(&mut g, player);
        curse_activate(&mut g, source, 2);
        settle(&mut g);
        g.add_extra_turn(player.other(), false);
        g.advance_to_actual_turn();
        assert_eq!(g.state.active, player.other());
        g.begin_turn();
        assert!(g.state.delayed_opponent_attack_until_next_turn.is_some());
        // Consume the ordinary successor after the extra turn.
        g.advance_to_actual_turn();
        g.begin_turn();
        assert!(g.state.delayed_opponent_attack_until_next_turn.is_some());
        let mut skips = [0; 2];
        skips[player.index()] = 1;
        g.state.turn_skips = Some(Box::new(skips));
        g.advance_to_actual_turn();
        assert_eq!(g.state.active, player.other());
        g.begin_turn();
        assert!(g.state.delayed_opponent_attack_until_next_turn.is_some());
        g.add_extra_turn(player, false);
        g.advance_to_actual_turn();
        assert_eq!(g.state.active, player);
        g.begin_turn();
        assert!(g.state.delayed_opponent_attack_until_next_turn.is_none());
    }
}
