//! Only actual damage recipients retain the lasting life-gain prohibition.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::engine::*;
use support::*;

#[test]
fn a_live_nonpositive_life_rule_only_prevents_the_life_based_loss() {
    const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CantLoseFromNonpositiveLife],
    }];
    const PROTECT: CardRules = CardRules {
        effect: Effect::Permanent,
        static_groups: GROUPS,
        ..CardRules::EMPTY
    };
    const LIFE_LOSS: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::NONE,
            &[Instruction::LoseLife(PlayerSet::Controller, 2)],
        ),
        ..CardRules::EMPTY
    };
    const GAME_LOSS: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::NONE,
            &[Instruction::LoseGame(PlayerSet::Controller)],
        ),
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Short Bow").unwrap(), &PROTECT),
        (Card::from_name("Opt").unwrap(), &LIFE_LOSS),
        (Card::from_name("Giant Growth").unwrap(), &GAME_LOSS),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let source = put(&mut game, player, "Short Bow", Zone::Battlefield);
        game.state.players[player.index()].life = 1;
        let spell = put(&mut game, player, "Opt", Zone::Hand);
        cast(&mut game, spell, None);
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, -1);
        assert_eq!(game.outcome(), None);

        let mut cleanup = game.clone();
        let turn = cleanup.state.turn;
        cleanup.begin_step(Step::End);
        act(&mut cleanup, Action::Pass);
        act(&mut cleanup, Action::Pass);
        assert_eq!(cleanup.state.turn, turn + 1);
        assert_eq!(cleanup.outcome(), None);

        let mut poison = game.clone();
        poison.state.players[player.index()].poison_counters = 10;
        act(&mut poison, Action::Pass);
        assert_eq!(poison.outcome(), Some(Outcome::Winner(player.other())));

        let mut empty_draw = game.clone();
        let amount = empty_draw.state.players[player.index()].library.len() as u128 + 1;
        empty_draw.draw_many(player, amount);
        act(&mut empty_draw, Action::Pass);
        assert_eq!(empty_draw.outcome(), Some(Outcome::Winner(player.other())));

        let mut explicit = game.clone();
        let spell = put(&mut explicit, player, "Giant Growth", Zone::Hand);
        cast(&mut explicit, spell, None);
        settle(&mut explicit);
        assert_eq!(explicit.outcome(), Some(Outcome::Winner(player.other())));

        let spell = put(&mut game, player, "Disenchant", Zone::Hand);
        cast(&mut game, spell, Some(Target::Card(source)));
        settle(&mut game);
        assert_eq!(game.object_view(source).zone, Zone::Graveyard);
        assert_eq!(game.outcome(), Some(Outcome::Winner(player.other())));
    }
}

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
    shock_and_choose_reflection(game, controller, source);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
}

