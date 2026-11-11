//! A creation replacement changes quantities of matching recipes in the same event.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const SPIRIT: Card = Card(5326);
const FOOD: Card = Card(5246);
const GNOME: Card = Card(5267);
const PROVIDER: &str = "Grizzly Bears";
const TRIPLE: ReplacementSpec = ReplacementSpec::MultiplyTokens {
    kinds: 1,
    factor: 3,
};
const TRIPLER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(0, TRIPLE)],
    }],
    extras: &CardExtras {
        replacement_effects: &[TRIPLE],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const ADD: ReplacementSpec = ReplacementSpec::AddToken(SPIRIT);
const ADDER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(0, ADD)],
    }],
    extras: &CardExtras {
        replacement_effects: &[ADD],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

const MIXED: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CreateTokenBatch(&[
            TokenGroupView {
                token: SPIRIT,
                amount: 2,
                tapped: false,
            },
            TokenGroupView {
                token: FOOD,
                amount: 2,
                tapped: true,
            },
            TokenGroupView {
                token: GNOME,
                amount: 1,
                tapped: false,
            },
        ])],
    ),
    ..CardRules::EMPTY
};
const SINGLE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CreateTokens {
            token: SPIRIT,
            amount: 1,
            tapped: false,
        }],
    ),
    ..CardRules::EMPTY
};
const ARTIFACT_TARGET: TargetPlan = TargetPlan::Single(TargetSpec::one(TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 16,
    relation: -1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
}));
const COPY: CardRules = CardRules {
    effect: Effect::Spell(
        ARTIFACT_TARGET,
        &[
            Instruction::CreateCopyToken {
                subject: Subject::Targets,
                amount: 1,
            },
            Instruction::CreateCounteredCopyToken {
                subject: Subject::Targets,
                exceptions: CopyExceptions::GOLEM,
                kind: CounterKind::PlusOne,
                amount: 2,
            },
        ],
    ),
    ..CardRules::EMPTY
};

fn definition(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn tokens(game: &Game, player: Player) -> Vec<ObjectView> {
    game.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, object)| {
            object.zone == Zone::Battlefield && object.is_token() && object.controller == player
        })
        .map(|(id, _)| game.object_view(id as CardId))
        .collect()
}

fn counts(game: &Game, player: Player) -> [usize; 3] {
    let created = tokens(game, player);
    [SPIRIT, FOOD, GNOME].map(|card| created.iter().filter(|token| token.card == card).count())
}

#[test]
fn current_controller_and_ability_availability_limit_which_mixed_groups_multiply() {
    let _rules = fixture_rules::activate(&[
        (definition(PROVIDER), &TRIPLER),
        (definition("Opt"), &MIXED),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let provider = put(&mut game, player.other(), PROVIDER, Zone::Battlefield);
        let spell = put(&mut game, player, "Opt", Zone::Hand);
        cast(&mut game, spell, None);
        let mut opposing = game.clone();
        settle(&mut opposing);
        assert_eq!(counts(&opposing, player), [2, 2, 1]);

        game.record_control(game.object_ref(provider), player, None);
        settle(&mut game);
        assert_eq!(counts(&game, player), [6, 2, 3]);
        for token in tokens(&game, player) {
            assert_eq!(token.tapped, token.card == FOOD);
        }

        let aura = put(&mut game, player, "Witness Protection", Zone::Hand);
        cast(&mut game, aura, Some(Target::Card(provider)));
        settle(&mut game);
        let spell = put(&mut game, player, "Opt", Zone::Hand);
        cast(&mut game, spell, None);
        settle(&mut game);
        assert_eq!(counts(&game, player), [8, 4, 4]);
    }
}

#[test]
fn affected_player_orders_multiplication_and_addition_once_with_owned_branches() {
    let _rules = fixture_rules::activate(&[
        (definition(PROVIDER), &TRIPLER),
        (definition("Short Bow"), &ADDER),
        (definition("Opt"), &SINGLE),
    ]);
    let player = Player::P1;
    let mut game = scenario(player);
    let tripler = put(&mut game, player, PROVIDER, Zone::Battlefield);
    let adder = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, spell, None);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.actor(), Some(player));
    assert!(matches!(game.state.pending, Pending::Replacement { .. }));

    let add = Action::ChooseReplacement {
        source: game.object_ref(adder),
        ability: 0,
    };
    let triple = Action::ChooseReplacement {
        source: game.object_ref(tripler),
        ability: 0,
    };
    let actions = game.legal_actions();
    assert_eq!(
        actions
            .iter()
            .filter(|action| matches!(action, Action::ChooseReplacement { .. }))
            .count(),
        2
    );
    assert!(actions.contains(&triple) && actions.contains(&add));

    let saved = game.snapshot();
    let revision = game.revision();
    let add_index = game
        .legal_actions()
        .iter()
        .position(|action| *action == add)
        .unwrap();
    let invalid = Action::ChooseReplacement {
        source: game.object_ref(tripler),
        ability: 1,
    };
    reject(&mut game, invalid);
    let mut raw = game.clone();
    let mut indexed = game.clone();
    drop(game);
    act(&mut raw, add);
    indexed.apply_index(player, revision, add_index).unwrap();
    settle(&mut raw);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(counts(&indexed, player), [6, 0, 0]);

    raw.restore(&saved).unwrap();
    act(&mut raw, triple);
    settle(&mut raw);
    assert_eq!(counts(&raw, player), [4, 0, 0]);
    assert_eq!(counts(&indexed, player), [6, 0, 0]);
}

#[test]
fn copies_use_the_creation_recipe_and_all_resulting_creatures_receive_counters() {
    let _rules =
        fixture_rules::activate(&[(definition(PROVIDER), &TRIPLER), (definition("Opt"), &COPY)]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, PROVIDER, Zone::Battlefield);
    let donor = put(&mut game, player, "Goldvein Pick", Zone::Battlefield);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(donor)));
    settle(&mut game);

    let created = tokens(&game, player);
    assert_eq!(created.len(), 4);
    let (creatures, other): (Vec<_>, Vec<_>) =
        created.into_iter().partition(ObjectView::is_creature);
    assert_eq!(other.len(), 1);
    assert_eq!(other[0].card, definition("Goldvein Pick"));
    assert_eq!(other[0].counters.get(CounterKind::PlusOne), 0);
    assert_eq!(creatures.len(), 3);
    for token in creatures {
        assert_eq!(token.kinds & 17, 17);
        assert!(token.has_subtype("Golem"));
        assert_eq!(token.counters.get(CounterKind::PlusOne), 2);
        assert_eq!(stats(&game, token.id), (5, 5));
    }
}

#[test]
fn every_extra_mobilize_token_is_tapped_attacking_and_sacrificed() {
    let _rules = fixture_rules::activate(&[(definition(PROVIDER), &TRIPLER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, PROVIDER, Zone::Battlefield);
    let source = put(&mut game, player, "Voice of Victory", Zone::Battlefield);
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: source });
    act(&mut game, Action::FinishAttackers);
    settle(&mut game);

    let created = tokens(&game, player);
    assert_eq!(created.len(), 6);
    for token in &created {
        assert!(token.tapped && token.has_subtype("Warrior"));
        assert!(game.state.attackers.iter().any(|a| a.card == token.id));
    }
    game.begin_step(Step::End);
    game.prepare_triggers();
    settle(&mut game);
    for token in created {
        assert_eq!(game.object_view(token.id).zone, Zone::Outside);
    }
}
