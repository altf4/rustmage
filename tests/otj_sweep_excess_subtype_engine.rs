//! Exact sweep recipients, tapped excess tokens, and foreign-owned creature subtypes.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const SWEEP: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::RemovePlayerCreatureAbilities {
                players: PlayerSet::Each,
                duration: AnimationDuration::EndOfTurn,
            },
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::RememberPermanents {
                    require_battlefield_source: false,
                },
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
                amount: 1,
                optional_skip: None,
            },
            Instruction::GrantRememberedCreatures(Keywords::INDESTRUCTIBLE),
            Instruction::MoveBattlefield {
                filters: &[BattlefieldFilter {
                    kinds: 1,
                    relation: -1,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: None,
                    max_mana: u32::MAX,
                }],
                exile: false,
            },
        ],
    ),
    ..CardRules::EMPTY
};
const CONTROLLED: CreatureFilter = CreatureFilter {
    relation: 0,
    other: false,
    subtype: None,
    keyword: Keywords::NONE,
    attacking: false,
    tokens: None,
    artifact: false,
};
const GRANTS: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::GrantCreatureKeywordAbilities {
                filter: CONTROLLED,
                keywords: Keywords::TRAMPLE.union(Keywords::MAX_ONE_BLOCKER),
            },
            Instruction::GrantCreatures {
                filter: CreatureFilter {
                    relation: 1,
                    ..CONTROLLED
                },
                keywords: Keywords::MAX_ONE_BLOCKER,
            },
        ],
    ),
    ..CardRules::EMPTY
};

const TREASURE: Card = Card::from_catalogue_id(5302);
const EXCESS: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[
            Instruction::DamageForExcess(AmountValue::ChosenX),
            Instruction::CreateTappedExcessTokens(TREASURE),
        ],
    ),
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const FOREIGN_GRANT: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Type,
        fragments: &[StaticAbilityFragment::ControlledForeignOwnedCreatureSubtype("Mercenary")],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Upkeep,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: true,
        })),
        program: &[
            Instruction::ExileTopLibraryRemember {
                players: PlayerSet::Targeted,
                amount: AmountValue::Stat(StatValue::Count(CountFilter {
                    zone: CountZone::Battlefield,
                    relation: 0,
                    kind: 0,
                    exclude: 0,
                    subtype: Some("Mercenary"),
                    basic: false,
                    other: false,
                    cards_only: false,
                })),
            },
            Instruction::GrantExiledResultFlexibleCast,
        ],
    }],
    ..CardRules::EMPTY
};

