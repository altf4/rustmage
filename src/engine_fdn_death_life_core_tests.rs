//! Regression tests for frozen death state and controller life payment.
use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::tests::spawn;
use super::*;

fn fixture(g: &mut Game, p: Player, name: &str) -> CardId {
    let card = Card::from_name(name).unwrap();
    assert!(card.is_supported());
    let id = spawn(g, p, card, Zone::Hand);
    cast(g, id, None, &[]);
    settle(g);
    assert_eq!(g.object_view(id).zone, Zone::Battlefield);
    id
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) {
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, targets);
    settle(g);
}
fn attack(g: &mut Game, id: CardId) {
    g.state.objects[id as usize].summoning_sick = false;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    checked(g, Action::Attack { card: id });
    checked(g, Action::FinishAttackers);
    assert!(g.object_view(id).attacking);
}
fn optional(g: &mut Game) {
    g.prepare_triggers();
    for _ in 0..60 {
        if matches!(g.state.pending, Pending::EffectOptional { .. }) {
            return;
        }
        let actions = g.legal_actions();
        let action = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            actions
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::PutTrigger { .. } | Action::FinishTargets | Action::PayCosts { .. }
                    )
                })
                .unwrap()
        };
        checked(g, action);
    }
    panic!("no optional payment {:?}", g.state.pending)
}
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}

#[test]
fn fdn_death_life_core_paid_death_selects_one_frozen_attacking_branch() {
    for p in [Player::P0, Player::P1] {
        for attacking in [false, true] {
            let mut g = setup(p);
            fixture(&mut g, p, "Garna, Bloodfist of Keld");
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            if attacking {
                attack(&mut g, bear);
            }
            let drawn = g.state.players[p.index()].cards_drawn_this_turn;
            let shock = named(&mut g, p, "Shock", Zone::Hand);
            cast(&mut g, shock, None, &[Target::Card(bear)]);
            settle(&mut g);
            assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                drawn + u64::from(attacking)
            );
            assert_eq!(
                g.state.players[p.other().index()].life,
                if attacking { 20 } else { 19 }
            );
            assert_eq!(g.state.players[p.index()].life, 20);
        }
    }
}
#[test]
fn fdn_death_life_core_simultaneous_departure_keeps_only_other_death_branches() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let garna = fixture(&mut g, p, "Garna, Bloodfist of Keld");
        let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        // A real attack establishes frozen combat membership; a simultaneous
        // sacrifice then removes all sources before their packets resolve.
        attack(&mut g, a);
        let drawn = g.state.players[p.index()].cards_drawn_this_turn;
        g.sacrifice_batch(&[(p, garna), (p, a), (p, b)]);
        assert_eq!(g.state.pending_triggers.len(), 2);
        g.move_object(a, Zone::Battlefield);
        g.state.objects[garna as usize].controller = p.other();
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, drawn + 1);
        assert_eq!(g.state.players[p.other().index()].life, 19);
        assert_eq!(g.state.players[p.index()].life, 20);
    }
}
#[test]
fn fdn_death_life_core_control_relations_and_removed_abilities_use_predeath_world() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let garna = fixture(&mut g, p, "Garna, Bloodfist of Keld");
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.move_object(enemy, Zone::Graveyard);
        assert!(g.state.pending_triggers.is_empty());
        g.state.objects[garna as usize].controller = p.other();
        let ally = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.move_object(ally, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 19);
        paid(&mut g, p, "Stop Cold", &[Target::Card(garna)]);
        let ally = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.move_object(ally, Zone::Graveyard);
        assert!(g.state.pending_triggers.is_empty());
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 19);
    }
}
#[test]
fn fdn_death_life_core_controller_payment_affordability_decline_exact_draw_and_privacy() {
    for p in [Player::P0, Player::P1] {
        for life in [1, 2, 5] {
            for accept in [false, true] {
                let mut g = setup(p);
                let c = fixture(&mut g, p, "Crossway Troublemakers");
                let vampire = named(&mut g, p, "Vampire Nighthawk", Zone::Battlefield);
                let top = *g.state.players[p.index()].library.last().unwrap();
                let drawn = g.state.players[p.index()].cards_drawn_this_turn;
                g.state.players[p.index()].life = life;
                g.move_object(vampire, Zone::Graveyard);
                if life >= 2 {
                    optional(&mut g);
                    assert_eq!(g.actor(), Some(p));
                    assert!(g.observe(p.other()).actions.is_empty());
                    assert_eq!(g.state.players[p.index()].life, life);
                    reject(&mut g, Action::DeclineEffectPayment);
                    reject(&mut g, Action::PayResolutionMana { payment: [0; 6] });
                    // The trigger's controller is captured, independent of source.
                    g.state.objects[c as usize].controller = p.other();
                    g.move_object(c, Zone::Exile);
                    checked(&mut g, Action::ChooseOptional { accept });
                    if g.outcome().is_none() {
                        settle(&mut g);
                    }
                } else {
                    settle(&mut g);
                }
                let paid = accept && life >= 2;
                assert_eq!(
                    g.state.players[p.index()].life,
                    life - if paid { 2 } else { 0 }
                );
                assert_eq!(
                    g.state.players[p.index()].life_lost_this_turn,
                    if paid { 2 } else { 0 }
                );
                assert_eq!(g.state.players[p.other().index()].life, 20);
                assert_eq!(
                    g.state.players[p.index()].cards_drawn_this_turn,
                    drawn + u64::from(paid)
                );
                assert_eq!(
                    g.object_view(top).zone,
                    if paid { Zone::Hand } else { Zone::Library }
                );
                assert!(!g.observe(p.other()).objects.iter().any(|v| v.id == top));
                if paid && life == 2 {
                    assert!(g.outcome().is_some());
                }
            }
        }
    }
}
#[test]
fn fdn_death_life_core_attacking_vampire_grants_and_self_death_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let c = fixture(&mut g, p, "Crossway Troublemakers");
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        assert!(!g.keywords(c).contains(Keywords::DEATHTOUCH));
        attack(&mut g, c);
        assert!(
            g.keywords(c)
                .contains(Keywords::DEATHTOUCH.union(Keywords::LIFELINK))
        );
        assert!(!g.keywords(bear).contains(Keywords::DEATHTOUCH));
        let drawn = g.state.players[p.index()].cards_drawn_this_turn;
        g.move_object(c, Zone::Graveyard);
        optional(&mut g);
        checked(&mut g, Action::ChooseOptional { accept: true });
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 18);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, drawn + 1);
    }
}