fn shock_and_choose_reflection(game: &mut Game, controller: Player, source: CardId) {
    let shock = put(game, controller, "Shock", Zone::Hand);
    cast(game, shock, Some(Target::Card(source)));
    act(game, Action::Pass);
    act(game, Action::Pass);
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
fn reflection_allows_both_players_and_excludes_its_surviving_source() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &REFLECT_DAMAGE)]);
    let controller = Player::P0;
    let mut game = scenario(controller);
    let source = put(&mut game, controller, "Grizzly Bears", Zone::Battlefield);
    game.state.objects[source as usize].growth.toughness = 1;
    shock_and_choose_reflection(&mut game, controller, source);
    assert_eq!(game.object_view(source).zone, Zone::Battlefield);
    reject(&mut game, target(Target::Card(source)));
    let legal = game.legal_actions();
    for player in [Player::P0, Player::P1] {
        assert!(legal.contains(&target(Target::Player(player))));
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
    let victim = put(&mut game, controller, "Hill Giant", Zone::Battlefield);
    reflect_lethal_damage(&mut game, controller);
    put(
        &mut game,
        opponent,
        "The Darkness Crystal",
        Zone::Battlefield,
    );
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

const PLAYER_AURA: CardRules = CardRules {
    effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(TargetRule::Player {
        opponent_only: false,
    }))),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::EnchantedPlayerCantGainLife],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AttachedPlayerDamaged,
        targets: TargetPlan::NONE,
        program: &[Instruction::LoseHalfLifeRoundedUp(PlayerSet::DamagedPlayer)],
    }],
    extras: &CardExtras {
        enchant_player: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

#[test]
fn player_aura_follows_actual_damage_batches_and_ends_its_live_restriction_on_departure() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Pacifism").unwrap(), &PLAYER_AURA),
        (Card::Divination, &GAIN_AND_DRAW),
    ]);
    let controller = Player::P0;
    let enchanted = controller.other();
    let mut game = scenario(controller);
    let attacker = put(&mut game, controller, "Bear Cub", Zone::Battlefield);
    let other_attacker = put(&mut game, controller, "Llanowar Elves", Zone::Battlefield);
    let aura = put(&mut game, controller, "Pacifism", Zone::Hand);
    cast(&mut game, aura, Some(Target::Player(enchanted)));
    settle(&mut game);
    let aura_view = game.object_view(aura);
    assert_eq!(aura_view.zone, Zone::Battlefield);
    assert_eq!(aura_view.attached_to_player, Some(enchanted));

    let gain = put(&mut game, controller, "Divination", Zone::Hand);
    let hand = game.hand_count(controller);
    cast(&mut game, gain, None);
    settle(&mut game);
    assert_eq!(game.state.players[controller.index()].life, 23);
    assert_eq!(game.state.players[enchanted.index()].life, 20);
    assert_eq!(game.hand_count(controller), hand);
    assert!(!prohibit_view(&game, enchanted));

    let shock = put(&mut game, controller, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Player(controller)));
    settle(&mut game);
    assert_eq!(game.state.players[controller.index()].life, 21);
    assert_eq!(game.state.players[enchanted.index()].life, 20);

    for _ in 0..4 {
        act(&mut game, Action::Pass);
    }
    for card in [attacker, other_attacker] {
        act(&mut game, Action::Attack { card });
    }
    act(&mut game, Action::FinishAttackers);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    act(&mut game, Action::FinishBlockers);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    if game.state.stack.is_empty() {
        choose(&mut game, |action| {
            matches!(action, Action::PutTrigger { .. })
        });
    }
    assert_eq!(game.state.stack.len(), 1);
    let trigger = game.state.stack.last().unwrap();
    assert_eq!(trigger.source.unwrap().object, aura_view.reference);
    let damage = trigger.damage_event.unwrap();
    assert_eq!(damage.player, enchanted);
    assert_eq!(damage.amount, 3);
    assert!(damage.combat);
    assert_eq!(game.state.players[enchanted.index()].life, 17);

    if game.actor() != Some(controller) {
        act(&mut game, Action::Pass);
    }
    game.state.players[controller.index()].mana[0] = 2;
    let removal = put(&mut game, controller, "Disenchant", Zone::Hand);
    cast(&mut game, removal, Some(Target::Card(aura)));
    settle(&mut game);
    let aura_view = game.object_view(aura);
    assert_eq!(aura_view.zone, Zone::Graveyard);
    assert_eq!(aura_view.attached_to_player, None);
    assert_eq!(game.state.players[enchanted.index()].life, 8);
    assert_eq!(life_gain::replacement_amounts(&game, enchanted, 3), vec![3]);
}

