//! Actual damage batches and exact graveyard results used by MKM triggers.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const CLUE: Card = Card::from_catalogue_id(5296);
const DAMAGE_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDamagedAtLeast(3),
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: CLUE,
            amount: 1,
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};

const WITHER_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledPermanentKeywords {
            kinds: 1,
            other: false,
            keywords: Keywords::WITHER,
        }],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreatureDiesWithMaximumToughness {
            relation: CastRelation::Opponent,
            maximum: 0,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

const ZERO_POWER: CardRules = CardRules {
    effect: Effect::Creature(0, 3),
    ..CardRules::EMPTY
};

const DEATH_OBSERVER: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreatureDiesWithCounter {
            relation: CastRelation::You,
            kind: CounterKind::Impostor,
        },
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::OtherThanEventObject {
                base: &TargetRule::Object {
                    zone: TargetZone::Graveyard,
                    kinds: 1,
                    relation: 0,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                excluded: None,
            },
            min: 0,
            max: 1,
        }),
        program: &[
            Instruction::ExileEventGraveyardCard,
            Instruction::ReturnToBattlefield,
        ],
    }],
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn kill(game: &mut Game, player: Player, victim: CardId) {
    let murder = put(game, player, "Murder", Zone::Hand);
    cast(game, murder, Some(Target::Card(victim)));
    pass_pair(game);
}

fn damage(game: &mut Game, events: &[DamageEvent]) {
    game.apply_damage_batch(events);
    assert!(!game.advance_replacements(false));
    game.state_based_actions();
    game.prepare_triggers();
}

#[test]
fn simultaneous_actual_damage_can_trigger_before_its_recipient_dies() {
    let _rules = fixture_rules::activate(&[(named("Bear Cub"), &DAMAGE_OBSERVER)]);
    let player = Player::P0;
    let other = player.other();
    let mut game = scenario(player);
    let observer = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let red = put(&mut game, other, "Hill Giant", Zone::Battlefield);
    let more_red = put(&mut game, other, "Goblin Piker", Zone::Battlefield);
    let hit = |id| DamageEvent {
        source: game.damage_source(id),
        target: Target::Card(observer),
        amount: 2,
        combat: false,
        preventable: true,
    };
    let events = [hit(red), hit(more_red)];
    let mut split = game.clone();
    damage(&mut split, &events[..1]);
    assert!(split.state.stack.is_empty());
    damage(&mut split, &events[1..]);
    assert_eq!(split.object_view(observer).zone, Zone::Graveyard);
    assert!(split.state.stack.is_empty());

    let mut prevented = game.clone();
    prevented.state.objects[observer as usize].gained_keywords = Keywords::PROTECTION_RED;
    damage(&mut prevented, &events);
    assert!(prevented.on_battlefield(observer));
    assert!(prevented.state.stack.is_empty());

    damage(&mut game, &events);
    assert_eq!(game.object_view(observer).zone, Zone::Graveyard);
    assert_eq!(game.state.stack.len(), 1);
    assert_eq!(
        game.state.stack[0].damage_received.as_ref().unwrap().amount,
        4
    );
    settle(&mut game);
    assert_eq!(
        game.controlled_token_ids(player)
            .filter(|&id| game.object_view(id).card == CLUE)
            .count(),
        1,
    );
}

