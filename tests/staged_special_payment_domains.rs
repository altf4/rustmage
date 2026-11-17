//! Special payments expose complete domains without enumerating all allocations.
mod support;

use rustmage::{ManaPaymentAction, ManaPaymentChoice};
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const MANY: ManaCost = ManaCost {
    colored: [0; 6],
    generic: 8,
};
const POOL: Mana = [16; 6];
const WHITE_PAYMENT: Mana = [8, 0, 0, 0, 0, 0];
const DISGUISE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    extras: &CardExtras {
        disguise: Some(DisguiseSpec::fixed(MANY)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const VARIABLE_DISGUISE: CardRules = CardRules {
    extras: &CardExtras {
        disguise: Some(DisguiseSpec {
            mana: ManaCost::ZERO,
            alternatives: &[],
            modifier: DisguiseCostModifier::X(std::num::NonZeroU8::MIN),
        }),
        ..CardExtras::EMPTY
    },
    ..DISGUISE
};
const RESOLUTION_X: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::PayXReflexive { ability: 1 }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::NONE,
            program: &[Instruction::PutCountersScaled {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: AmountValue::ChosenX,
            }],
        },
    ],
    ..CardRules::EMPTY
};
const FORETELL: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, &[]),
    extras: &CardExtras {
        foretell_cost: Some(MANY),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const ROOM: Card = Card::from_catalogue_id(176); // Anthem of Champions
const HALF: Card = Card::from_catalogue_id(1024); // Deeproot Pilgrimage
const OTHER_HALF: Card = Card::from_catalogue_id(2145); // Impact Tremors
const ROOM_RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        room: Some(&RoomSpec {
            halves: [HALF, OTHER_HALF],
            triggers: &[],
            static_groups: &[],
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const HALF_RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        room_parent: Some(ROOM),
        cost_alternatives: &[MANY],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const RESOLUTION: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::MayPayManaAlternatives {
                costs: &[
                    MANY,
                    ManaCost {
                        colored: [0, 1, 0, 0, 0, 0],
                        generic: 7,
                    },
                ],
                skip: 1,
            },
            Instruction::GainLife(PlayerSet::Controller, 3),
        ],
    ),
    ..CardRules::EMPTY
};
const TAX: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::AttackControllerTax {
            amount: 8,
            condition: StatValue::Fixed(1),
        }],
    }],
    ..CardRules::EMPTY
};
const RESTRICTED: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        exhaust: false,
        cycling: false,
        sorcery: false,
        once_per_turn: false,
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Tagged {
            production: &ManaProduction::Fixed(WHITE_PAYMENT),
            quality: &ManaQuality {
                restriction: ManaRestriction::CreatureSpells,
                spell_bonus: ManaSpellBonus::None,
                retention: ManaRetention::Step,
            },
        })],
    }],
    ..CardRules::EMPTY
};
const RESOLUTION_MATERIAL: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::MayPayCosts {
                costs: &[ResolutionCost {
                    mana: MANY,
                    life: Some(AmountValue::Fixed(2)),
                    objects: Some(ResolutionObjectCost {
                        cost: AdditionalCost {
                            kind: EffectChoiceKind::Sacrifice,
                            filter: SacrificeFilter {
                                kinds: 1,
                                source_only: false,
                                nonland: false,
                                other: false,
                                tokens: None,
                                subtype: None,
                                artifact_enchantment_or_token: false,
                                keywords: Keywords::NONE,
                            },
                        },
                        count: 1,
                    }),
                    ..ResolutionCost::FREE
                }],
                skip: 1,
            },
            Instruction::GainLife(PlayerSet::Controller, 3),
        ],
    ),
    ..CardRules::EMPTY
};

fn assert_indexed_payment(game: &Game, action: &Action) {
    let actor = game.actor().unwrap();
    let index = game
        .legal_actions()
        .iter()
        .position(|candidate| candidate == action)
        .unwrap();
    let mut raw = game.clone();
    let mut indexed = game.clone();
    act(&mut raw, action.clone());
    indexed
        .apply_index(actor, indexed.revision(), index)
        .unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
}

