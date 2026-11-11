//! A coin result creates a fresh ability with the original cast facts.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::rng::Rng;
use rustmage_core::{cards::*, engine::*};
use support::*;

const ARTIFACT: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 16,
    nonland: false,
    other: false,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};

const FLIP: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::Any,
                ordinal: Some(2),
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::ChooseObjects {
                    kind: EffectChoiceKind::Sacrifice,
                    players: PlayerSet::Controller,
                    filter: ARTIFACT,
                    amount: 1,
                    optional_skip: Some(1),
                },
                Instruction::FlipCoinForCapturedSpell { win: 1, loss: 2 },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::NONE,
            program: &[Instruction::CopyCapturedSpell {
                count: SpellCopyCount::One,
                retarget: true,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Any)),
            program: &[Instruction::DamageScaled(AmountValue::CastSpellManaValue)],
        },
    ],
    ..CardRules::EMPTY
};

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn separate_coin_trigger_uses_the_original_x_and_mana_value() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &FLIP)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let bear = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let first = put(&mut game, player, "Hill Giant", Zone::Hand);
    let second = put(&mut game, player, "Exsanguinate", Zone::Hand);
    let artifact = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let artifact = game.object_ref(artifact);
    cast(&mut game, first, None);
    pass_pair(&mut game);
    act(&mut game, Action::AnnounceCast { card: second });
    act(&mut game, Action::ChooseX { value: 3 });
    pay(&mut game);
    assert_eq!(game.state.stack.len(), 2);
    let original = game.state.stack[0].id;
    pass_pair(&mut game);

    let mut declined = game.clone();
    let rng = declined.state.rng;
    act(&mut declined, Action::DeclineEffectPayment);
    assert_eq!(declined.state.rng, rng);
    assert_eq!(declined.object_view(artifact.card).zone, Zone::Battlefield);
    assert_eq!(declined.state.stack.len(), 1);

    for win in [false, true] {
        let mut branch = game.clone();
        branch.state.rng = (1..1000)
            .map(Rng)
            .find(|state| {
                let mut state = *state;
                (state.below(2) == 1) == win
            })
            .unwrap();
        act(&mut branch, Action::Sacrifice { object: artifact });
        assert_eq!(branch.object_view(artifact.card).zone, Zone::Graveyard);
        if !win {
            act(
                &mut branch,
                Action::ChooseTarget {
                    target: Target::Player(opponent),
                },
            );
        }
        let trigger = branch.state.stack.last().unwrap();
        assert_eq!(trigger.source.unwrap().id, bear);
        assert_eq!(
            trigger.kind,
            StackKind::TriggeredAbility {
                ability: if win { 1 } else { 2 }
            }
        );
        assert_eq!(branch.state.players[opponent.index()].life, 20);
        branch.counter_stack_item(original, false);
        pass_pair(&mut branch);
        if win {
            let copy = branch.state.stack.last().unwrap();
            assert!(copy.is_copy);
            assert_eq!(copy.x, Some(3));
            assert_eq!(copy.mana_value(), 5);
            assert_eq!(
                branch.state.players[player.index()].spells_cast_this_turn,
                2
            );
        } else {
            assert_eq!(branch.state.players[opponent.index()].life, 15);
        }
    }
}

const SACRIFICE_EXILE: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Sacrificed {
            filter: SacrificeFilter {
                kinds: 1,
                other: true,
                ..ARTIFACT
            },
            own_turn: false,
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: false,
        })),
        program: &[
            Instruction::ExileTopLibraryRemember {
                players: PlayerSet::Targeted,
                amount: AmountValue::SacrificeEventManaValue,
            },
            Instruction::GrantExiledResultFlexiblePlayUntilNextEndStep,
        ],
    }],
    ..CardRules::EMPTY
};

#[test]
fn sacrifice_exiles_using_the_previous_battlefield_mana_value() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &SACRIFICE_EXILE)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let sacrifice = put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
    let mountain = put(&mut game, opponent, "Mountain", Zone::Library);
    let spell = put(&mut game, opponent, "Murder", Zone::Library);
    let hill = Card::from_name("Hill Giant").unwrap();
    game.apply_temporary_copy(
        game.object_ref(sacrifice),
        CopyRecipe::new(hill),
        CopyDeadline::EndOfTurn,
    );
    game.sacrifice_batch(&[(player, sacrifice)]);
    game.give_priority();
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(opponent),
        },
    );
    pass_pair(&mut game);
    assert_eq!(game.object_view(sacrifice).card.name(), "Healer's Hawk");
    assert_eq!(game.state.exile_permissions.len(), 4);
    assert!(game.state.exile_permissions.iter().all(|row| {
        row.player == player
            && row.expiry == ExilePlayExpiry::StartOfNextEndStep(player)
            && row.spend_any_mana
            && !row.cast_only
    }));
    assert_eq!(game.object_view(mountain).zone, Zone::Exile);
    assert_eq!(game.object_view(spell).zone, Zone::Exile);
    assert!(
        game.legal_actions()
            .contains(&Action::PlayLand { card: mountain })
    );
    game.state.step = Step::Main2;
    pass_pair(&mut game);
    assert!(game.state.exile_permissions.is_empty());
}
