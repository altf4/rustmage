//! Damage uses the ability source independently of a sampled entrant.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const POWER_DAMAGE: CardRules = CardRules {
    effect: Effect::Creature(5, 4),
    keywords: Keywords::LIFELINK,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::LIFELINK)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OtherCreatureEntersYou,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Any)),
        program: &[Instruction::DamageByEnteringPower],
    }],
    ..CardRules::EMPTY
};

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn the_ability_source_deals_damage_using_live_or_last_known_entrant_power() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &POWER_DAMAGE)]);
    let player = Player::P0;
    let enemy = player.other();
    let mut parent = scenario(player);
    put(&mut parent, player, "Grizzly Bears", Zone::Battlefield);
    let entrant = put(&mut parent, player, "Savannah Lions", Zone::Hand);
    let growth = put(&mut parent, player, "Giant Growth", Zone::Hand);
    let murder = put(&mut parent, player, "Murder", Zone::Hand);

    cast(&mut parent, entrant, None);
    pass_pair(&mut parent);
    act(
        &mut parent,
        Action::ChooseTarget {
            target: Target::Player(enemy),
        },
    );
    assert_eq!(parent.state.stack.len(), 1);
    cast(&mut parent, growth, Some(Target::Card(entrant)));
    pass_pair(&mut parent);
    assert_eq!(stats(&parent, entrant), (5, 4));

    let before = parent.state.players[player.index()].life;
    let opposing_before = parent.state.players[enemy.index()].life;
    for depart in [false, true] {
        let mut game = parent.clone();
        if depart {
            cast(&mut game, murder, Some(Target::Card(entrant)));
            pass_pair(&mut game);
            assert_eq!(game.object_view(entrant).zone, Zone::Graveyard);
        }
        settle(&mut game);
        assert_eq!(game.state.players[enemy.index()].life, opposing_before - 5);
        assert_eq!(game.state.players[player.index()].life, before + 5);
    }
}

const COMBAT_DRAW: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayerDamaged {
            subject: DamageSubject::ControlledLegendaryCreatures,
            combat_only: true,
            opponent_only: false,
            one_or_more: false,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

#[test]
fn each_legendary_creature_that_deals_combat_damage_to_a_player_triggers() {
    let provider = Card::from_name("The Wind Crystal").unwrap();
    let _rules = fixture_rules::activate(&[(provider, &COMBAT_DRAW)]);
    let player = Player::P0;
    let mut game = scenario(player);
    spawn(&mut game, player, provider, Zone::Battlefield);
    let first = put(
        &mut game,
        player,
        "Wylie Duke, Atiin Hero",
        Zone::Battlefield,
    );
    let second = put(
        &mut game,
        player,
        "Kolodin, Triumph Caster",
        Zone::Battlefield,
    );
    let nonlegend = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    game.apply_damage_batch(&[DamageEvent {
        preventable: true,
        source: game.damage_source(first),
        target: Target::Player(player.other()),
        amount: 1,
        combat: false,
    }]);
    assert!(game.state.pending_triggers.is_empty());

    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    for card in [first, second, nonlegend] {
        act(&mut game, Action::Attack { card });
    }
    act(&mut game, Action::FinishAttackers);
    pass_pair(&mut game);
    act(&mut game, Action::FinishBlockers);
    pass_pair(&mut game);
    while matches!(game.state.pending, Pending::OrderTriggers { .. }) {
        choose(&mut game, |action| {
            matches!(action, Action::PutTrigger { .. })
        });
    }
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
}