fn check_special_space(game: &Game, kind: ManaPaymentAction, unlisted: Mana) -> ManaPaymentChoice {
    let actor = game.actor().unwrap();
    let space = game.legal_action_space();
    let choices = &space.special_mana_payments;
    let choice = *choices
        .iter()
        .find(|choice| choice.action == kind && choice.domain.contains(unlisted))
        .unwrap();
    assert_eq!(game.observe(actor).special_mana_payments, *choices);
    assert!(game.observe(actor.other()).special_mana_payments.is_empty());
    let payment_actions: Vec<_> = space
        .choices
        .iter()
        .filter(|action| {
            matches!(
                action,
                Action::PayFaceUp { .. }
                    | Action::PayRoomUnlock { .. }
                    | Action::PayPlot { .. }
                    | Action::PayForetell { .. }
                    | Action::PayResolutionMana { .. }
            )
        })
        .collect();
    assert!(!payment_actions.is_empty());
    assert!(payment_actions.len() <= 64 * choices.len());
    assert!(!space.choices.contains(&kind.action(unlisted)));
    assert_indexed_payment(game, payment_actions[0]);
    let mut invalid = game.clone();
    reject(&mut invalid, kind.action([0; 6]));
    choice
}

fn no_future_work() {
    let work = Game::payment_planning_work();
    assert_eq!(
        (
            work.projected_moves,
            work.projected_steps,
            work.continuation_plans
        ),
        (0, 0, 0)
    );
}

#[test]
fn plot_x_has_bounded_choices_and_accepts_unlisted_exact_allocations() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        put(
            &mut game,
            player,
            "Fblthp, Lost on the Range",
            Zone::Battlefield,
        );
        let card = put(&mut game, player, "Goldvein Hydra", Zone::Library);
        game.state.players[player.index()].mana = POOL;
        let object = game.object_ref(card);
        act(
            &mut game,
            Action::AnnounceLibraryTopPlot {
                object,
                cost: ManaCost {
                    colored: [0, 0, 0, 0, 1, 0],
                    generic: 0,
                },
                x_symbols: std::num::NonZeroU32::new(1),
            },
        );
        let mut impossible = game.clone();
        Game::reset_payment_planning_work();
        act(&mut impossible, Action::ChooseX { value: 1 << 20 });
        assert!(impossible.special_mana_payments().is_empty());
        assert!(impossible.legal_actions().len() < 16);
        reject(&mut impossible, Action::PayPlot { payment: POOL });
        act(&mut impossible, Action::EndManaWindow);
        act(&mut impossible, Action::SubmitAttempt);
        assert_eq!(
            impossible
                .last_attempt_outcome_view(player)
                .unwrap()
                .outcome,
            AttemptOutcome::Failed
        );
        assert_eq!(impossible.object_ref(card), object);
        assert_eq!(impossible.object_view(card).zone, Zone::Library);
        no_future_work();

        act(&mut game, Action::ChooseX { value: 8 });
        let payment = [8, 0, 0, 0, 1, 0];
        check_special_space(&game, ManaPaymentAction::PayPlot, payment);
        let before = game.snapshot();
        act(&mut game, Action::PayPlot { payment });
        assert_eq!(game.object_view(card).zone, Zone::Exile);
        assert_eq!(
            game.state.players[player.index()].mana,
            [8, 16, 16, 16, 15, 16]
        );
        game.restore(&before).unwrap();
        assert_eq!(game.snapshot(), before);
        assert!(game.special_mana_payments()[0].domain.contains(payment));
    }
}

