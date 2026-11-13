//! Exact combat and cleanup triggers, and their owned resolution choices.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardRules, one};
use rustmage_core::{cards::*, engine::*};
use support::*;

const P: Player = Player::P0;
const ENEMY: Player = Player::P1;

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn declare_attacks(game: &mut Game, attackers: &[CardId]) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    for &card in attackers {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

const TAP_GROW: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    keywords: Keywords::VIGILANCE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::TapAnyUntappedPermanents,
                players: PlayerSet::Controller,
                filter: SacrificeFilter {
                    kinds: 1,
                    keywords: Keywords::NONE,
                    source_only: false,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
                amount: 0,
                optional_skip: None,
            },
            Instruction::GrowScaled {
                subject: Subject::Source,
                amount: AmountValue::RememberedCount,
                power: 1,
                toughness: 1,
            },
        ],
    }],
    ..CardRules::EMPTY
};
const TAP_OBSERVER: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ControlledCreaturesBecomeTapped,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

#[test]
fn choose_any_number_of_untapped_creatures_and_tap_them_in_one_event() {
    let observer = card("Banishing Light");
    let _rules =
        fixture_rules::activate(&[(Card::GrizzlyBears, &TAP_GROW), (observer, &TAP_OBSERVER)]);
    let mut game = scenario(P);
    let source = put(&mut game, P, "Grizzly Bears", Zone::Battlefield);
    let ready = put(&mut game, P, "Hill Giant", Zone::Battlefield);
    let tapped = put(&mut game, P, "Hill Giant", Zone::Battlefield);
    let opposing = put(&mut game, ENEMY, "Hill Giant", Zone::Battlefield);
    spawn(&mut game, P, observer, Zone::Battlefield);
    game.state.objects[ready as usize].summoning_sick = true;
    game.state.objects[tapped as usize].tapped = true;
    declare_attacks(&mut game, &[source]);
    pass_pair(&mut game);

    let saved = game.snapshot();
    let mut empty = game.clone();
    act(&mut empty, Action::FinishEffectChoice);
    assert_eq!(stats(&empty, source), (2, 1));
    for (id, allowed) in [
        (source, true),
        (ready, true),
        (tapped, false),
        (opposing, false),
    ] {
        let action = Action::ChooseEffectObject {
            object: game.object_ref(id),
        };
        if allowed {
            act(&mut game, action.clone());
            reject(&mut game, action);
        } else {
            reject(&mut game, action);
        }
    }
    assert!(!game.object_view(source).tapped && !game.object_view(ready).tapped);
    let mut indexed = game.clone();
    let index = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == Action::FinishEffectChoice)
        .unwrap();
    indexed.apply_index(P, indexed.revision(), index).unwrap();
    act(&mut game, Action::FinishEffectChoice);
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert!(game.object_view(source).tapped && game.object_view(ready).tapped);
    assert_eq!(stats(&game, source), (4, 3));
    assert_eq!(game.state.stack.len(), 1);
    settle(&mut game);
    assert_eq!(game.state.players[P.index()].life, 21);

    game.restore(&saved).unwrap();
    act(&mut game, Action::FinishEffectChoice);
    assert_eq!(game.snapshot(), empty.snapshot());
}

const COUNTER_CHOICE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SURVIVAL,
        targets: TargetPlan::NONE,
        program: &[Instruction::ChooseSourceCounterKind(&[
            CounterKind::Flying,
            CounterKind::Lifelink,
            CounterKind::PlusOne,
        ])],
    }],
    ..CardRules::EMPTY
};

