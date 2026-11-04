//! Shared rounded-up loss; the existing rounded-down instruction remains distinct.
//! Typed fixtures establish the engine dependency without admitting Betor.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const UP_OPPONENT: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::LoseHalfLifeRoundedUp(PlayerSet::Opponent)],
    ),
    ..CardRules::EMPTY
};
const UP_CONTROLLER: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::LoseHalfLifeRoundedUp(PlayerSet::Controller)],
    ),
    ..CardRules::EMPTY
};
const UP_EACH: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::LoseHalfLifeRoundedUp(PlayerSet::Each)],
    ),
    ..CardRules::EMPTY
};
const DOWN_EACH: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::LoseHalfLife(PlayerSet::Each)],
    ),
    ..CardRules::EMPTY
};
fn drain(g: &mut Game) {
    for _ in 0..16 {
        if g.outcome().is_some()
            || (g.state.pending == Pending::Priority && g.state.stack.is_empty())
        {
            return;
        }
        act(g, Action::Pass);
    }
    panic!("16-action arithmetic resolution budget");
}
fn spell(g: &mut Game, p: Player) {
    let id = put(g, p, "Shock", Zone::Hand);
    cast(g, id, None);
}
#[test]
fn round_up_opponent_uses_exact_odd_even_one_and_maximum_life_without_overflow() {
    let _scope = fixture_rules::activate(&[(Card::Shock, &UP_OPPONENT)]);
    for p in [Player::P0, Player::P1] {
        for (initial, expected) in [
            (21, 10),
            (20, 10),
            (3, 1),
            (2, 1),
            (1, 0),
            (i128::MAX, i128::MAX / 2),
        ] {
            let mut g = scenario(p);
            g.state.players[p.other().index()].life = initial;
            spell(&mut g, p);
            drain(&mut g);
            assert_eq!(g.state.players[p.index()].life, 20);
            assert_eq!(g.state.players[p.other().index()].life, expected);
            assert_eq!(
                g.state.players[p.other().index()].life_lost_this_turn,
                (initial - expected) as u128
            );
            assert_eq!(g.outcome(), (initial == 1).then_some(Outcome::Winner(p)));
        }
    }
}
#[test]
fn live_player_sets_and_rounding_are_distinct_without_changing_old_floor_behavior() {
    for (rules, expected) in [
        (&UP_CONTROLLER, [10, 3]),
        (&UP_EACH, [10, 1]),
        (&DOWN_EACH, [11, 2]),
    ] {
        let _scope = fixture_rules::activate(&[(Card::Shock, rules)]);
        for p in [Player::P0, Player::P1] {
            let mut g = scenario(p);
            g.state.players[p.index()].life = 21;
            g.state.players[p.other().index()].life = 3;
            spell(&mut g, p);
            drain(&mut g);
            assert_eq!(g.state.players[p.index()].life, expected[0]);
            assert_eq!(g.state.players[p.other().index()].life, expected[1]);
        }
    }
}
#[test]
fn zero_and_negative_calculations_lose_zero_without_creating_life_loss_events() {
    let _scope = fixture_rules::activate(&[(Card::Shock, &UP_OPPONENT)]);
    for p in [Player::P0, Player::P1] {
        for initial in [0, -1, i128::MIN] {
            let mut g = scenario(p);
            spell(&mut g, p);
            // Direct resolution is intentional: ordinary SBAs would already end
            // a game with this setup before the printed instruction executes.
            g.state.players[p.other().index()].life = initial;
            g.resolve_top();
            assert_eq!(g.state.players[p.other().index()].life, initial);
            assert_eq!(g.state.players[p.other().index()].life_lost_this_turn, 0);
            assert_eq!(g.state.players[p.index()].life, 20);
        }
    }
}
#[test]
fn amount_uses_resolution_time_life_and_packet_controller_without_targeting() {
    let _scope = fixture_rules::activate(&[(Card::Shock, &UP_OPPONENT)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        spell(&mut g, p);
        g.state.stack.last_mut().unwrap().controller = p.other();
        g.state.players[p.index()].life = 23;
        g.state.players[p.index()].hexproof_until_end_turn = true;
        drain(&mut g);
        assert_eq!(g.state.players[p.index()].life, 11);
        assert_eq!(g.state.players[p.other().index()].life, 20);
    }
}
#[test]
fn pending_arithmetic_is_owned_raw_index_restorable_and_failed_actions_are_atomic() {
    let _scope = fixture_rules::activate(&[(Card::Shock, &UP_OPPONENT)]);
    for p in [Player::P0, Player::P1] {
        let mut parent = scenario(p);
        parent.state.players[p.other().index()].life = 21;
        spell(&mut parent, p);
        act(&mut parent, Action::Pass);
        let actor = parent.actor().unwrap();
        let index = parent
            .legal_actions()
            .iter()
            .position(|a| *a == Action::Pass)
            .unwrap();
        let revision = parent.revision();
        let saved = parent.snapshot();
        let mut raw = parent.clone();
        let mut indexed = parent.clone();
        let sibling = parent.clone();
        drop(parent);
        assert_eq!(
            raw.apply(actor.other(), Action::Pass),
            Err(Error::WrongActor)
        );
        assert_eq!(raw.snapshot(), saved);
        reject(
            &mut raw,
            Action::ChooseTarget {
                target: Target::Player(p),
            },
        );
        act(&mut raw, Action::Pass);
        indexed.apply_index(actor, revision, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.state.players[p.other().index()].life, 10);
        assert_eq!(sibling.snapshot(), saved);
        for viewer in [Player::P0, Player::P1] {
            assert_eq!(
                serde_json::to_value(raw.observe(viewer)).unwrap(),
                serde_json::to_value(indexed.observe(viewer)).unwrap()
            );
        }
        indexed.restore(&saved).unwrap();
        assert_eq!(
            indexed.apply_index(actor, revision, index),
            Err(Error::StaleRevision)
        );
        act(&mut indexed, Action::Pass);
        assert_eq!(raw.snapshot(), indexed.snapshot());
    }
}
