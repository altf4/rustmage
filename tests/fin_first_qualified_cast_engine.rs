//! First qualified creature casts and exact, live cast-entry counter receipts.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
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
    let stack_id = raw.state.stack[0].id;
    act(&mut raw, Action::Pass);
    let scatter = put(&mut raw, opponent, "Essence Scatter", Zone::Hand);
    cast(&mut raw, scatter, Some(Target::Stack(stack_id)));
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
    let first_cast = game.state.stack[0].id;
    act(&mut game, Action::Pass);
    assert_eq!(game.actor(), Some(opponent));
    let scatter = put(&mut game, opponent, "Essence Scatter", Zone::Hand);
    cast(&mut game, scatter, Some(Target::Stack(first_cast)));
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
