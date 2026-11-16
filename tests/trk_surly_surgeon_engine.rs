//! Surly Surgeon's live destruction replacement, including competing shields.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardRules, one};
use rustmage_core::{cards::*, engine::zone_replacements::ZoneReplacementKind, engine::*};
use support::*;

const SURGEON: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::SaveOtherPlusOneCreatureFromDestruction],
    }],
    ..CardRules::EMPTY
};
const DESTROY: CardRules = CardRules {
    effect: Effect::Spell(one(TargetRule::Creature), &[Instruction::Destroy]),
    ..CardRules::EMPTY
};
const SACRIFICE: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::creature_you_control()),
        &[Instruction::SacrificeTarget { group: 0 }],
    ),
    ..CardRules::EMPTY
};

fn rules() -> fixture_rules::Guard {
    fixture_rules::activate(&[
        (Card::GrizzlyBears, &SURGEON),
        (Card::from_name("Opt").unwrap(), &DESTROY),
        (Card::from_name("Duress").unwrap(), &SACRIFICE),
    ])
}

fn destroy(game: &mut Game, by: Player, target: CardId) {
    let spell = put(game, by, "Opt", Zone::Hand);
    cast(game, spell, Some(Target::Card(target)));
    settle(game);
}

#[test]
fn effect_destruction_heals_exact_other_creature_and_ignores_sacrifice() {
    let _rules = rules();
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let surgeon = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let saved = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let opposing = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        let mut counters = Counters::of(CounterKind::PlusOne, 2);
        counters.add(CounterKind::Stun, 1);
        game.set_counters(saved, counters);
        game.set_counters(opposing, Counters::of(CounterKind::PlusOne, 1));
        game.set_counters(surgeon, Counters::of(CounterKind::PlusOne, 1));
        game.state.objects[saved as usize].damage = 3;
        destroy(&mut game, player, saved);
        assert!(game.on_battlefield(saved));
        assert_eq!(game.counter_count(saved, CounterKind::PlusOne), 0);
        assert_eq!(game.counter_count(saved, CounterKind::Stun), 1);
        assert_eq!(game.state.objects[saved as usize].damage, 0);
        let next = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        game.set_counters(next, Counters::of(CounterKind::PlusOne, 1));
        let sacrifice = put(&mut game, player, "Duress", Zone::Hand);
        cast(&mut game, sacrifice, Some(Target::Card(next)));
        settle(&mut game);
        assert!(!game.on_battlefield(next));

        destroy(&mut game, player, surgeon);
        assert!(
            !game.on_battlefield(surgeon),
            "the source is not its own other creature"
        );
        destroy(&mut game, player, opposing);
        assert!(
            !game.on_battlefield(opposing),
            "opposing creature is not controlled"
        );
    }
}

#[test]
fn lethal_deathtouch_and_zero_toughness_have_distinct_state_based_outcomes() {
    let _rules = rules();
    let player = Player::P0;
    let mut game = scenario(player);
    let provider = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let lethal = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let mut lethal_counters = Counters::of(CounterKind::PlusOne, 1);
    lethal_counters.add(CounterKind::Shield, 1);
    game.set_counters(lethal, lethal_counters);
    game.state.objects[lethal as usize].damage = 3;
    act(&mut game, Action::Pass);
    assert!(game.on_battlefield(lethal));
    assert_eq!(game.counter_count(lethal, CounterKind::PlusOne), 0);
    assert_eq!(game.counter_count(lethal, CounterKind::Shield), 1);
    assert_eq!(game.state.objects[lethal as usize].damage, 0);

    let paired = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let mut simultaneous = Counters::of(CounterKind::PlusOne, 2);
    simultaneous.add(CounterKind::MinusOne, 1);
    game.set_counters(paired, simultaneous);
    game.state.objects[paired as usize].damage = 3;
    act(&mut game, Action::Pass);
    assert!(game.on_battlefield(paired));
    assert_eq!(game.counter_count(paired, CounterKind::PlusOne), 0);
    assert_eq!(game.counter_count(paired, CounterKind::MinusOne), 0);
    assert_eq!(game.state.objects[paired as usize].damage, 0);

    let deathtouch = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    game.set_counters(deathtouch, Counters::of(CounterKind::PlusOne, 1));
    game.state.objects[deathtouch as usize].deathtouch_since_sba = true;
    act(&mut game, Action::Pass);
    assert!(game.on_battlefield(deathtouch));
    assert_eq!(game.counter_count(deathtouch, CounterKind::PlusOne), 0);

    let zero = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    game.set_counters(zero, Counters::of(CounterKind::PlusOne, 1));
    game.state.objects[zero as usize].growth.toughness = -3;
    act(&mut game, Action::Pass);
    assert!(!game.on_battlefield(zero));
    assert!(game.on_battlefield(provider));
}

