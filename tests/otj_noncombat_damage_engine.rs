//! Paid X damage replacements and exact toughness damage events.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const DAMAGE_RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ControlledNoncombatDamageMatchesCreatureToughness,
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            tap: true,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ActivationX,
            Instruction::IncreaseControlledNoncombatDamageThisTurn,
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

fn activate_damage_increase(game: &mut Game, source: CardId, value: u32) {
    act(
        game,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(game, Action::ChooseX { value });
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
    settle(game);
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn shock(game: &mut Game, player: Player, target: Target) {
    let spell = put(game, player, "Shock", Zone::Hand);
    cast(game, spell, Some(target));
}

#[test]
fn a_replaced_noncombat_hit_compares_to_toughness_and_ignores_marked_damage() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &DAMAGE_RULES)]);
    let player = Player::P0;
    let enemy = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let equal = put(&mut game, enemy, "Hill Giant", Zone::Battlefield);
    let smaller = put(&mut game, enemy, "Savannah Lions", Zone::Battlefield);
    let shielded = put(&mut game, enemy, "Hill Giant", Zone::Battlefield);
    game.state.objects[equal as usize].damage = 1;
    game.add_counter(shielded, CounterKind::Shield, 1);
    activate_damage_increase(&mut game, source, 1);

    shock(&mut game, player, Target::Card(equal));
    settle(&mut game);
    assert_eq!(game.object_view(equal).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);

    shock(&mut game, player, Target::Card(smaller));
    settle(&mut game);
    assert_eq!(game.object_view(smaller).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);

    shock(&mut game, player, Target::Card(shielded));
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(enemy));
    let shield = game.object_ref(shielded);
    act(
        &mut game,
        Action::ChooseDamagePrevention {
            source: shield,
            prevention: DamagePreventionKind::ShieldCounter,
        },
    );
    settle(&mut game);
    assert_eq!(game.object_view(shielded).damage, 0);
    assert_eq!(game.counter_count(shielded, CounterKind::Shield), 0);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

#[test]
fn additive_damage_replacement_is_owned_and_ordered_with_a_doubler() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &DAMAGE_RULES)]);
    let player = Player::P1;
    let enemy = player.other();
    let mut parent = scenario(player);
    let source = put(&mut parent, player, "Grizzly Bears", Zone::Battlefield);
    let source_ref = parent.object_ref(source);
    let doubler = put(&mut parent, player, "Twinflame Tyrant", Zone::Battlefield);
    activate_damage_increase(&mut parent, source, 1);
    let bounce = put(&mut parent, player, "Unsummon", Zone::Hand);
    cast(&mut parent, bounce, Some(Target::Card(source)));
    settle(&mut parent);
    assert_eq!(parent.object_view(source).zone, Zone::Hand);

    let life = parent.state.players[enemy.index()].life;
    shock(&mut parent, player, Target::Player(enemy));
    pass_pair(&mut parent);
    assert_eq!(parent.actor(), Some(enemy));
    let choice = parent.observe(enemy).replacement_choice.unwrap();
    let increase = choice.noncombat_damage_increases[0];
    assert_eq!(increase.source, source_ref);
    assert_eq!(increase.controller, player);
    assert_eq!(increase.amount, 1);
    assert_eq!(choice.damage.unwrap().amount, 2);

    let add = Action::ChooseResolvedDamageReplacement {
        effect: increase.id,
    };
    let double = Action::ChooseReplacement {
        source: parent.object_ref(doubler),
        ability: 0,
    };
    let index = parent
        .legal_actions()
        .iter()
        .position(|action| *action == add)
        .unwrap();
    let saved = parent.snapshot();
    let revision = parent.revision();
    let mut raw = parent.clone();
    let mut indexed = parent;
    act(&mut raw, add);
    indexed.apply_index(enemy, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.state.players[enemy.index()].life, life - 6);

    indexed.restore(&saved).unwrap();
    act(&mut indexed, double);
    assert_eq!(indexed.state.players[enemy.index()].life, life - 5);

    let turn = raw.state.turn;
    for _ in 0..32 {
        if raw.state.turn != turn {
            break;
        }
        choose(&mut raw, |action| {
            matches!(
                action,
                Action::Pass | Action::FinishAttackers | Action::FinishBlockers
            )
        });
    }
    assert_ne!(raw.state.turn, turn);
    assert!(raw.observe(player).noncombat_damage_increases.is_empty());
}
