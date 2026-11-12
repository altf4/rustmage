//! A source's exact chosen subtype determines a single colored mana amount.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const CITY: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[ActivatedAbility {
        guard: ActivationGuard::Always,
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        origin: ActivationOrigin::Battlefield,
        exhaust: false,
        cycling: false,
        sorcery: false,
        once_per_turn: false,
        program: &[Instruction::AddMana(ManaProduction::ChoiceStat {
            colors: 31,
            amount: &StatValue::CreaturesOfChosenType,
        })],
    }],
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::CreatureType),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

#[test]
fn mana_quantity_uses_live_controlled_creatures_of_the_chosen_type() {
    let definition = Card::from_name("Evolving Wilds").unwrap();
    let _rules = fixture_rules::activate(&[(definition, &CITY)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let land = spawn(&mut game, player, definition, Zone::Hand);
        act(&mut game, Action::PlayLand { card: land });
        act(
            &mut game,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::CreatureType(CreatureType::from_name("Bird").unwrap()),
            },
        );
        put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
        let changeling = put(&mut game, player, "Three Tree Mascot", Zone::Battlefield);
        put(
            &mut game,
            player.other(),
            "Healer's Hawk",
            Zone::Battlefield,
        );
        put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let reference = game.object_ref(land);
        let value =
            |game: &Game| game.stat_value(StatValue::CreaturesOfChosenType, reference, player);
        assert_eq!(value(&game), 2);
        let snapshot = game.snapshot();
        game.move_object(changeling, Zone::Exile);
        assert_eq!(value(&game), 1);
        game.restore(&snapshot).unwrap();

        let activate = Action::ActivateMana {
            card: land,
            ability: 0,
            payment: [0, 0, 0, 0, 0, 2],
            counter_payment: Counters::default(),
            production: [0, 0, 0, 0, 2, 0],
        };
        game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];
        reject(&mut game, activate.clone());
        game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
        let deferred = Action::ActivateMana {
            card: land,
            ability: 0,
            payment: [0, 0, 0, 0, 0, 2],
            counter_payment: Counters::default(),
            production: [0; 6],
        };
        if game.legal_actions().contains(&activate) {
            act(&mut game, activate);
        } else {
            act(&mut game, deferred);
            reject(
                &mut game,
                Action::ChooseMana {
                    production: [1, 0, 0, 0, 1, 0],
                },
            );
            act(
                &mut game,
                Action::ChooseMana {
                    production: [0, 0, 0, 0, 2, 0],
                },
            );
        }
        assert!(game.object_view(land).tapped);
        assert_eq!(game.state.players[player.index()].mana, [0, 0, 0, 0, 2, 0]);
    }
}
