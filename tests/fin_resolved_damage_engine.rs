mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const STAGGER: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    keywords: Keywords::FIRST_STRIKE
        .union(Keywords::TRAMPLE)
        .union(Keywords::LIFELINK),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayerDamaged {
            subject: DamageSubject::Source,
            combat_only: true,
            opponent_only: false,
            one_or_more: false,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::DoubleDamageToDamagedPlayerUntilNextTurn],
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
const PLUS: ReplacementSpec = ReplacementSpec::NoncombatDamageToOpponentPlus(1);
const BOOSTER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(0, PLUS)],
    }],
    extras: &CardExtras {
        replacement_effects: &[PLUS],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn pass_priority(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn first_strike(game: &mut Game, dealer: CardId, other_attacker: Option<CardId>) {
    pass_priority(game);
    pass_priority(game);
    act(game, Action::Attack { card: dealer });
    if let Some(card) = other_attacker {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
    pass_priority(game);
    act(game, Action::FinishBlockers);
    pass_priority(game);
    assert_eq!(game.state.step, Step::FirstStrikeDamage);
    assert_eq!(game.state.stack.len(), 1);
}

fn next_turn(game: &mut Game) {
    game.state.step = Step::End;
    pass_priority(game);
}

fn shock(game: &mut Game, player: Player, land: CardId, target: CardId) {
    let spell = put(game, player, "Shock", Zone::Hand);
    act(game, Action::TapMana { card: land });
    cast(game, spell, Some(Target::Card(target)));
}

fn copy_trigger(game: &mut Game, player: Player, island: CardId) {
    let trigger = game.state.stack[0].id;
    if game.actor() != Some(player) {
        act(game, Action::Pass);
    }
    let fork = put(game, player, "Opt", Zone::Hand);
    act(game, Action::TapMana { card: island });
    cast(game, fork, Some(Target::Stack(trigger)));
    settle(game);
}

#[test]
fn resolved_doubler_survives_source_departure_and_expires_on_the_actual_next_turn() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Hill Giant").unwrap(), &STAGGER)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let dealer = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let source = game.object_ref(dealer);
    let other = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let island = put(&mut game, player, "Island", Zone::Battlefield);
    first_strike(&mut game, dealer, Some(other));
    assert_eq!(game.state.players[player.index()].life, 23);
    assert_eq!(game.state.players[opponent.index()].life, 17);
    assert!(game.observe(player).damage_doublers.is_empty());

    let bounce = put(&mut game, player, "Unsummon", Zone::Hand);
    act(&mut game, Action::TapMana { card: island });
    cast(&mut game, bounce, Some(Target::Card(dealer)));
    settle(&mut game);
    assert_eq!(game.object_view(dealer).zone, Zone::Hand);
    let effects = game.observe(opponent).damage_doublers;
    assert_eq!(effects.len(), 1);
    assert_eq!(effects[0].source, source);
    assert_eq!(effects[0].controller, player);
    assert_eq!(effects[0].recipient, opponent);
    pass_priority(&mut game);
    assert_eq!(game.state.step, Step::CombatDamage);
    assert_eq!(game.state.players[opponent.index()].life, 13);

    let mut extra_turn = game.clone();
    extra_turn.add_extra_turn(player, false);
    next_turn(&mut extra_turn);
    assert_eq!(extra_turn.state.active, player);
    assert!(extra_turn.observe(opponent).damage_doublers.is_empty());

    game.state.turn_skips.get_or_insert_default()[player.index()] = 1;
    next_turn(&mut game);
    assert_eq!(game.state.active, opponent);
    assert_eq!(game.observe(player).damage_doublers, effects);
    let incoming = put(&mut game, player, "Gigantosaurus", Zone::Battlefield);
    let outgoing = put(&mut game, opponent, "Gigantosaurus", Zone::Battlefield);
    game.change_controller(game.object_ref(incoming), opponent);
    game.change_controller(game.object_ref(outgoing), player);
    let first_mountain = put(&mut game, opponent, "Mountain", Zone::Battlefield);
    let second_mountain = put(&mut game, opponent, "Mountain", Zone::Battlefield);
    shock(&mut game, opponent, first_mountain, incoming);
    settle(&mut game);
    shock(&mut game, opponent, second_mountain, outgoing);
    settle(&mut game);
    assert_eq!(game.object_view(incoming).damage, 4);
    assert_eq!(game.object_view(outgoing).damage, 2);
    assert_eq!(game.state.players[player.index()].life, 23);

    next_turn(&mut game);
    assert_eq!(game.state.active, opponent);
    assert_eq!(game.observe(player).damage_doublers, effects);
    next_turn(&mut game);
    assert_eq!(game.state.active, player);
    assert!(game.observe(opponent).damage_doublers.is_empty());
}

#[test]
fn a_copied_trigger_expires_on_its_own_controllers_next_turn() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Hill Giant").unwrap(), &STAGGER),
        (Card::from_name("Opt").unwrap(), &FORK),
    ]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let dealer = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let island = put(&mut game, opponent, "Island", Zone::Battlefield);
    first_strike(&mut game, dealer, None);
    copy_trigger(&mut game, opponent, island);
    let effects = game.observe(player).damage_doublers;
    assert_eq!(effects.len(), 2);
    assert_eq!(effects[0].controller, opponent);
    assert_eq!(effects[1].controller, player);
    assert!(effects.iter().all(|effect| effect.recipient == opponent));

    next_turn(&mut game);
    assert_eq!(game.state.active, opponent);
    assert_eq!(game.observe(player).damage_doublers, vec![effects[1]]);
    next_turn(&mut game);
    assert_eq!(game.state.active, player);
    assert!(game.observe(player).damage_doublers.is_empty());
}

