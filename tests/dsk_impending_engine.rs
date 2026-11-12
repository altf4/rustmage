//! Impending records a paid cast decision on the exact resulting permanent.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const SOURCE: &str = "Appendage Amalgam";
const BLACK: Mana = [0, 0, 1, 0, 0, 0];
const IMPENDING_COST: ManaCost = ManaCost {
    colored: BLACK,
    generic: 0,
};
const IMPENDING: ImpendingSpec = ImpendingSpec {
    cost: IMPENDING_COST,
    time_counters: 2,
};
const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 2)],
        },
        ImpendingSpec::END_STEP,
    ],
    extras: &CardExtras {
        impending: Some(IMPENDING),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const EQUAL_COST: CardRules = CardRules {
    extras: &CardExtras {
        impending: Some(ImpendingSpec {
            cost: ManaCost {
                colored: BLACK,
                generic: 2,
            },
            ..IMPENDING
        }),
        ..CardExtras::EMPTY
    },
    ..RULES
};

const EDIT_PERMANENT: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    sorcery: false,
    once_per_turn: false,
    cost: ActivationCost::ZERO,
    targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
        zone: TargetZone::Battlefield,
        kinds: 0,
        relation: -1,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    })),
    program: &[],
};
const HELPER: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    activated: &[
        ActivatedAbility {
            program: &[Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::Time,
                amount: 1,
            }],
            ..EDIT_PERMANENT
        },
        ActivatedAbility {
            program: &[Instruction::GainControl {
                subject: Subject::Targets,
                controller: PlayerSet::Controller,
            }],
            ..EDIT_PERMANENT
        },
        ActivatedAbility {
            program: &[Instruction::BlinkImmediately {
                counter: None,
                tapped: false,
            }],
            ..EDIT_PERMANENT
        },
        ActivatedAbility {
            program: &[Instruction::CreateCopyTokenWithExceptions {
                subject: Subject::Targets,
                amount: 1,
                exceptions: CopyExceptions::NONE,
            }],
            ..EDIT_PERMANENT
        },
        ActivatedAbility {
            program: &[Instruction::AnimateTargets(AnimationSpec {
                artifact: false,
                creature_types: AnimationCreatureTypes::Additional(&["Golem"]),
                base: Some((4, 4)),
                keywords: Keywords::NONE,
                duration: AnimationDuration::EndOfTurn,
            })],
            ..EDIT_PERMANENT
        },
    ],
    ..CardRules::EMPTY
};

fn choice(game: &Game, actor: Player, impending: bool) -> Action {
    let option = game
        .observe(actor)
        .cost_choices
        .into_iter()
        .find(|option| option.impending == impending)
        .expect("spell cost route");
    Action::ChooseSpellCost { cost: option.index }
}

fn cast_impending(game: &mut Game, player: Player, source: CardId) {
    act(game, Action::AnnounceCast { card: source });
    let option = choice(game, player, true);
    act(game, option);
    pay(game);
    assert!(game.state.stack.iter().any(|item| {
        item.source.is_some_and(|view| view.id == source)
            && item.cast.is_some_and(|cast| cast.impending)
    }));
    settle(game);
}

fn edit_permanent(game: &mut Game, helper: CardId, ability: u16, target: CardId) {
    act(
        game,
        Action::AnnounceActivate {
            card: helper,
            ability,
            counter_payment: Counters::default(),
        },
    );
    act(
        game,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
    settle(game);
}

fn advance_to_end(game: &mut Game, player: Player) {
    for _ in 0..80 {
        if game.state.active == player
            && game.state.step == Step::End
            && game.state.pending == Pending::Priority
        {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass | Action::FinishAttackers | Action::FinishBlockers
            )
        });
    }
    panic!("did not reach {player:?}'s end step");
}

fn assert_form(game: &Game, card: CardId, paid: bool, time: u64, creature: bool) {
    let view = game.object_view(card);
    assert_eq!(view.zone, Zone::Battlefield);
    assert_eq!(view.cast_for_impending, paid);
    assert_eq!(view.counters.get(CounterKind::Time), time);
    assert_eq!(view.kinds & 32, 32);
    assert_eq!(view.kinds & 1 != 0, creature);
}

