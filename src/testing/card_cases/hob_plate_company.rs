//! Focused printed outcomes; shared engine invariants live in core tests.
//! Printed outcomes and bounded ownership checks for amass attachment and live land allowances.
use super::*;
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..48 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("plate/company decision budget {:?}", g.state.pending);
}
fn quiet(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.resolving.is_none()
        && g.state.pending_triggers.is_empty()
}
fn drive(g: &mut Game, preferred: Option<CardId>) {
    for _ in 0..96 {
        if quiet(g) {
            return;
        }
        let actions = g.legal_actions();
        let action = preferred
            .and_then(|id| {
                actions.iter().find(|a| match a {
                    Action::ChooseArmy { object } => object.card == id,
                    Action::ChooseTarget { target } => *target == Target::Card(id),
                    _ => false,
                })
            })
            .or_else(|| {
                actions
                    .iter()
                    .find(|a| matches!(a, Action::ChooseReplacement { .. }))
            })
            .or_else(|| {
                actions.iter().find(|a| {
                    matches!(
                        a,
                        Action::ChooseArmy { .. }
                            | Action::FinishTargets
                            | Action::PutTrigger { .. }
                            | Action::Pass
                    )
                })
            })
            .unwrap_or_else(|| panic!("explicit choice {:?}", g.state.pending))
            .clone();
        act(g, action);
    }
    panic!("plate/company resolution budget");
}

