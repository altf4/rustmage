//! Delayed step packets own separate exact batches of exiled cards.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const EXILE: &[Instruction] = &[
    Instruction::ExileTopLibraryRemember {
        players: PlayerSet::Controller,
        amount: AmountValue::ControlledCreaturePowerCount { minimum: 4 },
    },
    Instruction::ScheduleStepWithExiledResult {
        step: TriggerStep::End,
        relation: CastRelation::You,
        ability: 1,
    },
    Instruction::GrantExiledResultPlayUntilNextEndStep,
];
const HAWK: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    keywords: Keywords::FLASH,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: EXILE,
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::NONE,
            program: &[Instruction::DamagePlayersScaled(
                PlayerSet::Opponent,
                AmountValue::Multiple {
                    value: &AmountValue::DelayedExiledCardCount,
                    factor: 2,
                },
            )],
        },
    ],
    ..CardRules::EMPTY
};

fn setup() -> fixture_rules::Guard {
    fixture_rules::activate(&[(Card::from_name("Hill Giant").unwrap(), &HAWK)])
}
fn drain(g: &mut Game) {
    for _ in 0..40 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
        {
            return;
        }
        choose(g, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("delayed trigger resolution exceeded action limit");
}
fn new_hawk(g: &mut Game, p: Player) -> CardId {
    if g.actor() != Some(p) {
        act(g, Action::Pass);
    }
    let hawk = put(g, p, "Hill Giant", Zone::Hand);
    cast(g, hawk, None);
    hawk
}
fn end_step(g: &mut Game) {
    g.begin_step(Step::End);
    g.prepare_triggers();
}
fn permitted(g: &Game, p: Player) -> Vec<ObjectRef> {
    let mut rows = g.observe(p).exile_permissions;
    rows.sort_by_key(|row| row.object.card);
    rows.into_iter()
        .map(|row| {
            assert_eq!(row.player, p);
            assert_eq!(row.expiry, ExilePlayExpiry::StartOfNextEndStep(p));
            assert!(!row.cast_only);
            assert!(!row.spend_any_mana);
            row.object
        })
        .collect()
}

#[test]
fn delayed_exile_batches_use_only_their_exact_cards_and_expire_at_the_own_end_step() {
    let _rules = setup();
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p.other());
        let grower = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        put(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
        let zap = put(&mut g, p, "Shock", Zone::Library);
        let land = put(&mut g, p, "Forest", Zone::Library);
        let first = new_hawk(&mut g, p);
        act(&mut g, Action::Pass);
        act(&mut g, Action::Pass);
        // The battlefield changes while the entry trigger is on the stack.
        g.add_counter(grower, CounterKind::PlusOne, 2);
        drain(&mut g);
        let initial_land = g.object_ref(land);
        let initial_zap = g.object_ref(zap);
        assert_eq!(permitted(&g, p), [initial_zap, initial_land]);
        assert_eq!(
            g.observe(p.other()).exile_permissions,
            g.observe(p).exile_permissions
        );
        end_step(&mut g);
        assert!(g.state.stack.is_empty());
        assert_eq!(permitted(&g, p), [initial_zap, initial_land]);
        assert_eq!(g.observe(p).delayed_steps.unwrap().len(), 1);

        g.finish_cleanup();
        assert_eq!(g.state.active, p);
        for _ in 0..24 {
            if g.state.step == Step::Main1 {
                break;
            }
            act(&mut g, Action::Pass);
        }
        assert_eq!(g.state.step, Step::Main1);
        g.state.players[p.index()].mana = [8; 6];
        act(&mut g, Action::PlayLand { card: land });
        g.move_object(land, Zone::Exile);
        assert_ne!(g.object_ref(land), initial_land);
        assert_eq!(permitted(&g, p), [initial_zap]);

        let exile2 = put(&mut g, p, "Shock", Zone::Library);
        let exile3 = put(&mut g, p, "Forest", Zone::Library);
        let exile4 = put(&mut g, p, "Forest", Zone::Library);
        let second = new_hawk(&mut g, p);
        drain(&mut g);
        let first_ref = g.object_ref(first);
        let second_ref = g.object_ref(second);
        let rows = g.observe(p).delayed_steps.unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].item.source.unwrap().object, first_ref);
        assert_eq!(rows[1].item.source.unwrap().object, second_ref);
        assert_eq!(permitted(&g, p).len(), 4);
        g.move_object(first, Zone::Graveyard);
        g.move_object(second, Zone::Graveyard);
        end_step(&mut g);
        assert!(g.observe(p).exile_permissions.is_empty());
        assert!(
            g.observe(p)
                .delayed_steps
                .is_none_or(|rows| rows.is_empty())
        );

        let action = g
            .legal_actions()
            .into_iter()
            .find(|action| matches!(action, Action::PutTrigger { .. }))
            .unwrap();
        let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
        let checkpoint = g.snapshot();
        let revision = g.revision();
        let mut indexed = g.clone();
        act(&mut g, action.clone());
        indexed.apply_index(p, revision, index).unwrap();
        assert_eq!(g.snapshot(), indexed.snapshot());
        indexed.restore(&checkpoint).unwrap();
        assert_eq!(
            indexed.apply_index(p, revision, index),
            Err(Error::StaleRevision)
        );
        act(&mut indexed, action);
        // A card that leaves and returns after the delayed trigger has fired
        // is no longer the incarnation the packet is allowed to count.
        for branch in [&mut g, &mut indexed] {
            branch.move_object(exile2, Zone::Hand);
            branch.move_object(exile2, Zone::Exile);
            drain(branch);
            assert_eq!(branch.state.players[p.other().index()].life, 14);
            assert_eq!(branch.state.players[p.index()].life, 20);
            assert_eq!(branch.object_view(exile3).zone, Zone::Exile);
            assert_eq!(branch.object_view(exile4).zone, Zone::Exile);
        }
        assert_eq!(g.snapshot(), indexed.snapshot());
    }
}

#[test]
fn an_empty_batch_has_no_permission_and_its_delayed_trigger_deals_zero_damage() {
    let _rules = setup();
    let p = Player::P0;
    let mut g = scenario(p);
    let hawk = new_hawk(&mut g, p);
    act(&mut g, Action::Pass);
    act(&mut g, Action::Pass);
    g.move_object(hawk, Zone::Graveyard);
    drain(&mut g);
    assert!(permitted(&g, p).is_empty());
    assert_eq!(g.observe(p).delayed_steps.unwrap().len(), 1);
    end_step(&mut g);
    assert_eq!(g.state.stack.len(), 1);
    drain(&mut g);
    assert_eq!(g.state.players[p.other().index()].life, 20);
}
