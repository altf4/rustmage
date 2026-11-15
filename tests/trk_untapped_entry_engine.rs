//! Entry-time untapped trigger for the Ni'Var, Q Continuum, and Quark's Bar land cycle.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const LAND: CardRules = CardRules {
    effect: Effect::Land,
    entry: EntryTapped::UnlessAtLeastTwo,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersUntapped,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

const UNTAP_LANDS: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::ControlledEntryUntapped],
    }],
    extras: &CardExtras {
        controlled_entry_untapped: 2,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn rules() -> fixture_rules::Guard {
    fixture_rules::activate(&[(Card::Forest, &LAND), (Card::GrizzlyBears, &UNTAP_LANDS)])
}

#[test]
fn trigger_uses_tapped_status_at_entry_even_when_land_changes_later() {
    let _rules = rules();
    for player in [Player::P0, Player::P1] {
        for other_lands in [0, 1, 2] {
            let mut game = scenario(player);
            for _ in 0..other_lands {
                put(&mut game, player, "Forest", Zone::Battlefield);
            }
            let land = put(&mut game, player, "Forest", Zone::Hand);
            let starting_life = game.state.players[player.index()].life;
            act(&mut game, Action::PlayLand { card: land });
            assert_eq!(game.object_view(land).tapped, other_lands < 2);
            if other_lands == 2 {
                assert_eq!(game.state.stack.len(), 1);
                game.tap_objects(&[land]);
                assert!(game.object_view(land).tapped);
                settle(&mut game);
                assert_eq!(game.state.players[player.index()].life, starting_life + 1);
            } else {
                assert!(game.state.stack.is_empty());
                game.untap_objects(&[land], untap_rules::UntapCause::Effect);
                game.prepare_triggers();
                assert!(game.state.stack.is_empty());
                assert_eq!(game.state.players[player.index()].life, starting_life);
            }
        }
    }
}

#[test]
fn competing_tapped_and_untapped_replacements_choose_the_entry_trigger_outcome() {
    let _rules = rules();
    let player = Player::P0;
    let mut parent = scenario(player);
    let provider = put(&mut parent, player, "Grizzly Bears", Zone::Battlefield);
    let land = put(&mut parent, player, "Forest", Zone::Hand);
    act(&mut parent, Action::PlayLand { card: land });
    assert_eq!(parent.state.pending, Pending::EntryReplacement { player });
    let tapped_first = Action::ChooseEntryReplacement {
        source: parent.object_ref(land),
        intrinsic: true,
    };
    let untapped_first = Action::ChooseEntryReplacement {
        source: parent.object_ref(provider),
        intrinsic: false,
    };
    let legal = parent.legal_actions();
    assert!(legal.contains(&tapped_first) && legal.contains(&untapped_first));
    let checkpoint = parent.snapshot();
    let index = legal
        .iter()
        .position(|action| *action == tapped_first)
        .unwrap();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    act(&mut raw, tapped_first);
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert!(!raw.object_view(land).tapped);
    assert_eq!(raw.state.stack.len(), 1);
    settle(&mut raw);
    assert_eq!(raw.state.players[player.index()].life, 21);

    indexed.restore(&checkpoint).unwrap();
    act(&mut indexed, untapped_first);
    assert!(indexed.object_view(land).tapped);
    assert!(indexed.state.stack.is_empty());
    assert_eq!(indexed.state.players[player.index()].life, 20);
}
