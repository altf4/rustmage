mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const PREVENT: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::PreventControlledCreatureDamageThisTurn],
    ),
    ..CardRules::EMPTY
};
const ANIMATE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Any)),
        &[Instruction::AnimateTargets(AnimationSpec {
            artifact: false,
            creature_types: AnimationCreatureTypes::Unchanged,
            base: Some((4, 4)),
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        })],
    ),
    ..CardRules::EMPTY
};
const UNPREVENTABLE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Any)),
        &[
            Instruction::DamageCannotBePreventedThisTurn,
            Instruction::Damage(2),
        ],
    ),
    ..CardRules::EMPTY
};

fn pass_priority(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn resolve(game: &mut Game, player: Player, name: &str, target: Option<Target>) -> CardId {
    if game.actor() != Some(player) {
        act(game, Action::Pass);
    }
    let spell = put(game, player, name, Zone::Hand);
    cast(game, spell, target);
    settle(game);
    spell
}

#[test]
fn prevention_tracks_current_creatures_and_controller_after_the_source_departs() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Opt").unwrap(), &PREVENT),
        (Card::from_name("Shore Up").unwrap(), &ANIMATE),
        (Card::from_name("Unsummon").unwrap(), &UNPREVENTABLE),
    ]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = resolve(&mut game, player, "Opt", None);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    let late = resolve(&mut game, player, "Hill Giant", None);
    let other = put(&mut game, opponent, "Gigantosaurus", Zone::Battlefield);
    let walker = put(
        &mut game,
        player,
        "Elspeth, Storm Slayer",
        Zone::Battlefield,
    );
    game.add_counter(walker, CounterKind::Loyalty, 9);

    for target in [
        Target::Card(late),
        Target::Card(other),
        Target::Card(walker),
        Target::Player(player),
    ] {
        resolve(&mut game, opponent, "Shock", Some(target));
    }
    assert_eq!(game.object_view(late).damage, 0);
    assert_eq!(game.object_view(other).damage, 2);
    assert_eq!(game.counter_count(walker, CounterKind::Loyalty), 7);
    assert_eq!(game.state.players[player.index()].life, 18);

    resolve(&mut game, player, "Shore Up", Some(Target::Card(walker)));
    assert!(game.is_creature(walker));
    resolve(&mut game, opponent, "Shock", Some(Target::Card(walker)));
    assert_eq!(game.counter_count(walker, CounterKind::Loyalty), 7);
    assert_eq!(game.object_view(walker).damage, 0);

    game.change_controller(game.object_ref(other), player);
    game.change_controller(game.object_ref(late), opponent);
    resolve(&mut game, opponent, "Shock", Some(Target::Card(other)));
    resolve(&mut game, opponent, "Shock", Some(Target::Card(late)));
    assert_eq!(game.object_view(other).damage, 2);
    assert_eq!(game.object_view(late).damage, 2);

    resolve(&mut game, opponent, "Unsummon", Some(Target::Card(other)));
    assert_eq!(game.object_view(other).damage, 4);
}

#[test]
fn affected_player_can_order_prevention_and_the_rule_expires_before_cleanup_priority() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &PREVENT)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let mountain = put(&mut game, opponent, "Mountain", Zone::Battlefield);
    let target = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    game.add_counter(target, CounterKind::Shield, 1);
    let source = resolve(&mut game, player, "Opt", None);
    let shields = game.observe(opponent).creature_damage_shields;
    assert_eq!(shields.len(), 1);
    assert_eq!(shields[0].controller, player);
    assert_eq!(shields[0].source.card, source);

    act(&mut game, Action::Pass);
    let shock = put(&mut game, opponent, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Card(target)));
    pass_priority(&mut game);
    assert_eq!(game.actor(), Some(player));
    let continuous = Action::ChooseDamagePrevention {
        source: shields[0].source,
        prevention: DamagePreventionKind::ControlledCreaturesThisTurn(shields[0].id),
    };
    let counter = Action::ChooseDamagePrevention {
        source: game.object_ref(target),
        prevention: DamagePreventionKind::ShieldCounter,
    };
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == continuous)
        .unwrap();
    let saved = game.snapshot();
    let revision = game.revision();
    let mut indexed = game.clone();
    assert_eq!(
        game.apply(opponent, continuous.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(game.snapshot(), saved);
    act(&mut game, continuous);
    indexed.apply_index(player, revision, index).unwrap();
    settle(&mut game);
    settle(&mut indexed);
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.object_view(target).damage, 0);
    assert_eq!(game.counter_count(target, CounterKind::Shield), 1);

    game.restore(&saved).unwrap();
    act(&mut game, counter);
    settle(&mut game);
    assert_eq!(game.object_view(target).damage, 0);
    assert_eq!(game.counter_count(target, CounterKind::Shield), 0);
    let doomed = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    game.state.objects[doomed as usize].growth.toughness = 1;
    game.add_counter(doomed, CounterKind::MinusOne, 2);
    game.state.step = Step::End;
    let turn = game.state.turn;
    pass_priority(&mut game);
    assert_eq!(game.state.step, Step::Cleanup);
    assert_eq!(game.state.turn, turn);
    assert_eq!(game.object_view(doomed).zone, Zone::Graveyard);
    assert!(game.observe(player).creature_damage_shields.is_empty());

    act(&mut game, Action::Pass);
    act(&mut game, Action::TapMana { card: mountain });
    resolve(&mut game, opponent, "Shock", Some(Target::Card(target)));
    assert_eq!(game.state.turn, turn);
    assert_eq!(game.object_view(target).damage, 2);
}
