//! A copy-on-entry limit can read committed mana while preserving copiable exceptions.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::{cards::*, engine::*};
use support::*;

const REDUCE: SpellReduction = SpellReduction {
    source_only: true,
    filter: CastFilter::Any,
    amount: SpellReductionAmount::Fixed(2),
    turn: CostTurn::Any,
    guard: StepGuard::Always,
};
const COPY_ENTRY: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    keywords: Keywords::FLYING,
    extras: &CardExtras {
        x_symbols: 1,
        cost_reductions: &[REDUCE],
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
        entry_copy_options: Some(&EntryCopyOptions {
            source: EntryCopySource::BattlefieldAtMostManaSpent,
            exceptions: CopyExceptions {
                bird: true,
                ..CopyExceptions::NONE
            },
            ..EntryCopyOptions::DEFAULT
        }),
        entry_copy_keywords: Keywords::FLYING,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const COUNTERS_ON_X: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    extras: &CardExtras {
        x_symbols: 2,
        x_entry_counters: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const FREE_CREATURE: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::FreeSpellCast(CastFilter::Creature)],
    }],
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn choice(game: &Game, blueprint: CardId) -> Action {
    Action::ChooseEntryCopy {
        object: Some(game.object_ref(blueprint)),
    }
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn reduced_x_uses_only_mana_spent_and_copies_additive_characteristics() {
    let _rules = fixture_rules::activate(&[
        (named("Spyglass Siren"), &COPY_ENTRY),
        (named("Marketback Walker"), &COUNTERS_ON_X),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let bear = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let expensive = put(
        &mut game,
        player.other(),
        "Aven Interrupter",
        Zone::Battlefield,
    );
    let walker = put(
        &mut game,
        player.other(),
        "Marketback Walker",
        Zone::Battlefield,
    );
    game.add_counter(bear, CounterKind::PlusOne, 2);
    game.add_counter(walker, CounterKind::PlusOne, 2);
    let incoming = put(&mut game, player, "Spyglass Siren", Zone::Hand);
    game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 1];
    act(&mut game, Action::AnnounceCast { card: incoming });
    act(&mut game, Action::ChooseX { value: 3 });
    pay(&mut game);
    assert_eq!(game.state.stack.last().unwrap().mana_value(), 4);
    assert_eq!(
        game.state.stack.last().unwrap().cast.unwrap().payment,
        [0, 1, 0, 0, 0, 1]
    );
    pass_pair(&mut game);
    assert!(matches!(game.state.pending, Pending::EntryCopy { .. }));
    let too_large = choice(&game, expensive);
    reject(&mut game, too_large);

    let before = game.snapshot();
    let mut from_x_copy = game.clone();
    let use_walker = choice(&from_x_copy, walker);
    act(&mut from_x_copy, use_walker);
    settle(&mut from_x_copy);
    assert_eq!(
        from_x_copy.object_view(incoming).card,
        named("Marketback Walker")
    );
    assert_eq!(from_x_copy.counter_count(incoming, CounterKind::PlusOne), 0);
    assert_eq!(from_x_copy.object_mana_value(incoming), 0);

    let bear_copy = choice(&game, bear);
    let mut indexed = game.clone();
    let index = indexed
        .legal_actions()
        .iter()
        .position(|action| *action == bear_copy)
        .unwrap();
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    act(&mut game, bear_copy);
    assert_eq!(indexed.snapshot(), game.snapshot());
    let entered = game.object_view(incoming);
    assert_eq!(entered.card, Card::GrizzlyBears);
    assert!(entered.has_subtype("Bird") && entered.has_subtype("Bear"));
    assert!(!entered.has_subtype("Pirate"));
    assert!(entered.keywords.contains(Keywords::FLYING));
    assert_eq!(game.counter_count(incoming, CounterKind::PlusOne), 0);

    game.state.players[player.index()].mana = [8; 6];
    let copy = put(&mut game, player, "Visage Bandit", Zone::Hand);
    cast(&mut game, copy, None);
    pass_pair(&mut game);
    let flattened = choice(&game, incoming);
    act(&mut game, flattened);
    settle(&mut game);
    let second = game.object_view(copy);
    assert!(second.has_subtype("Bird") && second.has_subtype("Bear"));
    assert!(second.keywords.contains(Keywords::FLYING));

    indexed.restore(&before).unwrap();
    act(&mut indexed, Action::ChooseEntryCopy { object: None });
    assert_eq!(indexed.object_view(incoming).card, named("Spyglass Siren"));
}

#[test]
fn free_and_uncast_entries_can_only_choose_zero_mana_value() {
    let _rules = fixture_rules::activate(&[
        (named("Spyglass Siren"), &COPY_ENTRY),
        (named("Short Bow"), &FREE_CREATURE),
    ]);
    let player = Player::P1;
    let mut game = scenario(player);
    let expensive = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    game.create_tokens(player, Card::DrakeToken, 1, false);
    let token = game
        .state
        .objects
        .iter()
        .position(|object| object.is_token())
        .unwrap() as CardId;
    put(&mut game, player, "Short Bow", Zone::Battlefield);
    let free = put(&mut game, player, "Spyglass Siren", Zone::Hand);
    let uncast = put(&mut game, player, "Spyglass Siren", Zone::Graveyard);
    let zombify = put(&mut game, player, "Zombify", Zone::Hand);
    let mut branch = game.clone();

    act(
        &mut game,
        Action::AnnounceFree {
            card: free,
            face: CastFace::Normal,
            flash_surcharge: false,
        },
    );
    pay(&mut game);
    assert_eq!(game.state.stack.last().unwrap().x, Some(0));
    pass_pair(&mut game);
    let rejected = choice(&game, expensive);
    reject(&mut game, rejected);
    let accepted = choice(&game, token);
    act(&mut game, accepted);
    assert!(game.object_view(free).has_subtype("Bird"));

    cast(&mut branch, zombify, Some(Target::Card(uncast)));
    pass_pair(&mut branch);
    let rejected = choice(&branch, expensive);
    reject(&mut branch, rejected);
    let accepted = choice(&branch, token);
    act(&mut branch, accepted);
    assert!(branch.object_view(uncast).has_subtype("Bird"));
    assert!(!branch.object_view(uncast).is_token());
}

#[test]
fn projected_entry_uses_the_same_exact_payment_limit() {
    let _rules = fixture_rules::activate(&[(named("Spyglass Siren"), &COPY_ENTRY)]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    put(
        &mut game,
        player.other(),
        "Aven Interrupter",
        Zone::Battlefield,
    );
    let incoming = put(&mut game, player, "Spyglass Siren", Zone::Hand);
    let mut movement = game.entry_move(incoming, Zone::Battlefield, false);
    movement.cast_payment = Some([0, 1, 0, 0, 0, 1]);
    movement.x = 8;
    let goal = FundingGoal {
        actor: player,
        required: [0; 6],
        generic: 0,
        reserve: None,
        source: None,
    };
    let world = FundingState::from_game(&game, &goal);
    let mut choices = Vec::new();
    world.visit_entry_moves(vec![movement], true, None, &mut |branch| {
        choices.push(branch.object(incoming).card);
        false
    });
    assert!(choices.contains(&named("Spyglass Siren")));
    assert!(choices.contains(&Card::GrizzlyBears));
    assert!(!choices.contains(&named("Aven Interrupter")));
}
