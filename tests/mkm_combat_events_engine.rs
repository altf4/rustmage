//! Exact combat event packets shared by MKM triggers.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const BLOCKED_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::MENACE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfBecomesBlocked,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::UntapPlayerCreatures(PlayerSet::Controller),
            Instruction::AdditionalCombat,
        ],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost::ZERO,
        targets: TargetPlan::NONE,
        program: &[Instruction::RequireSubjectBlocked {
            subject: Subject::Source,
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

const DEFENDER_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreaturesAttackYouOrPlaneswalkersAtLeast(2),
        targets: TargetPlan::NONE,
        program: &[
            Instruction::LoseLife(PlayerSet::EventPlayer, 3),
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    }],
    ..CardRules::EMPTY
};

const REPLACE_ATTACKER: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[
            Instruction::Destroy,
            Instruction::CreateTokensAttacking {
                token: Card::from_catalogue_id(5329),
                amount: 1,
            },
        ],
    ),
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn start_attackers(game: &mut Game) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
}

#[test]
fn an_exact_source_must_be_blocked_and_two_blockers_trigger_it_once() {
    let _rules = fixture_rules::activate(&[(named("Hill Giant"), &BLOCKED_OBSERVER)]);
    let player = Player::P0;
    let defender = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let reserve = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
    let first = put(&mut game, defender, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, defender, "Bear Cub", Zone::Battlefield);
    game.state.objects[reserve as usize].tapped = true;
    choose(&mut game, |action| {
        matches!(action,
            Action::Activate { card, ability: 0, .. }
            | Action::AnnounceActivate { card, ability: 0, .. } if *card == source)
    });
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    settle(&mut game);
    assert_eq!(game.block_requirement_weight(source), 1);

    start_attackers(&mut game);
    act(&mut game, Action::Attack { card: source });
    act(&mut game, Action::FinishAttackers);
    pass_pair(&mut game);
    reject(&mut game, Action::FinishBlockers);
    for blocker in [first, second] {
        act(
            &mut game,
            Action::Block {
                blocker,
                attacker: source,
            },
        );
    }
    act(&mut game, Action::FinishBlockers);
    assert_eq!(game.state.stack.len(), 1);
    let exact = game.object_ref(source);
    assert_eq!(
        game.state.stack[0].event_object.as_ref().unwrap().object,
        exact
    );
    settle(&mut game);
    assert!(!game.object_view(source).tapped);
    assert!(!game.object_view(reserve).tapped);
    assert_eq!(
        game.state.combat_rules.as_ref().unwrap().after_combat[0].remaining,
        1
    );

    game.move_object(source, Zone::Exile);
    game.move_object(source, Zone::Battlefield);
    assert_ne!(game.object_ref(source), exact);
    assert_eq!(game.block_requirement_weight(source), 0);
}

#[test]
fn a_defender_trigger_rechecks_only_the_original_attack_declaration() {
    let _rules = fixture_rules::activate(&[
        (named("Hill Giant"), &DEFENDER_OBSERVER),
        (named("Opt"), &REPLACE_ATTACKER),
    ]);
    let attacker = Player::P0;
    let defender = attacker.other();
    let mut game = scenario(attacker);
    put(&mut game, defender, "Hill Giant", Zone::Battlefield);
    let first = put(&mut game, attacker, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, attacker, "Bear Cub", Zone::Battlefield);
    let walker = put(
        &mut game,
        defender,
        "Ajani, Caller of the Pride",
        Zone::Battlefield,
    );
    game.add_counter(walker, CounterKind::Loyalty, 4);
    let siege = spawn(
        &mut game,
        attacker,
        named("@fixture/siege-front"),
        Zone::Battlefield,
    );
    game.add_counter(siege, CounterKind::Defense, 8);
    game.set_battle_protector(game.object_ref(siege), defender);
    let battle = game.object_ref(siege);
    let walker = game.object_ref(walker);
    start_attackers(&mut game);
    act(&mut game, Action::Attack { card: first });
    let mut battle_route = game.clone();
    act(
        &mut battle_route,
        Action::AttackBattle {
            card: second,
            defender: battle,
        },
    );
    act(&mut battle_route, Action::FinishAttackers);
    assert!(battle_route.state.stack.is_empty());

    act(
        &mut game,
        Action::AttackPlaneswalker {
            card: second,
            defender: walker,
        },
    );
    act(&mut game, Action::FinishAttackers);
    assert_eq!(game.state.stack.len(), 1);
    assert_eq!(game.state.stack[0].event_player, Some(attacker));
    let mut replaced = game.clone();
    let answer = put(&mut replaced, attacker, "Opt", Zone::Hand);
    cast(&mut replaced, answer, Some(Target::Card(first)));
    pass_pair(&mut replaced);
    if matches!(replaced.state.pending, Pending::Mobilize { .. }) {
        act(
            &mut replaced,
            Action::ChooseMobilizeDefender {
                defender: CombatDefender::Player(defender),
            },
        );
    }
    assert_eq!(replaced.object_view(first).zone, Zone::Graveyard);
    assert_eq!(replaced.state.attackers.len(), 2);
    assert!(
        replaced
            .state
            .attackers
            .iter()
            .any(|row| replaced.object_view(row.card).is_token())
    );
    settle(&mut replaced);
    assert_eq!(replaced.state.players[attacker.index()].life, 20);
    assert_eq!(
        replaced.state.players[defender.index()].cards_drawn_this_turn,
        0
    );

    settle(&mut game);
    assert_eq!(game.state.players[attacker.index()].life, 17);
    assert_eq!(
        game.state.players[defender.index()].cards_drawn_this_turn,
        1
    );
}
