//! Kaya’s printed abilities through ordinary actions.
use super::big_helpers::{cast as paid_cast, checked, settle as finish};
use super::*;

fn activate_kaya(g: &mut Game, kaya: CardId, ability: u16) {
    checked(
        g,
        Action::AnnounceActivate {
            card: kaya,
            ability,
            counter_payment: Counters::default(),
        },
    );
}

fn finish_token_copy(g: &mut Game, token: CardId, exiled: CardId) {
    g.prepare_triggers();
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(token),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        checked(g, Action::FinishTargets);
    }
    g.resolve_top();
    let exiled = g.object_ref(exiled);
    checked(g, Action::SelectResolutionExile { object: exiled });
    finish(g);
    assert_eq!(stats(g, token), (2, 2));
    assert!(g.keywords(token).contains(Keywords::FLYING));
}

#[test]
fn kaya_spirits_justice_whole_card() {
    let spirit = Card::from_name("@token/mkm-kaya-spirit").unwrap();
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let kaya = put(&mut g, player, "Kaya, Spirits' Justice", Zone::Hand);
        paid_cast(&mut g, kaya, None, &[]);
        finish(&mut g);
        assert_eq!(g.counter_count(kaya, CounterKind::Loyalty), 3);
        activate_kaya(&mut g, kaya, 1);
        pay(&mut g);
        finish(&mut g);
        let token = g
            .state
            .objects
            .iter()
            .position(|o| o.card == spirit && o.zone == Zone::Battlefield)
            .unwrap() as CardId;
        assert_eq!(g.object_view(token).colors, 5);
        assert!(g.keywords(token).contains(Keywords::FLYING));

        let mut grave = g.clone();
        grave.state.objects[kaya as usize].loyalty_activated_this_turn = false;
        grave.state.players[player.index()].library.clear();
        let bear = put(&mut grave, player, "Bear Cub", Zone::Graveyard);
        let other = put(&mut grave, player.other(), "Bear Cub", Zone::Graveyard);
        activate_kaya(&mut grave, kaya, 0);
        pay(&mut grave);
        grave.resolve_top();
        assert!(
            grave
                .legal_actions()
                .contains(&Action::SelectResolutionExile {
                    object: grave.object_ref(other)
                })
        );
        let exiled = grave.object_ref(bear);
        checked(&mut grave, Action::SelectResolutionExile { object: exiled });
        finish_token_copy(&mut grave, token, bear);
        assert_eq!(grave.object_view(bear).zone, Zone::Exile);
        grave.finish_cleanup();
        assert_eq!(stats(&grave, token), (1, 1));

        let mut exile = g.clone();
        exile.state.objects[kaya as usize].loyalty_activated_this_turn = false;
        let ours = put(&mut exile, player, "Bear Cub", Zone::Battlefield);
        let theirs = put(&mut exile, player.other(), "Bear Cub", Zone::Battlefield);
        activate_kaya(&mut exile, kaya, 2);
        for card in [ours, theirs] {
            checked(
                &mut exile,
                Action::ChooseTarget {
                    target: Target::Card(card),
                },
            );
        }
        if exile.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut exile, Action::FinishTargets);
        }
        pay(&mut exile);
        exile.resolve_top();
        assert_eq!(exile.object_view(ours).zone, Zone::Exile);
        assert_eq!(exile.object_view(theirs).zone, Zone::Exile);
        finish_token_copy(&mut exile, token, ours);
    }
}
