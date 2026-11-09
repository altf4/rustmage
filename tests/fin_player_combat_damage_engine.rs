mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const BALL: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::OpponentWasDealtLegendaryCombatDamageThisTurn,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            tap: true,
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 2)],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const QUEST: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            guard: StepGuard::AnyPlayerCombatDamageAtLeast(6),
            event: &TriggerCondition::AtStep {
                step: TriggerStep::EndCombat,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};
const EXTRA_COMBAT: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, &[Instruction::AdditionalCombat]),
    ..CardRules::EMPTY
};
const REDIRECT: ReplacementSpec = ReplacementSpec::RedirectControlledDamageToSource;
const TURTLE: CardRules = CardRules {
    effect: Effect::Creature(8, 20),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(0, REDIRECT)],
    }],
    extras: &CardExtras {
        replacement_effects: &[REDIRECT],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const PROTECTOR: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::PlayerProtectionFromOpponents],
    }],
    ..CardRules::EMPTY
};

fn pass_priority(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn combat(game: &mut Game, attacker: CardId) {
    for _ in 0..8 {
        if game.state.pending == Pending::Attackers {
            break;
        }
        act(game, Action::Pass);
    }
    assert_eq!(game.state.pending, Pending::Attackers);
    act(game, Action::Attack { card: attacker });
    act(game, Action::FinishAttackers);
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
    settle(game);
    pass_priority(game);
    act(game, Action::FinishBlockers);
    pass_priority(game);
}

fn history(game: &Game) -> PlayerCombatDamageView {
    game.observe(Player::P0)
        .player_combat_damage_this_turn
        .unwrap_or_default()
}

#[test]
fn legendary_status_is_frozen_at_first_strike_and_activation_replays_exactly() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Hedron Archive").unwrap(), &BALL)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let attacker = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let first_plains = put(&mut game, player, "Plains", Zone::Battlefield);
    let second_plains = put(&mut game, player, "Plains", Zone::Battlefield);
    let serum = put(&mut game, player, "Super-Soldier Serum", Zone::Hand);
    cast(&mut game, serum, Some(Target::Card(attacker)));
    settle(&mut game);
    assert!(game.object_view(attacker).is_legendary());
    game.grant_keyword_effects(
        &[attacker],
        game.object_ref(attacker),
        Keywords::DOUBLE_STRIKE,
    );
    combat(&mut game, attacker);
    assert_eq!(game.state.step, Step::FirstStrikeDamage);
    assert_eq!(history(&game).amounts, [0, 4]);
    assert_eq!(history(&game).by_legendary_creature, [false, true]);

    let disenchant = put(&mut game, player, "Disenchant", Zone::Hand);
    for land in [first_plains, second_plains] {
        act(&mut game, Action::TapMana { card: land });
    }
    cast(&mut game, disenchant, Some(Target::Card(serum)));
    settle(&mut game);
    assert!(!game.object_view(attacker).is_legendary());
    let ball = put(&mut game, player, "Hedron Archive", Zone::Battlefield);
    assert!(!game.activation_guard_available(BALL.activated[0].guard, ball, opponent));
    let actions = game.legal_actions();
    let index = actions
        .iter()
        .position(
            |action| matches!(action, Action::Activate { card, ability: 0, .. } if *card == ball),
        )
        .unwrap();
    let saved = game.snapshot();
    let revision = game.revision();
    let mut indexed = game.clone();
    act(&mut game, actions[index].clone());
    indexed.apply_index(player, revision, index).unwrap();
    settle(&mut game);
    settle(&mut indexed);
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.object_view(ball).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
    game.restore(&saved).unwrap();
    assert_eq!(game.object_view(ball).zone, Zone::Battlefield);
    pass_priority(&mut game);
    assert_eq!(game.state.step, Step::CombatDamage);
    assert_eq!(history(&game).amounts, [0, 6]);
    assert_eq!(history(&game).by_legendary_creature, [false, true]);
}