#[test]
fn shield_or_surgeon_choice_replays_exactly_on_raw_index_and_restore() {
    let _rules = rules();
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let target = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let mut counters = Counters::of(CounterKind::PlusOne, 2);
    counters.add(CounterKind::Shield, 1);
    game.set_counters(target, counters);
    game.state.objects[target as usize].damage = 2;
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(target)));
    for _ in 0..8 {
        if game.state.pending != Pending::Priority {
            break;
        }
        act(&mut game, Action::Pass);
    }
    assert_eq!(game.state.pending, Pending::ZoneReplacement { player });
    let view = game.observe(player).zone_replacement_choice.unwrap();
    assert_eq!(view.from, Zone::Battlefield);
    assert_eq!(view.card, Some(Card::from_name("Bear Cub").unwrap()));
    assert!(
        game.observe(player.other())
            .zone_replacement_choice
            .is_none()
    );
    let shield = Action::ChooseZoneReplacement {
        effect: ZoneReplacementKind::DestructionShield,
    };
    let heal = Action::ChooseZoneReplacement {
        effect: ZoneReplacementKind::DestructionHeal,
    };
    let actions = game.legal_actions();
    assert!(actions.contains(&shield) && actions.contains(&heal));
    let index = actions.iter().position(|a| *a == heal).unwrap();
    let checkpoint = game.snapshot();
    let mut raw = game.clone();
    let mut indexed = game.clone();
    act(&mut raw, heal);
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.counter_count(target, CounterKind::PlusOne), 0);
    assert_eq!(raw.counter_count(target, CounterKind::Shield), 1);
    assert_eq!(raw.state.objects[target as usize].damage, 0);
    indexed.restore(&checkpoint).unwrap();
    act(&mut indexed, shield);
    assert_eq!(indexed.counter_count(target, CounterKind::PlusOne), 2);
    assert_eq!(indexed.counter_count(target, CounterKind::Shield), 0);
    assert_eq!(indexed.state.objects[target as usize].damage, 2);
}

#[test]
fn copied_provider_control_and_ability_loss_are_live() {
    let _rules = rules();
    let player = Player::P0;
    let mut game = scenario(player);
    let copier = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let target = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let copy = game.object_ref(copier);
    game.set_copy_recipe(copy, CopyRecipe::new(Card::GrizzlyBears));
    let copied = game.snapshot();
    game.set_counters(target, Counters::of(CounterKind::PlusOne, 1));
    destroy(&mut game, player, target);
    assert!(game.on_battlefield(target));

    game.restore(&copied).unwrap();
    game.set_counters(target, Counters::of(CounterKind::PlusOne, 1));
    game.remove_abilities_exact(&[copy], copy, player, AnimationDuration::Permanent);
    destroy(&mut game, player, target);
    assert!(!game.on_battlefield(target));

    game.restore(&copied).unwrap();
    game.set_counters(target, Counters::of(CounterKind::PlusOne, 1));
    game.clear_copy_recipe(copy);
    destroy(&mut game, player, target);
    assert!(!game.on_battlefield(target));

    game.restore(&copied).unwrap();
    game.set_counters(target, Counters::of(CounterKind::PlusOne, 1));
    game.state.objects[copier as usize].controller = player.other();
    destroy(&mut game, player, target);
    assert!(!game.on_battlefield(target));
}
