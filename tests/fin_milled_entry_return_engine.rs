//! Milled cards enter together and one delayed trigger returns that exact cohort.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const ENCOUNTER: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::ShuffleYourLibrary,
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(4)),
            Instruction::SelectMilled(MilledSelectionSpec {
                filter: MilledFilter::Kinds(1),
                count: MilledCount::All,
                destination: MilledDestination::BattlefieldWithHasteAndReturn { ability: 0 },
            }),
        ],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::NONE,
        program: &[Instruction::ReturnDelayedBattlefieldObjectsToHand],
    }],
    extras: &CardExtras {
        flashback_cost: Some(ManaCost {
            colored: [0, 0, 0, 2, 0, 0],
            generic: 6,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

struct ResetPrevention;

impl Drop for ResetPrevention {
    fn drop(&mut self) {
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(None));
    }
}

fn encounter() -> Card {
    Card::from_name("Random Encounter").unwrap()
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn empty_library(game: &mut Game, player: Player) {
    let cards = game.state.players[player.index()].library.clone();
    for &card in cards.iter() {
        game.move_object(card, Zone::Exile);
    }
}

#[test]
fn public_replacement_mill_and_prevented_entry_produce_one_actual_owned_cohort() {
    let _rules = fixture_rules::activate(&[(encounter(), &ENCOUNTER)]);
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    empty_library(&mut game, player);
    let [first, second, blocked, noncreature] =
        ["Grizzly Bears", "Hill Giant", "Llanowar Elves", "Shock"]
            .map(|name| put(&mut game, player, name, Zone::Library));
    let old = put(&mut game, player, "Bear Cub", Zone::Graveyard);
    let spell = spawn(&mut game, player, encounter(), Zone::Graveyard);
    put(&mut game, opponent, "Rest in Peace", Zone::Battlefield);
    let mut expected_rng = game.state.rng;
    let mut library = game.state.players[player.index()].library.to_vec();
    expected_rng.shuffle(&mut library);

    let _reset = ResetPrevention;
    let blocked_after_mill = ObjectRef {
        card: blocked,
        generation: game.object_ref(blocked).generation + 1,
    };
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(Some(blocked_after_mill)));
    game.state.players[player.index()].mana = [0, 0, 0, 2, 0, 6];
    act(&mut game, Action::AnnounceFlashback { card: spell });
    pay(&mut game);
    let source = game.state.stack[0].source().unwrap().object;
    act(&mut game, Action::Pass);
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == Action::Pass)
        .unwrap();
    let revision = game.revision();
    let checkpoint = game.snapshot();
    let mut raw = game.clone();
    act(&mut raw, Action::Pass);
    game.apply_index(opponent, revision, index).unwrap();
    assert_eq!(raw.snapshot(), game.snapshot());
    for branch in [&raw, &game] {
        assert_eq!(branch.state.rng, expected_rng);
        assert_eq!(branch.object_view(spell).zone, Zone::Exile);
        assert_eq!(branch.object_view(old).zone, Zone::Graveyard);
        assert_eq!(branch.object_view(noncreature).zone, Zone::Exile);
        assert_eq!(branch.object_ref(blocked), blocked_after_mill);
        assert_eq!(branch.object_view(blocked).zone, Zone::Exile);
        let expected = [branch.object_ref(first), branch.object_ref(second)];
        for exact in expected {
            assert_eq!(branch.object_view(exact.card).zone, Zone::Battlefield);
            assert!(branch.keywords(exact.card).contains(Keywords::HASTE));
        }
        let delayed = branch.observe(opponent).delayed_steps.unwrap();
        assert_eq!(delayed.len(), 1);
        assert_eq!(delayed[0].item.source().unwrap().object, source);
        assert_eq!(delayed[0].item.controller, player);
        assert_eq!(delayed[0].active_player, None);
        let group = delayed[0].item.event_object.as_ref().unwrap();
        let companion = group.semantic_objects.as_ref().unwrap()[0];
        assert!(expected.contains(&group.object));
        assert!(
            expected
                .iter()
                .any(|&r| { r != group.object && companion == DamageSourceRef::Object(r) })
        );
    }

    raw.restore(&checkpoint).unwrap();
    act(&mut raw, Action::Pass);
    assert_eq!(raw.snapshot(), game.snapshot());
    let departed = raw.object_ref(first);
    raw.move_object(first, Zone::Exile);
    raw.move_object(first, Zone::Battlefield);
    assert_ne!(raw.object_ref(first), departed);
    assert!(!raw.keywords(first).contains(Keywords::HASTE));
    raw.record_control(raw.object_ref(second), opponent, None);
    assert_eq!(raw.object_view(second).controller, opponent);
    for branch in [&mut raw, &mut game] {
        branch.state.active = opponent;
        branch.state.step = Step::Main2;
        branch.give_priority();
        pass_pair(branch);
        assert_eq!(branch.state.step, Step::End);
        assert_eq!(branch.state.stack.len(), 1);
        assert!(branch.state.stack[0].targets.is_empty());
        assert_eq!(branch.state.stack[0].controller, player);
        settle(branch);
        assert_eq!(branch.object_view(second).zone, Zone::Hand);
        assert_eq!(branch.object_view(second).owner, player);
        assert!(branch.observe(player).delayed_steps.is_none());
    }
    assert_eq!(raw.object_view(first).zone, Zone::Battlefield);
    assert_eq!(game.object_view(first).zone, Zone::Hand);
}

#[test]
fn an_added_end_step_can_receive_the_trigger_and_countering_it_keeps_haste() {
    let card = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(card, &ENCOUNTER)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    empty_library(&mut game, player);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Library);
    let spell = spawn(&mut game, player, card, Zone::Hand);
    let lands = [(); 3].map(|_| put(&mut game, opponent, "Island", Zone::Battlefield));
    let turn = game.state.turn;
    game.state.step = Step::End;
    game.add_end_step_if_first();
    cast(&mut game, spell, None);
    settle(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Battlefield);
    assert!(game.keywords(creature).contains(Keywords::HASTE));
    pass_pair(&mut game);
    assert_eq!(game.state.turn, turn);
    assert!(
        game.observe(player)
            .additional_end_steps
            .unwrap()
            .current_is_additional
    );
    let trigger = game.state.stack[0].id;

    act(&mut game, Action::Pass);
    for land in lands {
        act(&mut game, Action::TapMana { card: land });
    }
    let counter = put(&mut game, opponent, "Tishana's Tidebinder", Zone::Hand);
    cast(&mut game, counter, None);
    pass_pair(&mut game);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Stack(trigger),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    settle(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Battlefield);
    pass_pair(&mut game);
    assert!(game.state.turn > turn);
    assert!(game.keywords(creature).contains(Keywords::HASTE));
}

#[test]
fn an_empty_actual_cohort_still_creates_one_no_target_end_step_trigger() {
    let _rules = fixture_rules::activate(&[(encounter(), &ENCOUNTER)]);
    let player = Player::P0;
    for has_land in [false, true] {
        let mut game = scenario(player);
        empty_library(&mut game, player);
        let land = has_land.then(|| put(&mut game, player, "Island", Zone::Library));
        let spell = spawn(&mut game, player, encounter(), Zone::Hand);
        cast(&mut game, spell, None);
        settle(&mut game);
        let waiting = game.observe(player).delayed_steps.unwrap();
        assert_eq!(waiting.len(), 1);
        assert!(waiting[0].item.event_object.is_none());
        game.state.step = Step::Main2;
        pass_pair(&mut game);
        assert_eq!(game.state.stack.len(), 1);
        settle(&mut game);
        assert!(game.observe(player).delayed_steps.is_none());
        if let Some(land) = land {
            assert_eq!(game.object_view(land).zone, Zone::Graveyard);
        }
    }
}
