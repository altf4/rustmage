//! Aang's two faces and waterbend transformation.
use super::big_helpers::{cast, checked, settle, setup};
use super::*;

#[test]
fn aang_airbends_then_waterbends_and_counters_tapped_controlled_creatures() {
    for player in [Player::P0, Player::P1] {
        let mut game = setup(player);
        let aang = put(&mut game, player, "Aang, Swift Savior", Zone::Hand);
        let victim = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        game.state.objects[enemy as usize].tapped = true;
        let contributors: Vec<_> = (0..8)
            .map(|_| put(&mut game, player, "Bear Cub", Zone::Battlefield))
            .collect();
        cast(&mut game, aang, None, &[]);
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
        assert!(game.has_airbend_permission(player.other(), victim));
        assert!(
            game.keywords(aang)
                .contains(Keywords::FLYING.union(Keywords::FLASH))
        );
        game.state.players[player.index()].mana = [0; 6];
        checked(
            &mut game,
            Action::AnnounceActivate {
                card: aang,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(&mut game, Action::BeginWaterbend);
        for &id in &contributors {
            let object = game.object_ref(id);
            checked(&mut game, Action::ChooseWaterbender { object });
        }
        checked(&mut game, Action::PayCosts { payment: [0; 6] });
        settle(&mut game);
        assert_eq!(game.object_view(aang).name(), "Aang and La, Ocean's Fury");
        assert_eq!((game.stats(aang).power, game.stats(aang).toughness), (5, 5));
        assert!(
            game.keywords(aang)
                .contains(Keywords::REACH.union(Keywords::TRAMPLE))
        );
        game.state.objects[aang as usize].summoning_sick = false;
        game.state.step = Step::Attackers;
        game.state.pending = Pending::Attackers;
        checked(&mut game, Action::Attack { card: aang });
        checked(&mut game, Action::FinishAttackers);
        settle(&mut game);
        assert_eq!(game.counter_count(aang, CounterKind::PlusOne), 1);
        assert_eq!(game.counter_count(enemy, CounterKind::PlusOne), 0);
        for id in contributors {
            assert_eq!(game.counter_count(id, CounterKind::PlusOne), 1);
        }
    }
}
