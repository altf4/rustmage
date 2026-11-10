//! Restrict a mana lot to expensive spells or spells with a printed X.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const RESTRICTED_MANA: Mana = [0, 1, 0, 0, 1, 0];

const PRODUCER: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        program: &[Instruction::AddMana(ManaProduction::Tagged {
            production: &ManaProduction::Fixed(RESTRICTED_MANA),
            quality: &ManaQuality {
                restriction: ManaRestriction::SpellManaValueAtLeastOrX(5),
                spell_bonus: ManaSpellBonus::None,
                retention: ManaRetention::Step,
            },
        })],
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        targets: TargetPlan::NONE,
        cycling: false,
        exhaust: false,
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

const SPELL_WITH_X: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Draw(PlayerSet::Controller, 1)],
    ),
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn produce(game: &mut Game, card: CardId) {
    act(
        game,
        Action::ActivateMana {
            card,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: RESTRICTED_MANA,
        },
    );
}

#[test]
fn the_printed_x_qualifies_at_zero_in_floating_and_prospective_funding() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Short Bow").unwrap(), &PRODUCER),
        (Card::Divination, &SPELL_WITH_X),
    ]);
    for floated in [false, true] {
        for player in [Player::P0, Player::P1] {
            let mut game = scenario(player);
            let mana_source = put(&mut game, player, "Short Bow", Zone::Battlefield);
            let variable = put(&mut game, player, "Divination", Zone::Hand);
            let cheap = put(&mut game, player, "Opt", Zone::Hand);
            game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];
            if floated {
                produce(&mut game, mana_source);
                game.move_object(mana_source, Zone::Graveyard);
            }
            assert!(!game.legal_actions().iter().any(|action| matches!(
                action,
                Action::AnnounceCast { card } | Action::Cast { card, .. } if *card == cheap
            )));
            act(&mut game, Action::AnnounceCast { card: variable });
            act(&mut game, Action::ChooseX { value: 0 });
            if !floated {
                produce(&mut game, mana_source);
            }
            pay(&mut game);
            settle(&mut game);
            assert_eq!(game.object_view(variable).zone, Zone::Graveyard);
            assert_eq!(game.state.players[player.index()].mana, [0; 6]);
        }
    }
}

#[test]
fn an_expensive_normal_face_does_not_qualify_its_cheaper_adventure() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Short Bow").unwrap(), &PRODUCER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let mana_source = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let adventure = put(&mut game, player, "Beanstalk Wurm", Zone::Hand);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 3];

    produce(&mut game, mana_source);
    assert!(
        !game
            .legal_actions()
            .contains(&Action::AnnounceAdventure { card: adventure })
    );
    cast(&mut game, adventure, None);
    settle(&mut game);
    assert_eq!(game.object_view(adventure).zone, Zone::Battlefield);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
}
