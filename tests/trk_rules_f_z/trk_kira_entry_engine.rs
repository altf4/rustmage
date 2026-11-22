//! Kira's controlled-creature tapped-and-attacking entry replacement.
use crate::support::*;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};

const KIRA_REPLACEMENT: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::ControlledCreatureEntryTappedAttacking],
    }],
    ..CardRules::EMPTY
};

const ENTRANT: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfBecomesTapped,
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 2)],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 4)],
        },
    ],
    ..CardRules::EMPTY
};

const OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OtherCreatureEntersYou,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 8)],
    }],
    ..CardRules::EMPTY
};

fn setup(player: Player) -> (fixture_rules::Guard, Game, CardId, CardId) {
    let rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &KIRA_REPLACEMENT),
        (Card::HillGiant, &ENTRANT),
        (Card::from_name("Bear Cub").unwrap(), &OBSERVER),
    ]);
    let mut game = scenario(player);
    let provider = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let attacker = put(&mut game, player, "Burglar Rat", Zone::Battlefield);
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: attacker });
    act(&mut game, Action::FinishAttackers);
    (rules, game, provider, attacker)
}

#[test]
fn controlled_creature_enters_tapped_attacking_with_defender_choice_and_entry_trigger() {
    for player in [Player::P0, Player::P1] {
        let (_rules, mut game, _, attacker) = setup(player);
        let walker = put(
            &mut game,
            player.other(),
            "Kaito, Cunning Infiltrator",
            Zone::Battlefield,
        );
        game.add_counter(walker, CounterKind::Loyalty, 3);
        put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let entrant = put(&mut game, player, "Hill Giant", Zone::Hand);
        let movement = game.entry_move(entrant, Zone::Battlefield, false);
        game.stage_entries(vec![movement], None);
        assert!(game.advance_entries());
        assert_eq!(
            game.state.pending,
            Pending::EntryAttackingDefender { player }
        );

        let choice = Action::ChooseEntryAttackingDefender {
            defender: CombatDefender::Planeswalker(game.object_ref(walker)),
        };
        let before = game.snapshot();
        let revision = game.revision();
        let index = game
            .legal_actions()
            .iter()
            .position(|action| *action == choice)
            .unwrap();
        let mut indexed = game.clone();
        indexed.apply_index(player, revision, index).unwrap();
        act(&mut game, choice);
        assert_eq!(indexed.snapshot(), game.snapshot());
        assert!(game.object_view(entrant).tapped);
        assert!(game.state.attackers.iter().any(|row| {
            row.card == entrant
                && row.defender == Some(CombatDefender::Planeswalker(game.object_ref(walker)))
        }));
        assert_eq!(game.declared_attackers_this_turn(player), 1);
        assert!(game.state.attackers.iter().any(|row| row.card == attacker));
        // Self-entry and the other-creature entry observer fire. Entering
        // tapped is neither a tap event nor an attack declaration.
        assert_eq!(
            game.state.stack.len()
                + game.state.pending_triggers.len()
                + game.state.ordering_triggers.len(),
            2,
            "entry trigger queue: {:?}",
            game.state.pending,
        );

        indexed.restore(&before).unwrap();
        act(
            &mut indexed,
            Action::ChooseEntryAttackingDefender {
                defender: CombatDefender::Player(player.other()),
            },
        );
        assert!(indexed.state.attackers.iter().any(|row| {
            row.card == entrant && row.defender == Some(CombatDefender::Player(player.other()))
        }));
        assert_eq!(indexed.declared_attackers_this_turn(player), 1);
    }
}

#[test]
fn replacement_requires_a_live_controlled_attacker_and_provider() {
    let (_rules, game, provider, attacker) = setup(Player::P0);
    let mut no_attacker = game.clone();
    no_attacker.move_object(attacker, Zone::Graveyard);
    let entrant = put(&mut no_attacker, Player::P0, "Hill Giant", Zone::Hand);
    let movement = no_attacker.entry_move(entrant, Zone::Battlefield, false);
    no_attacker.stage_entries(vec![movement], None);
    assert!(!no_attacker.advance_entries());
    assert!(!no_attacker.object_view(entrant).tapped);
    assert!(
        !no_attacker
            .state
            .attackers
            .iter()
            .any(|row| row.card == entrant)
    );

    let mut no_provider = game.clone();
    no_provider.move_object(provider, Zone::Graveyard);
    let entrant = put(&mut no_provider, Player::P0, "Hill Giant", Zone::Hand);
    let movement = no_provider.entry_move(entrant, Zone::Battlefield, false);
    no_provider.stage_entries(vec![movement], None);
    assert!(!no_provider.advance_entries());
    assert!(!no_provider.object_view(entrant).tapped);
    assert!(
        !no_provider
            .state
            .attackers
            .iter()
            .any(|row| row.card == entrant)
    );

    let mut opposing = game;
    let entrant = put(&mut opposing, Player::P1, "Hill Giant", Zone::Hand);
    let movement = opposing.entry_move(entrant, Zone::Battlefield, false);
    opposing.stage_entries(vec![movement], None);
    assert!(!opposing.advance_entries());
    assert!(!opposing.object_view(entrant).tapped);
    assert!(
        !opposing
            .state
            .attackers
            .iter()
            .any(|row| row.card == entrant)
    );
}

#[test]
fn illegal_or_stale_defender_rejects_without_mutating_pending_entry() {
    let (_rules, mut game, _, _) = setup(Player::P0);
    let walker = put(
        &mut game,
        Player::P1,
        "Kaito, Cunning Infiltrator",
        Zone::Battlefield,
    );
    let entrant = put(&mut game, Player::P0, "Hill Giant", Zone::Hand);
    let movement = game.entry_move(entrant, Zone::Battlefield, false);
    game.stage_entries(vec![movement], None);
    assert!(game.advance_entries());
    let before = game.snapshot();
    assert!(
        game.apply(
            Player::P0,
            Action::ChooseEntryAttackingDefender {
                defender: CombatDefender::Player(Player::P0),
            }
        )
        .is_err()
    );
    assert_eq!(game.snapshot(), before);
    let stale = game.object_ref(walker);
    game.move_object(walker, Zone::Graveyard);
    assert!(
        game.apply(
            Player::P0,
            Action::ChooseEntryAttackingDefender {
                defender: CombatDefender::Planeswalker(stale),
            }
        )
        .is_err()
    );
}
