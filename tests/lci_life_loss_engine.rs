//! Life-loss replacements preserve damage, costs and owned mana receipts.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const DOUBLE_LOSS: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::OpponentLifeLossDoubleOnOwnTurn],
    }],
    ..CardRules::EMPTY
};
const LOSS: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: false,
        })),
        &[Instruction::LoseLife(PlayerSet::Targeted, 2)],
    ),
    ..CardRules::EMPTY
};
const LIFELINK: CardRules = CardRules {
    keywords: Keywords::LIFELINK,
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Any)),
        &[Instruction::Damage(2)],
    ),
    ..CardRules::EMPTY
};
const LIFE_MANA: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            tap: true,
            life: 2,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Fixed([
            0, 1, 0, 0, 0, 0,
        ]))],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

#[test]
fn life_loss_doubles_by_live_controller_and_turn_while_damage_and_lifelink_keep_their_amounts() {
    let _rules = fixture_rules::activate(&[
        (Card::HillGiant, &DOUBLE_LOSS),
        (named("Shock"), &LIFELINK),
        (named("Opt"), &LOSS),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let provider = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let shock = put(&mut game, player, "Shock", Zone::Hand);
        let loss = put(&mut game, player, "Opt", Zone::Hand);

        cast(&mut game, shock, Some(Target::Player(player.other())));
        settle(&mut game);
        assert_eq!(
            game.state.players[player.other().index()].life,
            STARTING_LIFE - 4
        );
        assert_eq!(game.state.players[player.index()].life, STARTING_LIFE + 2);
        assert_eq!(
            game.state.players[player.other().index()].life_lost_this_turn,
            4
        );
        cast(&mut game, loss, Some(Target::Player(player)));
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, STARTING_LIFE);

        game.record_control(game.object_ref(provider), player.other(), None);
        let second = put(&mut game, player, "Opt", Zone::Hand);
        cast(&mut game, second, Some(Target::Player(player)));
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, STARTING_LIFE - 2);
    }
}

#[test]
fn multiple_providers_stack_and_ability_loss_stops_the_replacement() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &DOUBLE_LOSS), (named("Opt"), &LOSS)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let first = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let loss = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, loss, Some(Target::Player(player.other())));
    settle(&mut game);
    assert_eq!(
        game.state.players[player.other().index()].life,
        STARTING_LIFE - 8
    );

    let protection = put(&mut game, player, "Witness Protection", Zone::Hand);
    cast(&mut game, protection, Some(Target::Card(first)));
    settle(&mut game);
    let loss = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, loss, Some(Target::Player(player.other())));
    settle(&mut game);
    assert_eq!(
        game.state.players[player.other().index()].life,
        STARTING_LIFE - 12
    );
}

#[test]
fn simultaneous_loss_and_life_exchange_report_the_replaced_amounts() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &DOUBLE_LOSS)]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Hill Giant", Zone::Battlefield);
    game.lose_life_simultaneously([1, 1]);
    assert_eq!(game.state.players[player.index()].life, STARTING_LIFE - 1);
    assert_eq!(
        game.state.players[player.other().index()].life,
        STARTING_LIFE - 2
    );

    game.state.players[player.index()].life = 10;
    game.state.players[player.other().index()].life = 15;
    game.exchange_life_totals(player, player.other(), false);
    assert_eq!(game.state.players[player.index()].life, 15);
    assert_eq!(game.state.players[player.other().index()].life, 5);
    assert_eq!(
        game.state.players[player.other().index()].life_lost_this_turn,
        12
    );
}

#[test]
fn projected_mana_accounts_for_replaced_life_costs_and_raw_index_restore() {
    let _rules = fixture_rules::activate(&[
        (Card::HillGiant, &DOUBLE_LOSS),
        (named("Nutrient Block"), &LIFE_MANA),
    ]);
    for active in [Player::P0, Player::P1] {
        let player = active.other();
        let mut game = scenario(active);
        game.state.priority = player;
        game.state.players[player.index()].mana = [0; 6];
        game.state.players[player.index()].life = 5;
        put(&mut game, active, "Hill Giant", Zone::Battlefield);
        let first = put(&mut game, player, "Nutrient Block", Zone::Battlefield);
        put(&mut game, player, "Nutrient Block", Zone::Battlefield);
        let negate = put(&mut game, player, "Negate", Zone::Hand);
        let target = put(&mut game, active, "Shock", Zone::Hand);
        game.state.priority = active;
        cast(&mut game, target, Some(Target::Player(player)));
        act(&mut game, Action::Pass);
        assert_eq!(game.actor(), Some(player));
        let announce = Action::AnnounceCast { card: negate };
        reject(&mut game, announce.clone());

        game.state.players[player.index()].life = 9;
        act(&mut game, announce);
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Spell(target),
            },
        );
        let activation = Action::ActivateMana {
            card: first,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: [0, 1, 0, 0, 0, 0],
        };
        let saved = game.snapshot();
        let revision = game.revision();
        let index = game
            .legal_actions()
            .iter()
            .position(|action| *action == activation)
            .unwrap();
        let mut raw = game.clone();
        let mut indexed = game.clone();
        drop(game);
        act(&mut raw, activation.clone());
        indexed.apply_index(player, revision, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.state.players[player.index()].life, 5);
        assert_eq!(raw.state.players[player.index()].life_lost_this_turn, 4);
        indexed.restore(&saved).unwrap();
        act(&mut indexed, activation);
        assert_eq!(raw.snapshot(), indexed.snapshot());
    }
}
