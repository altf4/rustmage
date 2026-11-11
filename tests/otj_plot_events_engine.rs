//! Plotted designations and exact physical source events.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const PLOT_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    extras: &CardExtras {
        plot_cost: Some(ManaCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 1,
        }),
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfBecomesPlotted,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 2)],
    }],
    ..CardRules::EMPTY
};

#[test]
fn special_action_and_hand_effect_plot_the_current_exile_incarnation_once() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &PLOT_OBSERVER)]);
    for through_effect in [false, true] {
        let player = Player::P0;
        let mut game = scenario(player);
        let creature = put(&mut game, player, "Grizzly Bears", Zone::Hand);
        let hand = game.object_ref(creature);
        if through_effect {
            let jace = put(&mut game, player, "Jace Reawakened", Zone::Battlefield);
            game.add_counter(jace, CounterKind::Loyalty, 3);
            act(
                &mut game,
                Action::AnnounceActivate {
                    card: jace,
                    ability: 1,
                    counter_payment: Counters::default(),
                },
            );
            pay(&mut game);
            for _ in 0..8 {
                if matches!(game.state.pending, Pending::EffectChoice { .. }) {
                    break;
                }
                act(&mut game, Action::Pass);
            }
            act(&mut game, Action::SelectResolutionExile { object: hand });
        } else {
            act(&mut game, Action::AnnouncePlot { card: creature });
            assert_eq!(game.object_ref(creature), hand);
            assert!(game.observe(player.other()).plot_payment.is_none());
            choose(&mut game, |action| matches!(action, Action::PayPlot { .. }));
        }
        let exile = game.object_ref(creature);
        assert_ne!(exile, hand);
        assert_eq!(game.object_view(creature).zone, Zone::Exile);
        assert_eq!(game.observe(player.other()).plotted_cards[0].object, exile);
        assert_eq!(game.state.stack.len(), 1);
        assert_eq!(game.state.stack[0].source.unwrap().object, exile);
        game.remember_plotted(player, exile);
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, 22);
        assert_eq!(game.state.players[player.index()].spells_cast_this_turn, 0);
        reject(&mut game, Action::AnnouncePlotted { card: creature });

        game.move_object(creature, Zone::Graveyard);
        game.move_object(creature, Zone::Exile);
        game.state.turn += 1;
        reject(&mut game, Action::AnnouncePlotted { card: creature });
    }
}

#[test]
fn redirected_plot_exile_does_not_designate_or_trigger() {
    struct ResetRedirect;
    impl Drop for ResetRedirect {
        fn drop(&mut self) {
            zone_replacements::TEST_REDIRECT_EXILE_TO_BATTLEFIELD.with(|row| row.set(None));
        }
    }

    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &PLOT_OBSERVER)]);
    let _reset = ResetRedirect;
    let player = Player::P1;
    let mut game = scenario(player);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    zone_replacements::TEST_REDIRECT_EXILE_TO_BATTLEFIELD
        .with(|row| row.set(Some(game.object_ref(creature))));
    act(&mut game, Action::AnnouncePlot { card: creature });
    choose(&mut game, |action| matches!(action, Action::PayPlot { .. }));
    settle(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Battlefield);
    assert!(game.observe(player).plotted_cards.is_empty());
    assert_eq!(game.state.players[player.index()].life, 20);
}
