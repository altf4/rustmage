//! Paid printed producer scenarios; explicit primitive seams are labeled.
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
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
pub(super) fn paid(g: &mut Game, p: Player, name: &str, back: bool, targets: &[Target]) -> CardId {
    priority(g, p);
    let card = Card::from_name(name).unwrap();
    assert!(card.is_supported());
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
            let wrong_ref = g.object_ref(wrong);
            reject(&mut g, Action::PutHandPermanent { object: wrong_ref });
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
                    g.change_controller(g.object_ref(id), p.other());
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

const ARMOR: &str = "Iron Man Armor";
pub(super) fn armor(g: &mut Game, p: Player, target: Option<CardId>) -> CardId {
    priority(g, p);
    let c = Card::from_name(ARMOR).unwrap();
    let id = named(g, p, ARMOR, Zone::Hand);
    assert!(c.is_supported());
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
    cast(g, id, None, &[]);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    if let Some(t) = target {
        until(g, |g| {
            g.state
                .announcing
                .as_ref()
                .is_some_and(|a| a.item.card == id)
        });
        checked(
            g,
            Action::ChooseTarget {
                target: Target::Card(t),
            },
        );
        pay(g);
    }
    settle(g);
    id
}
pub(super) fn animate(g: &mut Game, p: Player, id: CardId) {
    activate(g, p, id, 0, [0, 0, 0, 0, 0, 2]);
    settle(g);
}
#[test]
fn msh_armor_tony_armor_paid_entry_target_own_creature_and_equipped_flying_stats() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let a = armor(&mut g, p, Some(bear));
        assert_eq!(g.object_view(a).attached_to, Some(g.object_ref(bear)));
        assert_eq!(
            (g.object_view(bear).power, g.object_view(bear).toughness),
            (4, 3)
        );
        assert!(g.keywords(bear).contains(Keywords::FLYING));
        assert!(!g.object_view(a).is_creature());
        let other = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: a,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(other),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(a).attached_to, Some(g.object_ref(bear)));
    }
}
#[test]
fn msh_armor_tony_armor_paid_animation_retains_equipment_detaches_and_expires() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let a = armor(&mut g, p, Some(bear));
        animate(&mut g, p, a);
        let v = g.object_view(a);
        assert!(v.is_creature() && v.card.is_artifact());
        assert!(v.has_subtype("Construct") && v.has_subtype("Hero") && v.has_subtype("Equipment"));
        assert!(v.keywords.contains(Keywords::FLYING));
        assert_eq!((v.power, v.toughness), (1, 1));
        assert_eq!(v.attached_to, None);
        assert_eq!(
            g.stats(bear),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        let row = g.self_granted_ability_views().unwrap()[0];
        assert_eq!(
            row.timestamp,
            g.state
                .roles
                .as_ref()
                .unwrap()
                .animations
                .last()
                .unwrap()
                .timestamp
        );
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: a,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(a).attached_to, None);
        next_turn(&mut g);
        assert!(!g.object_view(a).is_creature());
        assert!(g.state.self_granted_abilities.is_none());
        assert_eq!(g.object_view(a).attached_to, None);
    }
}
#[test]
fn msh_armor_tony_armor_paid_resolution_only_condition_allows_repeat_without_duplicate_grant() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = armor(&mut g, p, None);
        activate(&mut g, p, a, 0, [0, 0, 0, 0, 0, 2]);
        activate(&mut g, p, a, 0, [0, 0, 0, 0, 0, 2]);
        settle(&mut g);
        assert_eq!(g.state.self_granted_abilities.as_ref().unwrap().len(), 1);
        let before = g.state.roles.as_ref().unwrap().animations.len();
        animate(&mut g, p, a);
        assert_eq!(g.state.roles.as_ref().unwrap().animations.len(), before);
        assert_eq!(g.state.self_granted_abilities.as_ref().unwrap().len(), 1);
    }
}
#[test]
fn msh_armor_tony_armor_paid_live_artifact_count_control_and_source_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = armor(&mut g, p, None);
        animate(&mut g, p, a);
        let r = paid(&mut g, p, "Arc Reactor", false, &[]);
        assert_eq!(
            g.stats(a),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        g.move_object(r, Zone::Graveyard);
        assert_eq!(
            g.stats(a),
            Stats {
                power: 1,
                toughness: 1
            }
        );
        let r = named(&mut g, p.other(), "Arc Reactor", Zone::Battlefield);
        g.change_controller(g.object_ref(a), p.other());
        assert_eq!(
            g.stats(a),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        assert_eq!(g.object_view(a).controller, p.other());
        g.move_object(a, Zone::Hand);
        assert!(g.state.self_granted_abilities.is_none());
        g.move_object(a, Zone::Battlefield);
        assert!(!g.object_view(a).is_creature());
        assert_eq!(g.object_view(r).controller, p.other());
    }
}
#[test]
fn msh_armor_tony_armor_paid_ability_removal_suppresses_bonus_without_undoing_type() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = armor(&mut g, p, None);
        let _ = paid(&mut g, p, "Arc Reactor", false, &[]);
        animate(&mut g, p, a);
        assert_eq!(
            g.stats(a),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        let u = paid(&mut g, p, "Witness Protection", false, &[Target::Card(a)]);
        assert!(g.object_view(a).is_creature());
        assert_eq!(
            g.stats(a),
            Stats {
                power: 1,
                toughness: 1
            }
        );
        assert!(!g.keywords(a).contains(Keywords::FLYING));
        assert!(!g.self_granted_ability_views().unwrap()[0].available);
        paid(&mut g, p, "Disenchant", false, &[Target::Card(u)]);
        assert_eq!(
            g.stats(a),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        assert!(g.self_granted_ability_views().unwrap()[0].available);
        assert!(g.keywords(a).contains(Keywords::FLYING));
    }
}
#[test]
fn msh_armor_tony_armor_paid_later_copy_preserves_resolved_granted_ability() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = armor(&mut g, p, None);
        let r = paid(&mut g, p, "Arc Reactor", false, &[]);
        animate(&mut g, p, a);
        let shuri = paid(&mut g, p, "Shuri, Wakandan Inventor", false, &[]);
        g.state.objects[shuri as usize].summoning_sick = false;
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: shuri,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(a),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(r),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(a).name(), "Arc Reactor");
        assert!(g.object_view(a).is_creature());
        assert_eq!(
            g.stats(a),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        assert!(g.self_granted_ability_views().unwrap()[0].available);
        assert!(
            g.copiable_recipe(g.object_ref(a)).definition
                == Card::from_name("Arc Reactor").unwrap()
        );
    }
}
#[test]
fn msh_armor_tony_armor_paid_short_payment_foreign_and_source_stale_are_atomic() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = armor(&mut g, p, None);
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0; 6];
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: a,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        activate(&mut g, p, a, 0, [0, 0, 0, 0, 0, 2]);
        g.move_object(a, Zone::Hand);
        g.move_object(a, Zone::Battlefield);
        settle(&mut g);
        assert!(!g.object_view(a).is_creature());
        assert!(g.state.self_granted_abilities.is_none());
    }
}