#[test]
fn noncast_player_aura_chooses_through_hexproof_and_keeps_exact_owned_attachments() {
    const RETURN: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 0,
                relation: 0,
                other: false,
                nonland: true,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            })),
            &[Instruction::ReturnToBattlefield],
        ),
        ..CardRules::EMPTY
    };
    const HEXPROOF: CardRules = CardRules {
        effect: Effect::Permanent,
        static_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::PlayerHexproof],
        }],
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Pacifism").unwrap(), &PLAYER_AURA),
        (Card::from_name("Opt").unwrap(), &RETURN),
        (Card::from_name("Swiftfoot Boots").unwrap(), &HEXPROOF),
    ]);
    let controller = Player::P1;
    let opponent = controller.other();
    let mut parent = scenario(controller);
    put(&mut parent, opponent, "Swiftfoot Boots", Zone::Battlefield);
    let aura = put(&mut parent, controller, "Pacifism", Zone::Graveyard);

    let mut targeted = parent.clone();
    let aura_in_hand = put(&mut targeted, controller, "Pacifism", Zone::Hand);
    act(&mut targeted, Action::AnnounceCast { card: aura_in_hand });
    reject(&mut targeted, target(Target::Player(opponent)));
    assert!(
        targeted
            .legal_actions()
            .contains(&target(Target::Player(controller)))
    );

    let spell = put(&mut parent, controller, "Opt", Zone::Hand);
    cast(&mut parent, spell, Some(Target::Card(aura)));
    act(&mut parent, Action::Pass);
    act(&mut parent, Action::Pass);
    assert_eq!(
        parent.state.pending,
        Pending::AuraEntry { player: controller }
    );
    for viewer in [Player::P0, Player::P1] {
        let entry = parent.observe(viewer).aura_entry.unwrap();
        assert_eq!(entry.aura, parent.object_ref(aura));
        assert_eq!(entry.player_candidates, vec![Player::P0, Player::P1]);
    }
    let attach_opponent = Action::ChooseAuraPlayerAttachment { player: opponent };
    let index = parent
        .legal_actions()
        .iter()
        .position(|action| *action == attach_opponent)
        .unwrap();
    let saved = parent.snapshot();
    let revision = parent.revision();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    drop(parent);
    act(&mut raw, attach_opponent);
    indexed.apply_index(controller, revision, index).unwrap();
    settle(&mut raw);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.object_view(aura).attached_to_player, Some(opponent));

    let aura_ref = raw.object_ref(aura);
    let funding = FundingState::from_game(
        &raw,
        &FundingGoal {
            actor: controller,
            required: [0; 6],
            generic: 0,
            reserve: Some((
                aura_ref,
                AdditionalCost {
                    kind: EffectChoiceKind::Sacrifice,
                    filter: SacrificeFilter {
                        kinds: 0,
                        nonland: true,
                        source_only: false,
                        other: false,
                        tokens: None,
                        subtype: None,
                        artifact_enchantment_or_token: false,
                        keywords: Keywords::NONE,
                    },
                },
            )),
            source: None,
        },
    );
    assert_eq!(
        life_gain::replacement_amounts(&funding, opponent, 3),
        vec![0]
    );
    assert_eq!(
        life_gain::replacement_amounts(&funding, controller, 3),
        vec![3]
    );
    assert!(
        funding.cost_departures(&[aura_ref], Zone::Graveyard, &mut |projected| {
            assert_eq!(
                life_gain::replacement_amounts(projected, opponent, 3),
                vec![3]
            );
            true
        })
    );
    assert_eq!(raw.object_view(aura).attached_to_player, Some(opponent));

    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(controller, revision, index),
        Err(Error::StaleRevision)
    );
    act(
        &mut indexed,
        Action::ChooseAuraPlayerAttachment { player: controller },
    );
    settle(&mut indexed);
    assert_eq!(
        indexed.object_view(aura).attached_to_player,
        Some(controller)
    );
    assert_eq!(raw.object_view(aura).attached_to_player, Some(opponent));

    reach_next_turn(&mut raw);
    let protection = put(&mut raw, opponent, "Absolute Virtue", Zone::Hand);
    raw.state.players[opponent.index()].mana = [8; 6];
    cast(&mut raw, protection, None);
    settle(&mut raw);
    assert_eq!(raw.object_view(protection).zone, Zone::Battlefield);
    assert_eq!(raw.object_view(aura).zone, Zone::Graveyard);
    assert_eq!(raw.object_view(aura).attached_to_player, None);
    assert_eq!(life_gain::replacement_amounts(&raw, opponent, 3), vec![3]);
}
