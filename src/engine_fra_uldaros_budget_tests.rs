use super::*;

// A real resolving Chandra ability supplies the ordinary cast continuation;
// replace its permission with the budgeted free offer under test.
fn budget_offer(g: &mut Game, player: Player, names: &[&str], budget: u64) -> Vec<ObjectRef> {
    let chandra = put(g, player, "Chandra, Torch of Defiance", Zone::Battlefield);
    g.state.objects[chandra as usize].counters.loyalty = 4;
    let cards: Vec<_> = names.iter().map(|name| put(g, player, name, Zone::Exile)).collect();
    put(g, player, "Mountain", Zone::Library);
    act(g, Action::AnnounceActivate { card: chandra, ability: 0, counter_payment: Counters::default() });
    pay(g);
    g.resolve_top();
    let refs: Vec<_> = cards.iter().map(|&id| g.object_ref(id)).collect();
    let offer = g.state.resolution_free_casts.as_mut().unwrap();
    offer.paid = None;
    offer.casts_remaining = None;
    offer.remaining_mana_value = Some(budget);
    offer.remaining = refs.clone();
    refs
}

#[test]
fn budget_is_inclusive_cumulative_and_replays_for_both_seats() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let cards = budget_offer(&mut g, player, &["Bear Cub", "Hill Giant", "Bear Cub"], 6);
        let saved = g.snapshot();
        for _ in 0..2 {
            let a = Action::CastExiledDuringResolution { object: cards[0], face: CastFace::Normal };
            assert!(g.legal_actions().contains(&a));
            act(&mut g, a);
            pay(&mut g);
            assert_eq!(g.state.resolution_free_casts.as_ref().unwrap().remaining_mana_value, Some(4));
            let b = Action::CastExiledDuringResolution { object: cards[1], face: CastFace::Normal };
            assert!(g.legal_actions().contains(&b));
            act(&mut g, b);
            pay(&mut g);
            assert_eq!(g.state.resolution_free_casts.as_ref().unwrap().remaining_mana_value, Some(0));
            assert!(!g.legal_actions().contains(&Action::CastExiledDuringResolution { object: cards[2], face: CastFace::Normal }));
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn budget_uses_selected_adventure_face_and_rejects_over_budget() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let cards = budget_offer(&mut g, player, &["Great Ugly-Looking Goblin // Clap! Snap!"], 2);
        let object = cards[0];
        assert!(!g.legal_actions().contains(&Action::CastExiledDuringResolution { object, face: CastFace::Normal }));
        let saved = g.snapshot();
        for _ in 0..2 {
            let a = Action::CastExiledDuringResolution { object, face: CastFace::Adventure };
            assert!(g.legal_actions().contains(&a));
            act(&mut g, a);
            pay(&mut g);
            assert_eq!(g.state.stack.last().unwrap().mana_value(), 2);
            assert_eq!(g.state.resolution_free_casts.as_ref().unwrap().remaining_mana_value, Some(0));
            g.restore(&saved).unwrap();
        }
    }
}
