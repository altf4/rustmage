//! A same-origin Celebration effect across types, abilities and base stats.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const SOURCE: &str = "Linden, the Steadfast Queen";
const RED: Mana = [0, 0, 0, 1, 0, 0];
const BASE: BaseStatSetter = BaseStatSetter {
    power: Some(4),
    toughness: Some(4),
    own_turn: false,
    condition: Some(StatValue::AtLeast {
        value: &StatValue::NonlandEntriesThisTurn,
        minimum: 2,
    }),
};
const GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::HASTE)],
    },
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Type,
        fragments: &[
            StaticAbilityFragment::SourceCelebrationCreatureForm {
                subtypes: &["Dragon"],
                keywords: Keywords::FLYING,
                activation: 0,
            },
            StaticAbilityFragment::BaseStatSetter(0, BASE),
        ],
    },
];
const FORM: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::HASTE,
    static_groups: GROUPS,
    extras: &CardExtras {
        base_stat_setter: Some(BASE),
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            colored: RED,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GrowCreatures {
            filter: CreatureFilter {
                relation: 0,
                other: false,
                subtype: Some("Dragon"),
                keyword: Keywords::NONE,
                attacking: false,
                tokens: None,
                artifact: false,
            },
            power: 1,
            toughness: 0,
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const BLANK: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::RemoveAbilities {
            subject: Subject::Targets,
            duration: AnimationDuration::EndOfTurn,
        }],
    ),
    ..CardRules::EMPTY
};
const ARTIFACTIZE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::AddArtifactToTargets],
    ),
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn enter(game: &mut Game, player: Player, name: &str) -> CardId {
    let card = put(game, player, name, Zone::Hand);
    cast(game, card, None);
    settle(game);
    card
}

fn finish_turn(game: &mut Game) {
    game.state.step = Step::End;
    game.give_priority();
    pass_pair(game);
}

fn grant(game: &Game, card: CardId) -> Action {
    Action::AnnounceSourceGrant {
        source: game.object_ref(card),
        ability: 0,
    }
}

fn assert_form(game: &Game, card: CardId, dragon: bool, power: i128, toughness: i128) {
    let view = game.object_view(card);
    assert_eq!((view.power, view.toughness), (power, toughness));
    assert!(view.is_legendary());
    assert_eq!(view.has_subtype("Dragon"), dragon);
    assert_eq!(view.has_subtype("Human"), !dragon);
    assert_eq!(view.has_subtype("Noble"), !dragon);
}

#[test]
fn second_entry_grants_paid_activation_and_earlier_layers_survive_ability_loss() {
    let _rules = fixture_rules::activate(&[
        (named(SOURCE), &FORM),
        (named("Opt"), &BLANK),
        (named("Giant Growth"), &ARTIFACTIZE),
    ]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let dragon = put(&mut game, player, "Shivan Dragon", Zone::Battlefield);
        let enemy_dragon = put(&mut game, opponent, "Shivan Dragon", Zone::Battlefield);
        let source = enter(&mut game, player, SOURCE);
        game.add_counter(source, CounterKind::PlusOne, 1);
        assert_form(&game, source, false, 4, 4);
        let pump = grant(&game, source);
        reject(&mut game, pump.clone());
        let land = put(&mut game, player, "Mountain", Zone::Hand);
        act(&mut game, Action::PlayLand { card: land });
        assert_eq!(
            game.observe(player).players[player.index()].nonland_permanents_entered_this_turn,
            1
        );

        let artifactize = put(&mut game, player, "Giant Growth", Zone::Hand);
        cast(&mut game, artifactize, Some(Target::Card(source)));
        settle(&mut game);
        let second = enter(&mut game, player, "Short Bow");
        assert_eq!(
            game.observe(player).players[player.index()].nonland_permanents_entered_this_turn,
            2
        );
        game.move_object(second, Zone::Graveyard);
        assert_form(&game, source, true, 5, 5);
        let view = game.object_view(source);
        assert_eq!(view.kinds, 17);
        assert!(
            view.keywords
                .contains(Keywords::HASTE.union(Keywords::FLYING))
        );
        reject(
            &mut game,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );

        act(&mut game, pump.clone());
        let red_before = game.state.players[player.index()].mana[3];
        let pay_red = Action::PayCosts { payment: RED };
        reject(&mut game, Action::PayCosts { payment: [0; 6] });
        let revision = game.revision();
        let index = game
            .legal_actions()
            .iter()
            .position(|a| *a == pay_red)
            .unwrap();
        let checkpoint = game.snapshot();
        let mut indexed = game.clone();
        act(&mut game, pay_red);
        indexed.apply_index(player, revision, index).unwrap();
        assert_eq!(game.snapshot(), indexed.snapshot());
        assert_eq!(game.state.players[player.index()].mana[3], red_before - 1);
        indexed.restore(&checkpoint).unwrap();
        assert_eq!(indexed.state.players[player.index()].mana[3], red_before);

        act(&mut game, Action::Pass);
        let blank = put(&mut game, opponent, "Opt", Zone::Hand);
        cast(&mut game, blank, Some(Target::Card(source)));
        pass_pair(&mut game);
        assert_form(&game, source, true, 5, 5);
        let view = game.object_view(source);
        assert!(
            !view
                .keywords
                .intersects(Keywords::HASTE.union(Keywords::FLYING))
        );
        reject(&mut game, pump.clone());

        settle(&mut game);
        assert_form(&game, source, true, 6, 5);
        assert_eq!(game.object_view(dragon).power, 6);
        assert_eq!(game.object_view(enemy_dragon).power, 5);
        let later_dragon = enter(&mut game, player, "Shivan Dragon");
        assert_eq!(game.object_view(later_dragon).power, 5);
        finish_turn(&mut game);
        assert_form(&game, source, false, 4, 4);
        let view = game.object_view(source);
        assert_eq!(view.kinds, 1);
        assert!(view.keywords.contains(Keywords::HASTE));
        assert!(!view.keywords.contains(Keywords::FLYING));
        assert_eq!(game.object_view(dragon).power, 5);
    }
}