#[test]
fn own_end_of_combat_uses_damage_from_both_combats_and_resets_at_the_next_turn() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Short Bow").unwrap(), &QUEST),
        (Card::from_name("Opt").unwrap(), &EXTRA_COMBAT),
    ]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let attacker = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    game.grant_keyword_effects(&[attacker], game.object_ref(attacker), Keywords::VIGILANCE);
    put(&mut game, player, "Short Bow", Zone::Battlefield);
    put(&mut game, opponent, "Short Bow", Zone::Battlefield);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, spell, None);
    settle(&mut game);

    combat(&mut game, attacker);
    pass_priority(&mut game);
    assert_eq!(game.state.step, Step::EndCombat);
    assert!(game.state.stack.is_empty());
    assert_eq!(history(&game).amounts, [0, 3]);

    combat(&mut game, attacker);
    pass_priority(&mut game);
    assert_eq!(game.state.step, Step::EndCombat);
    assert_eq!(game.state.stack.len(), 1);
    assert_eq!(game.state.stack[0].controller, player);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 21);
    assert_eq!(history(&game).amounts, [0, 6]);
    assert_eq!(history(&game).by_legendary_creature, [false, false]);

    game.state.step = Step::End;
    pass_priority(&mut game);
    assert_eq!(game.state.active, opponent);
    assert!(
        game.observe(player)
            .player_combat_damage_this_turn
            .is_none()
    );
}

#[test]
fn redirected_and_prevented_combat_damage_do_not_record_player_hits() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Gigantosaurus").unwrap(), &TURTLE),
        (Card::from_name("Cathar Commando").unwrap(), &PROTECTOR),
    ]);
    let player = Player::P0;
    let opponent = player.other();
    for (provider, damage) in [("Gigantosaurus", 3), ("Cathar Commando", 0)] {
        let mut game = scenario(player);
        let attacker = put(
            &mut game,
            player,
            "Captain America, Super-Soldier",
            Zone::Battlefield,
        );
        let prevention = put(&mut game, opponent, provider, Zone::Battlefield);
        combat(&mut game, attacker);
        assert_eq!(game.state.players[opponent.index()].life, 20);
        assert_eq!(game.object_view(prevention).damage, damage);
        assert!(game.observe(player).combat_damaged_players.is_empty());
        assert!(
            game.observe(player)
                .player_combat_damage_this_turn
                .is_none()
        );
    }
}

#[test]
fn totals_ignore_other_damage_and_keep_recipients_separate() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Short Bow").unwrap(), &QUEST)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let legend = put(
        &mut game,
        player,
        "Captain America, Super-Soldier",
        Zone::Battlefield,
    );
    let plain = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let walker = put(
        &mut game,
        opponent,
        "Elspeth, Storm Slayer",
        Zone::Battlefield,
    );
    let quest = put(&mut game, player, "Short Bow", Zone::Battlefield);
    game.add_counter(walker, CounterKind::Loyalty, 9);
    let legendary = game.damage_source(legend);
    let ordinary = game.damage_source(plain);
    let hit = |source, target, amount, combat| DamageEvent {
        source,
        target,
        amount,
        combat,
        preventable: true,
    };
    game.apply_damage_batch(&[
        hit(legendary, Target::Player(opponent), 7, false),
        hit(legendary, Target::Card(walker), 7, true),
        hit(legendary, Target::Player(opponent), 0, true),
        hit(ordinary, Target::Player(opponent), 3, true),
        hit(ordinary, Target::Player(player), 3, true),
    ]);
    assert_eq!(game.counter_count(walker, CounterKind::Loyalty), 2);
    assert_eq!(history(&game).amounts, [3, 3]);
    assert_eq!(history(&game).by_legendary_creature, [false, false]);
    assert!(!game.any_player_combat_damage_at_least(6));

    game.apply_damage_batch(&[hit(ordinary, Target::Player(player), 3, true)]);
    assert_eq!(history(&game).amounts, [6, 3]);
    game.begin_step(Step::EndCombat);
    assert_eq!(game.state.pending_triggers.len(), 1);
    assert_eq!(
        game.state.pending_triggers[0].source.unwrap().object,
        game.object_ref(quest)
    );
}
