//! First qualified creature casts and exact, live cast-entry counter receipts.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const DOG_OR_WOLF: &[&str] = &["Dog", "Wolf"];
const WOLF: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::FirstQualifiedCreature(CreatureCastQualifier::Human),
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddCastEntryCountersForControlledSubtypes {
            kind: CounterKind::PlusOne,
            subtypes: DOG_OR_WOLF,
        }],
    }],
    ..CardRules::EMPTY
};

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn start_next_turn(game: &mut Game) {
    let previous = game.state.active;
    game.state.step = Step::End;
    game.give_priority();
    pass_pair(game);
    assert_eq!(game.state.active, previous.other());
    assert_eq!(game.state.step, Step::Upkeep);
    for player in &mut game.state.players {
        player.mana = [8; 6];
    }
}

#[test]
fn first_human_spell_counts_live_distinct_dogs_or_wolves_on_its_exact_entry() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Wargling").unwrap(), &WOLF)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut parent = scenario(player);
    let wolf = put(&mut parent, player, "Wargling", Zone::Battlefield);
    let dog = put(&mut parent, player, "Chrome Companion", Zone::Battlefield);
    let changeling = put(&mut parent, player, "Mutable Explorer", Zone::Battlefield);
    assert!(parent.has_subtype(changeling, "Dog") && parent.has_subtype(changeling, "Wolf"));
    put(&mut parent, opponent, "Wargling", Zone::Battlefield);
    let bear = put(&mut parent, player, "Grizzly Bears", Zone::Hand);
    cast(&mut parent, bear, None);
    settle(&mut parent);

    let giant = put(&mut parent, opponent, "Hill Giant", Zone::Battlefield);
    let inversion = put(&mut parent, player, "Nameless Inversion", Zone::Hand);
    cast(&mut parent, inversion, Some(Target::Card(giant)));
    settle(&mut parent);
    assert_eq!(
        parent.observe(player).turn_facts.unwrap()[player.index()].qualified_creature_casts,
        [0, 0]
    );

    let explorer = put(&mut parent, player, "Mutable Explorer", Zone::Hand);
    cast(&mut parent, explorer, None);
    assert_eq!(parent.state.stack.len(), 2);
    let spell = parent.object_ref(explorer);
    pass_pair(&mut parent);
    let observation = parent.observe(player);
    assert_eq!(
        observation.turn_facts.unwrap()[player.index()].qualified_creature_casts,
        [1, 0]
    );
    assert_eq!(observation.cast_entry_counters.len(), 1);
    let receipt = observation.cast_entry_counters[0];
    assert_eq!(receipt.spell, spell);
    let count = receipt.per_controlled_subtypes.unwrap();
    assert_eq!(count.player, player);
    assert_eq!(count.subtypes, DOG_OR_WOLF);

    let saved = parent.snapshot();
    let revision = parent.revision();
    let index = parent
        .legal_actions()
        .iter()
        .position(|action| *action == Action::Pass)
        .unwrap();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    drop(parent);
    act(&mut raw, Action::Pass);
    indexed.apply_index(player, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    settle(&mut raw);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.counter_count(explorer, CounterKind::PlusOne), 3);
    assert_eq!(raw.counter_count(changeling, CounterKind::PlusOne), 0);
    assert!(raw.observe(player).cast_entry_counters.is_empty());

    raw.restore(&saved).unwrap();
    raw.move_object(wolf, Zone::Graveyard);
    raw.move_object(dog, Zone::Graveyard);
    put(&mut raw, player, "Chrome Companion", Zone::Battlefield);
    put(&mut raw, player, "Doubling Season", Zone::Battlefield);
    settle(&mut raw);
    assert_eq!(raw.counter_count(explorer, CounterKind::PlusOne), 4);
    assert_eq!(indexed.counter_count(explorer, CounterKind::PlusOne), 3);

    raw.restore(&saved).unwrap();
    act(&mut raw, Action::Pass);
    let scatter = put(&mut raw, opponent, "Essence Scatter", Zone::Hand);
    cast(&mut raw, scatter, Some(Target::Spell(explorer)));
    settle(&mut raw);
    assert_eq!(raw.object_view(explorer).zone, Zone::Graveyard);
    assert!(raw.observe(player).cast_entry_counters.is_empty());
}

#[test]
fn countered_cast_before_observer_consumes_first_slot_until_the_next_turn() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Wargling").unwrap(), &WOLF)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let inspector = put(&mut game, player, "Novice Inspector", Zone::Hand);
    cast(&mut game, inspector, None);
    act(&mut game, Action::Pass);
    assert_eq!(game.actor(), Some(opponent));
    let scatter = put(&mut game, opponent, "Essence Scatter", Zone::Hand);
    cast(&mut game, scatter, Some(Target::Spell(inspector)));
    settle(&mut game);
    assert_eq!(game.object_view(inspector).zone, Zone::Graveyard);

    put(&mut game, player, "Wargling", Zone::Battlefield);
    put(&mut game, player, "Chrome Companion", Zone::Battlefield);
    let second = put(&mut game, player, "Cathar Commando", Zone::Hand);
    cast(&mut game, second, None);
    assert_eq!(game.state.stack.len(), 1);
    settle(&mut game);
    assert_eq!(game.counter_count(second, CounterKind::PlusOne), 0);

    start_next_turn(&mut game);
    let other = put(&mut game, opponent, "Cathar Commando", Zone::Hand);
    cast(&mut game, other, None);
    assert_eq!(game.state.stack.len(), 1);
    settle(&mut game);
    assert_eq!(game.counter_count(other, CounterKind::PlusOne), 0);
    act(&mut game, Action::Pass);
    assert_eq!(game.actor(), Some(player));
    let off_turn = put(&mut game, player, "Cathar Commando", Zone::Hand);
    cast(&mut game, off_turn, None);
    assert_eq!(game.state.stack.len(), 2);
    settle(&mut game);
    assert_eq!(game.counter_count(off_turn, CounterKind::PlusOne), 2);
    for seat in [player, opponent] {
        assert_eq!(
            game.observe(player).turn_facts.unwrap()[seat.index()].qualified_creature_casts,
            [1, 0]
        );
    }
}