#[test]
fn counter_kind_is_chosen_at_resolution_for_the_exact_source() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &COUNTER_CHOICE)]);
    let mut game = scenario(P);
    let source = put(&mut game, P, "Grizzly Bears", Zone::Battlefield);
    game.state.objects[source as usize].tapped = true;
    game.state.players[P.index()].turn_facts.main_phases_started = 1;
    game.begin_step(Step::Main2);
    game.prepare_triggers();
    assert!(
        !game
            .legal_actions()
            .iter()
            .any(|a| matches!(a, Action::ChooseSourceCounterKind { .. }))
    );

    let mut blink = game.clone();
    blink.move_object(source, Zone::Exile);
    blink.move_object(source, Zone::Battlefield);
    settle(&mut blink);
    assert_eq!(blink.counter_count(source, CounterKind::Flying), 0);

    pass_pair(&mut game);
    reject(
        &mut game,
        Action::ChooseSourceCounterKind {
            counter: CounterKind::Stun,
        },
    );
    let checkpoint = game.snapshot();
    for counter in [
        CounterKind::Flying,
        CounterKind::Lifelink,
        CounterKind::PlusOne,
    ] {
        let mut branch = game.clone();
        act(&mut branch, Action::ChooseSourceCounterKind { counter });
        assert_eq!(branch.counter_count(source, counter), 1);
        assert_eq!(branch.state.pending, Pending::Priority);
        branch.restore(&checkpoint).unwrap();
        assert_eq!(branch.snapshot(), checkpoint);
    }
}

const BLOCKED: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ControlledCreatureBecomesBlocked,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayIfEventPermanentOnBattlefield { skip: 1 },
            Instruction::ExileEventPermanentPlayThisTurn,
        ],
    }],
    ..CardRules::EMPTY
};

#[test]
fn each_newly_blocked_attacker_gets_one_optional_exact_play_permission() {
    let observer = card("Bear Cub");
    let _rules = fixture_rules::activate(&[(observer, &BLOCKED)]);
    let mut game = scenario(P);
    let source = spawn(&mut game, P, observer, Zone::Battlefield);
    let attackers = [
        put(&mut game, P, "Grizzly Bears", Zone::Battlefield),
        put(&mut game, P, "Hill Giant", Zone::Battlefield),
    ];
    let b1 = put(&mut game, ENEMY, "Grizzly Bears", Zone::Battlefield);
    let b2 = put(&mut game, ENEMY, "Grizzly Bears", Zone::Battlefield);
    let b3 = put(&mut game, ENEMY, "Grizzly Bears", Zone::Battlefield);
    declare_attacks(&mut game, &attackers);
    pass_pair(&mut game);
    for (blocker, attacker) in [(b1, attackers[0]), (b2, attackers[0]), (b3, attackers[1])] {
        act(&mut game, Action::Block { blocker, attacker });
    }
    act(&mut game, Action::FinishBlockers);
    assert_eq!(game.state.ordering_triggers.len(), 2);
    choose(&mut game, |action| {
        matches!(action, Action::PutTrigger { .. })
    });
    assert_eq!(game.state.stack.len(), 2);
    assert!(
        game.state
            .stack
            .iter()
            .all(|item| item.source.unwrap().id == source)
    );

    let subject = game.state.stack.last().unwrap().entering_object.unwrap();
    let mut stale = game.clone();
    stale.move_object(subject.card, Zone::Exile);
    stale.move_object(subject.card, Zone::Battlefield);
    pass_pair(&mut stale);
    assert_eq!(stale.state.pending, Pending::Priority);
    assert!(stale.observe(P).exile_permissions.is_empty());

    game.move_object(source, Zone::Graveyard);
    pass_pair(&mut game);
    act(&mut game, Action::ChooseOptional { accept: true });
    assert_eq!(game.object_view(subject.card).zone, Zone::Exile);
    let view = game.observe(P);
    let permission = view
        .exile_permissions
        .iter()
        .find(|row| row.object == game.object_ref(subject.card))
        .unwrap();
    assert_eq!(permission.expiry, ExilePlayExpiry::Turn(game.state.turn));
    pass_pair(&mut game);
    act(&mut game, Action::ChooseOptional { accept: false });
    let other = attackers
        .into_iter()
        .find(|&id| id != subject.card)
        .unwrap();
    assert_eq!(game.object_view(other).zone, Zone::Battlefield);
}

