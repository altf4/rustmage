//! Only actual damage recipients retain the lasting life-gain prohibition.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::engine::*;
use support::*;

const REFLECT_DAMAGE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDamaged,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
            base: &TargetRule::Any,
            predicate: &TargetPredicate::OtherThanSource,
        })),
        program: &[
            Instruction::DamageRememberPlayersScaled(AmountValue::DamageReceivedAmount),
            Instruction::DamagedPlayersCannotGainLifeForGame,
        ],
    }],
    ..CardRules::EMPTY
};

const GAIN_AND_DRAW: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::GainLife(PlayerSet::Each, 3),
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    ),
    ..CardRules::EMPTY
};

fn target(target: Target) -> Action {
    Action::ChooseTarget { target }
}

fn reflect_lethal_damage(game: &mut Game, controller: Player) {
    let source = put(game, controller, "Grizzly Bears", Zone::Battlefield);
    let shock = put(game, controller, "Shock", Zone::Hand);
    cast(game, shock, Some(Target::Card(source)));
    act(game, Action::Pass);
    act(game, Action::Pass);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    if !matches!(game.state.pending, Pending::Targets { .. }) {
        choose(game, |action| matches!(action, Action::PutTrigger { .. }));
    }
    assert!(matches!(game.state.pending, Pending::Targets { .. }));
}

fn finish_targets(game: &mut Game) {
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
}

fn prohibit_view(game: &Game, player: Player) -> bool {
    for viewer in [Player::P0, Player::P1] {
        assert_eq!(
            game.observe(viewer).players[player.index()].cannot_gain_life_for_game,
            game.state.players[player.index()].cannot_gain_life_for_game
        );
    }
    game.state.players[player.index()].cannot_gain_life_for_game
}

fn reach_next_turn(game: &mut Game) {
    let turn = game.state.turn;
    for _ in 0..48 {
        if game.state.turn > turn && game.state.step == Step::Main1 {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass | Action::FinishAttackers | Action::FinishBlockers
            )
        });
    }
    panic!("expected the next main phase");
}

#[test]
fn exact_damage_recipient_and_persistent_prohibition_are_owned_by_branches() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &REFLECT_DAMAGE),
        (Card::Divination, &GAIN_AND_DRAW),
    ]);
    for controller in [Player::P0, Player::P1] {
        let opponent = controller.other();
        let mut parent = scenario(controller);
        let creature = put(&mut parent, opponent, "Hill Giant", Zone::Battlefield);
        reflect_lethal_damage(&mut parent, controller);

        let action = target(Target::Player(opponent));
        let index = parent
            .legal_actions()
            .iter()
            .position(|candidate| *candidate == action)
            .unwrap();
        let saved = parent.snapshot();
        let revision = parent.revision();
        let mut raw = parent.clone();
        let mut indexed = parent.clone();
        drop(parent);
        reject(&mut raw, target(Target::Stack(u64::MAX)));
        act(&mut raw, action);
        indexed.apply_index(controller, revision, index).unwrap();
        for game in [&mut raw, &mut indexed] {
            finish_targets(game);
            settle(game);
            assert!(prohibit_view(game, opponent));
            assert!(!prohibit_view(game, controller));
            assert_eq!(game.state.players[opponent.index()].life, 18);

            reach_next_turn(game);
            assert_eq!(game.state.active, opponent);
            game.state.players[opponent.index()].mana = [0, 3, 0, 0, 0, 0];
            let gain = put(game, opponent, "Divination", Zone::Hand);
            let hand = game.hand_count(opponent);
            cast(game, gain, None);
            settle(game);
            assert_eq!(game.state.players[opponent.index()].life, 18);
            assert_eq!(
                game.state.players[opponent.index()].life_gain_events_this_turn,
                0
            );
            assert_eq!(game.state.players[controller.index()].life, 23);
            assert_eq!(game.hand_count(opponent), hand);
        }
        assert_eq!(raw.snapshot(), indexed.snapshot());

        indexed.restore(&saved).unwrap();
        assert_eq!(
            indexed.apply_index(controller, revision, index),
            Err(Error::StaleRevision)
        );
        act(&mut indexed, target(Target::Card(creature)));
        finish_targets(&mut indexed);
        settle(&mut indexed);
        assert_eq!(indexed.object_view(creature).damage, 2);
        assert!(!prohibit_view(&indexed, opponent));
        assert!(prohibit_view(&raw, opponent));
    }
}

