//! Exact destruction, linked exile, and resolved entry witnesses.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::engine::characteristic_layers::AnimationTypeMode;
use rustmage_core::{cards::*, engine::*};
use support::*;

const CARD_GIFT: GiftSpec = GiftSpec {
    kind: GiftKind::Card,
    token: None,
    promised_targets: TargetPlan::NONE,
};

const TARGET_OPPOSING_CREATURE: TargetPlan =
    TargetPlan::Single(TargetSpec::one(TargetRule::Object {
        zone: TargetZone::Battlefield,
        kinds: 1,
        relation: 1,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    }));

const TARGET_OWN_GRAVEYARD_CREATURE: TargetPlan =
    TargetPlan::Single(TargetSpec::one(TargetRule::Object {
        zone: TargetZone::Graveyard,
        kinds: 1,
        relation: 0,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    }));

const SWEEP: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::GiveGift,
            Instruction::DestroyAllCreaturesRememberOwned,
            Instruction::IfGift {
                promised: true,
                skip: 1,
            },
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::ReturnRememberedCreature,
                players: PlayerSet::Controller,
                filter: SacrificeFilter {
                    artifact_enchantment_or_token: false,
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 1,
                    nonland: false,
                    other: false,
                    tokens: None,
                    subtype: None,
                },
                amount: 1,
                optional_skip: None,
            },
        ],
    ),
    extras: &CardExtras {
        gift: Some(CARD_GIFT),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

const EXILER: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TARGET_OPPOSING_CREATURE,
        program: &[
            Instruction::ExileUntilSourceLeavesRememberPowerAtMost(2),
            Instruction::IfAmountAtLeast {
                value: AmountValue::MovedCount,
                minimum: 1,
                skip: 1,
            },
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
        ],
    }],
    ..CardRules::EMPTY
};

