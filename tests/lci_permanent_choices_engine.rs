//! Exact non-targeted selections and opponent-owned resolution choices.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const ANY: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 0,
    nonland: false,
    other: false,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};
const COPY: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost::ZERO,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::RememberPermanents {
                    require_battlefield_source: true,
                },
                players: PlayerSet::Controller,
                filter: SacrificeFilter {
                    kinds: 1,
                    other: true,
                    ..ANY
                },
                amount: 1,
                optional_skip: Some(1),
            },
            Instruction::CopySourceFromRememberedPermanent,
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const PROTECTED_CREATURE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::PROTECTION_EVERYTHING,
    ..CardRules::EMPTY
};
const CHOOSE_CAVE: Instruction = Instruction::ChooseObjects {
    kind: EffectChoiceKind::RememberPermanents {
        require_battlefield_source: false,
    },
    players: PlayerSet::Controller,
    filter: SacrificeFilter {
        kinds: 2,
        subtype: Some("Cave"),
        ..ANY
    },
    amount: 1,
    optional_skip: None,
};
const COUNTERS: Instruction = Instruction::PutRememberedPermanentCounters {
    kind: CounterKind::PlusOne,
    amount: 3,
};
const ANIMATE: Instruction = Instruction::AnimateRememberedPermanents(AnimationSpec {
    artifact: false,
    creature_types: AnimationCreatureTypes::Additional(&["Elemental"]),
    base: Some((0, 0)),
    keywords: Keywords::HASTE,
    duration: AnimationDuration::Permanent,
});
const TWO_MODES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            CHOOSE_CAVE,
            COUNTERS,
            ANIMATE,
            CHOOSE_CAVE,
            COUNTERS,
            ANIMATE,
        ],
    ),
    ..CardRules::EMPTY
};
const DOUBLE: ReplacementSpec = ReplacementSpec::Counters {
    filter: ReplacementFilter::Creature,
    kind: Some(CounterKind::PlusOne),
    factor: 2,
    add: 0,
    effects_only: false,
};
const CREATURE_DOUBLER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(0, DOUBLE)],
    }],
    extras: &CardExtras {
        replacement_effects: &[DOUBLE],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const PAYMENT: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::SacrificeOrDiscard { nonland: false },
                players: PlayerSet::Opponent,
                filter: ANY,
                amount: 1,
                optional_skip: Some(1),
            },
            Instruction::Skip(1),
            Instruction::DamagePlayers(PlayerSet::Opponent, 3),
        ],
    ),
    ..CardRules::EMPTY
};
const SEARCH: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 2,
            relation: 1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[
            Instruction::DestroyLandThenControllerMaySearch,
            Instruction::GainLife(PlayerSet::Controller, 1),
        ],
    ),
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn wait_for_choice(game: &mut Game) {
    for _ in 0..8 {
        if game.state.pending != Pending::Priority {
            return;
        }
        act(game, Action::Pass);
    }
    panic!("expected a resolving choice");
}

fn choose_permanent(game: &Game, id: CardId) -> Action {
    Action::ChooseEffectObject {
        object: game.object_ref(id),
    }
}