#[test]
fn copied_doublers_keep_distinct_ids_across_affected_player_choices_and_restore() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Hill Giant").unwrap(), &STAGGER),
        (Card::from_name("Opt").unwrap(), &FORK),
        (Card::from_name("Short Bow").unwrap(), &BOOSTER),
    ]);
    let player = Player::P0;
    let opponent = player.other();
    let mut raw = scenario(player);
    let dealer = put(&mut raw, player, "Hill Giant", Zone::Battlefield);
    let island = put(&mut raw, player, "Island", Zone::Battlefield);
    let mountain = put(&mut raw, player, "Mountain", Zone::Battlefield);
    let booster = put(&mut raw, player, "Short Bow", Zone::Battlefield);
    let target = put(&mut raw, opponent, "Gigantosaurus", Zone::Battlefield);
    raw.state.objects[target as usize].growth.toughness = 5;
    first_strike(&mut raw, dealer, None);
    copy_trigger(&mut raw, player, island);
    let effects = raw.observe(opponent).damage_doublers;
    assert_eq!(effects.len(), 2);
    assert_eq!(effects[0].source, effects[1].source);
    assert_ne!(effects[0].id, effects[1].id);

    let mut prevented = raw.clone();
    prevented.add_counter(target, CounterKind::Shield, 1);
    shock(&mut prevented, player, mountain, target);
    pass_priority(&mut prevented);
    assert_eq!(prevented.actor(), Some(opponent));
    let shield = Action::ChooseDamagePrevention {
        source: prevented.object_ref(target),
        prevention: DamagePreventionKind::ShieldCounter,
    };
    act(&mut prevented, shield);
    settle(&mut prevented);
    assert_eq!(prevented.object_view(target).damage, 0);
    assert_eq!(prevented.counter_count(target, CounterKind::Shield), 0);
    assert_eq!(prevented.observe(player).damage_doublers, effects);

    shock(&mut raw, player, mountain, target);
    pass_priority(&mut raw);
    assert_eq!(raw.actor(), Some(opponent));
    let pending = raw.observe(opponent).replacement_choice.unwrap();
    assert_eq!(pending.damage.unwrap().amount, 2);
    assert_eq!(pending.resolved_damage_effects, effects);
    let first = Action::ChooseResolvedDamageReplacement {
        effect: effects[0].id,
    };
    let second = Action::ChooseResolvedDamageReplacement {
        effect: effects[1].id,
    };
    let plus = Action::ChooseReplacement {
        source: raw.object_ref(booster),
        ability: 0,
    };
    let serialized = serde_json::to_string(&first).unwrap();
    assert_eq!(serde_json::from_str::<Action>(&serialized).unwrap(), first);
    let index = raw
        .legal_actions()
        .iter()
        .position(|action| *action == first)
        .unwrap();
    let saved = raw.snapshot();
    let revision = raw.revision();
    let mut indexed = raw.clone();
    let mut sibling = raw.clone();
    assert_eq!(raw.apply(player, first.clone()), Err(Error::WrongActor));
    assert_eq!(raw.snapshot(), saved);

    act(&mut raw, first.clone());
    indexed.apply_index(opponent, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    reject(&mut raw, first);
    let pending = raw.observe(opponent).replacement_choice.unwrap();
    assert_eq!(pending.damage.unwrap().amount, 4);
    assert_eq!(pending.resolved_damage_effects, vec![effects[1]]);
    act(&mut raw, plus.clone());
    act(&mut indexed, plus.clone());
    settle(&mut raw);
    settle(&mut indexed);
    assert_eq!(raw.object_view(target).damage, 10);
    assert_eq!(raw.snapshot(), indexed.snapshot());

    act(&mut sibling, plus.clone());
    act(&mut sibling, second.clone());
    settle(&mut sibling);
    assert_eq!(sibling.object_view(target).damage, 12);
    indexed.restore(&saved).unwrap();
    act(&mut indexed, plus);
    act(&mut indexed, second);
    settle(&mut indexed);
    assert_eq!(indexed.snapshot(), sibling.snapshot());
}