fn armies(g: &Game, p: Player) -> Vec<CardId> {
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
fn plate(g: &mut Game, p: Player, black: bool) -> CardId {
    let id = put(g, p, "Goblin Plate Mail", Zone::Hand);
    g.state.players[p.index()].mana = if black {
        [0, 0, 1, 0, 0, 1]
    } else {
        [0, 0, 0, 1, 0, 1]
    };
    cast(g, id, None);
    id
}
fn company(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Thranduil's Company", Zone::Hand);
    g.state.players[p.index()].mana = [0, 1, 0, 0, 1, 2];
    cast(g, id, None);
    drive(g, None);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert_eq!(stats(g, id), (3, 4));
    assert!(g.has_subtype(id, "Elf") && g.has_subtype(id, "Soldier"));
    id
}
fn paid(g: &mut Game, p: Player, name: &str, target: Option<Target>) -> CardId {
    let id = put(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = [12; 6];
    cast(g, id, target);
    drive(g, None);
    id
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
fn play(g: &mut Game, land: CardId, target: CardId) {
    act(g, Action::PlayLand { card: land });
    drive(g, Some(target));
}
#[test]
fn hob_plate_company_plate_hybrid_amass_recipe_paid_equip_and_menace() {
    {
        let p = Player::P0;
        for black in [false, true] {
            let mut g = scenario(p);
            let plate = plate(&mut g, p, black);
            drive(&mut g, None);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            let a = armies(&g, p);
            assert_eq!(a.len(), 1);
            let army = a[0];
            let v = g.object_view(army);
            assert_eq!((v.colors, v.kinds, v.power, v.toughness), (4, 1, 2, 1));
            assert!(g.has_subtype(army, "Goblin"));
            assert_eq!(g.counter_count(army, CounterKind::PlusOne), 1);
            assert_eq!(g.object_view(plate).attached_to, Some(g.object_ref(army)));
            assert!(g.keywords(army).contains(Keywords::MENACE));
            let bearer = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let first = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let second = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: plate,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(first),
                },
            );
            bind(&mut g, bearer);
            pay(&mut g);
            drive(&mut g, None);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(stats(&g, bearer), (3, 2));
            assert_eq!(stats(&g, army), (1, 1));
            assert!(!g.keywords(army).contains(Keywords::MENACE));
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            act(&mut g, Action::Attack { card: bearer });
            act(&mut g, Action::FinishAttackers);
            assert_eq!(g.actor(), Some(p));
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
            reject(
                &mut g,
                Action::AnnounceActivate {
                    card: plate,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            until(&mut g, |g| g.state.pending == Pending::Blockers);
            act(
                &mut g,
                Action::Block {
                    blocker: first,
                    attacker: bearer,
                },
            );
            reject(&mut g, Action::FinishBlockers);
            act(
                &mut g,
                Action::Block {
                    blocker: second,
                    attacker: bearer,
                },
            );
            act(&mut g, Action::FinishBlockers);
        }
    }
}

#[test]
fn hob_plate_company_plate_old_source_incarnation_does_not_attach_returned_equipment() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let a = put(&mut g, p, "@token/goblin_army", Zone::Battlefield);
        g.set_counters(a, Counters::of(CounterKind::PlusOne, 1));
        let b = put(&mut g, p, "@token/goblin_army", Zone::Battlefield);
        g.set_counters(b, Counters::of(CounterKind::PlusOne, 1));
        let source = plate(&mut g, p, false);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectChoice { .. })
        });
        let old = g.object_ref(source);
        g.move_object(source, Zone::Hand);
        g.move_object(source, Zone::Battlefield);
        assert_ne!(old, g.object_ref(source));
        let object = g.object_ref(a);
        act(&mut g, Action::ChooseArmy { object });
        assert_eq!(g.counter_count(a, CounterKind::PlusOne), 2);
        assert_eq!(g.object_view(source).attached_to, None);
        drive(&mut g, Some(b));
        assert_eq!(g.object_view(source).attached_to, Some(g.object_ref(b)));
        assert_eq!(g.counter_count(a, CounterKind::PlusOne), 2);
        assert_eq!(g.counter_count(b, CounterKind::PlusOne), 2);
    }
}
#[test]
fn hob_plate_company_company_live_qualifier_suppression_control_and_used_count() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = company(&mut g, p);
        let first = put(&mut g, p, "Forest", Zone::Hand);
        let second = put(&mut g, p, "Island", Zone::Hand);
        let third = put(&mut g, p, "Forest", Zone::Hand);
        put(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
        play(&mut g, first, source);
        reject(&mut g, Action::PlayLand { card: second });
        let elf = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        assert!(
            g.legal_actions()
                .contains(&Action::PlayLand { card: second })
        );
        let aura = paid(&mut g, p, "Witness Protection", Some(Target::Card(elf)));
        reject(&mut g, Action::PlayLand { card: second });
        paid(&mut g, p, "Disenchant", Some(Target::Card(aura)));
        assert!(
            g.legal_actions()
                .contains(&Action::PlayLand { card: second })
        );
        let aura = paid(&mut g, p, "Witness Protection", Some(Target::Card(source)));
        reject(&mut g, Action::PlayLand { card: second });
        paid(&mut g, p, "Disenchant", Some(Target::Card(aura)));
        play(&mut g, second, source);
        assert_eq!(g.state.continuous_land_plays_used, 1);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 4);
        g.move_object(elf, Zone::Hand);
        reject(&mut g, Action::PlayLand { card: third });
        g.move_object(elf, Zone::Battlefield);
        reject(&mut g, Action::PlayLand { card: third });
        g.move_object(source, Zone::Hand);
        g.move_object(source, Zone::Battlefield);
        reject(&mut g, Action::PlayLand { card: third });
        let extra = put(&mut g, p, "Thranduil's Company", Zone::Battlefield);
        assert!(
            g.legal_actions()
                .contains(&Action::PlayLand { card: third })
        );
        play(&mut g, third, extra);
        assert_eq!(g.state.continuous_land_plays_used, 2);
        let fourth = put(&mut g, p, "Forest", Zone::Hand);
        g.record_control(g.object_ref(extra), p.other(), None);
        reject(&mut g, Action::PlayLand { card: fourth });
        g.record_control(g.object_ref(extra), p, None);
        reject(&mut g, Action::PlayLand { card: fourth });
    }
}
