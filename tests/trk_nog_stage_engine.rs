//! Permanent exact-source evolution, granted tap triggers, and live land permissions.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const SOURCE_CARD: &str = "Mine Raider"; // Printed Human Rogue supplies the first stage.
const SCOUT: &[&str] = &["Ferengi", "Scout"];
const OFFICER: &[&str] = &["Ferengi", "Officer"];
const SOURCE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                generic: 1,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::EvolveSource {
                required: "Rogue",
                types: SCOUT,
                stats: Some(Stats {
                    power: 3,
                    toughness: 4,
                }),
                keywords: Keywords::NONE,
                ability: Some(0),
            }],
            sorcery: false,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                generic: 3,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::IfSubjectSubtype {
                    subject: Subject::Source,
                    subtype: "Scout",
                    skip: 3,
                },
                Instruction::EvolveSource {
                    required: "Scout",
                    types: OFFICER,
                    stats: None,
                    keywords: Keywords::NONE,
                    ability: None,
                },
                Instruction::GrantPermanentSelfAbility(
                    SelfGrantedAbilityKind::PlayLandFromGraveyard,
                ),
                Instruction::GrantPermanentSelfAbility(
                    SelfGrantedAbilityKind::AdditionalLandPlays(1),
                ),
            ],
            sorcery: false,
            once_per_turn: false,
        },
    ],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Granted(&TriggerCondition::SelfBecomesTapped),
        targets: TargetPlan::NONE,
        program: &[Instruction::Surveil(2)],
    }],
    ..CardRules::EMPTY
};
const TAKE_CONTROL: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::GainControl {
            subject: Subject::Targets,
            controller: PlayerSet::Controller,
        }],
    ),
    ..CardRules::EMPTY
};
const REPLACE_TYPES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::SetCreatureTypes {
            subject: Subject::Targets,
            types: &["Officer"],
        }],
    ),
    ..CardRules::EMPTY
};
const RESTORE_SCOUT: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::SetCreatureTypes {
            subject: Subject::Targets,
            types: &["Scout"],
        }],
    ),
    ..CardRules::EMPTY
};
const REMOVE_ABILITIES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::RemoveAbilities {
            subject: Subject::Targets,
            duration: AnimationDuration::EndOfTurn,
        }],
    ),
    ..CardRules::EMPTY
};

fn rules() -> fixture_rules::Guard {
    fixture_rules::activate(&[
        (Card::from_name(SOURCE_CARD).unwrap(), &SOURCE),
        (Card::from_name("Unsummon").unwrap(), &TAKE_CONTROL),
        (Card::from_name("Giant Growth").unwrap(), &REPLACE_TYPES),
        (Card::from_name("Opt").unwrap(), &RESTORE_SCOUT),
        (Card::from_name("Shock").unwrap(), &REMOVE_ABILITIES),
    ])
}
fn activation(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}
fn activate(game: &mut Game, card: CardId, ability: u16) {
    act(game, activation(card, ability));
    pay(game);
    settle(game);
}
fn library_choice(game: &mut Game, player: Player) {
    for _ in 0..16 {
        game.prepare_triggers();
        if game.observe(player).library_choice.is_some() {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("granted tap trigger did not reach surveil");
}

#[test]
fn stages_check_subtypes_on_resolution_and_granted_tap_surveil_is_private() {
    let _rules = rules();
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let source = put(&mut game, player, SOURCE_CARD, Zone::Battlefield);
        let source_ref = game.object_ref(source);
        assert!(game.has_subtype(source, "Rogue"));
        // The final stage is legal to activate in the first form but does nothing.
        assert!(game.legal_actions().contains(&activation(source, 1)));
        activate(&mut game, source, 1);
        assert!(game.has_subtype(source, "Rogue"));
        assert!(game.observe(player).self_granted_abilities.is_none());

        activate(&mut game, source, 0);
        assert_eq!(game.object_ref(source), source_ref);
        assert!(game.has_subtype(source, "Scout"));
        assert!(game.has_subtype(source, "Ferengi"));
        assert!(!game.has_subtype(source, "Rogue"));
        assert_eq!(
            (game.stats(source).power, game.stats(source).toughness),
            (3, 4)
        );
        let first = game.top_library_card(player).unwrap();
        game.tap_objects_as(player, &[source]);
        library_choice(&mut game, player);
        let choice = game.observe(player).library_choice.unwrap();
        assert!(choice.surveil);
        assert_eq!(choice.remaining.len(), 2);
        assert!(game.observe(player.other()).library_choice.is_none());
        act(
            &mut game,
            Action::ArrangeLibrary {
                card: first,
                destination: LibraryDestination::Graveyard,
            },
        );
        choose(&mut game, |action| {
            matches!(
                action,
                Action::ArrangeLibrary {
                    destination: LibraryDestination::Top,
                    ..
                }
            )
        });
        settle(&mut game);
        assert_eq!(game.object_view(first).zone, Zone::Graveyard);
    }
}

#[test]
fn officer_grants_cumulative_land_play_and_graveyard_origin_with_owned_restore() {
    let _rules = rules();
    let player = Player::P0;
    let mut parent = scenario(player);
    let source = put(&mut parent, player, SOURCE_CARD, Zone::Battlefield);
    activate(&mut parent, source, 0);
    let action = activation(source, 1);
    let index = parent
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let checkpoint = parent.snapshot();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    act(&mut raw, action);
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    pay(&mut raw);
    pay(&mut indexed);
    settle(&mut raw);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    drop(parent);
    indexed.restore(&checkpoint).unwrap();
    assert_eq!(indexed.snapshot(), checkpoint);
    let mut replay = indexed.clone();
    activate(&mut replay, source, 1);
    activate(&mut indexed, source, 1);
    assert_eq!(indexed.snapshot(), replay.snapshot());

    assert!(raw.has_subtype(source, "Officer"));
    assert!(raw.has_subtype(source, "Ferengi"));
    assert!(!raw.has_subtype(source, "Scout"));
    assert_eq!(
        (raw.stats(source).power, raw.stats(source).toughness),
        (3, 4)
    );
    let grave_land = put(&mut raw, player, "Forest", Zone::Graveyard);
    let hand_land = put(&mut raw, player, "Island", Zone::Hand);
    let third_land = put(&mut raw, player, "Plains", Zone::Hand);
    assert!(
        raw.legal_actions()
            .contains(&Action::PlayLand { card: grave_land })
    );
    act(&mut raw, Action::PlayLand { card: grave_land });
    assert!(
        raw.legal_actions()
            .contains(&Action::PlayLand { card: hand_land })
    );
    act(&mut raw, Action::PlayLand { card: hand_land });
    reject(&mut raw, Action::PlayLand { card: third_land });
    let restore = put(&mut raw, player, "Opt", Zone::Hand);
    cast(&mut raw, restore, Some(Target::Card(source)));
    settle(&mut raw);
    assert!(raw.has_subtype(source, "Scout"));
    activate(&mut raw, source, 1);
    assert!(raw.has_subtype(source, "Officer"));
    // Each resolution granted its own static ability, so the third play is now legal.
    act(&mut raw, Action::PlayLand { card: third_land });
}

#[test]
fn later_ability_removal_suppresses_both_acquired_land_permissions_and_tap_trigger() {
    let _rules = rules();
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, SOURCE_CARD, Zone::Battlefield);
    activate(&mut game, source, 0);
    activate(&mut game, source, 1);
    let first = put(&mut game, player, "Forest", Zone::Hand);
    act(&mut game, Action::PlayLand { card: first });
    let grave_land = put(&mut game, player, "Island", Zone::Graveyard);
    assert!(
        game.legal_actions()
            .contains(&Action::PlayLand { card: grave_land })
    );
    let blank = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, blank, Some(Target::Card(source)));
    settle(&mut game);
    reject(&mut game, Action::PlayLand { card: grave_land });
    assert!(!game.can_play_static_land(player, grave_land));
    game.tap_objects_as(player, &[source]);
    game.prepare_triggers();
    assert!(game.state.pending_triggers.is_empty());
    assert!(game.state.stack.is_empty());
}

