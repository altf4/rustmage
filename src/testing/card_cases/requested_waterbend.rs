//! Focused printed outcomes; shared engine invariants live in core tests.
use super::big_helpers::{cast, checked, settle, setup};
use super::*;

#[test]
fn iceberg_waterbend_uses_creatures_and_returns_the_exiled_permanent() {
    {
        let player = Player::P0;
        let mut game = setup(player);
        let iceberg = put(&mut game, player, "Aang's Iceberg", Zone::Hand);
        let victim = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        let contributors = [
            put(&mut game, player, "Bear Cub", Zone::Battlefield),
            put(&mut game, player, "Bear Cub", Zone::Battlefield),
            put(&mut game, player, "Bear Cub", Zone::Battlefield),
        ];
        game.state.objects[contributors[0] as usize].summoning_sick = true;
        cast(&mut game, iceberg, None, &[]);
        for _ in 0..20 {
            if game.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(victim),
            }) {
                break;
            }
            let action = game
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PutTrigger { .. } | Action::Pass))
                .unwrap();
            checked(&mut game, action);
        }
        checked(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(victim),
            },
        );
        if game.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut game, Action::FinishTargets);
        }
        settle(&mut game);
        assert_eq!(game.object_view(victim).zone, Zone::Exile);
        assert_eq!(waterbend::masks(&game, player, 2, &[])[0], 2);
        game.state.players[player.index()].mana = [0; 6];
        checked(
            &mut game,
            Action::AnnounceActivate {
                card: iceberg,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(&mut game, Action::BeginWaterbend);
        for id in contributors {
            let object = game.object_ref(id);
            checked(&mut game, Action::ChooseWaterbender { object });
        }
        checked(&mut game, Action::PayCosts { payment: [0; 6] });
        assert_eq!(game.object_view(iceberg).zone, Zone::Battlefield);
        settle(&mut game);
        assert_eq!(game.object_view(iceberg).zone, Zone::Graveyard);
        assert_eq!(game.object_view(victim).zone, Zone::Battlefield);
        assert!(game.state.players[player.index()].scried_or_surveilled_this_turn);
        for id in contributors {
            assert!(game.object_view(id).tapped);
        }
    }
}