#[test]
fn equal_cost_choices_and_the_paid_incarnation_survive_owned_raw_index_and_restore() {
    let source = Card::from_name(SOURCE).unwrap();
    let _rules = fixture_rules::activate(&[(source, &EQUAL_COST)]);
    for player in [Player::P0, Player::P1] {
        let mut parent = scenario(player);
        let card = put(&mut parent, player, SOURCE, Zone::Hand);
        act(&mut parent, Action::AnnounceCast { card });
        let cost = parent.observe(player).cost_choices;
        assert_eq!(cost.len(), 2);
        assert_eq!(cost[0].cost, cost[1].cost);
        assert!(parent.observe(player.other()).cost_choices.is_empty());
        reject(&mut parent, Action::ChooseSpellCost { cost: u16::MAX });
        let action = choice(&parent, player, true);
        let index = parent
            .legal_actions()
            .iter()
            .position(|a| a == &action)
            .unwrap();
        let saved = parent.snapshot();
        let mut raw = parent.clone();
        let mut indexed = parent.clone();
        drop(parent);

        act(&mut raw, action);
        indexed
            .apply_index(player, indexed.revision(), index)
            .unwrap();
        pay(&mut raw);
        pay(&mut indexed);
        settle(&mut raw);
        settle(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_form(&raw, card, true, 2, false);
        assert!(!raw.has_subtype(card, "Horror"));
        assert_eq!(raw.state.players[player.index()].life, 22);

        indexed.restore(&saved).unwrap();
        let normal = choice(&indexed, player, false);
        act(&mut indexed, normal);
        pay(&mut indexed);
        settle(&mut indexed);
        assert_form(&indexed, card, false, 0, true);
        assert!(indexed.has_subtype(card, "Horror"));
        assert_form(&raw, card, true, 2, false);
    }
}

#[test]
fn the_current_controller_ticks_and_zero_counters_do_not_erase_the_cast_decision() {
    let source = Card::from_name(SOURCE).unwrap();
    let _rules = fixture_rules::activate(&[(source, &RULES), (Card::HillGiant, &HELPER)]);
    let player = Player::P0;
    let other = player.other();
    let mut game = scenario(player);
    let helper = put(&mut game, other, "Hill Giant", Zone::Battlefield);
    let card = put(&mut game, player, SOURCE, Zone::Hand);
    game.state.players[player.index()].mana = BLACK;
    cast_impending(&mut game, player, card);
    assert_form(&game, card, true, 2, false);

    act(&mut game, Action::Pass);
    edit_permanent(&mut game, helper, 1, card);
    assert_eq!(game.object_view(card).controller, other);

    advance_to_end(&mut game, player);
    settle(&mut game);
    assert_form(&game, card, true, 2, false);
    advance_to_end(&mut game, other);
    settle(&mut game);
    assert_form(&game, card, true, 1, false);

    advance_to_end(&mut game, player);
    settle(&mut game);
    advance_to_end(&mut game, other);
    settle(&mut game);
    assert_form(&game, card, true, 0, true);
    assert!(game.has_subtype(card, "Horror"));

    edit_permanent(&mut game, helper, 0, card);
    assert_form(&game, card, true, 1, false);
    let previous = game.object_ref(card);
    edit_permanent(&mut game, helper, 2, card);
    assert_ne!(game.object_ref(card), previous);
    assert_form(&game, card, false, 0, true);
    assert_eq!(game.object_view(card).controller, player);
    edit_permanent(&mut game, helper, 0, card);
    assert_form(&game, card, false, 1, true);
}

#[test]
fn a_spell_copy_retains_the_paid_choice_and_a_battlefield_copy_does_not() {
    const COPY_SPELL: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        triggers: &[TriggeredAbility {
            condition: TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::Creature,
                ordinal: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::CopyCapturedSpell {
                count: SpellCopyCount::One,
                retarget: false,
            }],
        }],
        ..CardRules::EMPTY
    };
    let source = Card::from_name(SOURCE).unwrap();
    let _rules = fixture_rules::activate(&[
        (source, &RULES),
        (Card::GrizzlyBears, &COPY_SPELL),
        (Card::HillGiant, &HELPER),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let helper = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let card = put(&mut game, player, SOURCE, Zone::Hand);
    cast_impending(&mut game, player, card);
    let copies = |game: &Game| {
        game.state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, object)| {
                object.card == source && object.zone == Zone::Battlefield && object.is_token()
            })
            .map(|(id, _)| id as CardId)
            .collect::<Vec<_>>()
    };
    let first = copies(&game);
    assert_eq!(first.len(), 1);
    assert_form(&game, first[0], true, 2, false);
    assert_form(&game, card, true, 2, false);
    edit_permanent(&mut game, helper, 3, card);
    let both = copies(&game);
    assert_eq!(both.len(), 2);
    let copy = both.into_iter().find(|id| *id != first[0]).unwrap();
    assert_form(&game, copy, false, 0, true);
}