#[test]
fn source_control_and_departure_move_permissions_and_expire_exact_stage() {
    let _rules = rules();
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, SOURCE_CARD, Zone::Battlefield);
    activate(&mut game, source, 0);
    activate(&mut game, source, 1);
    let grave_land = put(&mut game, player, "Forest", Zone::Graveyard);
    assert!(
        game.legal_actions()
            .contains(&Action::PlayLand { card: grave_land })
    );
    act(&mut game, Action::Pass);
    let steal = put(&mut game, player.other(), "Unsummon", Zone::Hand);
    cast(&mut game, steal, Some(Target::Card(source)));
    settle(&mut game);
    assert_eq!(game.object_view(source).controller, player.other());
    reject(&mut game, Action::PlayLand { card: grave_land });
    assert!(!game.can_play_static_land(player, grave_land));
    let other_land = put(&mut game, player.other(), "Island", Zone::Graveyard);
    assert!(game.can_play_static_land(player.other(), other_land));

    let old = game.object_ref(source);
    game.move_object(source, Zone::Graveyard);
    assert!(!game.can_play_static_land(player.other(), other_land));
    game.move_object(source, Zone::Battlefield);
    assert_ne!(game.object_ref(source), old);
    assert!(game.has_subtype(source, "Rogue"));
    assert!(!game.has_subtype(source, "Officer"));
    assert_eq!(
        (game.stats(source).power, game.stats(source).toughness),
        (2, 2)
    );
    assert!(game.observe(player).self_granted_abilities.is_none());
}

#[test]
fn failed_resolution_after_type_change_or_reentry_never_grants_a_stage() {
    let _rules = rules();
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, SOURCE_CARD, Zone::Battlefield);
    act(&mut game, activation(source, 0));
    pay(&mut game);
    let replace = put(&mut game, player, "Giant Growth", Zone::Hand);
    cast(&mut game, replace, Some(Target::Card(source)));
    settle(&mut game);
    assert!(game.has_subtype(source, "Officer"));
    assert!(!game.has_subtype(source, "Scout"));
    assert!(game.observe(player).temporary_abilities.is_empty());

    let mut departed = scenario(player);
    let source = put(&mut departed, player, SOURCE_CARD, Zone::Battlefield);
    let original = departed.object_ref(source);
    act(&mut departed, activation(source, 0));
    pay(&mut departed);
    departed.move_object(source, Zone::Hand);
    departed.move_object(source, Zone::Battlefield);
    settle(&mut departed);
    assert_ne!(departed.object_ref(source), original);
    assert!(departed.has_subtype(source, "Rogue"));
    assert!(!departed.has_subtype(source, "Scout"));
    assert!(departed.observe(player).temporary_abilities.is_empty());
}