#[test]
fn prevention_and_redirection_do_not_lock_the_original_player_target() {
    const PREVENTION: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::NONE,
            &[Instruction::PreventPlayerDamageUntilNextTurn],
        ),
        ..CardRules::EMPTY
    };
    const REDIRECTION: ReplacementSpec =
        ReplacementSpec::RedirectControllerDamageToEnchantedCreature;
    const REDIRECTING_AURA: CardRules = CardRules {
        effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(
            TargetRule::creature_you_control(),
        ))),
        aura: Some(AuraSpec {
            kinds: 1,
            relation: 0,
            bonus: EquipmentSpec {
                stats: Stats {
                    power: 0,
                    toughness: 0,
                },
                keywords: Keywords::NONE,
            },
        }),
        static_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::Replacement(0, REDIRECTION)],
        }],
        extras: &CardExtras {
            replacement_effects: &[REDIRECTION],
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &REFLECT_DAMAGE),
        (Card::from_name("Opt").unwrap(), &PREVENTION),
        (Card::from_name("Pacifism").unwrap(), &REDIRECTING_AURA),
    ]);
    let controller = Player::P0;
    let opponent = controller.other();
    for redirect in [false, true] {
        let mut game = scenario(controller);
        let recipient = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
        if redirect {
            let aura = put(&mut game, opponent, "Pacifism", Zone::Battlefield);
            game.state.objects[aura as usize].attached_to = Some(game.object_ref(recipient));
        }
        reflect_lethal_damage(&mut game, controller);
        act(&mut game, target(Target::Player(opponent)));
        finish_targets(&mut game);
        if !redirect {
            if game.actor() != Some(opponent) {
                act(&mut game, Action::Pass);
            }
            let protection = put(&mut game, opponent, "Opt", Zone::Hand);
            cast(&mut game, protection, None);
        }
        settle(&mut game);
        assert_eq!(game.state.players[opponent.index()].life, 20);
        assert_eq!(
            game.object_view(recipient).damage,
            if redirect { 2 } else { 0 }
        );
        assert!(!prohibit_view(&game, opponent));
    }
}

#[test]
fn projected_cost_departures_respect_the_same_life_gain_prohibition() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &REFLECT_DAMAGE)]);
    let controller = Player::P1;
    let opponent = controller.other();
    let mut game = scenario(controller);
    put(
        &mut game,
        opponent,
        "The Darkness Crystal",
        Zone::Battlefield,
    );
    let victim = put(&mut game, controller, "Hill Giant", Zone::Battlefield);
    reflect_lethal_damage(&mut game, controller);
    let before_damage = game.snapshot();
    for lock in [true, false] {
        game.restore(&before_damage).unwrap();
        act(
            &mut game,
            target(Target::Player(if lock { opponent } else { controller })),
        );
        finish_targets(&mut game);
        settle(&mut game);
        let victim = game.object_ref(victim);
        let goal = FundingGoal {
            actor: controller,
            required: [0; 6],
            generic: 0,
            reserve: Some((
                victim,
                AdditionalCost {
                    kind: EffectChoiceKind::Sacrifice,
                    filter: SacrificeFilter {
                        keywords: Keywords::NONE,
                        source_only: false,
                        kinds: 1,
                        nonland: false,
                        other: false,
                        tokens: None,
                        artifact_enchantment_or_token: false,
                        subtype: None,
                    },
                },
            )),
            source: None,
        };
        let saved = game.snapshot();
        let before = game.state.players[opponent.index()].life;
        let world = FundingState::from_game(&game, &goal);
        assert!(
            world.cost_departures(&[victim], Zone::Graveyard, &mut |next| {
                assert_eq!(next.object(victim.card).zone, Zone::Exile);
                assert_eq!(next.life(opponent), before + if lock { 0 } else { 2 });
                assert_eq!(next.gained_life(opponent), !lock);
                true
            })
        );
        assert_eq!(game.snapshot(), saved);
    }
}
