//! Shared exact rules needed by Star Trek E–M cards.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const SACRIFICE_CREATURE: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 1,
    nonland: true,
    other: true,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};
const GOWRON_CONDITION: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::Sacrifice,
                filter: SACRIFICE_CREATURE,
            }),
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::IfSacrificedKlingon { skip: 2 },
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::LoseLife(PlayerSet::Controller, 1),
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const MILES_HISTORY: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::Any,
            guard: StepGuard::PermanentReturnedToYourHandThisTurn,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

fn drain(game: &mut Game) {
    for _ in 0..48 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.resolving.is_none()
        {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("resolution budget exhausted");
}

#[test]
fn sacrificed_klingon_uses_actual_predeparture_subtype_and_owns_index_restore() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &GOWRON_CONDITION)]);
    for (sacrifice, qualifies) in [("General Chang, Cold Warrior", true), ("Bear Cub", false)] {
        let player = Player::P0;
        let mut game = scenario(player);
        let source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let material = if qualifies {
            spawn(
                &mut game,
                player,
                Card::from_name(sacrifice).unwrap(),
                Zone::Battlefield,
            )
        } else {
            put(&mut game, player, sacrifice, Zone::Battlefield)
        };
        put(&mut game, player, "Island", Zone::Library);
        act(
            &mut game,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let action = Action::ChooseAdditionalCost {
            object: game.object_ref(material),
        };
        let checkpoint = game.snapshot();
        let revision = game.revision();
        let index = game
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        let mut indexed = game.clone();
        indexed.apply_index(player, revision, index).unwrap();
        act(&mut game, action);
        assert_eq!(game.snapshot(), indexed.snapshot());
        indexed.restore(&checkpoint).unwrap();
        assert_eq!(
            indexed.apply_index(player, revision, index),
            Err(Error::StaleRevision)
        );
        pay(&mut game);
        drain(&mut game);
        assert_eq!(game.object_view(material).zone, Zone::Graveyard);
        assert_eq!(
            game.state.players[player.index()].cards_drawn_this_turn,
            u64::from(qualifies)
        );
        assert_eq!(
            game.state.players[player.index()].life,
            20 - i128::from(qualifies)
        );
    }
}

#[test]
fn returned_land_counts_for_owner_but_unrelated_departure_does_not() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &MILES_HISTORY)]);
    for returned in [false, true] {
        let player = Player::P0;
        let mut game = scenario(player);
        put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let permanent = put(&mut game, player, "Forest", Zone::Battlefield);
        put(&mut game, player, "Island", Zone::Library);
        game.move_object(
            permanent,
            if returned {
                Zone::Hand
            } else {
                Zone::Graveyard
            },
        );
        assert_eq!(
            game.state.players[player.index()]
                .turn_facts
                .permanent_returned_to_your_hand,
            returned
        );
        game.state.step = Step::Main2;
        game.give_priority();
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        drain(&mut game);
        assert_eq!(
            game.state.players[player.index()].cards_drawn_this_turn,
            u64::from(returned)
        );
    }
}
