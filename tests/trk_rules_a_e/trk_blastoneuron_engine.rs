//! Exact graveyard exile, printed-card copy, lasting haste, and delayed sacrifice.
use crate::support::*;

use rustmage as _;
use rustmage_core::cards::authored::{CardRules, one};
use rustmage_core::engine::zone_replacements;
use rustmage_core::{cards::*, engine::*};

const TARGET: TargetRule = TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 1,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: 3,
};
const EXILE_COPY: CardRules = CardRules {
    effect: Effect::Spell(
        one(TARGET),
        &[
            Instruction::ExileGraveyardTargetCopyWithNextEndStepSacrifice {
                keywords: Keywords::HASTE,
            },
        ],
    ),
    ..CardRules::EMPTY
};
const LEAVE_WITNESS: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CardsLeaveGraveyard,
        targets: TargetPlan::NONE,
        program: &[Instruction::LoseOpponentGainSame(AmountValue::Fixed(1))],
    }],
    ..CardRules::EMPTY
};

fn copied_tokens(game: &Game) -> Vec<CardId> {
    game.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, object)| {
            (object.zone == Zone::Battlefield
                && object.controller == Player::P0
                && object.is_token())
            .then_some(id as CardId)
        })
        .collect()
}

fn settle_ordered(game: &mut Game) {
    for _ in 0..48 {
        if game.state.stack.is_empty()
            && game.state.resolving.is_none()
            && game.state.pending == Pending::Priority
        {
            return;
        }
        match game.state.pending {
            Pending::OrderTriggers { .. } => {
                choose(game, |action| matches!(action, Action::PutTrigger { .. }));
            }
            Pending::Priority => act(game, Action::Pass),
            ref pending => panic!("unexpected Blastoneuron continuation {pending:?}"),
        }
    }
    panic!("Blastoneuron resolution budget exhausted");
}

#[test]
fn exact_exile_copies_printed_card_and_its_graveyard_departure_triggers_once() {
    let opt = Card::from_name("Opt").unwrap();
    let watcher = Card::from_name("Serra Angel").unwrap();
    let _rules = fixture_rules::activate(&[(opt, &EXILE_COPY), (watcher, &LEAVE_WITNESS)]);
    let mut game = scenario(Player::P0);
    put(&mut game, Player::P0, "Serra Angel", Zone::Battlefield);
    let grave = put(&mut game, Player::P0, "Grizzly Bears", Zone::Graveyard);
    let too_large = put(&mut game, Player::P0, "Hill Giant", Zone::Graveyard);
    let foreign = put(&mut game, Player::P1, "Bear Cub", Zone::Graveyard);
    let spell = put(&mut game, Player::P0, "Opt", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    for illegal in [too_large, foreign] {
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(illegal),
            },
        );
    }
    let checkpoint = game.snapshot();
    let revision = game.revision();
    let action = Action::ChooseTarget {
        target: Target::Card(grave),
    };
    let index = game
        .legal_actions()
        .iter()
        .position(|legal| *legal == action)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(Player::P0, revision, index).unwrap();
    act(&mut game, action);
    assert_eq!(indexed.snapshot(), game.snapshot());
    indexed.restore(&checkpoint).unwrap();
    assert_eq!(indexed.snapshot(), checkpoint);
    pay(&mut game);
    settle_ordered(&mut game);
    assert_eq!(game.object_view(grave).zone, Zone::Exile);
    assert_eq!(game.state.players[0].life, 21);
    assert_eq!(game.state.players[1].life, 19);
    let created = copied_tokens(&game);
    assert_eq!(created.len(), 1);
    let token = created[0];
    assert_eq!(game.object_view(token).card, Card::GrizzlyBears);
    assert!(game.keywords(token).contains(Keywords::HASTE));
    assert_eq!(game.observe(Player::P0).delayed_sacrifices.len(), 1);

    let mut survivor = game.clone();
    survivor.state.step = Step::Cleanup;
    survivor.finish_cleanup();
    assert!(survivor.keywords(token).contains(Keywords::HASTE));

    game.begin_step(Step::End);
    game.prepare_triggers();
    settle_ordered(&mut game);
    assert_eq!(game.object_view(token).zone, Zone::Outside);
    assert_eq!(game.object_view(grave).zone, Zone::Exile);
}

#[test]
fn prevented_exile_cannot_create_a_copy_or_a_graveyard_departure_trigger() {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(None));
        }
    }
    let _reset = Reset;
    let opt = Card::from_name("Opt").unwrap();
    let watcher = Card::from_name("Serra Angel").unwrap();
    let _rules = fixture_rules::activate(&[(opt, &EXILE_COPY), (watcher, &LEAVE_WITNESS)]);
    let mut game = scenario(Player::P0);
    put(&mut game, Player::P0, "Serra Angel", Zone::Battlefield);
    let grave = put(&mut game, Player::P0, "Grizzly Bears", Zone::Graveyard);
    let spell = put(&mut game, Player::P0, "Opt", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(grave)));
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(Some(game.object_ref(grave))));
    settle(&mut game);
    assert_eq!(game.object_view(grave).zone, Zone::Graveyard);
    assert!(copied_tokens(&game).is_empty());
    assert_eq!(game.state.players[0].life, 20);
    assert_eq!(game.state.players[1].life, 20);
    assert!(game.observe(Player::P0).delayed_sacrifices.is_empty());
}