#[test]
fn the_later_cursed_role_wins_the_base_layer_when_it_is_the_second_entry() {
    let _rules = fixture_rules::activate(&[(named(SOURCE), &FORM)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let source = put(&mut game, player, SOURCE, Zone::Battlefield);
        let reference = game.object_ref(source);
        game.add_counter(source, CounterKind::PlusOne, 1);
        let hexmage = put(&mut game, player, "Spiteful Hexmage", Zone::Hand);
        cast(&mut game, hexmage, None);
        pass_pair(&mut game);
        assert_form(&game, source, false, 4, 4);
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(source),
            },
        );
        settle(&mut game);
        let observation = game.observe(player);
        assert_eq!(
            observation.players[player.index()].nonland_permanents_entered_this_turn,
            2
        );
        assert_form(&game, source, true, 2, 2);
        assert!(game.object_view(source).keywords.contains(Keywords::FLYING));
        let cursed = observation
            .role_timestamps
            .iter()
            .find(|row| row.recipient == reference)
            .unwrap()
            .aura;
        let remove = put(&mut game, player, "Disenchant", Zone::Hand);
        cast(&mut game, remove, Some(Target::Card(cursed.card)));
        settle(&mut game);
        assert_form(&game, source, true, 5, 5);
        assert!(game.legal_actions().contains(&grant(&game, source)));
    }
}

#[test]
fn celebration_recomputes_for_the_live_controller_and_resets_for_both_players() {
    let _rules = fixture_rules::activate(&[(named(SOURCE), &FORM)]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let source = enter(&mut game, player, SOURCE);
        let reference = game.object_ref(source);
        enter(&mut game, player, "Short Bow");
        assert_form(&game, source, true, 4, 4);

        game.change_controller(reference, opponent);
        assert_form(&game, source, false, 3, 3);
        act(&mut game, Action::Pass);
        let pump = grant(&game, source);
        reject(&mut game, pump.clone());
        enter(&mut game, opponent, "Resolute Reinforcements");
        assert_form(&game, source, true, 4, 4);
        assert_eq!(
            game.observe(opponent).players[opponent.index()].nonland_permanents_entered_this_turn,
            2
        );
        act(&mut game, Action::Pass);
        assert!(game.legal_actions().contains(&pump));
        finish_turn(&mut game);
        assert_eq!(game.actor(), Some(opponent));
        assert_form(&game, source, false, 3, 3);
        assert!(
            game.observe(opponent)
                .players
                .iter()
                .all(|p| p.nonland_permanents_entered_this_turn == 0)
        );
        game.state.players[opponent.index()].mana = RED;
        assert!(!game.legal_actions().contains(&pump));
    }
}