const WATCH_DEATHS: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::creature_you_control()),
        &[
            Instruction::CreaturePowerDamageBatch {
                source: 0,
                recipients: CreaturePowerDamageRecipients::OtherCreatures,
            },
            Instruction::ScheduleTurnTrigger {
                event: TurnCallbackEvent::ControlledCreatureDeath,
                ability: 0,
                once: false,
            },
        ],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::NONE,
        program: &[Instruction::ExileTopPlay {
            amount: 1,
            next_turn: true,
        }],
    }],
    ..CardRules::EMPTY
};

#[test]
fn death_callback_catches_simultaneous_tokens_and_cards_then_expires_in_cleanup() {
    let spell = card("Pyroclasm");
    let token = card("@token/cat_beast_2_2_white");
    let _rules = fixture_rules::activate(&[(spell, &WATCH_DEATHS)]);
    let mut game = scenario(P);
    let dealer = put(&mut game, P, "Hill Giant", Zone::Battlefield);
    let own = put(&mut game, P, "Grizzly Bears", Zone::Battlefield);
    let opposing = put(&mut game, ENEMY, "Grizzly Bears", Zone::Battlefield);
    let token_id = game.state.objects.len() as CardId;
    game.create_tokens(P, token, 1, false);
    let cast_id = spawn(&mut game, P, spell, Zone::Hand);
    cast(&mut game, cast_id, Some(Target::Card(dealer)));
    pass_pair(&mut game);
    assert_eq!(game.object_view(own).zone, Zone::Graveyard);
    assert_eq!(game.object_view(opposing).zone, Zone::Graveyard);
    assert_eq!(game.object_view(token_id).zone, Zone::Outside);
    assert_eq!(game.state.ordering_triggers.len(), 2);
    choose(&mut game, |action| {
        matches!(action, Action::PutTrigger { .. })
    });
    settle(&mut game);
    assert_eq!(game.observe(P).exile_permissions.len(), 2);
    let turn = game.state.turn;
    game.finish_cleanup();
    assert_eq!(game.state.turn, turn + 1);
    game.move_object(dealer, Zone::Graveyard);
    game.prepare_triggers();
    assert!(game.state.stack.is_empty());
}

const FEAR: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacksUnlessDefenderControlsCreatureSubtype("Glimmer"),
        targets: TargetPlan::NONE,
        program: &[Instruction::GrantKeywords {
            subject: Subject::Source,
            keywords: Keywords::MENACE.union(Keywords::DEATHTOUCH),
        }],
    }],
    ..CardRules::EMPTY
};

#[test]
fn the_defending_players_current_creature_subtype_is_checked_twice() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &FEAR)]);
    let mut game = scenario(P);
    let source = put(&mut game, P, "Grizzly Bears", Zone::Battlefield);
    let glimmer = put(&mut game, P, "Enduring Curiosity", Zone::Battlefield);
    let glimmer_ref = game.object_ref(glimmer);
    let mut too_early = game.clone();
    too_early.change_controller(glimmer_ref, ENEMY);
    declare_attacks(&mut too_early, &[source]);
    assert!(too_early.state.stack.is_empty());

    declare_attacks(&mut game, &[source]);
    let trigger = game.state.stack.last().unwrap();
    assert_eq!(trigger.event_player, Some(ENEMY));
    let mut response = game.clone();
    response.change_controller(glimmer_ref, ENEMY);
    settle(&mut response);
    assert!(
        !response
            .object_view(source)
            .keywords
            .contains(Keywords::MENACE)
    );
    settle(&mut game);
    let keywords = game.object_view(source).keywords;
    assert!(keywords.contains(Keywords::MENACE.union(Keywords::DEATHTOUCH)));
}