#[test]
fn msh_armor_tony_paid_equipment_entry_target_restriction_and_source_leave_response() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        priority(&mut g, p);
        let card = Card::from_name(ARMOR).unwrap();
        let a = named(&mut g, p, ARMOR, Zone::Hand);
        assert!(card.is_supported());
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        cast(&mut g, a, None, &[]);
        until(&mut g, |g| {
            g.state
                .announcing
                .as_ref()
                .is_some_and(|n| n.item.card == a)
        });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        pay(&mut g);
        priority(&mut g, p);
        let d = named(&mut g, p, "Disenchant", Zone::Hand);
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 1];
        cast(&mut g, d, None, &[Target::Card(a)]);
        settle(&mut g);
        assert_eq!(g.object_view(a).zone, Zone::Graveyard);
        assert_eq!(
            g.stats(own),
            Stats {
                power: 2,
                toughness: 2
            }
        );
    }
}
#[test]
fn msh_armor_tony_paid_equipment_recipient_protection_ward_and_noncreature_boundaries() {
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
                    g.state.objects[id as usize].gained_keywords = Keywords::HEXPROOF;
                }
                1 => {
                    g.state.objects[id as usize].gained_keywords = Keywords::PROTECTION_EVERYTHING;
                }
                _ => {
                    // Bounded exact-recipient type seam while the genuinely paid trigger is queued.
                    g.state.objects[id as usize].card = Card::from_name("Arc Reactor").unwrap();
                }
            }
            choice(&mut g, p);
            let object = g.object_ref(n);
            checked(&mut g, Action::PutHandPermanent { object });
            settle(&mut g);
            assert_eq!(g.object_view(n).zone, Zone::Battlefield);
            assert_eq!(g.object_view(n).attached_to.is_some(), mode == 0);
        }
    }
}
#[test]
fn msh_armor_tony_paid_equipment_entry_replacement_pause_preserves_exact_callback() {
    for p in [Player::P0, Player::P1] {
        for redirected in [false, true] {
            let mut g = setup(p);
            let id = paid(&mut g, p, TONY, true, &[]);
            let n = named(&mut g, p, "Adaptive Automaton", Zone::Hand);
            combat(&mut g, p);
            // Automaton provides a genuine entry-value pause. The real paid Tony
            // trigger owns its callback; as an artifact creature it enters unattached.
            // The alternate case redirects its staged move through a bounded replacement seam.
            let r = g.object_ref(n);
            g.choose_effect_object(p, r);
            assert!(g.advance_entries());
            let recipient = g.object_ref(id);
            let batch = g.state.entry_batch.as_mut().unwrap();
            assert_eq!(batch.moves[0].attach_equipment_to, Some(recipient));
            if redirected {
                batch.moves[0].destination = Zone::Exile;
            }
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::ChooseEntryValue { .. }))
                .unwrap();
            g = orphan(g, a);
            settle(&mut g);
            assert_eq!(
                g.object_view(n).zone,
                if redirected {
                    Zone::Exile
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(g.object_view(n).attached_to, None);
        }
    }
}
#[test]
fn msh_armor_tony_paid_hand_entry_mdfc_front_facts_and_hidden_choice_atomicity() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, TONY, true, &[]);
        let n = named(&mut g, p, "Monica Rambeau", Zone::Hand);
        combat(&mut g, p);
        let r = g.object_ref(n);
        reject(&mut g, Action::PutHandPermanent { object: r });
        checked(&mut g, Action::DeclineHandEntry);
        settle(&mut g);
        assert_eq!(g.object_view(n).zone, Zone::Hand);
        assert_eq!(g.object_view(id).name(), IRON);
    }
}
#[test]
fn msh_armor_tony_paid_transform_copy_token_uses_physical_modal_structure_and_face_mana() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, TONY, false, &[]);
        priority(&mut g, p);
        let m = named(&mut g, p, "Molten Duplication", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 1];
        cast(&mut g, m, None, &[Target::Card(id)]);
        g.prepare_triggers();
        for _ in 0..30 {
            if g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
                && g.state.resolving.is_none()
            {
                break;
            }
            let actions = g.legal_actions();
            let a=actions.iter().find(|a|matches!(a,Action::KeepLegend{object}if g.object_view(object.card).is_token())).or_else(||actions.iter().find(|a|matches!(a,Action::Pass))).or_else(||actions.iter().find(|a|!matches!(a,Action::Concede))).unwrap().clone();
            checked(&mut g, a);
        }
        let copy = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| o.zone == Zone::Battlefield && o.is_copy_token())
            .unwrap()
            .0 as CardId;
        assert!(g.object_view(copy).face.is_some());
        activate(&mut g, p, copy, 1, [0, 1, 0, 1, 0, 4]);
        settle(&mut g);
        assert_eq!(g.object_view(copy).name(), IRON);
        assert_eq!(
            g.copiable_recipe(g.object_ref(copy))
                .definition
                .fixed_mana_value(),
            6
        );
        assert!(g.object_view(copy).face.unwrap().back);
    }
}