#[test]
fn non_targeted_copy_can_choose_a_protected_creature_and_keeps_the_exact_sources_counters() {
    let _rules = fixture_rules::activate(&[
        (Card::HillGiant, &COPY),
        (named("Grizzly Bears"), &PROTECTED_CREATURE),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let foreign = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    game.add_counter(source, CounterKind::PlusOne, 1);
    let original = game.object_ref(source);
    act(
        &mut game,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    let mut departed = game.clone();
    departed.move_object(source, Zone::Graveyard);
    settle(&mut departed);
    assert_eq!(departed.object_view(source).zone, Zone::Graveyard);
    wait_for_choice(&mut game);
    let invalid = choose_permanent(&game, foreign);
    reject(&mut game, invalid);
    let self_choice = choose_permanent(&game, source);
    reject(&mut game, self_choice);
    let choice = choose_permanent(&game, creature);
    act(&mut game, choice);
    settle(&mut game);
    assert_eq!(game.object_ref(source), original);
    assert_eq!(game.object_view(source).card, named("Grizzly Bears"));
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 1);
    assert_eq!(stats(&game, source), (3, 3));
}

#[test]
fn repeatable_cave_choice_places_counters_before_adding_creature_type() {
    let _rules = fixture_rules::activate(&[
        (named("Opt"), &TWO_MODES),
        (named("Doubling Season"), &CREATURE_DOUBLER),
    ]);
    let player = Player::P1;
    let mut game = scenario(player);
    let cave = spawn(
        &mut game,
        player,
        named("Hidden Courtyard"),
        Zone::Battlefield,
    );
    let foreign = spawn(
        &mut game,
        player.other(),
        named("Hidden Courtyard"),
        Zone::Battlefield,
    );
    put(&mut game, player, "Doubling Season", Zone::Battlefield);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, spell, None);
    wait_for_choice(&mut game);
    let invalid = choose_permanent(&game, foreign);
    reject(&mut game, invalid);
    let choice = choose_permanent(&game, cave);
    act(&mut game, choice.clone());
    assert_eq!(game.counter_count(cave, CounterKind::PlusOne), 3);
    act(&mut game, choice);
    settle(&mut game);
    let result = game.object_view(cave);
    assert_eq!(result.kinds, 1 | 2);
    assert!(result.has_subtype("Cave") && result.has_subtype("Elemental"));
    assert!(result.keywords.contains(Keywords::HASTE));
    assert_eq!(result.counters.get(CounterKind::PlusOne), 9);
    assert_eq!((result.power, result.toughness), (9, 9));
}

#[test]
fn opponent_can_discard_or_sacrifice_privately_and_damage_follows_only_declining() {
    let _rules = fixture_rules::activate(&[(named("Opt"), &PAYMENT)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    put(&mut game, player, "Rest in Peace", Zone::Battlefield);
    let permanent = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    let hand = put(&mut game, opponent, "Forest", Zone::Hand);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, spell, None);
    wait_for_choice(&mut game);
    assert_eq!(game.actor(), Some(opponent));
    assert!(game.observe(player).effect_choice.is_none());
    let checkpoint = game.snapshot();
    let object = game.object_ref(permanent);
    for action in [
        Action::Discard { card: hand },
        Action::Sacrifice { object },
        Action::DeclineEffectPayment,
    ] {
        game.restore(&checkpoint).unwrap();
        let declined = action == Action::DeclineEffectPayment;
        act(&mut game, action);
        settle(&mut game);
        assert_eq!(
            game.state.players[opponent.index()].life,
            STARTING_LIFE - 3 * i128::from(declined)
        );
    }
}

#[test]
fn only_the_destroyed_lands_controller_gets_the_optional_search() {
    let _rules = fixture_rules::activate(&[(named("Opt"), &SEARCH)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let target = put(&mut game, opponent, "Swiftwater Cliffs", Zone::Battlefield);
    let basic = put(&mut game, opponent, "Island", Zone::Library);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(target)));
    wait_for_choice(&mut game);
    assert_eq!(game.state.pending, Pending::OptionalSearch(opponent));
    assert_eq!(game.object_view(target).zone, Zone::Graveyard);
    let checkpoint = game.snapshot();
    let deck = game.state.players[opponent.index()].library.clone();
    act(&mut game, Action::ChooseOptional { accept: false });
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].library, deck);
    assert_eq!(game.state.players[player.index()].life, STARTING_LIFE + 1);

    game.restore(&checkpoint).unwrap();
    act(&mut game, Action::ChooseOptional { accept: true });
    act(&mut game, Action::ChooseSearch { card: basic });
    if matches!(game.state.pending, Pending::EffectSearch { .. }) {
        act(&mut game, Action::FinishSearch);
    }
    settle(&mut game);
    assert_eq!(game.object_view(basic).zone, Zone::Battlefield);
    assert_eq!(game.object_view(basic).controller, opponent);
    assert_eq!(game.state.players[player.index()].life, STARTING_LIFE + 1);
}