const RETURN_AND_COPY: CardRules = CardRules {
    effect: Effect::Spell(
        TARGET_OWN_GRAVEYARD_CREATURE,
        &[
            Instruction::GiveGift,
            Instruction::CaptureEntryResult,
            Instruction::ReturnToBattlefieldUnderControl,
            Instruction::IfGift {
                promised: true,
                skip: 1,
            },
            Instruction::CopyNonlegendaryEntryResult { base: (1, 1) },
        ],
    ),
    extras: &CardExtras {
        gift: Some(GiftSpec {
            promised_targets: TARGET_OWN_GRAVEYARD_CREATURE,
            ..CARD_GIFT
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

const COPY_CREATURE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    extras: &CardExtras {
        entry_copy: Some(EntryCopySpec {
            filters: &[BattlefieldFilter {
                kinds: 1,
                relation: -1,
                nonland: false,
                tapped: None,
                without: Keywords::NONE,
                subtype: None,
                max_mana: u32::MAX,
            }],
            optional: true,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn resolve_top(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn cast_with_gift(game: &mut Game, card: CardId, target: Option<CardId>, gift: bool) {
    let beneficiary = gift.then(|| game.actor().unwrap().other());
    act(game, Action::AnnounceCast { card });
    act(game, Action::ChooseGift { beneficiary });
    if let Some(target) = target {
        act(
            game,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
    }
    pay(game);
}

fn create_creature_token(game: &mut Game, player: Player) -> CardId {
    let id = game.state.objects.len() as CardId;
    game.create_tokens(
        player,
        named("@token/shapeshifter_1_1_colorless_changeling"),
        1,
        false,
    );
    id
}

#[test]
fn sweep_remembers_only_owned_creature_cards_actually_destroyed_into_the_graveyard() {
    let _rules = fixture_rules::activate(&[(named("Day of Judgment"), &SWEEP)]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let spell = put(&mut game, player, "Day of Judgment", Zone::Hand);
        let stolen = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        game.change_controller(game.object_ref(stolen), opponent);
        let foreign = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
        game.change_controller(game.object_ref(foreign), player);
        let second = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let finality = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        game.add_counter(finality, CounterKind::Finality, 1);
        let indestructible = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        game.state.objects[indestructible as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
        let animated = put(&mut game, player, "Soul-Guide Lantern", Zone::Battlefield);
        game.change_object_form(
            game.object_ref(animated),
            AnimationSpec {
                artifact: false,
                creature_types: AnimationCreatureTypes::Unchanged,
                base: Some((2, 2)),
                keywords: Keywords::NONE,
                duration: AnimationDuration::EndOfTurn,
            },
            AnimationTypeMode::AddCreature,
        );
        let old = put(&mut game, player, "Hill Giant", Zone::Graveyard);
        let token = create_creature_token(&mut game, player);
        let old_stolen = game.object_ref(stolen);

        let mut declined = game.clone();
        cast_with_gift(&mut declined, spell, None, false);
        settle(&mut declined);
        assert_eq!(declined.object_view(stolen).zone, Zone::Graveyard);

        cast_with_gift(&mut game, spell, None, true);
        resolve_top(&mut game);
        assert_eq!(
            game.state.players[opponent.index()].cards_drawn_this_turn,
            1
        );
        assert_eq!(game.object_view(finality).zone, Zone::Exile);
        assert_eq!(game.object_view(indestructible).zone, Zone::Battlefield);
        for bad in [foreign, finality, indestructible, animated, old, token] {
            let object = game.object_ref(bad);
            reject(&mut game, Action::SelectGraveyardReturn { object });
        }
        reject(
            &mut game,
            Action::SelectGraveyardReturn { object: old_stolen },
        );
        reject(&mut game, Action::FinishGraveyardReturn);
        let object = game.object_ref(stolen);
        act(&mut game, Action::SelectGraveyardReturn { object });
        assert_eq!(game.object_view(stolen).zone, Zone::Battlefield);
        assert_eq!(game.object_view(stolen).controller, player);
        assert_eq!(game.object_view(second).zone, Zone::Graveyard);
    }
}

fn target_with_exiler(game: &mut Game, player: Player, target: CardId) -> CardId {
    let source = put(game, player, "Hill Giant", Zone::Hand);
    cast(game, source, None);
    resolve_top(game);
    act(
        game,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    source
}

#[test]
fn linked_exile_uses_signed_power_when_the_move_actually_occurs() {
    let _rules = fixture_rules::activate(&[(named("Hill Giant"), &EXILER)]);
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let victim = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    let source = target_with_exiler(&mut game, player, victim);
    let mut larger = game.clone();
    larger.add_counter(victim, CounterKind::PlusOne, 1);
    settle(&mut larger);
    assert_eq!(larger.object_view(victim).zone, Zone::Exile);
    assert_eq!(larger.counter_count(source, CounterKind::PlusOne), 0);

    let mut gone = game.clone();
    gone.move_object(source, Zone::Graveyard);
    settle(&mut gone);
    assert_eq!(gone.object_view(victim).zone, Zone::Battlefield);

    game.state.objects[victim as usize].growth.power = -4;
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Exile);
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 1);
    game.move_object(source, Zone::Graveyard);
    assert_eq!(game.object_view(victim).zone, Zone::Battlefield);

    let mut tokens = scenario(player);
    let token = create_creature_token(&mut tokens, opponent);
    let source = target_with_exiler(&mut tokens, player, token);
    settle(&mut tokens);
    assert_eq!(tokens.counter_count(source, CounterKind::PlusOne), 1);
    tokens.move_object(source, Zone::Graveyard);
    assert_ne!(tokens.object_view(token).zone, Zone::Battlefield);
}

#[test]
fn copy_uses_the_entrants_actual_copiable_characteristics_and_legendary_status() {
    let _rules = fixture_rules::activate(&[
        (named("Zombify"), &RETURN_AND_COPY),
        (Card::GrizzlyBears, &COPY_CREATURE),
    ]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Zombify", Zone::Hand);
    let entrant = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    let ordinary = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let legendary = put(&mut game, opponent, "Krenko, Mob Boss", Zone::Battlefield);
    game.add_counter(ordinary, CounterKind::PlusOne, 2);
    let mut invalidated = game.clone();
    cast_with_gift(&mut invalidated, spell, Some(entrant), true);
    invalidated.move_object(entrant, Zone::Exile);
    settle(&mut invalidated);
    assert_eq!(
        invalidated.state.players[opponent.index()].cards_drawn_this_turn,
        0
    );

    cast_with_gift(&mut game, spell, Some(entrant), true);
    resolve_top(&mut game);
    assert_eq!(
        game.state.players[opponent.index()].cards_drawn_this_turn,
        1
    );
    let before_copy = game.snapshot();
    let mut legendary_copy = game.clone();
    let object = Some(game.object_ref(ordinary));
    let choice = Action::ChooseEntryCopy { object };
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == choice)
        .unwrap();
    let revision = game.revision();
    let mut indexed = game.clone();
    indexed.apply_index(player, revision, index).unwrap();
    act(&mut game, choice);
    settle(&mut indexed);
    settle(&mut game);
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.object_view(entrant).card, named("Hill Giant"));
    assert_eq!(
        game.stats(entrant),
        Stats {
            power: 3,
            toughness: 3
        }
    );
    let view = game.observe(player);
    let tokens: Vec<_> = view
        .objects
        .iter()
        .filter(|object| object.is_token())
        .collect();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].card, named("Hill Giant"));
    assert_eq!(tokens[0].power, 1);
    assert_eq!(tokens[0].toughness, 1);

    let object = Some(legendary_copy.object_ref(legendary));
    act(&mut legendary_copy, Action::ChooseEntryCopy { object });
    settle(&mut legendary_copy);
    assert!(legendary_copy.object_view(entrant).is_legendary());
    assert!(
        !legendary_copy
            .observe(player)
            .objects
            .iter()
            .any(|object| object.is_token())
    );
    indexed.restore(&before_copy).unwrap();
    assert_eq!(indexed.snapshot(), before_copy);
}
