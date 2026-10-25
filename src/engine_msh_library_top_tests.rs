//! Paid whole-card outcomes, information boundaries and owned operation rollback.
use super::ability_world::CharacteristicWorld;
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::*;
const LAD: &str = "Iron Lad, Diverging Destiny";
const DEVIL: &str = "Daredevil, Man Without Fear";
fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn finish(g: &mut Game) {
    for _ in 0..20 {
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
    panic!("unpaid library-top action");
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    priority(g, p);
    let id = named(g, p, name, Zone::Hand);
    let (colored, generic) = g.object_view(id).card.cost();
    g.state.players[p.index()].mana = colored;
    g.state.players[p.index()].mana[5] += generic;
    cast(g, id, None, targets);
    settle(g);
    id
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..180 {
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
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                        | Action::FinishTargets
                        | Action::PutTrigger { .. }
                )
            })
            .or_else(|| {
                g.legal_actions()
                    .into_iter()
                    .find(|a| !matches!(a, Action::Concede))
            })
            .unwrap();
        checked(g, a);
    }
    panic!("library-top continuation stalled {:?}", g.state.pending);
}
fn ready(g: &mut Game, p: Player) {
    next_turn(g);
    next_turn(g);
    until(g, |g| {
        g.state.active == p && g.state.step == Step::Main1 && g.actor() == Some(p)
    });
}
fn activate(g: &mut Game, p: Player, id: CardId) {
    priority(g, p);
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    finish(g);
}
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn observation(g: &Game, p: Player) -> String {
    serde_json::to_string(&g.observe(p)).unwrap()
}
fn top(g: &Game, p: Player) -> Option<ObjectRef> {
    g.observe(p).known_library_top[p.index()].map(|v| v.object)
}
fn declare(g: &mut Game, p: Player, attackers: &[CardId]) {
    until(g, |g| {
        g.state.pending == Pending::Attackers && g.state.active == p
    });
    for &card in attackers {
        checked(g, Action::Attack { card });
    }
    checked(g, Action::FinishAttackers);
}
fn optional(g: &mut Game) {
    until(g, |g| {
        matches!(g.state.pending, Pending::EffectOptional { .. })
    });
}
fn reveal(g: &Game) -> LibraryRevealEvent {
    g.state
        .library_top
        .as_ref()
        .unwrap()
        .reveal
        .clone()
        .unwrap()
}
#[test]
fn msh_library_top_paid_bodies_keywords_and_private_no_priority_look() {
    for p in [Player::P0, Player::P1] {
        for name in [LAD, DEVIL] {
            let mut g = setup(p);
            let id = paid(&mut g, p, name, &[]);
            let x = named(&mut g, p, "Shock", Zone::Library);
            let other = named(&mut g, p.other(), "Murder", Zone::Library);
            assert_eq!(top(&g, p), Some(g.object_ref(x)));
            assert!(g.observe(p.other()).known_library_top[p.index()].is_none());
            assert!(g.observe(p).known_library_top[p.other().index()].is_none());
            priority(&mut g, p.other());
            assert_eq!(top(&g, p), Some(g.object_ref(x)));
            assert_eq!(g.object_view(id).name(), name);
            assert!(g.keywords(id).contains(Keywords::VIGILANCE));
            assert!(g.keywords(id).contains(if name == LAD {
                Keywords::FLYING
            } else {
                Keywords::HASTE
            }));
            assert_ne!(x, other);
        }
    }
}
#[test]
fn msh_library_top_look_control_transfer_suppression_and_multiple_providers() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = paid(&mut g, p, LAD, &[]);
        let b = paid(&mut g, p, DEVIL, &[]);
        let mine = named(&mut g, p, "Shock", Zone::Library);
        let enemy = named(&mut g, p.other(), "Murder", Zone::Library);
        g.change_controller(g.object_ref(a), p.other());
        assert_eq!(top(&g, p), Some(g.object_ref(mine)));
        assert_eq!(top(&g, p.other()), Some(g.object_ref(enemy)));
        g.move_object(b, Zone::Hand);
        assert!(top(&g, p).is_none());
        priority(&mut g, p);
        paid(&mut g, p, "Stop Cold", &[Target::Card(a)]);
        assert!(top(&g, p.other()).is_none());
    }
}
#[test]
fn msh_library_top_lad_sickness_tap_cost_and_vigilance() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, LAD, &[]);
        priority(&mut g, p);
        let action = Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        };
        reject(&mut g, action.clone());
        ready(&mut g, p);
        named(&mut g, p, "Forest", Zone::Library);
        activate(&mut g, p, id);
        assert!(g.state.objects[id as usize].tapped);
        reject(&mut g, action);
        settle(&mut g);
        assert_eq!(reveal(&g).revealed.card.name(), "Forest");
    }
}
#[test]
fn msh_library_top_lad_artifact_draw_is_public_and_ordinary() {
    for p in [Player::P0, Player::P1] {
        for name in ["Gilded Lotus", LAD, "Prophetic Prism"] {
            let mut g = setup(p);
            let id = paid(&mut g, p, LAD, &[]);
            ready(&mut g, p);
            let next = named(&mut g, p, "Murder", Zone::Library);
            let x = named(&mut g, p, name, Zone::Library);
            let before = g.hand_count(p);
            let draws = g.state.players[p.index()].cards_drawn_this_turn;
            activate(&mut g, p, id);
            settle(&mut g);
            assert_eq!(g.hand_count(p), before + 1);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, draws + 1);
            let event = reveal(&g);
            assert_eq!(event.revealed.card.name(), name);
            assert_eq!(event.drawn, Some(g.object_ref(x)));
            assert_eq!(
                g.observe(p).last_library_reveal,
                g.observe(p.other()).last_library_reveal
            );
            assert!(g.observe(p.other()).known_hand.contains(&g.object_ref(x)));
            assert!(g.observe(p.other()).known_library_top[p.index()].is_none());
            assert_eq!(top(&g, p), Some(g.object_ref(next)));
        }
    }
}
#[test]
fn msh_library_top_lad_nonartifact_reveal_then_shuffle_invalidates_top() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, LAD, &[]);
        ready(&mut g, p);
        let x = named(&mut g, p, "Murder", Zone::Library);
        let before = g.hand_count(p);
        activate(&mut g, p, id);
        settle(&mut g);
        assert_eq!(g.hand_count(p), before);
        assert!(reveal(&g).drawn.is_none());
        assert_eq!(
            g.observe(p.other()).known_library_top[p.index()]
                .unwrap()
                .object,
            g.object_ref(x)
        );
        g.forget_library_top(p);
        g.state.players[p.index()].library.reverse();
        assert!(g.observe(p.other()).known_library_top[p.index()].is_none());
        assert_eq!(reveal(&g).revealed.card.name(), "Murder");
    }
}
#[test]
fn msh_library_top_lad_empty_library_does_not_attempt_draw() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, LAD, &[]);
        ready(&mut g, p);
        let ids = g.state.players[p.index()].library.to_vec();
        for x in ids {
            g.move_object(x, Zone::Exile);
        }
        activate(&mut g, p, id);
        settle(&mut g);
        assert!(!g.state.players[p.index()].attempted_empty_draw);
        assert!(g.outcome().is_none());
        assert!(g.observe(p).last_library_reveal.is_none());
    }
}
#[test]
fn msh_library_top_lad_reads_resolution_top_after_response() {
    for p in [Player::P0, Player::P1] {
        for artifact in [false, true] {
            let mut g = setup(p);
            let id = paid(&mut g, p, LAD, &[]);
            ready(&mut g, p);
            named(
                &mut g,
                p,
                if artifact { "Forest" } else { "Gilded Lotus" },
                Zone::Library,
            );
            activate(&mut g, p, id);
            let x = named(
                &mut g,
                p,
                if artifact { "Gilded Lotus" } else { "Forest" },
                Zone::Library,
            );
            settle(&mut g);
            assert_eq!(reveal(&g).revealed.object.card, x);
            assert_eq!(
                g.state.objects[x as usize].zone,
                if artifact { Zone::Hand } else { Zone::Library }
            );
        }
    }
}
#[test]
fn msh_library_top_lad_departure_blink_theft_keep_captured_controller() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = setup(p);
            let id = paid(&mut g, p, LAD, &[]);
            ready(&mut g, p);
            let x = named(&mut g, p, "Gilded Lotus", Zone::Library);
            activate(&mut g, p, id);
            let old = g.object_ref(id);
            if mode == 2 {
                g.change_controller(old, p.other());
            } else {
                g.move_object(id, Zone::Exile);
                if mode == 1 {
                    g.move_object(id, Zone::Battlefield);
                }
            }
            settle(&mut g);
            assert_eq!(reveal(&g).source, old);
            assert_eq!(reveal(&g).player, p);
            assert_eq!(g.state.objects[x as usize].zone, Zone::Hand);
        }
    }
}
#[test]
fn msh_library_top_lad_borrowed_tap_activation_uses_forge_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let lad = named(&mut g, p.other(), LAD, Zone::Battlefield);
        let forge = named(&mut g, p, "Territory Forge", Zone::Hand);
        g.state.players[p.index()].mana = [30; 6];
        cast(&mut g, forge, None, &[]);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(lad),
            })
        });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(lad),
            },
        );
        settle(&mut g);
        let b = g
            .borrowed_ability_views()
            .into_iter()
            .find(|b| b.source.card == forge && b.ability == 0)
            .unwrap();
        let x = named(&mut g, p, "Gilded Lotus", Zone::Library);
        priority(&mut g, p);
        checked(
            &mut g,
            Action::AnnounceBorrowedAbility {
                card: forge,
                exiled: b.exiled,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        finish(&mut g);
        settle(&mut g);
        assert_eq!(reveal(&g).source.card, forge);
        assert_eq!(reveal(&g).player, p);
        assert_eq!(g.state.objects[x as usize].zone, Zone::Hand);
    }
}
#[test]
fn msh_library_top_revealed_draw_owned_orphan_and_restore() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, LAD, &[]);
        ready(&mut g, p);
        named(&mut g, p, "Gilded Lotus", Zone::Library);
        activate(&mut g, p, id);
        checked(&mut g, Action::Pass);
        let before = g.snapshot();
        let actor = g.actor().unwrap();
        let rev = g.revision();
        let at = g
            .legal_actions()
            .iter()
            .position(|a| *a == Action::Pass)
            .unwrap();
        let mut raw = g.clone();
        let mut indexed = g.clone();
        drop(g);
        raw.apply(actor, Action::Pass).unwrap();
        indexed.apply_index(actor, rev, at).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        let result = raw.snapshot();
        raw.restore(&before).unwrap();
        assert_eq!(raw.snapshot(), before);
        checked(&mut raw, Action::Pass);
        assert_eq!(raw.snapshot(), result);
    }
}
#[test]
fn msh_library_top_daredevil_haste_vigilance_and_self_attack_boost() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, DEVIL, &[]);
        let x = named(&mut g, p, LAD, Zone::Library);
        declare(&mut g, p, &[id]);
        assert!(!g.state.objects[id as usize].tapped);
        optional(&mut g);
        checked(&mut g, Action::ChooseOptional { accept: true });
        settle(&mut g);
        assert_eq!((g.stats(id).power, g.stats(id).toughness), (5, 5));
        assert!(g.can_play_exiled(p, x));
    }
}
#[test]
fn msh_library_top_daredevil_other_attackers_once_per_declaration() {
    for p in [Player::P0, Player::P1] {
        for count in [1, 2] {
            let mut g = setup(p);
            let id = paid(&mut g, p, DEVIL, &[]);
            let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let b = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let x = named(&mut g, p, "Forest", Zone::Library);
            let attackers = [a, b];
            declare(&mut g, p, &attackers[..count]);
            optional(&mut g);
            checked(&mut g, Action::ChooseOptional { accept: true });
            settle(&mut g);
            assert_eq!(
                g.state
                    .exile_permissions
                    .iter()
                    .filter(|e| e.player == p)
                    .count(),
                1
            );
            assert!(g.can_play_exiled(p, x));
            assert_eq!(g.stats(id).power, 3);
        }
    }
}
#[test]
fn msh_library_top_daredevil_decline_and_empty_optional() {
    for p in [Player::P0, Player::P1] {
        for empty in [false, true] {
            let mut g = setup(p);
            let id = paid(&mut g, p, DEVIL, &[]);
            if empty {
                let ids = g.state.players[p.index()].library.to_vec();
                for x in ids {
                    g.move_object(x, Zone::Exile);
                }
            }
            let before = g.state.players[p.index()].library.clone();
            declare(&mut g, p, &[id]);
            optional(&mut g);
            checked(&mut g, Action::ChooseOptional { accept: empty });
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].library, before);
            assert!(g.state.exile_permissions.is_empty());
            assert_eq!(g.stats(id).power, 3);
        }
    }
}
#[test]
fn msh_library_top_daredevil_kindred_changeling_is_hero_without_creature_type() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, DEVIL, &[]);
        // A gated Kindred card supplies intrinsic metadata only; no unrelated behavior is admitted.
        let x = named(&mut g, p, "Forest", Zone::Library);
        g.state.objects[x as usize].card = Card::from_name("Nameless Inversion").unwrap();
        assert!(!g.state.objects[x as usize].card.is_creature());
        assert!(g.state.objects[x as usize].card.has_subtype("Hero"));
        declare(&mut g, p, &[id]);
        optional(&mut g);
        checked(&mut g, Action::ChooseOptional { accept: true });
        settle(&mut g);
        assert_eq!(g.stats(id).power, 5);
        assert!(g.can_play_exiled(p, x));
    }
}
#[test]
fn msh_library_top_daredevil_source_departure_blink_and_theft() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = setup(p);
            let id = paid(&mut g, p, DEVIL, &[]);
            let x = named(&mut g, p, LAD, Zone::Library);
            let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            declare(&mut g, p, &[a]);
            optional(&mut g);
            let old = g.object_ref(id);
            if mode == 2 {
                g.change_controller(old, p.other());
            } else {
                g.move_object(id, Zone::Exile);
                if mode == 1 {
                    g.move_object(id, Zone::Battlefield);
                }
            }
            checked(&mut g, Action::ChooseOptional { accept: true });
            settle(&mut g);
            assert!(g.can_play_exiled(p, x));
            assert!(!g.can_play_exiled(p.other(), x));
            if mode != 0 {
                assert_eq!(g.stats(id).power, if mode == 2 { 5 } else { 3 });
            }
        }
    }
}
#[test]
fn msh_library_top_daredevil_paid_exile_instant_and_sorcery_timing() {
    for p in [Player::P0, Player::P1] {
        for instant in [false, true] {
            let mut g = setup(p);
            let id = paid(&mut g, p, DEVIL, &[]);
            let x = named(
                &mut g,
                p,
                if instant { "Shock" } else { "Divination" },
                Zone::Library,
            );
            declare(&mut g, p, &[id]);
            optional(&mut g);
            checked(&mut g, Action::ChooseOptional { accept: true });
            settle(&mut g);
            priority(&mut g, p);
            g.state.players[p.index()].mana = [0; 6];
            reject(&mut g, Action::AnnounceExile { card: x });
            g.state.players[p.index()].mana = if instant {
                [0, 0, 0, 1, 0, 0]
            } else {
                [0, 1, 0, 0, 0, 2]
            };
            if instant {
                checked(&mut g, Action::AnnounceExile { card: x });
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Player(p.other()),
                    },
                );
                finish(&mut g);
                assert_eq!(
                    g.state.stack.last().unwrap().cast.unwrap().origin,
                    Zone::Exile
                );
                settle(&mut g);
                assert_eq!(g.life(p.other()), 18);
            } else {
                reject(&mut g, Action::AnnounceExile { card: x });
                until(&mut g, |g| {
                    g.state.step == Step::Main2 && g.actor() == Some(p)
                });
                g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 2];
                assert_eq!(g.state.active, p);
                assert!(
                    g.state.stack.is_empty()
                        && g.state.resolving.is_none()
                        && g.state.announcing.is_none()
                );
                checked(&mut g, Action::AnnounceExile { card: x });
                finish(&mut g);
                settle(&mut g);
            }
        }
    }
}
#[test]
fn msh_library_top_daredevil_land_permission_normal_allowance_and_source_independence() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, DEVIL, &[]);
        let x = named(&mut g, p, "Forest", Zone::Library);
        declare(&mut g, p, &[id]);
        optional(&mut g);
        checked(&mut g, Action::ChooseOptional { accept: true });
        settle(&mut g);
        priority(&mut g, p);
        reject(&mut g, Action::PlayLand { card: x });
        g.move_object(id, Zone::Exile);
        until(&mut g, |g| {
            g.state.step == Step::Main2 && g.actor() == Some(p)
        });
        checked(&mut g, Action::PlayLand { card: x });
        assert_eq!(g.state.objects[x as usize].zone, Zone::Battlefield);
        let y = named(&mut g, p, "Forest", Zone::Hand);
        reject(&mut g, Action::PlayLand { card: y });
    }
}
#[test]
fn msh_library_top_daredevil_generation_and_cleanup_expiry() {
    for p in [Player::P0, Player::P1] {
        for moved in [false, true] {
            let mut g = setup(p);
            let id = paid(&mut g, p, DEVIL, &[]);
            let x = named(&mut g, p, LAD, Zone::Library);
            declare(&mut g, p, &[id]);
            optional(&mut g);
            checked(&mut g, Action::ChooseOptional { accept: true });
            settle(&mut g);
            let old = g.object_ref(x);
            if moved {
                g.move_object(x, Zone::Graveyard);
                g.move_object(x, Zone::Exile);
                assert_ne!(g.object_ref(x), old);
                assert!(!g.can_play_exiled(p, x));
            } else {
                next_turn(&mut g);
                assert!(!g.can_play_exiled(p, x));
                assert_eq!(g.stats(id).power, 3);
            }
        }
    }
}
#[test]
fn msh_library_top_library_cast_defers_new_card_until_costs_finish() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, LAD, &[]);
        paid(&mut g, p, "Vizier of the Menagerie", &[]);
        let below = named(&mut g, p, "Shock", Zone::Library);
        let x = named(&mut g, p, "Bear Cub", Zone::Library);
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
        checked(&mut g, Action::AnnounceCast { card: x });
        assert!(top(&g, p).is_none());
        assert!(g.state.library_top.as_ref().unwrap().operation.is_some());
        finish(&mut g);
        assert_eq!(top(&g, p), Some(g.object_ref(below)));
        settle(&mut g);
    }
}
#[test]
fn msh_library_top_unchanged_top_visible_during_hand_cast_and_activation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, LAD, &[]);
        ready(&mut g, p);
        let x = named(&mut g, p, "Forest", Zone::Library);
        let spell = named(&mut g, p, "Bear Cub", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
        checked(&mut g, Action::AnnounceCast { card: spell });
        assert_eq!(top(&g, p), Some(g.object_ref(x)));
        finish(&mut g);
        settle(&mut g);
        activate(&mut g, p, id);
        assert_eq!(top(&g, p), Some(g.object_ref(x)));
        settle(&mut g);
    }
}
#[test]
fn msh_library_top_activation_changed_top_snapshot_is_owned_and_private() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, LAD, &[]);
        let a = named(&mut g, p, "Forest", Zone::Library);
        let b = named(&mut g, p, "Gilded Lotus", Zone::Library);
        let source = named(&mut g, p, "Collector's Vault", Zone::Battlefield);
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let start = g
            .state
            .library_top
            .as_ref()
            .unwrap()
            .operation
            .as_ref()
            .unwrap()
            .starting_top;
        assert_eq!(start[p.index()], Some(g.object_ref(b)));
        g.move_object(b, Zone::Exile);
        assert!(top(&g, p).is_none());
        let mut before = g.observe(p.other());
        let saved = g.snapshot();
        let mut orphan = g.clone();
        drop(g);
        assert!(top(&orphan, p).is_none());
        finish(&mut orphan);
        assert_eq!(top(&orphan, p), Some(orphan.object_ref(a)));
        orphan.restore(&saved).unwrap();
        assert_eq!(orphan.snapshot(), saved);
        before.revision = orphan.revision();
        assert_eq!(
            serde_json::to_value(orphan.observe(p.other())).unwrap(),
            serde_json::to_value(before).unwrap()
        );
        assert!(top(&orphan, p).is_none());
    }
}
#[test]
fn msh_library_top_hidden_libraries_full_nonactor_observation_matches() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, LAD, &[]);
        let x = named(&mut g, p, "Shock", Zone::Library);
        let y = named(&mut g, p, "Murder", Zone::Library);
        let before = observation(&g, p.other());
        let n = g.state.players[p.index()].library.len();
        g.state.players[p.index()].library.swap(n - 1, n - 2);
        assert_eq!(observation(&g, p.other()), before);
        assert_eq!(top(&g, p), Some(g.object_ref(x)));
        assert_ne!(x, y);
        let snapshot = g.snapshot();
        for _ in 0..3 {
            let _ = g.legal_actions();
            let _ = g.observe(p);
            let _ = g.observe(p.other());
        }
        assert_eq!(g.snapshot(), snapshot);
    }
}
#[test]
fn msh_library_top_lad_countered_activation_keeps_paid_tap_without_reveal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, LAD, &[]);
        ready(&mut g, p);
        let x = named(&mut g, p, "Gilded Lotus", Zone::Library);
        activate(&mut g, p, id);
        let stack = g.state.stack.last().unwrap().id;
        g.counter_stack_item(stack, false);
        settle(&mut g);
        assert!(g.state.objects[id as usize].tapped);
        assert!(g.observe(p).last_library_reveal.is_none());
        assert_eq!(g.state.objects[x as usize].zone, Zone::Library);
    }
}
#[test]
fn msh_library_top_lad_ordinary_first_draw_miracle_pause_is_owned() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, LAD, &[]);
        ready(&mut g, p);
        named(&mut g, p, "Lorehold, the Historian", Zone::Battlefield);
        g.state.players[p.index()].cards_drawn_this_turn = 0;
        let x = named(&mut g, p, "Gilded Lotus", Zone::Library);
        activate(&mut g, p, id);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::MiracleReveal { .. })
        });
        assert_eq!(reveal(&g).drawn, Some(g.object_ref(x)));
        assert!(g.observe(p.other()).miracle.is_none());
        assert!(g.observe(p.other()).known_hand.contains(&g.object_ref(x)));
        assert!(!g.observe(p).miracle.unwrap().can_reveal);
        checked(&mut g, Action::DeclineMiracle);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
    }
}
#[test]
fn msh_library_top_lad_reveal_sequence_capacity_failure_is_transactional() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, LAD, &[]);
        ready(&mut g, p);
        named(&mut g, p, "Forest", Zone::Library);
        activate(&mut g, p, id);
        settle(&mut g);
        ready(&mut g, p);
        g.state
            .library_top
            .as_mut()
            .unwrap()
            .reveal
            .as_mut()
            .unwrap()
            .sequence = u64::MAX;
        named(&mut g, p, "Gilded Lotus", Zone::Library);
        activate(&mut g, p, id);
        checked(&mut g, Action::Pass);
        let before = g.snapshot();
        let revision = g.revision();
        let a = g.actor().unwrap();
        let view = [observation(&g, p), observation(&g, p.other())];
        assert!(matches!(
            g.apply(a, Action::Pass),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.revision(), revision);
        assert_eq!([observation(&g, p), observation(&g, p.other())], view);
    }
}
#[test]
fn msh_library_top_copied_lad_and_daredevil_have_independent_sources() {
    for p in [Player::P0, Player::P1] {
        for name in [LAD, DEVIL] {
            let mut g = setup(p);
            let original = paid(&mut g, p, name, &[]);
            let definition = g.object_view(original).card;
            let copy = g.create_copy_groups_inner(
                p.other(),
                &[TokenGroupView {
                    token: definition,
                    amount: 1,
                    tapped: false,
                }],
                &[Some(CopyRecipe::new(definition))],
                None,
            )[0]
            .card;
            g.advance_entries();
            settle(&mut g);
            let x = named(
                &mut g,
                p.other(),
                if name == LAD { "Gilded Lotus" } else { LAD },
                Zone::Library,
            );
            assert_eq!(top(&g, p.other()), Some(g.object_ref(x)));
            g.move_object(original, Zone::Exile);
            next_turn(&mut g);
            until(&mut g, |g| {
                g.state.active == p.other()
                    && g.state.step == Step::Main1
                    && g.actor() == Some(p.other())
            });
            let x = named(
                &mut g,
                p.other(),
                if name == LAD { "Gilded Lotus" } else { LAD },
                Zone::Library,
            );
            if name == LAD {
                activate(&mut g, p.other(), copy);
                settle(&mut g);
                assert_eq!(reveal(&g).source, g.object_ref(copy));
                assert_eq!(g.state.objects[x as usize].zone, Zone::Hand);
            } else {
                declare(&mut g, p.other(), &[copy]);
                optional(&mut g);
                checked(&mut g, Action::ChooseOptional { accept: true });
                settle(&mut g);
                assert_eq!(g.stats(copy).power, 5);
                assert!(g.can_play_exiled(p.other(), x));
            }
        }
    }
}
#[test]
fn msh_library_top_daredevil_no_attack_and_extra_combat_declarations() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, DEVIL, &[]);
        declare(&mut g, p, &[]);
        settle(&mut g);
        assert!(g.state.exile_permissions.is_empty());
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        g.state.priority = p;
        let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        for _ in 0..2 {
            let x = named(&mut g, p, LAD, Zone::Library);
            declare(&mut g, p, &[a]);
            optional(&mut g);
            checked(&mut g, Action::ChooseOptional { accept: true });
            settle(&mut g);
            assert!(g.can_play_exiled(p, x));
            g.state.objects[a as usize].tapped = false;
            g.state.attackers.clear();
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            g.state.priority = p;
        }
        assert_eq!(g.stats(id).power, 7);
    }
}
#[test]
fn msh_library_top_daredevil_top_changes_after_trigger_and_suppression_after_event() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, DEVIL, &[]);
        let old = named(&mut g, p, "Forest", Zone::Library);
        declare(&mut g, p, &[id]);
        optional(&mut g);
        let aura = named(&mut g, p, "Stop Cold", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(id));
        let x = named(&mut g, p, LAD, Zone::Library);
        checked(&mut g, Action::ChooseOptional { accept: true });
        settle(&mut g);
        assert!(g.can_play_exiled(p, x));
        assert_eq!(g.state.objects[old as usize].zone, Zone::Library);
        assert_eq!(g.stats(id).power, 5);
    }
}
#[test]
fn msh_library_top_lad_activation_prohibition_and_suppression_are_atomic() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, LAD, &[]);
        ready(&mut g, p);
        paid(&mut g, p, "Stop Cold", &[Target::Card(id)]);
        priority(&mut g, p);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: id,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        assert!(top(&g, p).is_none());
    }
}
#[test]
fn msh_library_top_nested_mana_frames_keep_outer_changed_top_hidden() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, LAD, &[]);
        let old = named(&mut g, p, "Forest", Zone::Library);
        let spell = named(&mut g, p, "Divination", Zone::Hand);
        let rubble = named(&mut g, p, "Rubble Rouser", Zone::Battlefield);
        let fuel = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 1];
        priority(&mut g, p);
        checked(&mut g, Action::AnnounceCast { card: spell });
        g.move_object(old, Zone::Exile);
        assert!(top(&g, p).is_none());
        while !matches!(g.state.pending, Pending::PayCosts { .. }) {
            checked(&mut g, Action::FinishTargets);
        }
        checked(
            &mut g,
            Action::AnnounceMana {
                card: rubble,
                ability: 0,
                counter_payment: Counters::default(),
                production: [0, 0, 0, 1, 0, 0],
            },
        );
        assert!(g.state.mana_announcement_stack.is_some());
        assert!(top(&g, p).is_none());
        let object = g.object_ref(fuel);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        for _ in 0..12 {
            if g.state.mana_announcement_stack.is_none() {
                break;
            }
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
                .unwrap();
            checked(&mut g, a);
            assert!(top(&g, p).is_none());
        }
        assert!(g.state.mana_announcement_stack.is_none());
        finish(&mut g);
        assert!(top(&g, p).is_some());
        settle(&mut g);
    }
}

