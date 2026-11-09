mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const PROVIDER: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    keywords: Keywords::FLASH,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::PlayerProtectionFromOpponents],
    }],
    ..CardRules::EMPTY
};
const STEAL: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::GainControlUntilEndTurn],
    ),
    ..CardRules::EMPTY
};
const MIXED: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::DamagePlayers(PlayerSet::Opponent, 2),
            Instruction::LoseLife(PlayerSet::Opponent, 1),
            Instruction::DamageCannotBePreventedThisTurn,
            Instruction::DamagePlayers(PlayerSet::Opponent, 2),
        ],
    ),
    ..CardRules::EMPTY
};
const DEALER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Any)),
        program: &[Instruction::Damage(2)],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const FORK: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Ability)),
        &[Instruction::CopyTargetAbility { group: 0 }],
    ),
    ..CardRules::EMPTY
};

fn pass_priority(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn live_player_protection_rechecks_targets_and_follows_the_providers_controller() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Cathar Commando").unwrap(), &PROVIDER),
        (Card::from_name("Opt").unwrap(), &STEAL),
    ]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let first = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, first, Some(Target::Player(opponent)));
    act(&mut game, Action::Pass);
    let provider = put(&mut game, opponent, "Cathar Commando", Zone::Hand);
    cast(&mut game, provider, None);
    settle(&mut game);
    assert_eq!(game.object_view(first).zone, Zone::Graveyard);
    assert_eq!(game.state.players[opponent.index()].life, 20);

    let second = put(&mut game, player, "Shock", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: second });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(opponent),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(player),
        },
    );
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 18);

    let steal = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, steal, Some(Target::Card(provider)));
    settle(&mut game);
    assert_eq!(game.object_view(provider).controller, player);
    let third = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, third, Some(Target::Player(opponent)));
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 18);

    let blank = put(&mut game, player, "Witness Protection", Zone::Hand);
    cast(&mut game, blank, Some(Target::Card(provider)));
    settle(&mut game);
    act(&mut game, Action::Pass);
    let fourth = put(&mut game, opponent, "Shock", Zone::Hand);
    cast(&mut game, fourth, Some(Target::Player(player)));
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 16);
}

#[test]
fn protection_prevents_combat_and_untargeted_damage_but_allows_life_loss_and_unpreventable_damage()
{
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Cathar Commando").unwrap(), &PROVIDER),
        (Card::from_name("Opt").unwrap(), &MIXED),
    ]);
    let player = Player::P0;
    let opponent = player.other();
    for combat in [false, true] {
        let mut game = scenario(player);
        put(&mut game, opponent, "Cathar Commando", Zone::Battlefield);
        if combat {
            let attacker = put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
            pass_priority(&mut game);
            pass_priority(&mut game);
            act(&mut game, Action::Attack { card: attacker });
            act(&mut game, Action::FinishAttackers);
            pass_priority(&mut game);
            act(&mut game, Action::FinishBlockers);
            pass_priority(&mut game);
        } else {
            let spell = put(&mut game, player, "Opt", Zone::Hand);
            cast(&mut game, spell, None);
            settle(&mut game);
        }
        assert_eq!(game.state.players[player.index()].life, 20);
        assert_eq!(
            game.state.players[opponent.index()].life,
            if combat { 20 } else { 17 }
        );
    }
}

#[test]
fn targeting_uses_the_ability_controller_and_prevention_uses_the_dealers_last_known_controller() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Cathar Commando").unwrap(), &PROVIDER),
        (Card::from_name("Grizzly Bears").unwrap(), &DEALER),
        (Card::from_name("Opt").unwrap(), &FORK),
    ]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    put(&mut game, opponent, "Cathar Commando", Zone::Battlefield);
    let source = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    game.change_controller(game.object_ref(source), player);
    choose(&mut game, |action| {
        matches!(action, Action::Activate { card, ability: 0, target: Some(Target::Player(target)), .. }
            if *card == source && *target == player)
    });
    let original = game.state.stack[0].id;
    act(&mut game, Action::Pass);
    let fork = put(&mut game, opponent, "Opt", Zone::Hand);
    cast(&mut game, fork, Some(Target::Stack(original)));
    pass_priority(&mut game);
    assert_eq!(game.actor(), Some(opponent));
    act(
        &mut game,
        Action::SetStackTarget {
            index: 0,
            target: Target::Player(opponent),
        },
    );
    act(&mut game, Action::FinishStackTargets);

    let bounce = put(&mut game, player, "Unsummon", Zone::Hand);
    cast(&mut game, bounce, Some(Target::Card(source)));
    settle(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Hand);
    assert_eq!(game.object_view(source).controller, opponent);
    assert_eq!(game.state.players[opponent.index()].life, 20);
    assert_eq!(game.state.players[player.index()].life, 18);
}
