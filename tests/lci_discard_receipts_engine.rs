//! Discard outcomes remain exact through replacement, independent triggers,
//! and immediate casting permissions.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const FREE_CAST: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::DiscardOneRememberCard,
            Instruction::CastRememberedDiscardFree,
        ],
    ),
    ..CardRules::EMPTY
};
const REFLEXIVE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::DiscardOneReflexiveManaValue(0)],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: false,
        })),
        program: &[Instruction::Mill(
            PlayerSet::Targeted,
            MillAmount::Amount(AmountValue::DiscardedCardManaValue),
        )],
    }],
    ..CardRules::EMPTY
};
const LAND_WATCHER: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::DiscardedKinds {
            relation: CastRelation::Opponent,
            kinds: 2,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};
const EACH_DISCARDS: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Discard(PlayerSet::Each, 2)],
    ),
    ..CardRules::EMPTY
};

fn start(game: &mut Game, player: Player) {
    let spell = put(game, player, "Opt", Zone::Hand);
    cast(game, spell, None);
    for _ in 0..6 {
        if matches!(game.state.pending, Pending::EffectDiscard { .. }) {
            return;
        }
        act(game, Action::Pass);
    }
    panic!("discard choice did not open");
}

#[test]
fn only_the_actual_public_discard_result_can_be_cast_during_resolution() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &FREE_CAST)]);
    for replacement in [false, true] {
        let player = Player::P0;
        let opponent = player.other();
        let mut game = scenario(player);
        let discarded = put(&mut game, player, "Shock", Zone::Hand);
        let unrelated = put(&mut game, player, "Shock", Zone::Graveyard);
        let land = put(&mut game, player, "Forest", Zone::Hand);
        if replacement {
            put(&mut game, opponent, "Rest in Peace", Zone::Battlefield);
        }
        start(&mut game, player);
        let before_discard = game.snapshot();
        act(&mut game, Action::Discard { card: discarded });
        assert_eq!(
            game.object_view(discarded).zone,
            if replacement {
                Zone::Exile
            } else {
                Zone::Graveyard
            }
        );
        assert!(matches!(
            game.state.pending,
            Pending::ResolutionCasts { .. }
        ));
        let free = game
            .legal_actions()
            .into_iter()
            .find(|action| {
                matches!(action, Action::CastExiledDuringResolution { object, .. }
                    | Action::CastFreeDuringResolution { object, .. } if object.card == discarded)
            })
            .unwrap();
        let other = game.object_ref(unrelated);
        reject(
            &mut game,
            Action::CastExiledDuringResolution {
                object: other,
                face: CastFace::Normal,
            },
        );
        let before_offer = game.snapshot();
        let mana = game.state.players[player.index()].mana;
        act(&mut game, free);
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Player(opponent),
            },
        );
        if matches!(game.state.pending, Pending::PayCosts { .. }) {
            pay(&mut game);
        }
        if matches!(game.state.pending, Pending::ResolutionCasts { .. }) {
            act(&mut game, Action::FinishResolutionCasts);
        }
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].mana, mana);
        assert_eq!(game.state.players[opponent.index()].life, STARTING_LIFE - 2);

        game.restore(&before_offer).unwrap();
        act(&mut game, Action::FinishResolutionCasts);
        settle(&mut game);
        assert_eq!(game.state.players[opponent.index()].life, STARTING_LIFE);

        game.restore(&before_discard).unwrap();
        act(&mut game, Action::Discard { card: land });
        assert!(game.legal_actions().iter().all(|action| {
            !matches!(
                action,
                Action::CastExiledDuringResolution { .. }
                    | Action::CastFreeDuringResolution { .. }
                    | Action::PlayLand { .. }
            )
        }));
        act(&mut game, Action::FinishResolutionCasts);
        settle(&mut game);
    }
}

#[test]
fn a_replaced_discard_still_captures_mana_value_before_the_reflexive_target() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &REFLEXIVE)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let opponent = player.other();
        put(&mut game, player, "Rest in Peace", Zone::Battlefield);
        let discarded = put(&mut game, player, "Hill Giant", Zone::Hand);
        let library = game.state.players[opponent.index()].library.len();
        start(&mut game, player);
        assert!(game.state.stack.is_empty());
        act(&mut game, Action::Discard { card: discarded });
        assert_eq!(game.object_view(discarded).zone, Zone::Exile);
        for _ in 0..4 {
            if matches!(game.state.pending, Pending::Targets { .. }) {
                break;
            }
            choose(&mut game, |action| {
                matches!(action, Action::PutTrigger { .. } | Action::Pass)
            });
        }
        assert_eq!(game.state.players[opponent.index()].library.len(), library);
        assert!(matches!(game.state.pending, Pending::Targets { .. }));
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Player(opponent),
            },
        );
        settle(&mut game);
        assert_eq!(
            game.state.players[opponent.index()].library.len(),
            library - 4
        );
    }
}

#[test]
fn land_discard_observer_filters_both_player_and_card_kind() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Opt").unwrap(), &EACH_DISCARDS),
        (Card::HillGiant, &LAND_WATCHER),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let own_land = put(&mut game, player, "Forest", Zone::Hand);
    let own_nonland = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    let foe_land = put(&mut game, player.other(), "Island", Zone::Hand);
    let foe_nonland = put(&mut game, player.other(), "Shock", Zone::Hand);
    start(&mut game, player);
    for card in [own_land, own_nonland, foe_land, foe_nonland] {
        act(&mut game, Action::Discard { card });
    }
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, STARTING_LIFE + 1);
}
