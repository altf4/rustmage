//! Paid whole Saga chapters with owned decisions and exact-source continuations.
use super::assurance::big_helpers::{cast, checked, named, next_turn, setup};
use super::*;
const ARMOR: &str = "Armor Wars";
const HULK: &str = "World War Hulk";
fn views(g: &Game) -> [serde_json::Value; 2] {
    [Player::P0, Player::P1].map(|p| {
        let mut o = g.observe(p);
        o.revision = 0;
        serde_json::to_value(o).unwrap()
    })
}
fn reject(g: &mut Game, a: Action) {
    let before = (g.snapshot(), g.revision(), g.state_bytes(), views(g));
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(
        (g.snapshot(), g.revision(), g.state_bytes(), views(g)),
        before
    );
}
fn orphan(g: Game, a: Action) -> Game {
    let actor = g.actor().unwrap();
    let rev = g.revision();
    let ix = g.legal_actions().iter().position(|v| *v == a).unwrap();
    let before = g.snapshot();
    let v = views(&g);
    let (mut raw, mut index) = (g.clone(), g.clone());
    drop(g);
    raw.apply(actor, a.clone()).unwrap();
    index.apply_index(actor, rev, ix).unwrap();
    assert_eq!(raw.snapshot(), index.snapshot());
    assert_eq!(views(&raw), views(&index));
    let after = raw.snapshot();
    raw.restore(&before).unwrap();
    assert_eq!(views(&raw), v);
    assert_eq!(raw.apply_index(actor, rev, ix), Err(Error::StaleRevision));
    raw.apply(actor, a).unwrap();
    assert_eq!(raw.snapshot(), after);
    index
}
fn step(g: &mut Game, target: Option<Target>) {
    let aa = g.legal_actions();
    let a = target
        .map(|target| Action::ChooseTarget { target })
        .filter(|a| aa.contains(a))
        .or_else(|| {
            aa.iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::Pass | Action::PayCosts { .. } | Action::FinishTargets
                    )
                })
                .cloned()
        })
        .or_else(|| aa.iter().find(|a| !matches!(a, Action::Concede)).cloned())
        .unwrap_or_else(|| panic!("no Saga decision {:?}", g.state.pending));
    checked(g, a);
}
fn until(g: &mut Game, target: Option<Target>, pred: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..150 {
        if pred(g) {
            return;
        }
        assert!(g.outcome().is_none(), "premature terminal");
        step(g, target);
    }
    panic!("Saga boundary stalled {:?}", g.state.pending);
}
fn quiet(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.announcing.is_none()
        && g.state.resolving.is_none()
        && g.state.pending_triggers.is_empty()
}
fn finish(g: &mut Game, target: Option<Target>) {
    until(g, target, quiet);
}
fn priority(g: &mut Game, p: Player) {
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Main1;
    g.state.pending = Pending::Priority;
    g.state.passes = 0;
}
fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    priority(g, p);
    let d = Card::from_name(name).unwrap();
    assert!(d.is_supported());
    let id = named(g, p, name, Zone::Hand);
    let (mut m, n) = d.cost();
    m[5] += n;
    g.state.players[p.index()].mana = m;
    cast(g, id, None, &[]);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn producer(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = paid(g, p, name);
    finish(g, None);
    id
}
fn queued(g: &mut Game, id: CardId, ability: u16, target: Option<Target>) {
    assert!(g.queue_trigger(&g.object_view(id), ability as usize));
    until(g, target, |g| {
        g.state
            .stack
            .iter()
            .any(|s| s.card == id && s.kind == StackKind::TriggeredAbility { ability })
    });
}
fn chapter(g: &mut Game, id: CardId, ability: u16, target: Target) {
    queued(g, id, ability, Some(target));
    finish(g, Some(target));
}
fn main_lore(g: &mut Game, p: Player) {
    priority(g, p);
    g.begin_step(Step::Main1);
    g.prepare_triggers();
}
fn next_own_chapter(g: &mut Game, p: Player, id: CardId, lore: u64, target: Option<Target>) {
    next_turn(g);
    next_turn(g);
    until(g, target, |g| {
        g.state.active == p && g.counter_count(id, CounterKind::Lore) >= lore
    });
    finish(g, target);
}
fn free(id: CardId) -> Action {
    Action::AnnounceFree {
        card: id,
        face: CastFace::Normal,
        flash_surcharge: false,
    }
}

