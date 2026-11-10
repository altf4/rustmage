//! Bounded numeric choices, committed spell characteristics and live activation discounts.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const SWEEPER: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::ChooseNumberDestroyPowerAtLeast {
            minimum: 0,
            maximum: 10,
        }],
    ),
    ..CardRules::EMPTY
};

const OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::NumberRange {
            minimum: 1,
            maximum: 10,
        }),
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::Opponent,
            filter: CastFilter::SourceChosenNumberMatchesSpell,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::LoseLife(PlayerSet::Opponent, 2),
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    }],
    ..CardRules::EMPTY
};

const X_SPELL: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, &[]),
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

const OWN_TURN: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        activation_generic_reductions: &[(
            0,
            ActivationGenericReduction::Stat {
                value: StatValue::ControllerTurn,
                multiplier: 1,
            },
        )],
        ..CardExtras::EMPTY
    },
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        program: &[Instruction::Tap],
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

#[test]
fn power_threshold_uses_live_power_on_resolution_and_supports_owned_choices() {
    let _scope = fixture_rules::activate(&[(Card::Divination, &SWEEPER)]);
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let negative = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let zero = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let nine = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        let ten = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        let grows = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        for (id, modifier) in [(negative, -3), (zero, -2), (nine, 7), (ten, 8), (grows, 6)] {
            g.state.objects[id as usize].growth.power = modifier;
        }
        let sweeper = put(&mut g, player, "Divination", Zone::Hand);
        let pump = put(&mut g, player.other(), "Giant Growth", Zone::Hand);
        cast(&mut g, sweeper, None);
        act(&mut g, Action::Pass);
        cast(&mut g, pump, Some(Target::Card(grows)));
        for _ in 0..4 {
            act(&mut g, Action::Pass);
        }
        assert_eq!(g.object_view(grows).power, 11);
        assert_eq!(
            g.legal_actions()
                .into_iter()
                .filter(|action| *action != Action::Concede)
                .collect::<Vec<_>>(),
            (0..=10)
                .map(|value| Action::ChoosePowerThreshold { value })
                .collect::<Vec<_>>()
        );
        reject(&mut g, Action::ChoosePowerThreshold { value: 11 });
        let checkpoint = g.snapshot();
        let revision = g.revision();
        let maximum = Action::ChoosePowerThreshold { value: 10 };
        assert_eq!(
            g.apply(player.other(), maximum.clone()),
            Err(Error::WrongActor)
        );
        assert_eq!(g.snapshot(), checkpoint);
        let mut indexed = g.clone();
        let index = g
            .legal_actions()
            .iter()
            .position(|action| *action == maximum)
            .unwrap();
        act(&mut g, maximum);
        indexed.apply_index(player, revision, index).unwrap();
        settle(&mut g);
        settle(&mut indexed);
        assert_eq!(g.snapshot(), indexed.snapshot());
        for id in [negative, zero, nine] {
            assert_eq!(g.object_view(id).zone, Zone::Battlefield);
        }
        for id in [ten, grows] {
            assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        }

        indexed.restore(&checkpoint).unwrap();
        act(&mut indexed, Action::ChoosePowerThreshold { value: 0 });
        settle(&mut indexed);
        assert_eq!(indexed.object_view(negative).zone, Zone::Battlefield);
        for id in [zero, nine, ten, grows] {
            assert_eq!(indexed.object_view(id).zone, Zone::Graveyard);
        }
        assert_eq!(g.object_view(nine).zone, Zone::Battlefield);
    }
}

fn choose_number(number: u16) -> Action {
    Action::ChooseEntryValue {
        value: EntryChoiceValue::Number(number),
    }
}

#[test]
fn the_number_is_chosen_as_the_permanent_enters_and_observes_only_committed_opposing_casts() {
    let opt = Card::from_name("Opt").unwrap();
    let _scope = fixture_rules::activate(&[(Card::HillGiant, &OBSERVER), (opt, &X_SPELL)]);
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let other = player.other();
        let observer = put(&mut g, player, "Hill Giant", Zone::Hand);
        cast(&mut g, observer, None);
        act(&mut g, Action::Pass);
        act(&mut g, Action::Pass);
        assert_ne!(g.object_view(observer).zone, Zone::Battlefield);
        assert_eq!(
            g.legal_actions()
                .into_iter()
                .filter(|action| *action != Action::Concede)
                .collect::<Vec<_>>(),
            (1..=10).map(choose_number).collect::<Vec<_>>()
        );
        reject(&mut g, choose_number(0));
        reject(&mut g, choose_number(11));
        let mut chosen_two = g.clone();
        act(&mut g, choose_number(3));
        act(&mut chosen_two, choose_number(2));
        assert_eq!(
            g.object_view(observer).entry_choice,
            Some(EntryChoiceValue::Number(3))
        );

        let own = put(&mut g, player, "Divination", Zone::Hand);
        cast(&mut g, own, None);
        settle(&mut g);
        assert_eq!(g.state.players[other.index()].life, 20);
        let drawn = g.state.players[player.index()].cards_drawn_this_turn;

        // Start at the opposing main phase, using actual paid actions for every cast.
        g.state.active = other;
        g.state.priority = other;
        let variable = put(&mut g, other, "Opt", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: variable });
        act(&mut g, Action::ChooseX { value: 2 });
        assert_eq!(g.state.players[other.index()].life, 20);
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[other.index()].life, 18);

        for (name, life) in [
            ("Stingblade Assassin", 16),
            ("Barrow Naughty", 14),
            ("Bear Cub", 14),
        ] {
            let spell = put(&mut g, other, name, Zone::Hand);
            cast(&mut g, spell, None);
            settle(&mut g);
            assert_eq!(g.state.players[other.index()].life, life, "{name}");
        }
        assert_eq!(
            g.state.players[player.index()].cards_drawn_this_turn,
            drawn + 3
        );

        chosen_two.state.active = other;
        chosen_two.state.priority = other;
        let spell = put(&mut chosen_two, other, "Bear Cub", Zone::Hand);
        cast(&mut chosen_two, spell, None);
        settle(&mut chosen_two);
        // Mana value, power and toughness all match. There is one trigger.
        assert_eq!(chosen_two.state.players[other.index()].life, 18);
        assert_eq!(
            chosen_two.state.players[player.index()].cards_drawn_this_turn,
            1
        );
    }
}

#[test]
fn an_activation_discount_uses_the_sources_current_controllers_turn() {
    let crown = Card::from_name("Short Bow").unwrap();
    let _scope = fixture_rules::activate(&[(crown, &OWN_TURN)]);
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Short Bow", Zone::Battlefield);
        let target = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        let announce = Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        };
        g.state.players[player.index()].mana = [0; 6];
        let mut off_turn = g.clone();
        off_turn.state.active = player.other();
        reject(&mut off_turn, announce.clone());
        off_turn.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];
        for branch in [&mut g, &mut off_turn] {
            act(branch, announce.clone());
            act(
                branch,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            pay(branch);
            settle(branch);
            assert!(branch.object_view(source).tapped);
            assert!(branch.object_view(target).tapped);
            assert_eq!(branch.state.players[player.index()].mana, [0; 6]);
        }
    }
}