#[test]
fn msh_library_top_land_special_action_pending_entry_owns_visibility() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, LAD, &[]);
        let old = named(&mut g, p, "Forest", Zone::Library);
        let land = named(&mut g, p, "Breeding Pool", Zone::Hand);
        priority(&mut g, p);
        checked(&mut g, Action::PlayLand { card: land });
        assert!(g.state.entry_batch.is_some());
        assert!(
            g.state
                .library_top
                .as_ref()
                .unwrap()
                .operation
                .as_ref()
                .unwrap()
                .land
        );
        assert_eq!(top(&g, p), Some(g.object_ref(old)));
        g.move_object(old, Zone::Exile);
        assert!(top(&g, p).is_none());
        let snap = g.snapshot();
        let mut branch = g.clone();
        drop(g);
        let a = branch
            .legal_actions()
            .into_iter()
            .find(|a| !matches!(a, Action::Concede))
            .unwrap();
        checked(&mut branch, a);
        assert!(branch.state.entry_batch.is_none());
        assert!(top(&branch, p).is_some());
        branch.restore(&snap).unwrap();
        assert!(top(&branch, p).is_none());
    }
}
#[test]
fn msh_library_top_funding_does_not_treat_draw_as_mana_or_publish_hidden_top() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, LAD, &[]);
        ready(&mut g, p);
        g.state.players[p.index()].mana = [0; 6];
        let x = named(&mut g, p, "Gilded Lotus", Zone::Library);
        let spell = named(&mut g, p, "Bear Cub", Zone::Hand);
        let before = g.snapshot();
        let obs = observation(&g, p.other());
        assert!(!g.can_fund(FundingGoal {
            actor: p,
            required: [0, 0, 0, 0, 1, 0],
            generic: 1,
            reserve: None,
            source: None
        }));
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: spell })
        );
        assert_eq!(g.snapshot(), before);
        assert_eq!(observation(&g, p.other()), obs);
        assert_eq!(top(&g, p), Some(g.object_ref(x)));
    }
}
#[test]
fn msh_library_top_daredevil_paid_alternate_faces_keep_origin_costs_and_timing() {
    for p in [Player::P0, Player::P1] {
        for adventure in [false, true] {
            let mut g = setup(p);
            let id = paid(&mut g, p, DEVIL, &[]);
            let card = named(
                &mut g,
                p,
                if adventure {
                    "Bramble Familiar"
                } else {
                    "Monica Rambeau"
                },
                Zone::Library,
            );
            declare(&mut g, p, &[id]);
            optional(&mut g);
            checked(&mut g, Action::ChooseOptional { accept: true });
            settle(&mut g);
            priority(&mut g, p);
            let action = if adventure {
                Action::AnnounceAdventure { card }
            } else {
                Action::AnnounceBack { card }
            };
            g.state.players[p.index()].mana = [30; 6];
            reject(&mut g, action.clone());
            until(&mut g, |g| {
                g.state.step == Step::Main2 && g.actor() == Some(p)
            });
            g.state.players[p.index()].mana = if adventure {
                [0, 0, 0, 0, 2, 5]
            } else {
                [2, 0, 0, 1, 0, 2]
            };
            checked(&mut g, action);
            finish(&mut g);
            let item = g.state.stack.last().unwrap();
            assert_eq!(item.cast.unwrap().origin, Zone::Exile);
            assert_eq!(
                item.cast.unwrap().face,
                if adventure {
                    CastFace::Adventure
                } else {
                    CastFace::Back
                }
            );
            assert_eq!(
                item.source_card.name(),
                if adventure {
                    "Fetch Quest"
                } else {
                    "Photon, Living Light"
                }
            );
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}