const LEGENDARY_REDUCTION: SpellReduction = SpellReduction {
    source_only: false,
    filter: CastFilter::FirstQualifiedCreature(CreatureCastQualifier::Legendary),
    amount: SpellReductionAmount::Fixed(2),
    turn: CostTurn::Any,
    guard: StepGuard::Always,
};
const REDUCTION_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::CostReduction(0, LEGENDARY_REDUCTION)],
}];
const REDUCER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: REDUCTION_GROUPS,
    extras: &CardExtras {
        static_ability_groups: REDUCTION_GROUPS,
        cost_reductions: &[LEGENDARY_REDUCTION],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

#[test]
fn first_legendary_creature_reduces_projected_and_paid_generic_cost() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Wargling").unwrap(), &REDUCER)]);
    let player = Player::P0;
    let mut parent = scenario(player);
    let provider = put(&mut parent, player, "Wargling", Zone::Battlefield);
    for name in ["Grizzly Bears", "Captain America's Shield"] {
        let unqualified = put(&mut parent, player, name, Zone::Hand);
        cast(&mut parent, unqualified, None);
        settle(&mut parent);
    }
    let first = put(&mut parent, player, "Yargle, Glutton of Urborg", Zone::Hand);
    let lands = [
        put(&mut parent, player, "Swamp", Zone::Battlefield),
        put(&mut parent, player, "Forest", Zone::Battlefield),
        put(&mut parent, player, "Forest", Zone::Battlefield),
    ];
    parent.state.players[player.index()].mana = [0; 6];
    let announce_first = Action::AnnounceCast { card: first };
    let mut absent = parent.clone();
    absent.move_object(provider, Zone::Graveyard);
    reject(&mut absent, announce_first.clone());
    act(&mut parent, announce_first);
    let observation = parent.observe(player);
    let request = observation.payment_request.unwrap();
    assert_eq!(request.required, [0, 0, 1, 0, 0, 0]);
    assert_eq!(request.generic, 2);
    assert_eq!(
        observation.turn_facts.unwrap()[player.index()].qualified_creature_casts,
        [0, 0]
    );
    for land in lands {
        act(&mut parent, Action::TapMana { card: land });
    }

    let revision = parent.revision();
    let (index, action) = parent
        .legal_actions()
        .into_iter()
        .enumerate()
        .find(|(_, action)| matches!(action, Action::PayCosts { .. }))
        .unwrap();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    drop(parent);
    act(&mut raw, action);
    indexed.apply_index(player, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.state.stack.last().unwrap().mana_value(), 5);
    assert_eq!(raw.state.players[player.index()].mana, [0; 6]);
    assert_eq!(
        raw.observe(player).turn_facts.unwrap()[player.index()].qualified_creature_casts,
        [0, 1]
    );
    settle(&mut raw);

    let second = put(&mut raw, player, "Yargle, Goliath of Otaria", Zone::Hand);
    raw.state.players[player.index()].mana = [0, 1, 0, 0, 2, 0];
    let announce_second = Action::AnnounceCast { card: second };
    reject(&mut raw, announce_second.clone());
    raw.state.players[player.index()].mana = [0, 1, 0, 0, 4, 0];
    act(&mut raw, announce_second);
    assert_eq!(raw.observe(player).payment_request.unwrap().generic, 4);
    pay(&mut raw);
    settle(&mut raw);
    assert_eq!(raw.object_view(second).zone, Zone::Battlefield);
}

#[test]
fn a_prior_countered_legend_consumes_the_discount_for_that_player_and_turn() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Wargling").unwrap(), &REDUCER)]);
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let first = put(&mut game, player, "Yargle, Glutton of Urborg", Zone::Hand);
    cast(&mut game, first, None);
    act(&mut game, Action::Pass);
    let scatter = put(&mut game, opponent, "Essence Scatter", Zone::Hand);
    cast(&mut game, scatter, Some(Target::Spell(first)));
    settle(&mut game);
    assert_eq!(game.object_view(first).zone, Zone::Graveyard);

    put(&mut game, player, "Wargling", Zone::Battlefield);
    let flash_legend = put(&mut game, player, "Agent Venom", Zone::Hand);
    let black = [0, 0, 1, 0, 0, 0];
    game.state.players[player.index()].mana = black;
    let announce = Action::AnnounceCast { card: flash_legend };
    reject(&mut game, announce.clone());

    start_next_turn(&mut game);
    let opposing_legend = put(&mut game, opponent, "Agent Venom", Zone::Hand);
    cast(&mut game, opposing_legend, None);
    settle(&mut game);
    act(&mut game, Action::Pass);
    assert_eq!(game.actor(), Some(player));

    game.state.players[player.index()].mana = black;
    act(&mut game, announce);
    let request = game.observe(player).payment_request.unwrap();
    assert_eq!(request.required, black);
    assert_eq!(request.generic, 0);
    pay(&mut game);
    assert_eq!(game.state.stack.last().unwrap().mana_value(), 3);
    settle(&mut game);
    assert_eq!(game.object_view(flash_legend).zone, Zone::Battlefield);
}