#[test]
fn the_death_gate_uses_toughness_and_wither_can_qualify_indestructible_creatures() {
    let _rules = fixture_rules::activate(&[
        (named("Hill Giant"), &WITHER_OBSERVER),
        (Card::GrizzlyBears, &ZERO_POWER),
    ]);
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let dealer = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let zero_power = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    kill(&mut game, player, zero_power);
    settle(&mut game);
    assert_eq!(game.object_view(zero_power).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);

    let recipient = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    game.state.objects[recipient as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
    assert!(game.keywords(dealer).contains(Keywords::WITHER));
    let bite = put(&mut game, player, "Bite Down", Zone::Hand);
    announce(
        &mut game,
        bite,
        &[Target::Card(dealer), Target::Card(recipient)],
    );
    settle(&mut game);
    assert_eq!(game.object_view(recipient).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

#[test]
fn the_graveyard_receipt_supports_zero_targets_and_rejects_reincarnations() {
    let _rules = fixture_rules::activate(&[(named("Soul-Guide Lantern"), &DEATH_OBSERVER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Soul-Guide Lantern", Zone::Battlefield);
    let marked = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let other = put(&mut game, player, "Bear Cub", Zone::Graveyard);
    game.add_counter(marked, CounterKind::Impostor, 1);
    kill(&mut game, player, marked);
    assert_eq!(game.object_view(marked).zone, Zone::Graveyard);
    let death = game.object_ref(marked);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(marked),
        },
    );

    let checkpoint = game.snapshot();
    let mut no_target = game.clone();
    let mut indexed = game.clone();
    let finish = Action::FinishTargets;
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == finish)
        .unwrap();
    indexed.apply_index(player, game.revision(), index).unwrap();
    act(&mut no_target, finish);
    assert_eq!(indexed.snapshot(), no_target.snapshot());
    settle(&mut no_target);
    assert_eq!(no_target.object_view(marked).zone, Zone::Exile);
    assert_eq!(no_target.object_view(other).zone, Zone::Graveyard);
    indexed.restore(&checkpoint).unwrap();
    assert_eq!(indexed.snapshot(), game.snapshot());

    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(other),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    let mut invalid_target = game.clone();
    invalid_target.move_object(other, Zone::Exile);
    settle(&mut invalid_target);
    assert_eq!(invalid_target.object_ref(marked), death);
    assert_eq!(invalid_target.object_view(marked).zone, Zone::Graveyard);

    game.move_object(marked, Zone::Exile);
    game.move_object(marked, Zone::Graveyard);
    assert_ne!(game.object_ref(marked), death);
    settle(&mut game);
    assert_eq!(game.object_view(marked).zone, Zone::Graveyard);
    assert_eq!(game.object_view(other).zone, Zone::Battlefield);
}

#[test]
fn a_death_receipt_excludes_and_exiles_both_meld_fronts() {
    let _rules = fixture_rules::activate(&[(named("Soul-Guide Lantern"), &DEATH_OBSERVER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let lantern = put(&mut game, player, "Soul-Guide Lantern", Zone::Battlefield);
    let front = put(
        &mut game,
        player,
        "Vanille, Cheerful l'Cie",
        Zone::Battlefield,
    );
    let companion = put(&mut game, player, "Fang, Fearless l'Cie", Zone::Battlefield);
    game.begin_step(Step::Main1);
    game.prepare_triggers();
    pass_pair(&mut game);
    choose(&mut game, |action| {
        matches!(action, Action::PayResolutionMana { .. })
    });
    let partner = game.object_ref(companion);
    act(&mut game, Action::ChooseEffectObject { object: partner });
    settle(&mut game);

    let spare = put(&mut game, player, "Bear Cub", Zone::Graveyard);
    let melded = game.observe(player).melded_permanents[0].object;
    game.add_counter(melded.card, CounterKind::Impostor, 1);
    kill(&mut game, player, melded.card);
    choose(&mut game, |action| {
        matches!(action, Action::ChooseMeldOrder { .. })
    });
    // Place Ragnarok's own death trigger underneath the graveyard observer.
    let ragnarok_trigger = game
        .state
        .ordering_triggers
        .iter()
        .find(|item| item.source.is_some_and(|source| source.object == melded))
        .unwrap()
        .id;
    act(
        &mut game,
        Action::PutTrigger {
            id: ragnarok_trigger,
        },
    );
    for id in [lantern, spare] {
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
    for id in [front, companion] {
        assert_eq!(game.object_view(id).zone, Zone::Graveyard);
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(spare),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    settle(&mut game);
    assert_eq!(game.object_view(front).zone, Zone::Exile);
    assert_eq!(game.object_view(companion).zone, Zone::Exile);
    assert_eq!(game.object_view(spare).zone, Zone::Battlefield);
}
