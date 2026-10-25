//! Gated attachment core fixtures: real immutable definitions and paid actions.
//! Winter's return-and-choice continuation remains outside this first core scope.
use super::*;
use crate::engine::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use crate::engine::msh_thirteenth_combined_tests::owned_checked_step;
const SWORD: &str = "Swordsman, Sharp Scoundrel";
const WINTER: &str = "Winter Soldier, Icy Assassin";
fn seed(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    if card.is_supported() {
        return named(g, p, name, zone);
    }
    assert!([SWORD, WINTER].contains(&name));
    let id = CardId::try_from(g.state.objects.len()).unwrap();
    g.state.objects.push(Object {
        card,
        copy_kind: CopyKind::None,
        owner: p,
        controller: p,
        zone,
        generation: 0,
        tapped: false,
        summoning_sick: false,
        flashback: false,
        damage: 0,
        damaged_this_turn: false,
        dealt_damage_this_turn: false,
        deathtouch_since_sba: false,
        growth: Stats::default(),
        gained_keywords: Keywords::NONE,
        counters: InitialCounters::default(),
        used_abilities: 0,
        attached_to: None,
    });
    match zone {
        Zone::Battlefield => g.register_battlefield_sources(id),
        Zone::Library => g.state.players[p.index()].library.push(id),
        Zone::Graveyard => g.register_graveyard_sources(id),
        _ => (),
    }
    id
}
fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn packet(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    priority(g, p);
    let id = seed(g, p, name, Zone::Hand);
    let (mut mana, generic) = Card::from_name(name).unwrap().cost();
    mana[5] += generic;
    g.state.players[p.index()].mana = mana;
    cast(g, id, None, targets);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    let id = packet(g, p, name, targets);
    settle(g);
    id
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..240 {
        if stop(g) {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                        | Action::ChooseMode { mode: 0 }
                )
            })
            .unwrap_or_else(|| panic!("unexpected {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("stalled {:?}", g.state.pending);
}
fn finish(g: &mut Game) {
    for _ in 0..12 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .unwrap();
        checked(g, a);
    }
    panic!("unfinished");
}
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    let seen = [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap());
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), r);
    assert_eq!(
        [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap()),
        seen
    );
}
fn orphan(g: Game, a: Action) -> Game {
    let p = g.actor().unwrap();
    let i = g.legal_actions().iter().position(|v| *v == a).unwrap();
    owned_checked_step(g, p, i, a)
}
fn entry_targets(g: &mut Game, sword: CardId) {
    until(g, |g| {
        matches!(g.state.pending, Pending::Targets { .. })
            && g.state
                .announcing
                .as_ref()
                .is_some_and(|a| a.item.card == sword)
    });
}
fn next_main(g: &mut Game, p: Player) {
    next_turn(g);
    next_turn(g);
    until(g, |g| {
        g.state.active == p && g.state.step == Step::Main1 && g.state.pending == Pending::Priority
    });
    priority(g, p);
}
fn equip(g: &mut Game, p: Player, pole: CardId, bear: CardId) {
    priority(g, p);
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
    checked(
        g,
        Action::AnnounceActivate {
            card: pole,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    finish(g);
    settle(g);
}
#[test]
fn msh_attachment_core_paid_swordsman_optional_equipment_mandatory_creature_and_real_move() {
    for p in [Player::P0, Player::P1] {
        for choose in [false, true] {
            let mut g = setup(p);
            let sword = paid(&mut g, p, SWORD, &[]);
            let pole = paid(&mut g, p, "Fishing Pole", &[]);
            let bear = paid(&mut g, p, "Bear Cub", &[]);
            let foreign = named(&mut g, p.other(), "Fishing Pole", Zone::Battlefield);
            packet(&mut g, p, "Mister Hyde, Monster Within", &[]);
            entry_targets(&mut g, sword);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bear),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(foreign),
                },
            );
            g = orphan(
                g,
                if choose {
                    Action::ChooseTarget {
                        target: Target::Card(pole),
                    }
                } else {
                    Action::FinishTargets
                },
            );
            reject(&mut g, Action::FinishTargets);
            g = orphan(
                g,
                Action::ChooseTarget {
                    target: Target::Card(bear),
                },
            );
            finish(&mut g);
            settle(&mut g);
            assert_eq!(
                g.state.objects[pole as usize].attached_to,
                choose.then(|| g.object_ref(bear))
            );
            assert_eq!(g.object_view(pole).controller, p);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}
#[test]
fn msh_attachment_core_paid_swordsman_invalidated_target_preserves_old_attachment_and_source_independence()
 {
    for p in [Player::P0, Player::P1] {
        for departure in 0..3 {
            let mut g = setup(p);
            let sword = paid(&mut g, p, SWORD, &[]);
            let pole = paid(&mut g, p, "Fishing Pole", &[]);
            let old = paid(&mut g, p, "Bear Cub", &[]);
            let new = named(&mut g, p, "Hill Giant", Zone::Battlefield);
            equip(&mut g, p, pole, old);
            packet(&mut g, p, "Mister Hyde, Monster Within", &[]);
            entry_targets(&mut g, sword);
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(pole),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(new),
                },
            );
            finish(&mut g);
            let removed = match departure {
                0 => new,
                1 => pole,
                _ => sword,
            };
            paid(&mut g, p, "Unsummon", &[Target::Card(removed)]);
            settle(&mut g);
            assert_eq!(
                g.state.objects[pole as usize].attached_to,
                match departure {
                    0 => Some(g.object_ref(old)),
                    1 => None,
                    _ => Some(g.object_ref(new)),
                }
            );
        }
    }
}
#[test]
fn msh_attachment_core_paid_swordsman_declared_attack_connives_once_with_two_equipment() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let sword = paid(&mut g, p, SWORD, &[]);
        let a = paid(&mut g, p, "Fishing Pole", &[]);
        let b = paid(&mut g, p, "Fishing Pole", &[]);
        let bear = paid(&mut g, p, "Bear Cub", &[]);
        equip(&mut g, p, a, bear);
        equip(&mut g, p, b, bear);
        next_main(&mut g, p);
        let discard = named(&mut g, p, "Shock", Zone::Hand);
        let n = g.state.players[p.index()].library.len();
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        checked(&mut g, Action::Attack { card: bear });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectDiscard { .. })
        });
        assert_eq!(g.state.players[p.index()].library.len(), n - 1);
        assert_eq!(g.state.resolving.as_ref().unwrap().item.card, sword);
        assert!(g.state.resolving.as_ref().unwrap().item.targets.is_empty());
        g = orphan(g, Action::Discard { card: discard });
        settle(&mut g);
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 1);
        assert_eq!(g.state.players[p.index()].library.len(), n - 1);
    }
}
#[test]
fn msh_attachment_core_paid_swordsman_queued_attacker_current_or_departed_controller_connives() {
    for p in [Player::P0, Player::P1] {
        for leave in [false, true] {
            let mut g = setup(p);
            let sword = paid(&mut g, p, SWORD, &[]);
            let pole = paid(&mut g, p, "Fishing Pole", &[]);
            let bear = paid(&mut g, p, "Bear Cub", &[]);
            equip(&mut g, p, pole, bear);
            next_main(&mut g, p);
            until(&mut g, |g| g.state.pending == Pending::Attackers);
            checked(&mut g, Action::Attack { card: bear });
            checked(&mut g, Action::FinishAttackers);
            until(&mut g, |g| {
                g.state.stack.iter().any(|s| {
                    s.card == sword && matches!(s.kind, StackKind::TriggeredAbility { .. })
                })
            });
            let old = g.object_ref(bear);
            g.change_controller(old, p.other());
            if leave {
                packet(&mut g, p, "Unsummon", &[Target::Card(bear)]);
            }
            let discard = named(&mut g, p.other(), "Shock", Zone::Hand);
            let n = g.state.players[p.other().index()].library.len();
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectDiscard { .. })
            });
            assert_eq!(g.actor(), Some(p.other()));
            assert_eq!(g.state.players[p.other().index()].library.len(), n - 1);
            g = orphan(g, Action::Discard { card: discard });
            settle(&mut g);
            assert_eq!(
                g.counter_count(bear, CounterKind::PlusOne),
                u64::from(!leave)
            );
            assert_eq!(
                g.object_view(bear).zone,
                if leave { Zone::Hand } else { Zone::Battlefield }
            );
        }
    }
}
#[test]
fn msh_attachment_core_paid_winter_live_equipment_count_same_in_virtual_world() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let winter = paid(&mut g, p, WINTER, &[]);
        let a = paid(&mut g, p, "Fishing Pole", &[]);
        let b = paid(&mut g, p, "Fishing Pole", &[]);
        assert_eq!(g.stats(winter).power, 2);
        equip(&mut g, p, a, winter);
        equip(&mut g, p, b, winter);
        let base = Card::from_name("Fishing Pole")
            .unwrap()
            .equipment()
            .unwrap()
            .stats
            .power;
        assert_eq!(g.stats(winter).power, 2 + 4 + base * 2);
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 0,
            source: None,
            reserve: None,
        };
        let mut w = FundingState::from_game(&g, &goal);
        assert_eq!(w.value(StatValue::SourceEquipmentCount, winter), 2);
        assert_eq!(w.power(winter), g.stats(winter).power);
        let i = w.resources.iter().position(|r| r.id == a).unwrap();
        w.resources[i].object.zone = Zone::Graveyard;
        assert_eq!(w.value(StatValue::SourceEquipmentCount, winter), 1);
        assert_eq!(w.power(winter), 2 + 2 + base);
        assert_eq!(g.stats(winter).power, 2 + 4 + base * 2);
    }
}
