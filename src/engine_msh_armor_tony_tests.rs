//! Paid printed producer scenarios; explicit primitive seams are labeled.
use super::assurance::big_helpers::{cast, checked, named, settle, setup};
use super::*;
const TONY: &str = "Tony Stark";
const IRON: &str = "The Invincible Iron Man";
fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn pay(g: &mut Game) {
    for _ in 0..12 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::PayCosts { .. }
                        | Action::FinishTargets
                        | Action::ChooseActivationCost { .. }
                )
            })
            .expect("payment");
        checked(g, a);
    }
    panic!("unpaid");
}
fn paid(g: &mut Game, p: Player, name: &str, back: bool, targets: &[Target]) -> CardId {
    priority(g, p);
    let id = named(g, p, name, Zone::Hand);
    let card = if back {
        g.state.objects[id as usize].card.other_face().unwrap()
    } else {
        g.state.objects[id as usize].card
    };
    let (mut mana, generic) = card.cost();
    mana[5] += generic;
    g.state.players[p.index()].mana = mana;
    if back {
        checked(g, Action::AnnounceBack { card: id });
        pay(g);
    } else {
        cast(g, id, None, targets);
    }
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    settle(g);
    id
}
fn activate(g: &mut Game, p: Player, id: CardId, ability: u16, mana: Mana) {
    priority(g, p);
    g.state.players[p.index()].mana = mana;
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability,
            counter_payment: Counters::default(),
        },
    );
    pay(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..100 {
        if stop(g) {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass | Action::PutTrigger { .. } | Action::FinishTargets
                )
            })
            .unwrap_or_else(|| panic!("unexpected {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("stalled");
}
fn choice(g: &mut Game, p: Player) {
    until(g, |g| g.observe(p).effect_choice.is_some());
}
fn combat(g: &mut Game, p: Player) {
    g.state.active = p;
    g.begin_step(Step::BeginCombat);
    choice(g, p);
}
fn views(g: &Game) -> [serde_json::Value; 2] {
    [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap())
}
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let v = views(g);
    let rev = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), s);
    assert_eq!(views(g), v);
    assert_eq!(g.revision(), rev);
}
fn orphan(g: Game, a: Action) -> Game {
    let actor = g.actor().unwrap();
    let rev = g.revision();
    let before = g.snapshot();
    let seen = views(&g);
    let index = g.legal_actions().iter().position(|x| *x == a).unwrap();
    let (mut raw, mut idx) = (g.clone(), g.clone());
    drop(g);
    raw.apply(actor, a.clone()).unwrap();
    idx.apply_index(actor, rev, index).unwrap();
    let after = raw.snapshot();
    assert_eq!(after, idx.snapshot());
    assert_eq!(views(&raw), views(&idx));
    raw.restore(&before).unwrap();
    let mut expected = seen;
    for v in &mut expected {
        v["revision"] = serde_json::json!(raw.revision());
    }
    assert_eq!(views(&raw), expected);
    assert_eq!(
        raw.apply_index(actor, rev, index),
        Err(Error::StaleRevision)
    );
    raw.apply(actor, a).unwrap();
    assert_eq!(raw.snapshot(), after);
    idx
}
fn empty_library(g: &mut Game, p: Player) {
    let ids = g.state.players[p.index()].library.clone();
    for id in ids {
        g.move_object(id, Zone::Graveyard);
    }
}
#[test]
fn msh_armor_tony_modal_metadata_both_paid_faces_and_printed_characteristics() {
    for p in [Player::P0, Player::P1] {
        for back in [false, true] {
            let mut g = setup(p);
            let id = paid(&mut g, p, TONY, back, &[]);
            let v = g.object_view(id);
            assert_eq!(v.name(), if back { IRON } else { TONY });
            assert_eq!((v.power, v.toughness), if back { (5, 5) } else { (1, 3) });
            assert_eq!(v.card.fixed_mana_value(), if back { 6 } else { 2 });
            assert_eq!(v.card.is_artifact(), back);
            assert!(v.is_legendary());
            if back {
                assert!(v.keywords.contains(Keywords::FLYING.union(Keywords::HASTE)));
            } else {
                assert!(v.has_subtype("Artificer"));
            }
            assert_eq!(
                Card::from_name(TONY).unwrap(),
                Card::from_name("Tony Stark // The Invincible Iron Man").unwrap()
            );
            assert_eq!(
                Card::from_name(IRON).unwrap().front_face(),
                Card::from_name(TONY).unwrap()
            );
            g.move_object(id, Zone::Hand);
            assert_eq!(g.object_view(id).name(), TONY);
        }
    }
}
#[test]
fn msh_armor_tony_paid_transform_is_sorcery_exact_source_and_preserves_counters() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, TONY, false, &[]);
        let r = g.object_ref(id);
        g.state.objects[id as usize].counters.plus_one = 2;
        g.state.step = Step::BeginCombat;
        g.state.players[p.index()].mana = [0, 1, 0, 1, 0, 4];
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: id,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        g.state.step = Step::Main1;
        activate(&mut g, p, id, 1, [0, 1, 0, 1, 0, 4]);
        assert_eq!(g.state.objects[id as usize].card.name(), TONY);
        settle(&mut g);
        assert_eq!(g.object_ref(id), r);
        assert_eq!(g.object_view(id).name(), IRON);
        assert_eq!(
            (g.object_view(id).power, g.object_view(id).toughness),
            (7, 7)
        );
        assert_eq!(g.object_view(id).card.fixed_mana_value(), 6);
    }
}
#[test]
fn msh_armor_tony_paid_transform_stale_packet_and_departure_do_not_retransform() {
    for p in [Player::P0, Player::P1] {
        for leave in [false, true] {
            let mut g = setup(p);
            let id = paid(&mut g, p, TONY, false, &[]);
            activate(&mut g, p, id, 1, [0, 1, 0, 1, 0, 4]);
            if leave {
                g.move_object(id, Zone::Hand);
                g.move_object(id, Zone::Battlefield);
            } else {
                // Bounded duplicate of the genuinely paid ability packet, solely testing CR701 transform serial guarding.
                let mut duplicate = g.state.stack.last().unwrap().clone();
                duplicate.id = g.next_stack_id();
                g.state.stack.push(duplicate);
            }
            settle(&mut g);
            assert_eq!(g.object_view(id).name(), if leave { TONY } else { IRON });
        }
    }
}
#[test]
fn msh_armor_tony_paid_look_four_reveals_one_artifact_and_randomizes_only_remainder() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, TONY, false, &[]);
        empty_library(&mut g, p);
        let prefix = named(&mut g, p, "Plains", Zone::Library);
        let art = named(&mut g, p, "Arc Reactor", Zone::Library);
        let b = named(&mut g, p, "Bear Cub", Zone::Library);
        let island = named(&mut g, p, "Island", Zone::Library);
        let forest = named(&mut g, p, "Forest", Zone::Library);
        g.state.objects[id as usize].summoning_sick = false;
        activate(&mut g, p, id, 0, [0, 0, 0, 0, 0, 1]);
        assert!(g.state.objects[id as usize].tapped);
        until(&mut g, |g| g.observe(p).library_selection.is_some());
        assert!(g.observe(p.other()).library_selection.is_none());
        reject(&mut g, Action::SelectLibrary { card: b });
        g = orphan(g, Action::SelectLibrary { card: art });
        checked(&mut g, Action::FinishLibrarySelection);
        settle(&mut g);
        assert_eq!(g.object_view(art).zone, Zone::Hand);
        assert!(g.state.known_hand.contains(&g.object_ref(art)));
        let lib = &g.state.players[p.index()].library;
        assert_eq!(lib.len(), 4);
        assert!(
            lib.contains(&prefix)
                && lib.contains(&b)
                && lib.contains(&island)
                && lib.contains(&forest)
        );
        assert_eq!(lib.last(), Some(&prefix));
    }
}
#[test]
fn msh_armor_tony_paid_look_empty_short_optional_and_sickness_tap_cost() {
    for p in [Player::P0, Player::P1] {
        for count in [0, 1, 3] {
            let mut g = setup(p);
            let id = paid(&mut g, p, TONY, false, &[]);
            priority(&mut g, p);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
            reject(
                &mut g,
                Action::AnnounceActivate {
                    card: id,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            g.state.objects[id as usize].summoning_sick = false;
            empty_library(&mut g, p);
            for _ in 0..count {
                named(&mut g, p, "Arc Reactor", Zone::Library);
            }
            activate(&mut g, p, id, 0, [0, 0, 0, 0, 0, 1]);
            until(&mut g, |g| {
                g.observe(p).library_selection.is_some()
                    || g.state.resolving.is_none() && g.state.stack.is_empty()
            });
            if g.observe(p).library_selection.is_some() {
                checked(&mut g, Action::FinishLibrarySelection);
            }
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].library.len(), count);
            priority(&mut g, p);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
            reject(
                &mut g,
                Action::AnnounceActivate {
                    card: id,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
        }
    }
}
#[test]
fn msh_armor_tony_paid_combat_optional_artifact_equipment_attaches_after_entry() {
    for p in [Player::P0, Player::P1] {
        for equip in [false, true] {
            let mut g = setup(p);
            let id = paid(&mut g, p, TONY, true, &[]);
            let n = named(
                &mut g,
                p,
                if equip {
                    "Captain America's Shield"
                } else {
                    "Arc Reactor"
                },
                Zone::Hand,
            );
            let wrong = named(&mut g, p, "Bear Cub", Zone::Hand);
            combat(&mut g, p);
            assert!(g.observe(p.other()).effect_choice.is_none());
            reject(
                &mut g,
                Action::PutHandPermanent {
                    object: g.object_ref(wrong),
                },
            );
            let r = g.object_ref(n);
            g = orphan(g, Action::PutHandPermanent { object: r });
            settle(&mut g);
            assert_eq!(g.object_view(n).zone, Zone::Battlefield);
            assert_eq!(
                g.object_view(n).attached_to,
                if equip { Some(g.object_ref(id)) } else { None }
            );
            assert_eq!(g.object_view(id).toughness, if equip { 13 } else { 5 });
        }
    }
}
#[test]
fn msh_armor_tony_paid_combat_source_gone_blink_control_and_captured_hand_owner() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = setup(p);
            let id = paid(&mut g, p, TONY, true, &[]);
            let n = named(&mut g, p, "Captain America's Shield", Zone::Hand);
            g.state.active = p;
            g.begin_step(Step::BeginCombat);
            until(&mut g, |g| g.state.stack.iter().any(|s| s.card == id));
            match mode {
                0 => {
                    g.move_object(id, Zone::Graveyard);
                }
                1 => {
                    g.move_object(id, Zone::Exile);
                    g.move_object(id, Zone::Battlefield);
                }
                _ => {
                    g.change_controller(id, p.other());
                }
            }
            choice(&mut g, p);
            let r = g.object_ref(n);
            checked(&mut g, Action::PutHandPermanent { object: r });
            settle(&mut g);
            assert_eq!(g.object_view(n).controller, p);
            assert_eq!(
                g.object_view(n).attached_to,
                if mode == 2 {
                    Some(g.object_ref(id))
                } else {
                    None
                }
            );
        }
    }
}
#[test]
fn msh_armor_tony_paid_combat_private_empty_phase_and_opponent_step_no_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut a = setup(p);
        let mut b = setup(p);
        paid(&mut a, p, TONY, true, &[]);
        paid(&mut b, p, TONY, true, &[]);
        named(&mut a, p, "Captain America's Shield", Zone::Hand);
        named(&mut b, p, "Bear Cub", Zone::Hand);
        combat(&mut a, p);
        combat(&mut b, p);
        assert_eq!(
            serde_json::to_value(a.observe(p.other())).unwrap(),
            serde_json::to_value(b.observe(p.other())).unwrap()
        );
        assert!(b.observe(p).effect_choice.unwrap().candidates.is_empty());
        checked(&mut a, Action::DeclineHandEntry);
        checked(&mut b, Action::DeclineHandEntry);
        settle(&mut a);
        settle(&mut b);
        a.state.active = p.other();
        a.begin_step(Step::BeginCombat);
        settle(&mut a);
        assert!(a.observe(p).effect_choice.is_none());
    }
}
