use super::*;

// Use the real Chandra resolution as the existing budget tests do; the copied
// cards and budget are supplied directly because Uldaros remains gated.
fn copied_offer(g: &mut Game, player: Player) -> [ObjectRef; 2] {
    let chandra = put(g, player, "Chandra, Torch of Defiance", Zone::Battlefield);
    g.state.objects[chandra as usize].counters.loyalty = 4;
    put(g, player, "Mountain", Zone::Library);
    act(
        g,
        Action::AnnounceActivate {
            card: chandra,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(g);
    g.resolve_top();
    let bear = g.allocate_card_copy(Card::from_name("Bear Cub").unwrap(), player);
    let giant = g.allocate_card_copy(Card::from_name("Hill Giant").unwrap(), player);
    let offer = g.state.resolution_free_casts.as_mut().unwrap();
    offer.paid = None;
    offer.casts_remaining = None;
    offer.remaining_mana_value = Some(6);
    offer.remaining = vec![bear, giant];
    [bear, giant]
}

#[test]
fn multiple_card_copies_spend_budget_and_resolve_as_tokens() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let [bear, giant] = copied_offer(&mut g, player);
        let before = g.snapshot();
        for _ in 0..2 {
            for (object, expected) in [(bear, 4), (giant, 0)] {
                let action = Action::CastExiledDuringResolution {
                    object,
                    face: CastFace::Normal,
                };
                assert!(g.legal_actions().contains(&action));
                act(&mut g, action);
                pay(&mut g);
                assert_eq!(
                    g.state
                        .resolution_free_casts
                        .as_ref()
                        .unwrap()
                        .remaining_mana_value,
                    Some(expected)
                );
            }
            act(&mut g, Action::FinishResolutionCasts);
            g.resolve_top();
            g.resolve_top();
            for name in ["Bear Cub", "Hill Giant"] {
                let card = Card::from_name(name).unwrap();
                assert_eq!(
                    g.state
                        .objects
                        .iter()
                        .filter(|o| o.zone == Zone::Battlefield
                            && o.controller == player
                            && o.is_token()
                            && o.card == card)
                        .count(),
                    1
                );
            }
            g.restore(&before).unwrap();
        }
    }
}

#[test]
fn copy_offer_budget_and_remaining_choices_survive_owned_rollback() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let [bear, giant] = copied_offer(&mut g, player);
        let action = Action::CastExiledDuringResolution {
            object: bear,
            face: CastFace::Normal,
        };
        let saved = g.snapshot();
        let revision = g.revision();
        let index = g.legal_actions().iter().position(|a| a == &action).unwrap();
        let mut raw = g.clone();
        let mut indexed = g.clone();
        assert!(raw.apply(player.other(), action.clone()).is_err());
        assert_eq!(raw.snapshot(), saved);
        raw.apply(player, action.clone()).unwrap();
        indexed.apply_index(player, revision, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        pay(&mut raw);
        let after = raw.snapshot();
        let offer = raw.state.resolution_free_casts.as_ref().unwrap();
        assert_eq!(offer.remaining_mana_value, Some(4));
        assert!(offer.remaining.contains(&giant));
        assert!(!offer.remaining.contains(&bear));
        let choice = Action::CastExiledDuringResolution {
            object: giant,
            face: CastFace::Normal,
        };
        assert!(raw.legal_actions().contains(&choice));
        raw.restore(&saved).unwrap();
        act(&mut raw, action);
        pay(&mut raw);
        assert_eq!(raw.snapshot(), after);
        assert!(raw.legal_actions().contains(&choice));
        for viewer in [Player::P0, Player::P1] {
            let view = serde_json::to_value(raw.observe(viewer)).unwrap();
            assert_eq!(view["resolution_free_casts"]["remaining_mana_value"], 4);
        }
    }
}
