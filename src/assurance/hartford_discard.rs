//! Related discard consumers and sacrifice-scaled attack rewards.
use super::big_helpers::{cast, checked, settle, setup};
use super::*;

fn reach(g: &mut Game, wanted: impl Fn(&Game) -> bool) {
    for _ in 0..50 {
        if wanted(g) {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
            .unwrap();
        checked(g, action);
    }
    panic!("missing expected decision {:?}", g.state.pending);
}
fn attack(g: &mut Game, card: CardId) {
    g.state.objects[card as usize].summoning_sick = false;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    checked(g, Action::Attack { card });
    checked(g, Action::FinishAttackers);
}
fn token_count(g: &Game, p: Player, name: &str) -> usize {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(id, o)| {
            o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && g.has_subtype(*id as CardId, name)
        })
        .count()
}

#[test]
fn inti_attack_discard_creates_a_later_targeted_reflexive_trigger() {
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            let mut g = setup(p);
            put(&mut g, p, "Inti, Seneschal of the Sun", Zone::Battlefield);
            let attacker = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let idle = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let card = put(&mut g, p, "Mountain", Zone::Hand);
            attack(&mut g, attacker);
            reach(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectChoice { .. })
            });
            assert!(g.observe(p.other()).effect_choice.is_none());
            assert!(
                !g.legal_actions()
                    .iter()
                    .any(|a| matches!(a, Action::ChooseTarget { .. }))
            );
            if pay {
                checked(&mut g, Action::Discard { card });
                reach(&mut g, |g| {
                    matches!(g.state.pending, Pending::Targets { .. })
                });
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(idle),
                    },
                );
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(attacker),
                    },
                );
            } else {
                checked(&mut g, Action::DeclineEffectPayment);
            }
            settle(&mut g);
            assert_eq!(
                g.counter_count(attacker, CounterKind::PlusOne),
                u64::from(pay)
            );
            assert_eq!(g.keywords(attacker).contains(Keywords::TRAMPLE), pay);
            assert_eq!(g.state.exile_permissions.len(), usize::from(pay));
        }
    }
}

#[test]
fn inti_one_or_more_discard_permission_expires_at_the_next_own_end_step() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        put(&mut g, p, "Inti, Seneschal of the Sun", Zone::Battlefield);
        g.begin_step(Step::End);
        let a = put(&mut g, p, "Mountain", Zone::Hand);
        let b = put(&mut g, p, "Forest", Zone::Hand);
        g.discard_batch(&[a, b], None);
        settle(&mut g);
        assert_eq!(g.state.exile_permissions.len(), 1);
        let object = g.state.exile_permissions[0].object;
        assert!(g.can_play_exiled(p, object.card));
        g.state.active = p.other();
        g.begin_step(Step::End);
        assert!(g.can_play_exiled(p, object.card));
        g.state.active = p;
        g.begin_step(Step::End);
        assert!(!g.can_play_exiled(p, object.card));
    }
}

#[test]
fn monument_uses_each_mode_once_per_incarnation_per_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let monument = put(&mut g, p, "Monument to Endurance", Zone::Battlefield);
        let before = g.state.players[p.other().index()].life;
        for mode in [0, 1, 2] {
            let card = put(&mut g, p, "Mountain", Zone::Hand);
            g.discard_batch(&[card], None);
            g.prepare_triggers();
            reach(&mut g, |g| {
                g.legal_actions()
                    .iter()
                    .any(|a| matches!(a, Action::ChooseMode { .. }))
            });
            for used in 0..mode {
                assert!(
                    !g.legal_actions()
                        .contains(&Action::ChooseMode { mode: used })
                );
            }
            checked(&mut g, Action::ChooseMode { mode });
            settle(&mut g);
        }
        assert_eq!(g.hand_count(p), 1);
        assert_eq!(token_count(&g, p, "Treasure"), 1);
        assert_eq!(g.state.players[p.other().index()].life, before - 3);
        let card = put(&mut g, p, "Mountain", Zone::Hand);
        g.discard_batch(&[card], None);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, before - 3);
        g.state.turn += 1;
        g.begin_turn();
        let card = put(&mut g, p, "Mountain", Zone::Hand);
        g.discard_batch(&[card], None);
        g.prepare_triggers();
        reach(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseMode { mode: 2 })
        });
        checked(&mut g, Action::ChooseMode { mode: 2 });
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, before - 6);
        assert_eq!(g.object_view(monument).zone, Zone::Battlefield);
    }
}

#[test]
fn obsessive_counts_prior_sacrifices_and_clue_costs_then_scales_the_attack_reward() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = put(&mut g, p, "Mountain", Zone::Battlefield);
        let b = put(&mut g, p, "Forest", Zone::Battlefield);
        g.sacrifice_batch(&[(p, a), (p, b)]);
        let source = put(&mut g, p, "Obsessive Pursuit", Zone::Hand);
        let life = g.state.players[p.index()].life;
        cast(&mut g, source, None, &[]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life - 1);
        assert_eq!(token_count(&g, p, "Clue"), 1);
        let clue = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(id, o)| {
                o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.is_token()
                    && g.has_subtype(*id as CardId, "Clue")
            })
            .unwrap()
            .0 as CardId;
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: clue,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        settle(&mut g);
        assert_eq!(
            g.observe(p.other()).players[p.index()].permanents_sacrificed_this_turn,
            3
        );
        let attacker = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        attack(&mut g, attacker);
        reach(&mut g, |g| {
            matches!(g.state.pending, Pending::Targets { .. })
        });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(attacker),
            },
        );
        settle(&mut g);
        assert_eq!(g.counter_count(attacker, CounterKind::PlusOne), 3);
        assert!(g.keywords(attacker).contains(Keywords::LIFELINK));
        g.state.turn += 2;
        g.state.active = p;
        g.begin_turn();
        assert_eq!(
            g.state.players[p.index()].permanents_sacrificed_this_turn,
            0
        );
        g.begin_step(Step::Upkeep);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life - 2);
        assert_eq!(token_count(&g, p, "Clue"), 1);
    }
}
