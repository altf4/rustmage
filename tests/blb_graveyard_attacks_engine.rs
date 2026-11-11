//! Graveyard attack observers, intervening threshold and exact paid return.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const PAYMENT: Mana = [0, 0, 1, 0, 2, 0];

const GRAVEYARD_ATTACKER: CardRules = CardRules {
    effect: Effect::Creature(3, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InZone {
            zone: TargetZone::Graveyard,
            event: &TriggerCondition::InterveningIf {
                guard: StepGuard::Expression {
                    value: StatValue::GraveyardCards,
                    minimum: Some(7),
                    maximum: None,
                },
                event: &TriggerCondition::ControlledSubtypeAttackDeclaration("Rat"),
            },
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayPayCosts {
                costs: &[ResolutionCost {
                    mana: ManaCost {
                        colored: [0, 0, 1, 0, 0, 0],
                        generic: 2,
                    },
                    ..ResolutionCost::FREE
                }],
                skip: 1,
            },
            Instruction::ReturnSourceTappedAttacking,
        ],
    }],
    ..CardRules::EMPTY
};

fn graveyard(game: &mut Game, player: Player, filler_count: usize) -> (CardId, Vec<CardId>) {
    let source = put(game, player, "Hill Giant", Zone::Hand);
    game.move_object(source, Zone::Graveyard);
    let fillers = (0..filler_count)
        .map(|_| put(game, player, "Forest", Zone::Graveyard))
        .collect();
    (source, fillers)
}

fn attack(game: &mut Game, attackers: &[CardId]) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    game.state.priority = game.state.active;
    for &card in attackers {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

fn resolve_top(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn a_threshold_group_checks_the_graveyard_before_queueing_and_again_before_payment() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &GRAVEYARD_ATTACKER)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let (source, fillers) = graveyard(&mut game, player, 5);
        graveyard(&mut game, player.other(), 6);
        let first = put(&mut game, player, "Burglar Rat", Zone::Battlefield);
        let second = put(&mut game, player, "Burglar Rat", Zone::Battlefield);
        let non_rat = put(&mut game, player, "Bear Cub", Zone::Battlefield);

        let mut six_cards = game.clone();
        attack(&mut six_cards, &[first, second]);
        assert!(six_cards.state.stack.is_empty());

        put(&mut game, player, "Forest", Zone::Graveyard);
        let mut no_rat = game.clone();
        attack(&mut no_rat, &[non_rat]);
        assert!(no_rat.state.stack.is_empty());

        let mut not_in_graveyard = game.clone();
        not_in_graveyard.move_object(source, Zone::Battlefield);
        put(&mut not_in_graveyard, player, "Forest", Zone::Graveyard);
        attack(&mut not_in_graveyard, &[first, second]);
        assert!(not_in_graveyard.state.stack.is_empty());

        let watched = game.object_ref(source);
        attack(&mut game, &[first, second]);
        assert_eq!(game.state.stack.len(), 1);
        let packet = game.state.stack.last().unwrap();
        assert_eq!(packet.source().unwrap().object, watched);
        assert!(packet.targets.is_empty());
        assert_eq!(packet.controller, player);
        game.state.players[player.index()].mana = PAYMENT;

        let mut falls_below = game.clone();
        falls_below.move_object(fillers[0], Zone::Exile);
        resolve_top(&mut falls_below);
        assert_eq!(falls_below.state.pending, Pending::Priority);
        assert_eq!(falls_below.object_view(source).zone, Zone::Graveyard);
        assert_eq!(falls_below.state.players[player.index()].mana, PAYMENT);

        let mut decline = game.clone();
        resolve_top(&mut decline);
        act(&mut decline, Action::DeclineResolutionMana);
        assert_eq!(decline.object_view(source).zone, Zone::Graveyard);
        assert_eq!(decline.state.players[player.index()].mana, PAYMENT);

        let mut stale = game.clone();
        stale.move_object(source, Zone::Exile);
        stale.move_object(source, Zone::Graveyard);
        let replacement = stale.object_ref(source);
        resolve_top(&mut stale);
        act(&mut stale, Action::PayResolutionMana { payment: PAYMENT });
        assert_eq!(stale.object_ref(source), replacement);
        assert_eq!(stale.object_view(source).zone, Zone::Graveyard);
    }
}

#[test]
fn paid_return_chooses_the_defender_without_declaring_another_attacker() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &GRAVEYARD_ATTACKER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let (source, _) = graveyard(&mut game, player, 6);
    let rat = put(&mut game, player, "Burglar Rat", Zone::Battlefield);
    let opponent = player.other();
    let walker = put(
        &mut game,
        opponent,
        "Kaito, Cunning Infiltrator",
        Zone::Battlefield,
    );
    game.add_counter(walker, CounterKind::Loyalty, 3);
    let origin = game.object_ref(source);
    attack(&mut game, &[rat]);
    game.state.players[player.index()].mana = PAYMENT;
    resolve_top(&mut game);
    act(&mut game, Action::PayResolutionMana { payment: PAYMENT });
    assert_eq!(
        game.state.pending,
        Pending::EntryAttackingDefender { player }
    );

    let choose = Action::ChooseEntryAttackingDefender {
        defender: CombatDefender::Planeswalker(game.object_ref(walker)),
    };
    let saved = game.snapshot();
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == choose)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(player, game.revision(), index).unwrap();
    act(&mut game, choose);
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_ne!(game.object_ref(source), origin);
    assert_eq!(game.object_view(source).zone, Zone::Battlefield);
    assert!(game.object_view(source).tapped);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    assert_eq!(game.declared_attackers_this_turn(player), 1);
    assert!(
        game.state
            .attackers
            .iter()
            .any(|attack| attack.card == source
                && attack.defender == Some(CombatDefender::Planeswalker(game.object_ref(walker))))
    );
    assert!(game.state.stack.is_empty());
    assert!(game.state.pending_triggers.is_empty());

    indexed.restore(&saved).unwrap();
    act(
        &mut indexed,
        Action::ChooseEntryAttackingDefender {
            defender: CombatDefender::Player(opponent),
        },
    );
    assert!(
        indexed
            .state
            .attackers
            .iter()
            .any(|attack| attack.card == source
                && attack.defender == Some(CombatDefender::Player(opponent)))
    );
    assert_eq!(indexed.declared_attackers_this_turn(player), 1);
}
