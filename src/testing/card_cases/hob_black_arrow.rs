//! Focused printed outcomes; shared engine invariants live in core tests.
//! Actual damage, post-prevention Dragon destruction and paid Equipment wiring.
use super::*;
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..64 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Black Arrow decision budget: {:?}", g.state.pending);
}
fn drive(g: &mut Game) {
    until(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
    });
}
fn start(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "The Black Arrow", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
    cast(g, id, None);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    until(g, |g| matches!(g.state.pending, Pending::Targets { .. }));
    id
}
fn bind(g: &mut Game, target: Target) {
    act(g, Action::ChooseTarget { target });
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}

#[test]
fn black_arrow_paid_flash_entry_any_player_creature_planeswalker_and_mandatory_legality() {
    {
        let p = Player::P0;
        for row in 0..3 {
            let mut g = scenario(p);
            let bear = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let walker = put(
                &mut g,
                p.other(),
                "Ajani, Caller of the Pride",
                Zone::Battlefield,
            );
            g.state.objects[walker as usize].counters.loyalty = 5;
            let land = put(&mut g, p, "Forest", Zone::Battlefield);
            g.state.active = p.other();
            g.state.step = Step::End;
            g.state.priority = p;
            let arrow = start(&mut g, p);
            assert!(g.keywords(arrow).contains(Keywords::FLASH));
            assert_eq!(g.object_view(arrow).zone, Zone::Battlefield);
            for id in [land, arrow] {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(id),
                    },
                );
            }
            reject(&mut g, Action::FinishTargets);
            bind(
                &mut g,
                match row {
                    0 => Target::Player(p.other()),
                    1 => Target::Card(bear),
                    _ => Target::Card(walker),
                },
            );
            drive(&mut g);
            assert_eq!(
                g.state.players[p.other().index()].life,
                if row == 0 { 19 } else { 20 }
            );
            assert_eq!(
                g.state.objects[bear as usize].damage,
                if row == 1 { 1 } else { 0 }
            );
            assert_eq!(
                g.counter_count(walker, CounterKind::Loyalty),
                if row == 2 { 4 } else { 5 }
            );
        }
    }
}
#[test]
fn black_arrow_actual_positive_damage_destroys_dragon_but_shield_and_indestructible_differ() {
    {
        let p = Player::P0;
        for row in 0..3 {
            let mut g = scenario(p);
            let dragon = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
            // Existing damage must not qualify a subsequently prevented Arrow.
            let shock = put(&mut g, p, "Shock", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
            cast(&mut g, shock, Some(Target::Card(dragon)));
            drive(&mut g);
            assert_eq!(g.state.objects[dragon as usize].damage, 2);
            if row == 1 {
                g.add_counter(dragon, CounterKind::Shield, 1);
            }
            if row == 2 {
                g.add_counter(dragon, CounterKind::Indestructible, 1);
            }
            start(&mut g, p);
            bind(&mut g, Target::Card(dragon));
            drive(&mut g);
            assert_eq!(
                g.object_view(dragon).zone,
                if row == 0 {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            if row == 1 {
                assert_eq!(g.state.objects[dragon as usize].damage, 2);
                assert_eq!(g.counter_count(dragon, CounterKind::Shield), 0);
            }
            if row == 2 {
                assert_eq!(g.state.objects[dragon as usize].damage, 3);
            }
        }
    }
}

#[test]
fn black_arrow_shared_tracker_preserves_actual_goblin_negotiation_excess_tokens() {
    {
        let p = Player::P0;
        for shield in [false, true] {
            let mut g = scenario(p);
            let target = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            if shield {
                g.add_counter(target, CounterKind::Shield, 1);
            }
            let spell = put(&mut g, p, "Goblin Negotiation", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 2, 0, 5];
            act(&mut g, Action::AnnounceCast { card: spell });
            act(&mut g, Action::ChooseX { value: 5 });
            bind(&mut g, Target::Card(target));
            pay(&mut g);
            drive(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            let goblins: Vec<_> = g
                .state
                .objects
                .iter()
                .enumerate()
                .filter_map(|(i, o)| {
                    (o.zone == Zone::Battlefield
                        && o.controller == p
                        && o.card.is_token()
                        && o.card.has_subtype("Goblin"))
                    .then_some(i as CardId)
                })
                .collect();
            assert_eq!(goblins.len(), if shield { 0 } else { 2 });
            for id in goblins {
                assert_eq!(stats(&g, id), (1, 1));
                assert_eq!(g.state.objects[id as usize].card.colors(), 8);
            }
            assert_eq!(
                g.object_view(target).zone,
                if shield {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
        }
    }
}