#[test]
fn entry_replacement_and_later_animation_use_the_live_pending_type() {
    const NO_COUNTERS: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        static_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::CounterProhibition(32)],
        }],
        ..CardRules::EMPTY
    };
    let source = Card::from_name(SOURCE).unwrap();
    let _rules = fixture_rules::activate(&[
        (source, &RULES),
        (Card::GrizzlyBears, &NO_COUNTERS),
        (Card::HillGiant, &HELPER),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let prohibition = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let helper = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let card = put(&mut game, player, SOURCE, Zone::Exile);
    game.grant_exile_play_permission(player, game.object_ref(card), false);
    game.state.players[player.index()].mana = BLACK;
    act(&mut game, Action::AnnounceExile { card });
    let option = choice(&game, player, true);
    act(&mut game, option);
    pay(&mut game);
    settle(&mut game);
    assert_form(&game, card, true, 0, true);

    game.move_object(prohibition, Zone::Exile);
    edit_permanent(&mut game, helper, 0, card);
    assert_form(&game, card, true, 1, false);
    edit_permanent(&mut game, helper, 4, card);
    assert_form(&game, card, true, 1, true);
    assert!(game.has_subtype(card, "Golem"));
    assert!(!game.has_subtype(card, "Horror"));
}

#[test]
fn impending_keeps_mandatory_additional_costs_and_cannot_combine_with_free_casts() {
    const DISCARD: CardRules = CardRules {
        extras: &CardExtras {
            impending: Some(IMPENDING),
            additional_cost: Some(AdditionalCost {
                kind: EffectChoiceKind::Discard,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 0,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
            }),
            ..CardExtras::EMPTY
        },
        ..RULES
    };
    let source = Card::from_name(SOURCE).unwrap();
    let _rules = fixture_rules::activate(&[(source, &DISCARD)]);
    let player = Player::P1;
    let mut game = scenario(player);
    game.state.players[player.index()].mana = BLACK;
    let card = put(&mut game, player, SOURCE, Zone::Hand);
    reject(&mut game, Action::AnnounceCast { card });
    let discard = put(&mut game, player, "Forest", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card });
    let option = choice(&game, player, true);
    act(&mut game, option);
    let object = game.object_ref(discard);
    act(&mut game, Action::ChooseAdditionalCost { object });
    pay(&mut game);
    settle(&mut game);
    assert_form(&game, card, true, 2, false);
    assert_eq!(game.object_view(discard).zone, Zone::Graveyard);

    let card = put(&mut game, player, SOURCE, Zone::Exile);
    let discard = put(&mut game, player, "Forest", Zone::Hand);
    let object = game.object_ref(discard);
    game.grant_exile_free_permission(player, game.object_ref(card));
    act(
        &mut game,
        Action::AnnounceExileFree {
            card,
            face: CastFace::Normal,
            flash_surcharge: false,
        },
    );
    assert!(game.observe(player).cost_choices.is_empty());
    act(&mut game, Action::ChooseAdditionalCost { object });
    pay(&mut game);
    assert!(!game.state.stack.last().unwrap().cast.unwrap().impending);
    settle(&mut game);
    assert_form(&game, card, false, 0, true);
    assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
}