#[test]
fn ability_removal_precedes_a_non_targeted_grant_and_creature_destruction() {
    let _rules = fixture_rules::activate(&[(Card::Divination, &SWEEP)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let chosen = put(
        &mut game,
        player,
        "Sphinx of the Final Word",
        Zone::Battlefield,
    );
    let rival = put(
        &mut game,
        player.other(),
        "Kotis, the Fangkeeper",
        Zone::Battlefield,
    );
    let other = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let spell = put(&mut game, player, "Divination", Zone::Hand);
    assert!(game.keywords(chosen).contains(Keywords::HEXPROOF));
    assert!(game.keywords(rival).contains(Keywords::INDESTRUCTIBLE));
    cast(&mut game, spell, None);
    assert!(game.state.stack.last().unwrap().targets.is_empty());
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert!(!game.keywords(chosen).contains(Keywords::HEXPROOF));
    assert!(!game.keywords(rival).contains(Keywords::INDESTRUCTIBLE));
    let illegal = Action::ChooseEffectObject {
        object: game.object_ref(rival),
    };
    reject(&mut game, illegal);
    let selection = Action::ChooseEffectObject {
        object: game.object_ref(chosen),
    };
    act(&mut game, selection);
    settle(&mut game);
    assert_eq!(game.object_view(chosen).zone, Zone::Battlefield);
    assert!(game.keywords(chosen).contains(Keywords::INDESTRUCTIBLE));
    assert_eq!(game.object_view(rival).zone, Zone::Graveyard);
    assert_eq!(game.object_view(other).zone, Zone::Graveyard);
}

#[test]
fn removing_abilities_ends_quoted_but_not_direct_blocking_limits() {
    let _rules = fixture_rules::activate(&[(Card::Shock, &GRANTS), (Card::Divination, &SWEEP)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let recipient = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let opposing = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let grants = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, grants, None);
    settle(&mut game);
    assert!(game.keywords(recipient).contains(Keywords::TRAMPLE));
    assert!(game.keywords(recipient).contains(Keywords::MAX_ONE_BLOCKER));
    assert!(game.keywords(opposing).contains(Keywords::MAX_ONE_BLOCKER));

    let sweep = put(&mut game, player, "Divination", Zone::Hand);
    cast(&mut game, sweep, None);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert!(!game.keywords(recipient).contains(Keywords::TRAMPLE));
    assert!(!game.keywords(recipient).contains(Keywords::MAX_ONE_BLOCKER));
    assert!(game.keywords(opposing).contains(Keywords::MAX_ONE_BLOCKER));
}

#[test]
fn paid_x_creates_tapped_treasure_only_for_damage_beyond_remaining_lethal() {
    let definition = Card::from_name("Exsanguinate").unwrap();
    let _rules = fixture_rules::activate(&[(definition, &EXCESS)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let victim = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    game.state.objects[victim as usize].damage = 1;
    let spell = put(&mut game, player, "Exsanguinate", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    for _ in 0..8 {
        if game.state.announcing.is_none() {
            break;
        }
        choose(&mut game, |action| {
            matches!(
                action,
                Action::ChooseX { value: 4 } | Action::FinishTargets | Action::PayCosts { .. }
            ) || *action
                == Action::ChooseTarget {
                    target: Target::Card(victim),
                }
        });
    }
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
    let treasure: Vec<_> = game
        .observe(player)
        .objects
        .into_iter()
        .filter(|object| {
            object.card == TREASURE
                && object.controller == player
                && object.zone == Zone::Battlefield
        })
        .collect();
    assert_eq!(treasure.len(), 3);
    assert!(treasure.iter().all(|object| object.tapped));
}

#[test]
fn foreign_owned_creatures_keep_their_other_types_and_feed_spell_only_exile() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &FOREIGN_GRANT)]);
    let player = Player::P1;
    let other = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let own = put(&mut game, player, "Diregraf Ghoul", Zone::Battlefield);
    let first = put(&mut game, other, "Diregraf Ghoul", Zone::Battlefield);
    let second = put(&mut game, other, "Hill Giant", Zone::Battlefield);
    for creature in [first, second] {
        game.record_control(game.object_ref(creature), player, None);
        assert!(game.has_subtype(creature, "Mercenary"));
    }
    assert!(game.has_subtype(first, "Zombie"));
    assert!(!game.has_subtype(own, "Mercenary"));
    let land = put(&mut game, other, "Forest", Zone::Library);
    let spell = put(&mut game, other, "Shock", Zone::Library);
    game.begin_step(Step::Upkeep);
    game.give_priority();
    game.prepare_triggers();
    for _ in 0..12 {
        if game.object_view(land).zone == Zone::Exile {
            break;
        }
        choose(&mut game, |action| {
            matches!(
                action,
                Action::Pass | Action::FinishTargets | Action::PutTrigger { .. }
            ) || *action
                == Action::ChooseTarget {
                    target: Target::Player(other),
                }
        });
    }
    settle(&mut game);
    assert_eq!(game.object_view(land).zone, Zone::Exile);
    assert_eq!(game.object_view(spell).zone, Zone::Exile);
    game.move_object(source, Zone::Graveyard);
    assert!(!game.has_subtype(first, "Mercenary"));

    game.begin_step(Step::Main1);
    game.give_priority();
    game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 0];
    reject(&mut game, Action::PlayLand { card: land });
    act(&mut game, Action::AnnounceExile { card: spell });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(other),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    pay(&mut game);
    assert_eq!(
        game.state.stack.last().unwrap().cast.unwrap().payment,
        [0, 0, 0, 0, 1, 0]
    );
}
