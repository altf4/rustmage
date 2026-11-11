//! Descended is per-owner turn history of physical arrivals. Live graveyard
//! characteristics also govern activation legality and mana funding.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const WATCHER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::You,
            guard: StepGuard::Expression {
                value: StatValue::DescendedThisTurn,
                minimum: Some(1),
                maximum: None,
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLifeScaled(
            PlayerSet::Controller,
            AmountValue::Stat(StatValue::DescendedThisTurn),
        )],
    }],
    ..CardRules::EMPTY
};

const MILL_THREE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Mill(
            PlayerSet::Controller,
            MillAmount::Fixed(3),
        )],
    ),
    ..CardRules::EMPTY
};

const WHITE: Mana = [1, 0, 0, 0, 0, 0];
const GRAVEYARD_MANA: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::StatAtLeast {
            value: StatValue::GraveyardPermanentTypes,
            minimum: 4,
        },
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            colored: [0; 6],
            generic: 0,
            alternatives: &[],
            additional: None,
            mana_component: false,
            tap: true,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Fixed(WHITE))],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn descend_count(game: &Game, player: Player, source: CardId) -> i128 {
    game.stat_value(
        StatValue::DescendedThisTurn,
        game.object_ref(source),
        player,
    )
}

#[test]
fn simultaneous_deaths_record_owner_and_exclude_tokens_and_exile_replacements() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &WATCHER)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let defender = player.other();
        let own = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let stolen = put(&mut game, defender, "Bear Cub", Zone::Battlefield);
        game.record_control(game.object_ref(stolen), player, None);
        let finality = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        game.add_counter(finality, CounterKind::Finality, 1);
        // The same Rat recipe used by the Wilds combat tests.
        game.create_tokens(player, Card::test_index(5323), 1, false);
        let sweeper = put(&mut game, player, "Day of Judgment", Zone::Hand);

        cast(&mut game, sweeper, None);
        settle(&mut game);
        assert_eq!(game.object_view(own).zone, Zone::Graveyard);
        assert_eq!(game.object_view(stolen).zone, Zone::Graveyard);
        assert_eq!(game.object_view(finality).zone, Zone::Exile);
        assert_eq!(game.object_view(sweeper).zone, Zone::Graveyard);
        assert_eq!(descend_count(&game, player, sweeper), 1);
        assert_eq!(descend_count(&game, defender, sweeper), 1);
        for viewer in [player, defender] {
            let view = game.observe(viewer);
            assert_eq!(
                view.players[player.index()].controlled_creatures_died_this_turn,
                3
            );
            assert_eq!(
                view.players[player.index()].controlled_creature_departures_this_turn,
                4
            );
            assert_eq!(
                view.players[defender.index()].controlled_creature_departures_this_turn,
                0
            );
        }

        put(&mut game, player, "Hill Giant", Zone::Battlefield);
        game.begin_step(Step::End);
        game.prepare_triggers();
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, STARTING_LIFE + 1);
    }
}

#[test]
fn mill_batch_descend_history_survives_graveyard_departure_and_owned_restore() {
    let _rules =
        fixture_rules::activate(&[(named("Opt"), &MILL_THREE), (Card::HillGiant, &WATCHER)]);
    for player in [Player::P0, Player::P1] {
        let mut root = scenario(player);
        let land = put(&mut root, player, "Forest", Zone::Library);
        let instant = put(&mut root, player, "Opt", Zone::Library);
        let creature = put(&mut root, player, "Bear Cub", Zone::Library);
        let spell = put(&mut root, player, "Opt", Zone::Hand);
        // An existing graveyard card contributes to the live count only.
        put(&mut root, player, "Bear Cub", Zone::Graveyard);

        cast(&mut root, spell, None);
        act(&mut root, Action::Pass);
        let actor = root.actor().unwrap();
        let revision = root.revision();
        let pass = root
            .legal_actions()
            .iter()
            .position(|action| *action == Action::Pass)
            .unwrap();
        let checkpoint = root.snapshot();
        let mut raw = root.clone();
        let mut indexed = root.clone();
        drop(root);

        reject(&mut raw, Action::FinishAttackers);
        act(&mut raw, Action::Pass);
        indexed.apply_index(actor, revision, pass).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        for card in [land, instant, creature, spell] {
            assert_eq!(raw.object_view(card).zone, Zone::Graveyard);
        }
        assert_eq!(descend_count(&raw, player, spell), 2);
        assert_eq!(descend_count(&raw, player.other(), spell), 0);
        assert_eq!(
            raw.observe(player.other()).players[player.index()].descended_this_turn,
            2
        );
        indexed.restore(&checkpoint).unwrap();
        assert_eq!(
            indexed.observe(player.other()).players[player.index()].descended_this_turn,
            0
        );
        assert_eq!(descend_count(&indexed, player, spell), 0);
        assert_eq!(
            indexed.apply_index(actor, revision, pass),
            Err(Error::StaleRevision)
        );
        act(&mut indexed, Action::Pass);
        assert_eq!(raw.snapshot(), indexed.snapshot());

        raw.move_object(land, Zone::Exile);
        raw.move_object(creature, Zone::Exile);
        assert_eq!(descend_count(&raw, player, spell), 2);
        assert_eq!(indexed.object_view(land).zone, Zone::Graveyard);
        put(&mut raw, player, "Hill Giant", Zone::Battlefield);
        raw.begin_step(Step::End);
        raw.prepare_triggers();
        settle(&mut raw);
        assert_eq!(raw.state.players[player.index()].life, STARTING_LIFE + 2);

        let turn = raw.state.turn;
        for _ in 0..12 {
            if raw.state.turn != turn {
                break;
            }
            act(&mut raw, Action::Pass);
        }
        assert_ne!(raw.state.turn, turn);
        assert_eq!(descend_count(&raw, player, spell), 0);
        assert_eq!(
            raw.observe(player.other()).players[player.index()].descended_this_turn,
            0
        );
    }
}

#[test]
fn distinct_owned_permanent_types_control_the_mana_activation_and_funding() {
    let _rules = fixture_rules::activate(&[(named("Nutrient Block"), &GRAVEYARD_MANA)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        game.state.players[player.index()].mana = [0; 6];
        let source = put(&mut game, player, "Nutrient Block", Zone::Battlefield);
        let hawk = put(&mut game, player, "Healer's Hawk", Zone::Hand);
        put(&mut game, player, "Gingerbrute", Zone::Graveyard);
        put(&mut game, player, "Bear Cub", Zone::Graveyard);
        put(&mut game, player, "Forest", Zone::Graveyard);
        put(&mut game, player, "Opt", Zone::Graveyard);
        put(&mut game, player.other(), "Spelunking", Zone::Graveyard);
        let mana = Action::ActivateMana {
            card: source,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: WHITE,
        };
        let announce = Action::AnnounceCast { card: hawk };
        assert!(!game.legal_actions().contains(&announce));
        reject(&mut game, mana.clone());

        put(&mut game, player, "Spelunking", Zone::Graveyard);
        assert!(game.legal_actions().contains(&announce));
        act(&mut game, announce);
        act(&mut game, mana);
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.object_view(hawk).zone, Zone::Battlefield);
        assert!(game.object_view(source).tapped);
    }
}