#[test]
fn face_up_and_room_domains_preserve_source_identity_and_all_allocations() {
    let smuggler = Card::from_name("Bubble Smuggler").unwrap();
    let _rules = fixture_rules::activate(&[
        (smuggler, &DISGUISE),
        (ROOM, &ROOM_RULES),
        (HALF, &HALF_RULES),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let card = spawn(&mut game, player, smuggler, Zone::Hand);
        let hidden = announce_hand_disguise(&mut game, card).card;
        pay(&mut game);
        settle(&mut game);
        game.state.players[player.index()].mana = POOL;
        let object = game.object_ref(hidden);
        act(&mut game, Action::BeginFaceUp { object, cost: MANY });
        check_special_space(&game, ManaPaymentAction::PayFaceUp, WHITE_PAYMENT);
        let mut stale = game.clone();
        stale.move_object(hidden, Zone::Hand);
        assert!(stale.special_mana_payments().is_empty());
        reject(
            &mut stale,
            Action::PayFaceUp {
                payment: WHITE_PAYMENT,
            },
        );
        act(
            &mut game,
            Action::PayFaceUp {
                payment: WHITE_PAYMENT,
            },
        );
        assert_eq!(game.object_ref(hidden), object);
        assert!(!game.is_face_down(object));
        assert_eq!(
            game.state.players[player.index()].mana,
            [8, 16, 16, 16, 16, 16]
        );

        let room = spawn(&mut game, player, ROOM, Zone::Battlefield);
        let object = game.object_ref(room);
        game.state.players[player.index()].mana = POOL;
        act(
            &mut game,
            Action::BeginRoomUnlock {
                object,
                door: RoomDoor::Left,
                cost: MANY,
            },
        );
        check_special_space(&game, ManaPaymentAction::PayRoomUnlock, WHITE_PAYMENT);
        let mut stale = game.clone();
        stale.move_object(room, Zone::Graveyard);
        assert!(stale.special_mana_payments().is_empty());
        reject(
            &mut stale,
            Action::PayRoomUnlock {
                payment: WHITE_PAYMENT,
            },
        );
        act(
            &mut game,
            Action::PayRoomUnlock {
                payment: WHITE_PAYMENT,
            },
        );
        assert_eq!(game.object_ref(room), object);
        assert_eq!(
            game.object_view(room).room_doors,
            Some(RoomDoors {
                left: true,
                right: false
            })
        );
        assert_eq!(
            game.state.players[player.index()].mana,
            [8, 16, 16, 16, 16, 16]
        );
    }
}

#[test]
fn variable_face_up_and_resolution_payments_do_not_enumerate_huge_unpaid_x() {
    let smuggler = Card::from_name("Bubble Smuggler").unwrap();
    let _rules = fixture_rules::activate(&[
        (smuggler, &VARIABLE_DISGUISE),
        (Card::GrizzlyBears, &RESOLUTION_X),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let card = spawn(&mut game, player, smuggler, Zone::Hand);
    let hidden = announce_hand_disguise(&mut game, card).card;
    pay(&mut game);
    settle(&mut game);
    game.state.players[player.index()].mana = POOL;
    let object = game.object_ref(hidden);
    act(
        &mut game,
        Action::BeginFaceUpX {
            object,
            cost: ManaCost::ZERO,
            x_symbols: std::num::NonZeroU8::MIN,
        },
    );
    assert!(game.special_mana_payments().is_empty());
    reject(&mut game, Action::PayFaceUp { payment: [0; 6] });
    let mut impossible = game.clone();
    act(&mut impossible, Action::ChooseX { value: 1 << 20 });
    assert!(impossible.special_mana_payments().is_empty());
    assert!(impossible.legal_actions().len() < 16);
    act(&mut game, Action::ChooseX { value: 8 });
    check_special_space(&game, ManaPaymentAction::PayFaceUp, WHITE_PAYMENT);
    act(
        &mut game,
        Action::PayFaceUp {
            payment: WHITE_PAYMENT,
        },
    );
    assert!(!game.is_face_down(object));

    let source = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    cast(&mut game, source, None);
    for _ in 0..12 {
        if game.state.pending == (Pending::ResolutionX { player }) {
            break;
        }
        choose(&mut game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    assert_eq!(game.state.pending, Pending::ResolutionX { player });
    game.state.players[player.index()].mana = POOL;
    let mut impossible = game.clone();
    act(&mut impossible, Action::ChooseX { value: 1 << 20 });
    assert!(impossible.special_mana_payments().is_empty());
    assert!(impossible.legal_actions().len() < 16);
    reject(&mut impossible, Action::PayResolutionMana { payment: POOL });
    act(&mut impossible, Action::DeclineResolutionMana);
    assert_eq!(impossible.counter_count(source, CounterKind::PlusOne), 0);
    act(&mut game, Action::ChooseX { value: 8 });
    check_special_space(&game, ManaPaymentAction::PayResolutionMana, WHITE_PAYMENT);
    act(
        &mut game,
        Action::PayResolutionMana {
            payment: WHITE_PAYMENT,
        },
    );
    settle(&mut game);
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 8);
}

#[test]
fn resolution_alternatives_share_exact_domains_and_bounded_indexed_payments() {
    let _rules = fixture_rules::activate(&[(Card::Opt, &RESOLUTION)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, spell, None);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.state.pending, Pending::ResolutionMana { player });
    game.state.players[player.index()].mana = POOL;
    Game::reset_payment_planning_work();
    check_special_space(&game, ManaPaymentAction::PayResolutionMana, WHITE_PAYMENT);
    assert_eq!(game.special_mana_payments().len(), 2);
    let mut blue = game.clone();
    act(
        &mut blue,
        Action::PayResolutionMana {
            payment: [7, 1, 0, 0, 0, 0],
        },
    );
    assert_eq!(blue.state.players[player.index()].life, 23);
    act(
        &mut game,
        Action::PayResolutionMana {
            payment: WHITE_PAYMENT,
        },
    );
    assert_eq!(game.state.players[player.index()].life, 23);
    assert_eq!(
        game.state.players[player.index()].mana,
        [8, 16, 16, 16, 16, 16]
    );
    no_future_work();
}

#[test]
fn special_raw_payments_still_require_eligible_mana_and_selected_materials() {
    let _rules = fixture_rules::activate(&[
        (Card::Opt, &RESOLUTION_MATERIAL),
        (Card::GoblinPiker, &RESTRICTED),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Goblin Piker", Zone::Battlefield);
    let victim = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, spell, None);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    game.state.players[player.index()].mana = [0; 6];
    act(
        &mut game,
        Action::ActivateMana {
            card: source,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: WHITE_PAYMENT,
        },
    );
    let object = game.object_ref(victim);
    assert!(game.special_mana_payments().is_empty());
    reject(
        &mut game,
        Action::PayResolutionMana {
            payment: WHITE_PAYMENT,
        },
    );
    act(&mut game, Action::ChooseAdditionalCost { object });
    // The material is selected, but creature-spell-only mana cannot pay a
    // resolution cost, including through the direct numeric validation path.
    assert!(game.special_mana_payments().is_empty());
    reject(
        &mut game,
        Action::PayResolutionMana {
            payment: WHITE_PAYMENT,
        },
    );
    assert_eq!(game.object_ref(victim), object);
    assert_eq!(game.object_view(victim).zone, Zone::Battlefield);
    assert_eq!(game.observe(player).mana_lots[0].amounts[0], 8);
    game.state.players[player.index()].mana[0] += 8;
    assert_eq!(
        game.special_mana_payments()[0].domain.available,
        WHITE_PAYMENT
    );
    let mut no_life = game.clone();
    no_life.state.players[player.index()].life = 1;
    assert!(no_life.special_mana_payments().is_empty());
    reject(
        &mut no_life,
        Action::PayResolutionMana {
            payment: WHITE_PAYMENT,
        },
    );
    act(
        &mut game,
        Action::PayResolutionMana {
            payment: WHITE_PAYMENT,
        },
    );
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].life, 21);
    assert_eq!(game.state.players[player.index()].mana, WHITE_PAYMENT);
    assert_eq!(game.observe(player).mana_lots[0].amounts[0], 8);
}

#[test]
fn foretell_domain_is_private_and_only_accepts_its_action_kind() {
    let _rules = fixture_rules::activate(&[(Card::Opt, &FORETELL)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let card = put(&mut game, player, "Opt", Zone::Hand);
    let object = game.object_ref(card);
    game.state.players[player.index()].mana = POOL;
    act(&mut game, Action::AnnounceForetell { object });
    let choices = game.special_mana_payments();
    assert_eq!(choices.len(), 1);
    assert_eq!(choices[0].action, ManaPaymentAction::PayForetell);
    assert_eq!(choices[0].domain.generic, 2);
    assert!(choices[0].domain.contains([2, 0, 0, 0, 0, 0]));
    assert!(
        game.observe(player.other())
            .special_mana_payments
            .is_empty()
    );
    reject(
        &mut game,
        Action::PayPlot {
            payment: [2, 0, 0, 0, 0, 0],
        },
    );
    let action = Action::PayForetell {
        payment: [2, 0, 0, 0, 0, 0],
    };
    assert_indexed_payment(&game, &action);
    act(&mut game, action);
    assert_eq!(game.object_view(card).zone, Zone::Exile);
    assert_eq!(
        game.state.players[player.index()].mana,
        [14, 16, 16, 16, 16, 16]
    );
}

#[test]
fn any_number_sacrifice_declaration_is_not_capped_by_current_permanents() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let card = put(&mut game, player, "Rottenmouth Viper", Zone::Hand);
        let original = game.object_ref(card);
        game.state.players[player.index()].mana = [0, 0, 1, 0, 0, 0];
        Game::reset_payment_planning_work();
        act(&mut game, Action::AnnounceCast { card });
        assert_eq!(
            game.x_choice().unwrap().intervals,
            vec![XInterval {
                minimum: 0,
                maximum: u32::MAX
            }]
        );
        let declaration = Action::ChooseX { value: u32::MAX };
        assert_indexed_payment(&game, &declaration);
        act(&mut game, declaration);
        assert_eq!(
            game.state
                .announcing
                .as_ref()
                .unwrap()
                .sacrifice_discount_count(),
            Some(u32::MAX)
        );
        assert_eq!(
            game.state
                .announcing
                .as_ref()
                .unwrap()
                .total_cost
                .unwrap()
                .generic,
            0
        );
        act(&mut game, Action::BeginOrderedCosts);
        let clause = game
            .observe(player)
            .cast_costs
            .unwrap()
            .clauses
            .iter()
            .position(|row| row.clause == CastCostClause::Additional && !row.paid)
            .unwrap() as u16;
        act(&mut game, Action::ChooseCastCost { clause });
        assert_eq!(game.state.pending, Pending::AdditionalCost { player });
        if game.attempt_view(player).unwrap().can_end_mana_window {
            act(&mut game, Action::EndManaWindow);
        }
        assert_indexed_payment(&game, &Action::SubmitAttempt);
        act(&mut game, Action::SubmitAttempt);
        let outcome = game.last_attempt_outcome_view(player).unwrap();
        assert_eq!(outcome.outcome, AttemptOutcome::Failed);
        assert_eq!(
            outcome.reason,
            Some(AttemptFailureReason::UnsatisfiedMaterial)
        );
        assert_eq!(game.object_ref(card), original);
        assert_eq!(game.object_view(card).zone, Zone::Hand);
        assert_eq!(game.state.players[player.index()].mana, [0, 0, 1, 0, 0, 0]);
        assert!(game.state.stack.is_empty());
        assert!(
            game.legal_actions()
                .contains(&Action::AnnounceCast { card })
        );
        no_future_work();
    }
}

#[test]
fn combat_tax_uses_bounded_representatives_and_full_payment_membership() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Short Bow").unwrap(), &TAX)]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player.other(), "Short Bow", Zone::Battlefield);
    let attacker = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    game.state.players[player.index()].mana = POOL;
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: attacker });
    act(&mut game, Action::FinishAttackers);
    assert_eq!(game.state.pending, Pending::AttackPayment { player });
    let actions = game.legal_actions();
    assert_eq!(
        actions
            .iter()
            .filter(|a| matches!(a, Action::PayAttackCost { .. }))
            .count(),
        64
    );
    let payment = Action::PayAttackCost {
        payment: WHITE_PAYMENT,
    };
    assert!(!actions.contains(&payment));
    assert!(game.mana_payment_domain().unwrap().contains(WHITE_PAYMENT));
    let representative = actions
        .iter()
        .find(|a| matches!(a, Action::PayAttackCost { .. }))
        .unwrap();
    assert_indexed_payment(&game, representative);
    act(&mut game, payment);
    assert!(game.object_view(attacker).tapped);
    assert_eq!(
        game.state.players[player.index()].mana,
        [8, 16, 16, 16, 16, 16]
    );
}