#[test]
fn msh_armor_hulk_paid_armor_natural_three_chapters_discount_and_final_damage() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Gilded Lotus", Zone::Battlefield);
        let a = paid(&mut g, p, ARMOR);
        until(&mut g, None, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        let hands = [g.hand_count(p), g.hand_count(p.other())];
        g = orphan(g, Action::ChooseOptional { accept: true });
        finish(&mut g, None);
        assert_eq!(
            [g.hand_count(p), g.hand_count(p.other())],
            [hands[0] + 1, hands[1] + 1]
        );
        assert_eq!(g.counter_count(a, CounterKind::Lore), 1);
        next_own_chapter(&mut g, p, a, 2, None);
        assert_eq!(
            g.player_cast_grants_view(p.index())
                .unwrap()
                .artifact_generic_reduction,
            1
        );
        let l = named(&mut g, p, "Gilded Lotus", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
        cast(&mut g, l, None, &[]);
        finish(&mut g, None);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert!(g.on_battlefield(a));
        next_own_chapter(&mut g, p, a, 3, Some(Target::Player(p.other())));
        assert_eq!(g.state.players[p.other().index()].life, 15);
        assert_eq!(g.object_view(a).zone, Zone::Graveyard);
        assert!(g.player_cast_grants_view(p.index()).is_none());
    }
}
#[test]
fn msh_armor_hulk_armor_zero_draw_accepts_or_declines_without_java_positive_guard() {
    for p in [Player::P0, Player::P1] {
        for accept in [false, true] {
            let mut g = setup(p);
            paid(&mut g, p, ARMOR);
            until(&mut g, None, |g| {
                matches!(g.state.pending, Pending::EffectOptional { .. })
            });
            let h = [g.hand_count(p), g.hand_count(p.other())];
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p),
                },
            );
            g = orphan(g, Action::ChooseOptional { accept });
            finish(&mut g, None);
            assert_eq!(g.hand_count(p), h[0]);
            assert_eq!(g.hand_count(p.other()), h[1] + usize::from(accept));
        }
    }
}
#[test]
fn msh_armor_hulk_armor_counts_resolution_control_and_artifact_creatures_not_ownership() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let old = named(&mut g, p, "Gilded Lotus", Zone::Battlefield);
        paid(&mut g, p, ARMOR);
        until(&mut g, None, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        g.move_object(old, Zone::Graveyard);
        named(&mut g, p, "A.I.M. Synthoids", Zone::Battlefield);
        let stolen = named(&mut g, p.other(), "Gilded Lotus", Zone::Battlefield);
        let r = g.object_ref(stolen);
        g.change_controller(r, p);
        named(&mut g, p.other(), "Gilded Lotus", Zone::Battlefield);
        let h = g.hand_count(p);
        checked(&mut g, Action::ChooseOptional { accept: true });
        finish(&mut g, None);
        assert_eq!(g.hand_count(p), h + 2);
    }
}
#[test]
fn msh_armor_hulk_armor_draw_pause_freezes_count_and_defers_opponent_draw() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Lorehold, the Historian", Zone::Battlefield);
        let arts = [
            named(&mut g, p, "Gilded Lotus", Zone::Battlefield),
            named(&mut g, p, "A.I.M. Synthoids", Zone::Battlefield),
        ];
        paid(&mut g, p, ARMOR);
        until(&mut g, None, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        let h = [g.hand_count(p), g.hand_count(p.other())];
        checked(&mut g, Action::ChooseOptional { accept: true });
        until(&mut g, None, |g| {
            matches!(g.state.pending, Pending::MiracleReveal { .. })
        });
        assert_eq!(g.hand_count(p), h[0] + 1);
        assert_eq!(g.hand_count(p.other()), h[1]);
        assert!(g.observe(p.other()).miracle.is_none());
        for id in arts {
            g.move_object(id, Zone::Graveyard);
        }
        g = orphan(g, Action::DeclineMiracle);
        finish(&mut g, None);
        assert_eq!(g.hand_count(p), h[0] + 2);
        assert_eq!(g.hand_count(p.other()), h[1] + 1);
    }
}
#[test]
fn msh_armor_hulk_armor_optional_hidden_draw_and_queued_controller_survive_departure() {
    for p in [Player::P0, Player::P1] {
        for blink in [false, true] {
            let mut g = setup(p);
            named(&mut g, p, "Gilded Lotus", Zone::Battlefield);
            let a = paid(&mut g, p, ARMOR);
            until(&mut g, None, |g| {
                matches!(g.state.pending, Pending::EffectOptional { .. })
            });
            let old = g.object_ref(a);
            let h = [g.hand_count(p), g.hand_count(p.other())];
            if blink {
                g.move_object(a, Zone::Exile);
                g.move_object(a, Zone::Battlefield);
                assert_ne!(old, g.object_ref(a));
            } else {
                g.change_controller(old, p.other());
            }
            let mut hidden = g.clone();
            let top = *hidden.state.players[p.index()].library.last().unwrap();
            hidden.state.objects[top as usize].card = Card::from_name("Island").unwrap();
            assert_eq!(
                views(&g)[p.other().index()],
                views(&hidden)[p.other().index()]
            );
            assert_eq!(g.legal_actions(), hidden.legal_actions());
            // Stop at the old resolution's completion, before any blinked chapter I is announced.
            checked(&mut g, Action::ChooseOptional { accept: true });
            assert_eq!(g.hand_count(p), h[0] + 1);
            assert_eq!(g.hand_count(p.other()), h[1] + 1);
        }
    }
}
#[test]
fn msh_armor_hulk_armor_greatest_mana_value_is_current_and_zero_without_artifacts() {
    for p in [Player::P0, Player::P1] {
        for amount in [0, 2, 5] {
            let mut g = setup(p);
            let a = producer(&mut g, p, ARMOR);
            let old = named(&mut g, p, "Gilded Lotus", Zone::Battlefield);
            queued(&mut g, a, 2, Some(Target::Player(p.other())));
            g.move_object(old, Zone::Graveyard);
            if amount == 2 {
                named(&mut g, p, "A.I.M. Synthoids", Zone::Battlefield);
            }
            if amount == 5 {
                let id = named(&mut g, p.other(), "Gilded Lotus", Zone::Battlefield);
                let r = g.object_ref(id);
                g.change_controller(r, p);
            }
            named(&mut g, p.other(), "Gilded Lotus", Zone::Battlefield);
            finish(&mut g, None);
            assert_eq!(g.state.players[p.other().index()].life, 20 - amount);
        }
    }
}
#[test]
fn msh_armor_hulk_armor_target_is_opponent_not_source_controller_or_creature() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = producer(&mut g, p, ARMOR);
        let b = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        assert!(g.queue_trigger(&g.object_view(a), 2));
        until(&mut g, None, |g| {
            g.state
                .announcing
                .as_ref()
                .is_some_and(|a| a.item.kind == StackKind::TriggeredAbility { ability: 2 })
        });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(b),
            },
        );
        g = orphan(
            g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        finish(&mut g, None);
        assert_eq!(g.state.players[p.other().index()].life, 20);
    }
}
#[test]
fn msh_armor_hulk_armor_damage_source_current_and_departed_controller_lifelink() {
    for p in [Player::P0, Player::P1] {
        for depart in [false, true] {
            let mut g = setup(p);
            let a = producer(&mut g, p, ARMOR);
            named(&mut g, p, "Gilded Lotus", Zone::Battlefield);
            g.grant_keyword_effects(&[a], g.object_ref(a), Keywords::LIFELINK);
            queued(&mut g, a, 2, Some(Target::Player(p.other())));
            let r = g.object_ref(a);
            g.change_controller(r, p.other());
            if depart {
                g.move_object(a, Zone::Graveyard);
            }
            finish(&mut g, None);
            assert_eq!(
                g.state.players[p.other().index()].life,
                20,
                "five damage and source-controller lifelink"
            );
            assert_eq!(g.state.players[p.index()].life, 20);
        }
    }
}
#[test]
fn msh_armor_hulk_armor_damage_uses_actual_replacement_actor() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = producer(&mut g, p, ARMOR);
        named(&mut g, p, "Gilded Lotus", Zone::Battlefield);
        named(&mut g, p, "Twinflame Tyrant", Zone::Battlefield);
        chapter(&mut g, a, 2, Target::Player(p.other()));
        assert_eq!(g.state.players[p.other().index()].life, 10);
    }
}
#[test]
fn msh_armor_hulk_paid_hulk_natural_three_chapters_free_cast_counters_double_and_cleanup() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = producer(&mut g, p, HULK);
        let b = named(&mut g, p, "Bear Cub", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, free(b));
        finish(&mut g, None);
        assert!(g.on_battlefield(b));
        assert_eq!(
            g.player_cast_grants_view(p.index())
                .map_or(0, |row| row.next_red_or_green_creature_free),
            0
        );
        next_own_chapter(&mut g, p, a, 2, Some(Target::Card(b)));
        assert_eq!(g.counter_count(b, CounterKind::PlusOne), 3);
        assert_eq!(
            g.stats(b),
            Stats {
                power: 5,
                toughness: 5
            }
        );
        next_own_chapter(&mut g, p, a, 3, Some(Target::Card(b)));
        assert_eq!(
            g.stats(b),
            Stats {
                power: 10,
                toughness: 10
            }
        );
        assert!(g.keywords(b).contains(Keywords::TRAMPLE));
        assert_eq!(g.object_view(a).zone, Zone::Graveyard);
        next_turn(&mut g);
        assert_eq!(
            g.stats(b),
            Stats {
                power: 5,
                toughness: 5
            }
        );
        assert!(!g.keywords(b).contains(Keywords::TRAMPLE));
    }
}
#[test]
fn msh_armor_hulk_hulk_chapters_require_own_current_creature_and_fail_atomically() {
    for p in [Player::P0, Player::P1] {
        for ability in [1, 2] {
            let mut g = setup(p);
            let a = producer(&mut g, p, HULK);
            let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let other = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let land = named(&mut g, p, "Forest", Zone::Battlefield);
            assert!(g.queue_trigger(&g.object_view(a), ability));
            until(&mut g, None, |g| g.state.announcing.is_some());
            for id in [other, land, a] {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(id),
                    },
                );
            }
            g = orphan(
                g,
                Action::ChooseTarget {
                    target: Target::Card(own),
                },
            );
            finish(&mut g, None);
            assert_eq!(
                g.stats(own),
                if ability == 1 {
                    Stats {
                        power: 5,
                        toughness: 5,
                    }
                } else {
                    Stats {
                        power: 4,
                        toughness: 4,
                    }
                }
            );
        }
    }
}
#[test]
fn msh_armor_hulk_hulk_bound_recipient_control_change_and_blink_do_not_rebind() {
    for p in [Player::P0, Player::P1] {
        for ability in [1, 2] {
            for blink in [false, true] {
                let mut g = setup(p);
                let a = producer(&mut g, p, HULK);
                let b = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                queued(&mut g, a, ability, Some(Target::Card(b)));
                if blink {
                    let old = g.object_ref(b);
                    g.move_object(b, Zone::Exile);
                    g.move_object(b, Zone::Battlefield);
                    assert_ne!(old, g.object_ref(b));
                } else {
                    let r = g.object_ref(b);
                    g.change_controller(r, p.other());
                }
                finish(&mut g, None);
                assert_eq!(g.counter_count(b, CounterKind::PlusOne), 0);
                assert_eq!(
                    g.stats(b),
                    Stats {
                        power: 2,
                        toughness: 2
                    }
                );
                assert!(!g.keywords(b).contains(Keywords::TRAMPLE));
            }
        }
    }
}
#[test]
fn msh_armor_hulk_hulk_queued_chapters_survive_source_control_departure_and_reentry() {
    for p in [Player::P0, Player::P1] {
        for ability in [1, 2] {
            for depart in [false, true] {
                let mut g = setup(p);
                let a = producer(&mut g, p, HULK);
                let b = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                queued(&mut g, a, ability, Some(Target::Card(b)));
                if depart {
                    g.move_object(a, Zone::Exile);
                    g.move_object(a, Zone::Hand);
                } else {
                    let r = g.object_ref(a);
                    g.change_controller(r, p.other());
                }
                finish(&mut g, None);
                assert_eq!(
                    g.stats(b),
                    if ability == 1 {
                        Stats {
                            power: 5,
                            toughness: 5,
                        }
                    } else {
                        Stats {
                            power: 4,
                            toughness: 4,
                        }
                    }
                );
            }
        }
    }
}
#[test]
fn msh_armor_hulk_hulk_counter_replacement_uses_recipient_and_captured_ability_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = producer(&mut g, p, HULK);
        let b = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        queued(&mut g, a, 1, Some(Target::Card(b)));
        let r = g.object_ref(a);
        g.change_controller(r, p.other());
        g.move_object(a, Zone::Graveyard);
        finish(&mut g, None);
        assert_eq!(g.counter_count(b, CounterKind::PlusOne), 6);
    }
}
#[test]
fn msh_armor_hulk_hulk_fixed_signed_snapshot_does_not_double_later_growth() {
    for p in [Player::P0, Player::P1] {
        for negative in [false, true] {
            let mut g = setup(p);
            let a = producer(&mut g, p, HULK);
            let b = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            // A bounded ordinary continuous modifier supplies a signed snapshot edge.
            g.state.objects[b as usize].growth = if negative {
                Stats {
                    power: -5,
                    toughness: 1,
                }
            } else {
                Stats {
                    power: 3,
                    toughness: 1,
                }
            };
            chapter(&mut g, a, 2, Target::Card(b));
            assert_eq!(
                g.stats(b),
                if negative {
                    Stats {
                        power: -6,
                        toughness: 6,
                    }
                } else {
                    Stats {
                        power: 10,
                        toughness: 6,
                    }
                }
            );
            priority(&mut g, p);
            let grow = named(&mut g, p, "Giant Growth", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
            cast(&mut g, grow, None, &[Target::Card(b)]);
            finish(&mut g, None);
            assert_eq!(
                g.stats(b),
                if negative {
                    Stats {
                        power: -3,
                        toughness: 9,
                    }
                } else {
                    Stats {
                        power: 13,
                        toughness: 9,
                    }
                }
            );
            next_turn(&mut g);
            assert_eq!(
                g.stats(b),
                Stats {
                    power: 2,
                    toughness: 2
                }
            );
        }
    }
}
#[test]
fn msh_armor_hulk_hulk_resolution_snapshots_after_paid_response() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = producer(&mut g, p, HULK);
        let b = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        queued(&mut g, a, 2, Some(Target::Card(b)));
        // The actual instant resolves above the already targeted chapter.
        g.state.priority = p;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
        let grow = named(&mut g, p, "Giant Growth", Zone::Hand);
        cast(&mut g, grow, None, &[Target::Card(b)]);
        finish(&mut g, None);
        assert_eq!(
            g.stats(b),
            Stats {
                power: 10,
                toughness: 10
            }
        );
    }
}
#[test]
fn msh_armor_hulk_saga_entry_crosses_two_chapters_but_main_lore_is_not_effect_doubled() {
    for p in [Player::P0, Player::P1] {
        for name in [ARMOR, HULK] {
            let mut g = setup(p);
            named(&mut g, p, "Doubling Season", Zone::Battlefield);
            let b = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let a = paid(&mut g, p, name);
            finish(&mut g, Some(Target::Card(b)));
            assert_eq!(g.counter_count(a, CounterKind::Lore), 2);
            assert!(g.on_battlefield(a));
            if name == HULK {
                assert_eq!(g.counter_count(b, CounterKind::PlusOne), 6);
            } else {
                assert_eq!(
                    g.player_cast_grants_view(p.index())
                        .unwrap()
                        .artifact_generic_reduction,
                    1
                );
            }
            main_lore(&mut g, p);
            assert_eq!(g.counter_count(a, CounterKind::Lore), 3);
            finish(
                &mut g,
                Some(if name == ARMOR {
                    Target::Player(p.other())
                } else {
                    Target::Card(b)
                }),
            );
            assert_eq!(g.object_view(a).zone, Zone::Graveyard);
        }
    }
}
#[test]
fn msh_armor_hulk_saga_final_source_waits_for_targeted_packet_then_sacrifices() {
    for p in [Player::P0, Player::P1] {
        for name in [ARMOR, HULK] {
            let mut g = setup(p);
            let b = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let a = producer(&mut g, p, name);
            main_lore(&mut g, p);
            finish(&mut g, Some(Target::Card(b)));
            main_lore(&mut g, p);
            until(
                &mut g,
                Some(if name == ARMOR {
                    Target::Player(p.other())
                } else {
                    Target::Card(b)
                }),
                |g| {
                    g.state.stack.iter().any(|s| {
                        s.card == a && s.kind == StackKind::TriggeredAbility { ability: 2 }
                    })
                },
            );
            assert!(g.on_battlefield(a));
            assert_eq!(g.counter_count(a, CounterKind::Lore), 3);
            finish(&mut g, None);
            assert_eq!(g.object_view(a).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn msh_armor_hulk_armor_effective_artifacts_include_food_and_animated_land() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Heroic Feast");
        finish(&mut g, None);
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Battlefield && o.is_token())
                .count(),
            1
        );
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        let object = g.object_ref(land);
        let timestamp = g.next_role_timestamp();
        // An ordinary production form row supplies a bounded earlier-layer fixture.
        g.state.roles.get_or_insert_default().animations.push(
            characteristic_layers::AnimationView {
                object,
                timestamp,
                artifact: true,
                preserve_card_types: false,
                colors: None,
                creature_types: None,
                all_creature_types: false,
                base: Some((4, 4)),
                keywords: Keywords::NONE,
                expires_turn: None,
            },
        );
        assert_ne!(g.current_kinds(land) & 16, 0);
        let a = paid(&mut g, p, ARMOR);
        until(&mut g, None, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        let before = g.hand_count(p);
        checked(&mut g, Action::ChooseOptional { accept: true });
        finish(&mut g, None);
        assert_eq!(g.hand_count(p), before + 2);
        chapter(&mut g, a, 2, Target::Player(p.other()));
        assert_eq!(
            g.state.players[p.other().index()].life,
            20,
            "both token and land have mana value zero"
        );
    }
}
#[test]
fn msh_armor_hulk_armor_target_gaining_hexproof_fizzles_bound_damage() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = producer(&mut g, p, ARMOR);
        named(&mut g, p, "Gilded Lotus", Zone::Battlefield);
        queued(&mut g, a, 2, Some(Target::Player(p.other())));
        // The supported static producer is a bounded battlefield fixture after target binding.
        named(&mut g, p.other(), "Crystal Barricade", Zone::Battlefield);
        finish(&mut g, None);
        assert_eq!(g.state.players[p.other().index()].life, 20);
    }
}
#[test]
fn msh_armor_hulk_hulk_live_noncreature_target_is_illegal_on_resolution() {
    for p in [Player::P0, Player::P1] {
        for ability in [1, 2] {
            let mut g = setup(p);
            let a = producer(&mut g, p, HULK);
            let b = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            queued(&mut g, a, ability, Some(Target::Card(b)));
            // This bounded owned Aura attachment changes actual effective characteristics.
            let moon = named(
                &mut g,
                p.other(),
                "Imprisoned in the Moon",
                Zone::Battlefield,
            );
            g.state.objects[moon as usize].attached_to = Some(g.object_ref(b));
            assert_eq!(g.current_kinds(b) & 1, 0);
            finish(&mut g, None);
            assert_eq!(g.counter_count(b, CounterKind::PlusOne), 0);
            assert!(!g.keywords(b).contains(Keywords::TRAMPLE));
        }
    }
}
#[test]
fn msh_armor_hulk_hulk_trample_is_used_in_real_blocked_combat() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = producer(&mut g, p, HULK);
        let b = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let blocker = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        chapter(&mut g, a, 2, Target::Card(b));
        assert_eq!(
            g.stats(b),
            Stats {
                power: 4,
                toughness: 4
            }
        );
        g.state.active = p;
        g.state.priority = p;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        g.state.objects[b as usize].summoning_sick = false;
        checked(&mut g, Action::Attack { card: b });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, None, |g| g.state.pending == Pending::Blockers);
        checked(
            &mut g,
            Action::Block {
                blocker,
                attacker: b,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        until(&mut g, None, |g| g.state.pending == Pending::Damage);
        reject(
            &mut g,
            Action::AssignDamage {
                target: Target::Player(p.other()),
                amount: 3,
            },
        );
        g = orphan(
            g,
            Action::AssignDamage {
                target: Target::Card(blocker),
                amount: 2,
            },
        );
        until(&mut g, None, |g| {
            g.state.players[p.other().index()].life == 18
        });
        assert_eq!(g.object_view(blocker).zone, Zone::Graveyard);
        assert!(g.on_battlefield(b));
    }
}
