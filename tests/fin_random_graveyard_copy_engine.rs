//! Owned random graveyard exile and sequential tapped-copy entries.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::engine::zone_replacements;
use rustmage_core::rng::Rng;
use rustmage_core::{cards::*, engine::*};
use support::*;

const RANDOM_COPY: CardRules = CardRules {
    effect: Effect::Creature(7, 7),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersOrAttacks,
        targets: TargetPlan::NONE,
        program: &[Instruction::CopyRandomGraveyardPermanentUntilNonland],
    }],
    ..CardRules::EMPTY
};

const LANDFALL: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::LandEntersYou,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

const ENTERING_CREATURE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    enters: InitialCounters {
        plus_one: 1,
        ..CardRules::EMPTY.enters
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 3)],
    }],
    ..CardRules::EMPTY
};

const OPPOSING_ENTRY: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreatureEnters {
            relation: CastRelation::Opponent,
            other: true,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 7)],
    }],
    ..CardRules::EMPTY
};

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn copies(game: &Game, player: Player) -> Vec<ObjectView> {
    game.observe(player)
        .objects
        .into_iter()
        .filter(|object| object.is_token() && object.zone == Zone::Battlefield)
        .collect()
}

#[test]
fn attack_copy_repeats_through_an_owned_land_entry_then_stacks_triggers_in_apnap_order() {
    let landfall = Card::from_name("Llanowar Elves").unwrap();
    let opponent_watcher = Card::from_name("Goblin Piker").unwrap();
    let _rules = fixture_rules::activate(&[
        (Card::HillGiant, &RANDOM_COPY),
        (landfall, &LANDFALL),
        (Card::GrizzlyBears, &ENTERING_CREATURE),
        (opponent_watcher, &OPPOSING_ENTRY),
    ]);
    let player = Player::P1;
    let mut game = scenario(player);
    let forest = put(&mut game, player, "Forest", Zone::Graveyard);
    let market = put(&mut game, player, "Night Market", Zone::Graveyard);
    let unchosen = put(&mut game, player, "Bear Cub", Zone::Graveyard);
    let copied = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    game.set_counters(copied, Counters::of(CounterKind::PlusOne, 6));
    game.move_object(copied, Zone::Graveyard);
    let instant = put(&mut game, player, "Opt", Zone::Graveyard);
    let foreign = put(&mut game, player.other(), "Mountain", Zone::Graveyard);
    let source = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    spawn(&mut game, player, landfall, Zone::Battlefield);
    spawn(
        &mut game,
        player.other(),
        opponent_watcher,
        Zone::Battlefield,
    );
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    game.state.rng = Rng(3);
    let mut expected_rng = game.state.rng;
    assert_eq!(
        [
            expected_rng.below(4),
            expected_rng.below(3),
            expected_rng.below(2)
        ],
        [1, 0, 1]
    );
    act(&mut game, Action::Attack { card: source });
    act(&mut game, Action::FinishAttackers);
    assert_eq!(game.state.stack.len(), 1);
    pass_pair(&mut game);

    assert_eq!(game.state.pending, Pending::EntryValue { player });
    assert_eq!(game.object_view(market).zone, Zone::Exile);
    assert_eq!(game.object_view(forest).zone, Zone::Graveyard);
    assert!(copies(&game, player).is_empty());
    assert!(game.state.resolving.is_some());
    assert!(!game.legal_actions().contains(&Action::Pass));

    let color = |color| Action::ChooseEntryValue {
        value: EntryChoiceValue::Color(color),
    };
    let action = color(ChoiceColor::Blue);
    let saved = game.snapshot();
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(player, revision, index).unwrap();
    act(&mut game, action);
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(player, revision, index),
        Err(Error::StaleRevision)
    );
    act(&mut indexed, color(ChoiceColor::Green));

    for (branch, chosen) in [(&game, ChoiceColor::Blue), (&indexed, ChoiceColor::Green)] {
        assert_eq!(branch.state.rng, expected_rng);
        assert!(branch.state.resolving.is_none());
        assert_eq!(branch.state.pending, Pending::OrderTriggers { player });
        for card in [market, forest, copied] {
            assert_eq!(branch.object_view(card).zone, Zone::Exile);
        }
        for card in [unchosen, instant, foreign] {
            assert_eq!(branch.object_view(card).zone, Zone::Graveyard);
        }
        let tokens = copies(branch, player);
        assert_eq!(tokens.len(), 3);
        assert!(
            tokens
                .iter()
                .all(|token| token.tapped && token.controller == player)
        );
        assert!(tokens.iter().any(|token| {
            token.card == branch.object_view(market).card
                && token.entry_choice == Some(EntryChoiceValue::Color(chosen))
        }));
        assert!(tokens.iter().any(|token| {
            token.card == Card::GrizzlyBears && token.power == 3 && token.toughness == 3
        }));
        assert_eq!(branch.state.attackers.len(), 1);
        assert_eq!(branch.state.attackers[0].card, source);
    }

    for _ in 0..2 {
        assert_eq!(game.actor(), Some(player));
        choose(&mut game, |action| {
            matches!(action, Action::PutTrigger { .. })
        });
    }
    assert_eq!(game.state.stack.len(), 4);
    assert_eq!(game.state.stack.last().unwrap().controller, player.other());
    pass_pair(&mut game);
    assert_eq!(game.state.players[player.other().index()].life, 27);
    assert_eq!(game.state.players[player.index()].life, 20);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 25);
}

