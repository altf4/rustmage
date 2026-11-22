//! Surly Surgeon's entry counter and live destruction replacement.
use crate::support::*;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};

#[test]
fn printed_entry_target_is_other_controlled_creature_and_destroy_heals_it() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let target = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let enemy = put(
            &mut game,
            player.other(),
            "Grizzly Bears",
            Zone::Battlefield,
        );
        let surgeon = put(&mut game, player, "Dr. McCoy, Surly Surgeon", Zone::Hand);
        cast(&mut game, surgeon, None);
        for _ in 0..12 {
            if game.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(target),
            }) {
                break;
            }
            choose(&mut game, |action| {
                matches!(action, Action::Pass | Action::PutTrigger { .. })
            });
        }
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(surgeon),
            },
        );
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        if game.legal_actions().contains(&Action::FinishTargets) {
            act(&mut game, Action::FinishTargets);
        }
        settle(&mut game);
        assert_eq!(stats(&game, surgeon), (2, 2));
        assert_eq!(game.counter_count(target, CounterKind::PlusOne), 1);

        game.state.objects[target as usize].damage = 2;
        let removal = put(&mut game, player, "Murder", Zone::Hand);
        cast(&mut game, removal, Some(Target::Card(target)));
        settle(&mut game);
        assert!(game.on_battlefield(target));
        assert_eq!(game.counter_count(target, CounterKind::PlusOne), 0);
        assert_eq!(game.state.objects[target as usize].damage, 0);

        let second = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        game.set_counters(second, Counters::of(CounterKind::PlusOne, 1));
        game.move_object(surgeon, Zone::Graveyard);
        let removal = put(&mut game, player, "Murder", Zone::Hand);
        cast(&mut game, removal, Some(Target::Card(second)));
        settle(&mut game);
        assert!(!game.on_battlefield(second));
    }
}