#[test]
fn msh_armor_tony_paid_armor_live_toughness_enables_real_baxter_draw_activation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = armor(&mut g, p, None);
        for _ in 0..3 {
            paid(&mut g, p, "Arc Reactor", false, &[]);
        }
        let b = named(&mut g, p, "Baxter Building", Zone::Hand);
        checked(&mut g, Action::PlayLand { card: b });
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: b,
                ability: 2,
                counter_payment: Counters::default(),
            },
        );
        animate(&mut g, p, a);
        assert_eq!(
            g.stats(a),
            Stats {
                power: 4,
                toughness: 4
            }
        );
        let before = g.hand_count(p);
        activate(&mut g, p, b, 2, [0, 0, 0, 0, 0, 4]);
        settle(&mut g);
        assert!(g.object_view(b).tapped);
        assert_eq!(g.hand_count(p), before + 1);
    }
}
#[test]
fn msh_armor_tony_paid_back_haste_allows_same_turn_actual_attack() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, TONY, true, &[]);
        assert!(g.state.objects[id as usize].summoning_sick);
        combat(&mut g, p);
        checked(&mut g, Action::DeclineHandEntry);
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        checked(&mut g, Action::Attack { card: id });
        checked(&mut g, Action::FinishAttackers);
        assert!(g.object_view(id).tapped);
        assert!(g.state.attackers.iter().any(|a| a.card == id));
    }
}
#[test]
fn msh_armor_tony_paid_entry_attaches_before_armor_own_targeted_trigger_resolves() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, TONY, true, &[]);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let c = Card::from_name(ARMOR).unwrap();
        let a = named(&mut g, p, ARMOR, Zone::Hand);
        assert!(c.is_supported());
        combat(&mut g, p);
        let object = g.object_ref(a);
        checked(&mut g, Action::PutHandPermanent { object });
        assert_eq!(g.object_view(a).attached_to, Some(g.object_ref(id)));
        assert_eq!(
            g.stats(id),
            Stats {
                power: 7,
                toughness: 6
            }
        );
        until(&mut g, |g| {
            g.state
                .announcing
                .as_ref()
                .is_some_and(|x| x.item.card == a)
        });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(a).attached_to, Some(g.object_ref(bear)));
        assert_eq!(
            g.stats(id),
            Stats {
                power: 5,
                toughness: 5
            }
        );
        assert_eq!(
            g.stats(bear),
            Stats {
                power: 4,
                toughness: 3
            }
        );
    }
}
#[test]
fn msh_armor_tony_paid_taskmaster_single_face_copy_can_pay_transform_without_gaining_back() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let task = paid(&mut g, p, "Taskmaster, Mercenary Mimic", false, &[]);
        let t = paid(&mut g, p, TONY, false, &[]);
        let turn = g.state.turn;
        for _ in 0..240 {
            if g.state.turn > turn && g.state.active == p && g.state.step == Step::Main1 {
                break;
            }
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::Pass
                            | Action::FinishAttackers
                            | Action::FinishBlockers
                            | Action::Discard { .. }
                            | Action::PutTrigger { .. }
                            | Action::FinishTargets
                    )
                })
                .unwrap();
            checked(&mut g, a);
        }
        until(&mut g, |g| {
            g.state
                .announcing
                .as_ref()
                .is_some_and(|a| a.item.card == task)
        });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(t),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(task).name(), "Taskmaster, Mercenary Mimic");
        assert!(g.object_view(task).face.is_none());
        activate(&mut g, p, task, 1, [0, 1, 0, 1, 0, 4]);
        settle(&mut g);
        assert_eq!(g.object_view(task).name(), "Taskmaster, Mercenary Mimic");
        assert!(g.object_view(task).face.is_none());
        assert_eq!(g.object_view(task).card.fixed_mana_value(), 2);
    }
}
