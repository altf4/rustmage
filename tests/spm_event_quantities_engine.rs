//! Greatest power and exact counter quantities sampled in their rule contexts.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

const GREATEST_DEATHS: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OpponentHighestPowerCreatureDies,
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

#[test]
fn simultaneous_opposing_deaths_use_pre_event_power_and_preserve_ties() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &GREATEST_DEATHS)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let tied = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    game.add_counter(tied, CounterKind::PlusOne, 1);
    put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let own = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    game.add_counter(own, CounterKind::PlusOne, 5);
    let wrath = put(&mut game, player, "Day of Judgment", Zone::Hand);
    cast(&mut game, wrath, None);
    pass_pair(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    choose(&mut game, |action| {
        matches!(action, Action::PutTrigger { .. })
    });
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
}

const GRAVE_POWER: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::GainLifeScaled(
            PlayerSet::Controller,
            AmountValue::GreatestCreatureCardPowerInGraveyard,
        )],
    ),
    ..CardRules::EMPTY
};

#[test]
fn graveyard_power_uses_characteristic_defining_power_and_only_own_creature_cards() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &GRAVE_POWER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    for _ in 0..4 {
        put(&mut game, player, "Forest", Zone::Battlefield);
    }
    put(
        &mut game,
        player,
        "Sandman, Shifting Scoundrel",
        Zone::Graveyard,
    );
    put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    put(
        &mut game,
        player.other(),
        "Ancient Adamantoise",
        Zone::Graveyard,
    );
    put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, spell, None);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 24);
}

const LARGE_CREATURES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OtherFilteredEntersYou(
            EntryPredicate::CreatureManaValueAtLeast(4),
        ),
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 2)],
    }],
    ..CardRules::EMPTY
};

#[test]
fn entry_threshold_observes_creatures_of_the_required_mana_value() {
    let _rules =
        fixture_rules::activate(&[(Card::from_name("Nutrient Block").unwrap(), &LARGE_CREATURES)]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Nutrient Block", Zone::Battlefield);
    for (name, life) in [
        ("Grizzly Bears", 20),
        ("Hedron Archive", 20),
        ("Hill Giant", 22),
    ] {
        let spell = put(&mut game, player, name, Zone::Hand);
        cast(&mut game, spell, None);
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, life);
    }
}

const SOLO_COUNTERS: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ControlledCreatureAttacksAlone,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::PutCounters {
                subject: Subject::Entering,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::SurveilScaled(AmountValue::EventObjectTotalCounters),
        ],
    }],
    ..CardRules::EMPTY
};

#[test]
fn solo_attacker_uses_all_live_counters_or_its_exact_departure_total() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &SOLO_COUNTERS)]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let attacker = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    game.add_counter(attacker, CounterKind::Charge, 1);
    game.begin_step(Step::BeginCombat);
    pass_pair(&mut game);
    act(&mut game, Action::Attack { card: attacker });
    act(&mut game, Action::FinishAttackers);

    let counter = put(&mut game, player, "Snakeskin Veil", Zone::Hand);
    cast(&mut game, counter, Some(Target::Card(attacker)));
    pass_pair(&mut game);
    assert_eq!(game.counter_count(attacker, CounterKind::PlusOne), 1);

    let mut departed = game.clone();
    let bounce = put(&mut departed, player, "Unsummon", Zone::Hand);
    cast(&mut departed, bounce, Some(Target::Card(attacker)));
    pass_pair(&mut departed);
    assert_eq!(departed.object_view(attacker).zone, Zone::Hand);

    for (mut child, looked) in [(game, 3), (departed, 2)] {
        pass_pair(&mut child);
        let distinct: std::collections::BTreeSet<_> = child
            .legal_actions()
            .iter()
            .filter_map(|action| match action {
                Action::ArrangeLibrary { card, .. } => Some(*card),
                _ => None,
            })
            .collect();
        assert_eq!(distinct.len(), looked);
        assert!(child.observe(player.other()).library_choice.is_none());
    }
}