#[test]
fn entering_copy_exhausts_an_all_land_pool_and_ignores_an_empty_permanent_pool() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &RANDOM_COPY)]);
    let player = Player::P0;
    for count in [0, 3] {
        let mut game = scenario(player);
        let instant = put(&mut game, player, "Opt", Zone::Graveyard);
        let lands = (0..count)
            .map(|_| put(&mut game, player, "Forest", Zone::Graveyard))
            .collect::<Vec<_>>();
        put(&mut game, player, "Doubling Season", Zone::Battlefield);
        let source = put(&mut game, player, "Hill Giant", Zone::Hand);
        let mut expected_rng = game.state.rng;
        for remaining in (2..=count).rev() {
            expected_rng.below(remaining);
        }
        cast(&mut game, source, None);
        settle(&mut game);
        assert_eq!(game.state.rng, expected_rng);
        assert_eq!(game.object_view(instant).zone, Zone::Graveyard);
        assert!(
            lands
                .into_iter()
                .all(|land| game.object_view(land).zone == Zone::Exile)
        );
        let tokens = copies(&game, player);
        assert_eq!(tokens.len(), (2 * count) as usize);
        assert!(
            tokens
                .iter()
                .all(|token| token.tapped && token.card == Card::Forest)
        );
    }
}

#[test]
fn a_prevented_or_redirected_random_land_exile_stops_and_replays_the_same_sample() {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(None));
            zone_replacements::TEST_REDIRECT_EXILE_TO_BATTLEFIELD.with(|rule| rule.set(None));
        }
    }
    let _reset = Reset;
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &RANDOM_COPY)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Forest", Zone::Graveyard);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    let source = put(&mut game, player, "Hill Giant", Zone::Hand);
    cast(&mut game, source, None);
    pass_pair(&mut game);
    game.state.rng = Rng(2);
    game.state.zone_rules.get_or_insert_default();
    let land_ref = game.object_ref(land);
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(Some(land_ref)));
    zone_replacements::TEST_REDIRECT_EXILE_TO_BATTLEFIELD.with(|rule| rule.set(Some(land_ref)));
    let mut expected_rng = game.state.rng;
    assert_eq!(expected_rng.below(2), 0);
    pass_pair(&mut game);
    assert_eq!(game.state.pending, Pending::ZoneReplacement { player });
    assert_eq!(game.object_ref(land), land_ref);

    let prevent = Action::ChooseZoneReplacement {
        effect: ZoneReplacementKind::Prevent,
    };
    let saved = game.snapshot();
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == prevent)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(player, revision, index).unwrap();
    act(&mut game, prevent);
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.object_ref(land), land_ref);

    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(player, revision, index),
        Err(Error::StaleRevision)
    );
    act(
        &mut indexed,
        Action::ChooseZoneReplacement {
            effect: ZoneReplacementKind::Battlefield,
        },
    );
    assert_eq!(indexed.object_view(land).zone, Zone::Battlefield);
    for branch in [&game, &indexed] {
        assert_eq!(branch.object_view(creature).zone, Zone::Graveyard);
        assert_eq!(branch.state.rng, expected_rng);
        assert!(branch.state.resolving.is_none());
        assert!(copies(branch, player).is_empty());
    }
}
