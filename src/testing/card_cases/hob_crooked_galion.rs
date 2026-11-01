//! Focused printed outcomes; shared engine invariants live in core tests.
//! Full printed outcomes plus frozen event and resolved layer-7b contracts.
use super::*;
pub(super) fn quiet(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.pending_triggers.is_empty()
        && g.state.resolving.is_none()
}
pub(super) fn drive(g: &mut Game, preferred: Option<CardId>) {
    for _ in 0..96 {
        if quiet(g) {
            return;
        }
        let a = g.legal_actions();
        let action = preferred
            .and_then(|id| {
                a.iter().find(|a| match a {
                    Action::ChooseTarget { target } => *target == Target::Card(id),
                    Action::ChooseArmy { object } => object.card == id,
                    _ => false,
                })
            })
            .or_else(|| {
                a.iter()
                    .find(|a| matches!(a, Action::ChooseReplacement { .. }))
            })
            .or_else(|| {
                a.iter().find(|a| {
                    matches!(
                        a,
                        Action::Pass
                            | Action::PutTrigger { .. }
                            | Action::FinishTargets
                            | Action::ChooseArmy { .. }
                    )
                })
            })
            .unwrap_or_else(|| panic!("explicit choice {:?}", g.state.pending))
            .clone();
        act(g, action);
    }
    panic!("crooked/galion resolution budget");
}
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..48 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("crooked/galion decision budget {:?}", g.state.pending);
}
pub(super) fn armies(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && g.is_creature(id as CardId)
                && g.has_subtype(id as CardId, "Army"))
            .then_some(id as CardId)
        })
        .collect()
}

fn bind(g: &mut Game, id: CardId) {
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(id),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}
fn paid(g: &mut Game, p: Player, name: &str, target: Option<Target>) -> CardId {
    let id = put(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = [12; 6];
    cast(g, id, target);
    drive(g, None);
    id
}
fn galion(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Galion, Elvenking's Butler", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 0, 2, 2];
    cast(g, id, None);
    drive(g, None);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert_eq!(stats(g, id), (4, 4));
    assert!(g.has_subtype(id, "Elf") && g.has_subtype(id, "Advisor"));
    assert!(g.object_view(id).is_legendary());
    id
}
fn attack(g: &mut Game, source: CardId) {
    g.state.objects[source as usize].summoning_sick = false;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    act(g, Action::Attack { card: source });
    act(g, Action::FinishAttackers);
    until(g, |g| g.legal_actions().contains(&Action::FinishTargets));
}

#[test]
fn hob_crooked_galion_along_paid_etb_return_causes_amass_and_public_hand_knowledge() {
    {
        let p = Player::P0;
        for stale in [false, true] {
            let mut g = scenario(p);
            let own = put(&mut g, p, "Bear Cub", Zone::Graveyard);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
            let land = put(&mut g, p, "Forest", Zone::Graveyard);
            let source = put(&mut g, p, "Along the Crooked Way", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 2];
            cast(&mut g, source, None);
            until(&mut g, |g| {
                g.legal_actions().contains(&Action::ChooseTarget {
                    target: Target::Card(own),
                })
            });
            for illegal in [enemy, land] {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(illegal),
                    },
                );
            }
            bind(&mut g, own);
            if stale {
                g.move_object(own, Zone::Exile);
                drive(&mut g, None);
            }
            drive(&mut g, None);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(g.object_view(source).zone, Zone::Battlefield);
            assert_eq!(
                g.object_view(own).zone,
                if stale { Zone::Exile } else { Zone::Hand }
            );
            assert_eq!(armies(&g, p).len(), 1);
            assert_eq!(
                g.counter_count(armies(&g, p)[0], CounterKind::PlusOne),
                1,
                "stale ETB target still left graveyard via actual response"
            );
            if !stale {
                assert!(g.observe(p.other()).known_hand.contains(&g.object_ref(own)));
            }
        }
        let mut g = scenario(p);
        let source = put(&mut g, p, "Along the Crooked Way", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 2];
        cast(&mut g, source, None);
        drive(&mut g, None);
        assert!(armies(&g, p).is_empty());
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
    }
}
#[test]
fn hob_crooked_galion_along_actual_reanimation_and_paid_graveyard_cast_each_trigger() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        put(&mut g, p, "Along the Crooked Way", Zone::Battlefield);
        let body = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        paid(&mut g, p, "Zombify", Some(Target::Card(body)));
        assert_eq!(g.object_view(body).zone, Zone::Battlefield);
        let army = armies(&g, p)[0];
        assert_eq!(g.counter_count(army, CounterKind::PlusOne), 1);
        let casted = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        g.grant_graveyard_permission(p, g.object_ref(casted), GraveyardCastKind::Cast);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
        cast(&mut g, casted, None);
        drive(&mut g, None);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.object_view(casted).zone, Zone::Battlefield);
        assert_eq!(g.counter_count(army, CounterKind::PlusOne), 2);
    }
}

#[test]
fn hob_crooked_galion_galion_samples_resolution_axes_then_fixed_base_layer_and_modifiers() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = galion(&mut g, p);
        g.add_counter(source, CounterKind::PlusOne, 1);
        let body = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.add_counter(body, CounterKind::PlusOne, 2);
        paid(&mut g, p, "Witness Protection", Some(Target::Card(body)));
        paid(&mut g, p, "Giant Growth", Some(Target::Card(body)));
        assert_eq!(stats(&g, body), (6, 6));
        attack(&mut g, source);
        bind(&mut g, body);
        let trigger = g.state.stack.last().unwrap().id;
        let growth = put(&mut g, p, "Giant Growth", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
        cast(&mut g, growth, Some(Target::Card(source)));
        until(&mut g, |g| {
            g.state.stack.last().is_some_and(|s| s.id == trigger)
        });
        assert_eq!(stats(&g, source), (8, 8));
        drive(&mut g, None);
        assert_eq!(stats(&g, body), (13, 13));
        paid(&mut g, p, "Giant Growth", Some(Target::Card(source)));
        assert_eq!(stats(&g, source), (11, 11));
        assert_eq!(stats(&g, body), (13, 13));
        // The later Aura is a sorcery-speed base setter. Reach main phase two
        // through normal combat choices while both temporary growth effects last.
        for _ in 0..32 {
            if g.state.step == Step::Main2 && quiet(&g) {
                break;
            }
            choose(&mut g, |a| {
                matches!(a, Action::Pass | Action::FinishBlockers)
            });
        }
        assert_eq!(g.state.step, Step::Main2);
        paid(&mut g, p, "Witness Protection", Some(Target::Card(body)));
        assert_eq!(stats(&g, body), (6, 6));
    }
}
