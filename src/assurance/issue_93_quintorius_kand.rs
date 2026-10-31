//! Quintorius's printed abilities through ordinary legal gameplay.
use super::big_helpers::{cast as paid_cast, checked, settle as finish};
use super::*;

#[test]
fn quintorius_kand_full_card() {
    let spirit = Card::from_name("@token/lci-quintorius-spirit").unwrap();
    for player in [Player::P0, Player::P1] {
        let mut token = scenario(player);
        let walker = put(&mut token, player, "Quintorius Kand", Zone::Hand);
        paid_cast(&mut token, walker, None, &[]);
        finish(&mut token);
        assert_eq!(token.counter_count(walker, CounterKind::Loyalty), 4);
        checked(
            &mut token,
            Action::AnnounceActivate {
                card: walker,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut token);
        finish(&mut token);
        let id = token
            .state
            .objects
            .iter()
            .position(|o| o.card == spirit && o.zone == Zone::Battlefield && o.controller == player)
            .unwrap() as CardId;
        assert_eq!(stats(&token, id), (3, 2));
        assert_eq!(token.object_view(id).colors, 1 | 8);
        assert!(token.has_subtype(id, "Spirit"));
        assert_eq!(token.counter_count(walker, CounterKind::Loyalty), 5);

        let mut discover = scenario(player);
        let walker = put(&mut discover, player, "Quintorius Kand", Zone::Battlefield);
        discover.state.objects[walker as usize].counters.loyalty = 4;
        let shock = put(&mut discover, player, "Shock", Zone::Library);
        let land = put(&mut discover, player, "Forest", Zone::Library);
        checked(
            &mut discover,
            Action::AnnounceActivate {
                card: walker,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut discover);
        discover.resolve_top();
        assert_eq!(discover.object_view(shock).zone, Zone::Exile);
        assert_eq!(discover.object_view(land).zone, Zone::Exile);
        let mut hand = discover.clone();
        checked(&mut hand, Action::FinishResolutionCasts);
        finish(&mut hand);
        assert_eq!(hand.object_view(shock).zone, Zone::Hand);
        assert_eq!(hand.object_view(land).zone, Zone::Library);
        let object = discover.object_ref(shock);
        checked(
            &mut discover,
            Action::CastExiledDuringResolution {
                object,
                face: CastFace::Normal,
            },
        );
        checked(
            &mut discover,
            Action::ChooseTarget {
                target: Target::Player(player.other()),
            },
        );
        if discover.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut discover, Action::FinishTargets);
        }
        pay(&mut discover);
        checked(&mut discover, Action::FinishResolutionCasts);
        finish(&mut discover);
        assert_eq!(discover.object_view(land).zone, Zone::Library);
        assert_eq!(discover.state.players[player.index()].life, 22);
        assert_eq!(discover.state.players[player.other().index()].life, 16);

        let mut ultimate = scenario(player);
        let walker = put(&mut ultimate, player, "Quintorius Kand", Zone::Battlefield);
        ultimate.state.objects[walker as usize].counters.loyalty = 6;
        let land = put(&mut ultimate, player, "Forest", Zone::Graveyard);
        let bear = put(&mut ultimate, player, "Bear Cub", Zone::Graveyard);
        checked(
            &mut ultimate,
            Action::AnnounceActivate {
                card: walker,
                ability: 2,
                counter_payment: Counters::default(),
            },
        );
        for card in [land, bear] {
            checked(
                &mut ultimate,
                Action::ChooseTarget {
                    target: Target::Card(card),
                },
            );
        }
        checked(&mut ultimate, Action::FinishTargets);
        pay(&mut ultimate);
        finish(&mut ultimate);
        assert_eq!(ultimate.object_view(land).zone, Zone::Exile);
        assert_eq!(ultimate.object_view(bear).zone, Zone::Exile);
        assert_eq!(ultimate.state.players[player.index()].mana[3], 10);
        assert!(
            ultimate
                .legal_actions()
                .contains(&Action::PlayLand { card: land })
        );
        assert!(
            ultimate
                .legal_actions()
                .contains(&Action::AnnounceExile { card: bear })
        );
    }
}
