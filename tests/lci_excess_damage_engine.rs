//! Excess damage observers see one completed event and its exact recipients.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::engine::characteristic_layers::AnimationTypeMode;
use rustmage_core::{cards::*, engine::*};
use support::*;

const TREASURE: Card = Card::from_catalogue_id(5302);
const EXCESS: TriggeredAbility = TriggeredAbility {
    condition: TriggerCondition::ExcessNoncombatDamageToCreatures {
        relation: CastRelation::Opponent,
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::CreateTokens {
        token: TREASURE,
        amount: 1,
        tapped: false,
    }],
};
const OBSERVER: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[EXCESS],
    ..CardRules::EMPTY
};
const CREATURE_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    ..OBSERVER
};
const ALL_CREATURES: BattlefieldFilter = BattlefieldFilter {
    kinds: 1,
    relation: -1,
    nonland: false,
    tapped: None,
    without: Keywords::NONE,
    subtype: None,
    max_mana: u32::MAX,
};
const SWEEP: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::DamageBattlefield {
            filters: &[ALL_CREATURES],
            players: None,
            amount: 3,
        }],
    ),
    ..CardRules::EMPTY
};
const POWER_HITS: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::PermanentsDealPowerDamage {
            filters: &[BattlefieldFilter {
                relation: 0,
                subtype: Some("Bear"),
                ..ALL_CREATURES
            }],
            exclude_source: false,
        }],
    ),
    ..CardRules::EMPTY
};
const REDIRECT: ReplacementSpec = ReplacementSpec::RedirectControlledDamageToSource;
const REDIRECTOR: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(0, REDIRECT)],
    }],
    extras: &CardExtras {
        replacement_effects: &[REDIRECT],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const WITHER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::WITHER,
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn treasure_count(game: &Game, player: Player) -> usize {
    game.state
        .objects
        .iter()
        .filter(|object| {
            object.card == TREASURE
                && object.controller == player
                && object.zone == Zone::Battlefield
        })
        .count()
}

fn pass_priority(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn shock(game: &mut Game, caster: Player, target: CardId) {
    let spell = put(game, caster, "Shock", Zone::Hand);
    cast(game, spell, Some(Target::Card(target)));
    settle(game);
}

#[test]
fn two_opposing_recipients_trigger_once_even_when_the_observer_dies() {
    let _rules = fixture_rules::activate(&[
        (named("Bear Cub"), &CREATURE_OBSERVER),
        (named("Pyroclasm"), &SWEEP),
    ]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let observer = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let source = game.object_ref(observer);
        let first = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
        let second = put(&mut game, opponent, "Healer's Hawk", Zone::Battlefield);
        let sweep = put(&mut game, player, "Pyroclasm", Zone::Hand);
        cast(&mut game, sweep, None);
        pass_priority(&mut game);

        for id in [observer, first, second] {
            assert_eq!(game.object_view(id).zone, Zone::Graveyard);
        }
        assert_eq!(game.state.stack.len(), 1);
        assert_eq!(game.state.stack[0].source.unwrap().object, source);
        assert_eq!(treasure_count(&game, player), 0);
        settle(&mut game);
        assert_eq!(treasure_count(&game, player), 1);
    }
}

#[test]
fn excess_excludes_exact_lethal_friendly_creatures_and_combat() {
    let _rules = fixture_rules::activate(&[(named("Short Bow"), &OBSERVER)]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        put(&mut game, player, "Short Bow", Zone::Battlefield);
        let exact = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
        let friendly = put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
        let marked = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
        shock(&mut game, player, exact);
        shock(&mut game, player, friendly);
        shock(&mut game, player, marked);
        assert_eq!(game.object_view(exact).zone, Zone::Graveyard);
        assert_eq!(game.object_view(friendly).zone, Zone::Graveyard);
        assert_eq!(game.object_view(marked).damage, 2);
        assert_eq!(treasure_count(&game, player), 0);
        shock(&mut game, player, marked);
        assert_eq!(treasure_count(&game, player), 1);

        let attacker = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let blocker = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
        pass_priority(&mut game);
        pass_priority(&mut game);
        act(&mut game, Action::Attack { card: attacker });
        act(&mut game, Action::FinishAttackers);
        pass_priority(&mut game);
        act(&mut game, Action::Block { blocker, attacker });
        act(&mut game, Action::FinishBlockers);
        pass_priority(&mut game);
        assert_eq!(game.object_view(blocker).zone, Zone::Graveyard);
        assert_eq!(treasure_count(&game, player), 1);
    }
}

#[test]
fn actual_hits_from_several_sources_share_one_threshold() {
    let _rules =
        fixture_rules::activate(&[(named("Short Bow"), &OBSERVER), (named("Opt"), &POWER_HITS)]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        for (victim, deathtouch, shielded, expected) in [
            ("Hill Giant", false, false, 1),
            ("Hill Giant", false, true, 0),
            ("Gigantosaurus", false, false, 0),
            ("Gigantosaurus", true, false, 1),
        ] {
            let mut game = scenario(player);
            put(&mut game, player, "Short Bow", Zone::Battlefield);
            let first = put(&mut game, player, "Bear Cub", Zone::Battlefield);
            put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
            let target = put(&mut game, opponent, victim, Zone::Battlefield);
            if deathtouch {
                game.grant_keyword_effects(&[first], game.object_ref(first), Keywords::DEATHTOUCH);
            }
            if shielded {
                game.add_counter(target, CounterKind::Shield, 1);
            }
            let spell = put(&mut game, player, "Opt", Zone::Hand);
            cast(&mut game, spell, Some(Target::Card(target)));
            settle(&mut game);
            assert_eq!(treasure_count(&game, player), expected);
        }
    }
}

#[test]
fn redirection_and_multitype_recipients_use_the_final_event() {
    let _rules = fixture_rules::activate(&[
        (named("Short Bow"), &OBSERVER),
        (named("Hill Giant"), &REDIRECTOR),
    ]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        put(&mut game, player, "Short Bow", Zone::Battlefield);
        let hawk = put(&mut game, opponent, "Healer's Hawk", Zone::Battlefield);
        let giant = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
        shock(&mut game, player, hawk);
        assert_eq!(game.object_view(hawk).damage, 0);
        assert_eq!(game.object_view(giant).damage, 2);
        assert_eq!(treasure_count(&game, player), 0);
        game.move_object(giant, Zone::Graveyard);

        for animated in [false, true] {
            let walker = put(&mut game, opponent, "Jace Reawakened", Zone::Battlefield);
            game.add_counter(walker, CounterKind::Loyalty, 1);
            if animated {
                game.change_object_form(
                    game.object_ref(walker),
                    AnimationSpec {
                        artifact: false,
                        creature_types: AnimationCreatureTypes::Unchanged,
                        base: Some((4, 4)),
                        keywords: Keywords::NONE,
                        duration: AnimationDuration::EndOfTurn,
                    },
                    AnimationTypeMode::AddCreature,
                );
            }
            shock(&mut game, player, walker);
            assert_eq!(treasure_count(&game, player), usize::from(animated));
        }
    }
}

#[test]
fn frozen_excess_survives_wither_counter_replacement_and_owned_restore() {
    let _rules = fixture_rules::activate(&[
        (named("Short Bow"), &OBSERVER),
        (named("Bear Cub"), &WITHER),
        (named("Opt"), &POWER_HITS),
    ]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        for (victim, expected) in [("Healer's Hawk", 1), ("Hill Giant", 0)] {
            let mut root = scenario(player);
            put(&mut root, player, "Short Bow", Zone::Battlefield);
            put(&mut root, player, "Bear Cub", Zone::Battlefield);
            let target = put(&mut root, opponent, victim, Zone::Battlefield);
            for _ in 0..2 {
                put(&mut root, opponent, "Doubling Season", Zone::Battlefield);
            }
            let spell = put(&mut root, player, "Opt", Zone::Hand);
            cast(&mut root, spell, Some(Target::Card(target)));
            pass_priority(&mut root);
            assert_eq!(root.actor(), Some(opponent));
            assert!(root.state.pending_triggers.is_empty());
            let actions = root.legal_actions();
            let index = actions
                .iter()
                .position(|a| matches!(a, Action::ChooseReplacement { .. }))
                .unwrap();
            let action = actions[index].clone();
            let revision = root.revision();
            let saved = root.snapshot();
            let mut raw = root.clone();
            let mut indexed = root.clone();
            drop(root);

            act(&mut raw, action.clone());
            indexed.apply_index(opponent, revision, index).unwrap();
            settle(&mut raw);
            settle(&mut indexed);
            assert_eq!(raw.snapshot(), indexed.snapshot());
            assert_eq!(raw.object_view(target).zone, Zone::Graveyard);
            assert_eq!(treasure_count(&raw, player), expected);

            indexed.restore(&saved).unwrap();
            assert_eq!(
                indexed.apply_index(opponent, revision, index),
                Err(Error::StaleRevision)
            );
            act(&mut indexed, action);
            settle(&mut indexed);
            assert_eq!(raw.snapshot(), indexed.snapshot());
        }
    }
}
