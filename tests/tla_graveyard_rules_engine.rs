//! Live graveyard cast permission and exact draw observers.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const SERPENT: CardRules = CardRules {
    effect: Effect::Creature(6, 5),
    extras: &CardExtras {
        self_cast_timing: Some(SelfCastTiming::AsThoughFlashWithGraveyardSubtype {
            subtype: "Lesson",
            minimum: 3,
        }),
        cost_reductions: &[SpellReduction {
            source_only: true,
            filter: CastFilter::Any,
            amount: SpellReductionAmount::Stat {
                value: StatValue::Count(CountFilter {
                    zone: CountZone::Graveyard,
                    relation: 0,
                    kind: 0,
                    exclude: 1 | 2,
                    subtype: None,
                    basic: false,
                    other: false,
                    cards_only: true,
                }),
                factor: 1,
            },
            turn: CostTurn::Any,
            guard: StepGuard::Always,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const BAT: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InZone {
            zone: TargetZone::Graveyard,
            event: &TriggerCondition::CardDrawn {
                relation: CastRelation::You,
                ordinal: Some(2),
            },
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayPayMana {
                cost: ManaCost {
                    colored: [0, 0, 1, 0, 0, 0],
                    generic: 0,
                },
                skip: 1,
            },
            Instruction::ReturnSourceToBattlefieldWithCounter {
                tapped: false,
                counter: CounterKind::Finality,
            },
        ],
    }],
    ..CardRules::EMPTY
};

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn own_graveyard_lessons_allow_casting_off_turn_without_granting_flash() {
    let serpent = Card::from_name("Gearseeker Serpent").unwrap();
    let _rules = fixture_rules::activate(&[(serpent, &SERPENT)]);
    for actor in [Player::P0, Player::P1] {
        let mut game = scenario(actor.other());
        act(&mut game, Action::Pass);
        let spell = spawn(&mut game, actor, serpent, Zone::Hand);
        let third = put(&mut game, actor, "Firebending Lesson", Zone::Graveyard);
        for _ in 0..2 {
            put(&mut game, actor, "Firebending Lesson", Zone::Graveyard);
        }
        put(&mut game, actor, "Short Bow", Zone::Graveyard);
        put(&mut game, actor, "Forest", Zone::Graveyard);
        put(&mut game, actor, "Goblin Piker", Zone::Graveyard);
        put(
            &mut game,
            actor.other(),
            "Firebending Lesson",
            Zone::Graveyard,
        );
        game.state.players[actor.index()].mana = [0, 2, 0, 0, 0, 1];
        let announce = Action::AnnounceCast { card: spell };
        let mut below_threshold = game.clone();
        below_threshold.move_object(third, Zone::Exile);
        put(&mut below_threshold, actor, "Short Bow", Zone::Graveyard);
        reject(&mut below_threshold, announce.clone());

        let mut missing_color = game.clone();
        missing_color.state.players[actor.index()].mana = [0, 1, 0, 0, 0, 9];
        reject(&mut missing_color, announce.clone());
        act(&mut game, announce);
        assert!(!game.keywords(spell).contains(Keywords::FLASH));
        pay(&mut game);
        assert_eq!(
            game.state.stack.last().unwrap().cast.unwrap().payment,
            [0, 2, 0, 0, 0, 1]
        );
        assert!(!game.keywords(spell).contains(Keywords::FLASH));
        settle(&mut game);
        assert_eq!(game.object_view(spell).zone, Zone::Battlefield);
    }
}

#[test]
fn graveyard_draw_observer_owns_its_incarnation_through_payment_and_finality() {
    let _rules = fixture_rules::activate(&[(Card::GoblinPiker, &BAT)]);
    for actor in [Player::P0, Player::P1] {
        let mut game = scenario(actor.other());
        act(&mut game, Action::Pass);
        let bat = put(&mut game, actor, "Goblin Piker", Zone::Graveyard);
        let original = game.object_ref(bat);
        let enemy = put(&mut game, actor.other(), "Goblin Piker", Zone::Graveyard);
        put(&mut game, actor, "Goblin Piker", Zone::Battlefield);
        put(&mut game, actor, "Goblin Piker", Zone::Hand);
        let draw = put(&mut game, actor, "Quick Study", Zone::Hand);
        cast(&mut game, draw, None);
        pass_pair(&mut game);
        assert_eq!(game.state.players[actor.index()].cards_drawn_this_turn, 2);
        assert_eq!(game.state.stack.len(), 1);
        let packet = &game.state.stack[0];
        assert_eq!(packet.source().unwrap().object, original);
        assert_eq!(packet.draw_event.unwrap().ordinal, 2);

        let mut stale = game.clone();
        stale.move_object(bat, Zone::Exile);
        stale.move_object(bat, Zone::Graveyard);
        assert_ne!(stale.object_ref(bat), original);
        pass_pair(&mut stale);
        let pay_black = Action::PayResolutionMana {
            payment: [0, 0, 1, 0, 0, 0],
        };
        act(&mut stale, pay_black.clone());
        settle(&mut stale);
        assert_eq!(stale.object_view(bat).zone, Zone::Graveyard);

        pass_pair(&mut game);
        assert_eq!(game.actor(), Some(actor));
        let checkpoint = game.snapshot();
        let mut indexed = game.clone();
        let index = indexed
            .legal_actions()
            .iter()
            .position(|action| *action == pay_black)
            .unwrap();
        indexed
            .apply_index(actor, indexed.revision(), index)
            .unwrap();
        act(&mut game, pay_black);
        settle(&mut game);
        settle(&mut indexed);
        assert_eq!(game.snapshot(), indexed.snapshot());
        assert_eq!(game.object_view(bat).zone, Zone::Battlefield);
        assert_eq!(game.object_view(bat).controller, actor);
        assert_eq!(game.counter_count(bat, CounterKind::Finality), 1);
        assert_eq!(game.object_view(enemy).zone, Zone::Graveyard);

        indexed.restore(&checkpoint).unwrap();
        act(&mut indexed, Action::DeclineResolutionMana);
        settle(&mut indexed);
        assert_eq!(indexed.object_ref(bat), original);
        game.move_object(bat, Zone::Graveyard);
        assert_eq!(game.object_view(bat).zone, Zone::Exile);
    }
}
