use super::tests::{act, scenario, spawn};
use super::*;
const NAMES: [&str; 12] = [
    "Apothecary Stomper",
    "Coliseum Behemoth",
    "Cosmogrand Zenith",
    "Divining Duelist",
    "Downwind Ambusher",
    "Elven Raft-Steerer",
    "Ferocification",
    "Gaius van Baelsar",
    "Giant-Sized Flying Ant",
    "Glamermite",
    "Lord Skitter's Butcher",
    "Wardens of the Cycle",
];
fn named(g: &mut Game, p: Player, n: &str, z: Zone) -> CardId {
    let c = Card::from_name(n).unwrap();
    assert!(c.is_supported(), "{n}");
    spawn(g, p, c, z)
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for s in &mut g.state.players {
        s.mana = [20; 6];
    }
    g
}
fn pt(g: &Game, id: CardId) -> (i128, i128) {
    let o = g.object_view(id);
    (o.power, o.toughness)
}
fn replay(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    let p = g.actor().unwrap();
    let mut f = g.clone();
    act(&mut f, a.clone());
    act(g, a.clone());
    let e = g.snapshot();
    assert_eq!(f.snapshot(), e);
    g.restore(&s).unwrap();
    assert_eq!(g.apply_index(p, r, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), e);
}
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    assert!(g.apply(g.actor().unwrap(), a).is_err());
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), r);
}

fn choose(g: &mut Game, mode: u16, target: Option<CardId>) {
    let actions = g.legal_actions();
    let chosen = actions
        .iter()
        .find(|a| **a == Action::ChooseMode { mode })
        .or_else(|| {
            target.and_then(|id| {
                actions.iter().find(|a| {
                    **a == Action::ChooseTarget {
                        target: Target::Card(id),
                    }
                })
            })
        })
        .or_else(|| {
            target.and_then(|id| {
                actions
                    .iter()
                    .find(|a| matches!(a,Action::Sacrifice{object} if object.card==id))
            })
        })
        .or_else(|| {
            actions
                .iter()
                .find(|a| matches!(a, Action::ChooseOptional { accept: true }))
        })
        .or_else(|| {
            actions.iter().find(|a| {
                matches!(
                    a,
                    Action::ChooseTarget { .. }
                        | Action::FinishTargets
                        | Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::Sacrifice { .. }
                        | Action::Discard { .. }
                        | Action::ArrangeLibrary { .. }
                )
            })
        })
        .expect("modal fixture choice")
        .clone();
    replay(g, chosen);
}
fn settle(g: &mut Game, mode: u16, target: Option<CardId>) {
    g.prepare_triggers();
    for _ in 0..140 {
        if g.outcome().is_some() || g.state.stack.is_empty() && g.state.pending == Pending::Priority
        {
            return;
        }
        let p = g.actor().unwrap();
        let o = g.observe(p);
        if o.library_choice.is_some() || o.decision == "effect_discard" {
            let other = g.observe(p.other());
            assert!(
                other.library_choice.is_none()
                    && other.effect_discards.is_empty()
                    && other.actions.is_empty()
            );
        }
        choose(g, mode, target);
    }
    panic!("unsettled modal trigger");
}
fn pay(g: &mut Game) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    replay(g, a);
}
fn cast_body(g: &mut Game, id: CardId, mode: u16, target: Option<CardId>) {
    replay(g, Action::AnnounceCast { card: id });
    if let Some(target) = target
        && g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(target),
        })
    {
        replay(
            g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
    }
    pay(g);
    settle(g, mode, target);
}
fn token(g: &mut Game, p: Player) -> CardId {
    let c = Card::catalogue()
        .find(|c| c.token_info().is_some() && c.is_creature())
        .unwrap();
    spawn(g, p, c, Zone::Battlefield)
}
fn mode_window(g: &mut Game) {
    g.prepare_triggers();
    for _ in 0..12 {
        if !g.observe(g.actor().unwrap()).mode_choices.is_empty() {
            return;
        }
        choose(g, 0, None);
    }
    panic!("no mode window");
}
#[test]
fn every_new_whole_body_casts_and_all_twenty_seven_modes_resolve_for_both_seats() {
    let counts = [2, 2, 2, 3, 2, 2, 2, 3, 2, 2, 3, 2];
    for p in [Player::P0, Player::P1] {
        for (i, name) in NAMES.iter().enumerate() {
            for mode in 0..counts[i] {
                let mut g = setup(p);
                let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
                let artifact = named(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
                let tokens = [token(&mut g, p), token(&mut g, p.other())];
                if i == 7 {
                    for player in [p, p.other()] {
                        named(&mut g, player, NAMES[6], Zone::Battlefield);
                    }
                }
                if mode == 1 && [3, 5, 8, 9].contains(&i) {
                    let target = if i == 5 { own } else { enemy };
                    g.state.objects[target as usize].tapped = true;
                }
                if i == 4 && mode == 1 {
                    g.state.objects[enemy as usize].damage = 1;
                    g.state.objects[enemy as usize].damaged_this_turn = true;
                }
                let id = named(&mut g, p, name, Zone::Hand);
                let target = if [0, 6, 7, 10].contains(&i) || i == 5 && mode == 1 {
                    own
                } else if i == 1 {
                    artifact
                } else {
                    enemy
                };
                let library = g.state.players[p.index()].library.len();
                cast_body(&mut g, id, mode, Some(target));
                if i == 2 {
                    let opt = named(&mut g, p, "Opt", Zone::Hand);
                    cast_body(&mut g, opt, mode, None);
                }
                if i == 5 {
                    let land = named(&mut g, p, "Forest", Zone::Hand);
                    replay(&mut g, Action::PlayLand { card: land });
                    settle(&mut g, mode, Some(target));
                }
                if i == 6 {
                    g.begin_step(Step::BeginCombat);
                    settle(&mut g, mode, Some(target));
                }
                if i == 11 {
                    let shock = named(&mut g, p, "Shock", Zone::Hand);
                    cast_body(&mut g, shock, mode, Some(own));
                    assert_eq!(g.state.creatures_died_this_turn, 1);
                    g.begin_step(Step::End);
                    settle(&mut g, mode, None);
                }
                match (i, mode) {
                    (0, 0) => assert_eq!(g.object_view(own).counters.plus_one, 2),
                    (0, 1) => assert_eq!(g.state.players[p.index()].life, 24),
                    (1, 0) => assert_eq!(g.object_view(artifact).zone, Zone::Graveyard),
                    (1, 1) => assert_eq!(g.state.players[p.index()].library.len(), library - 1),
                    (2, 0) => assert_eq!(
                        g.state
                            .objects
                            .iter()
                            .filter(|o| o.owner == p
                                && o.zone == Zone::Battlefield
                                && o.card
                                    .token_info()
                                    .is_some_and(|t| t.subtypes == "Human Soldier"))
                            .count(),
                        2
                    ),
                    (2, 1) => {
                        assert_eq!(g.object_view(own).counters.plus_one, 1);
                        assert_eq!(g.object_view(id).counters.plus_one, 1);
                    }
                    (3, 0) | (5, 0) | (8, 0) | (9, 0) => assert!(g.object_view(target).tapped),
                    (3, 1) | (5, 1) | (8, 1) | (9, 1) => assert!(!g.object_view(target).tapped),
                    (3, 2) => {
                        assert_eq!(g.state.players[p.index()].library.len(), library - 1);
                        assert_eq!(g.hand_count(p), 0);
                    }
                    (4, 0) => assert_eq!(pt(&g, enemy), (1, 1)),
                    (4, 1) => assert_eq!(g.object_view(enemy).zone, Zone::Graveyard),
                    (6, 0) => assert_eq!(pt(&g, own), (4, 2)),
                    (6, 1) => assert!(
                        g.keywords(own)
                            .contains(Keywords::MENACE.union(Keywords::HASTE))
                    ),
                    (7, 0) => {
                        for t in tokens {
                            assert_eq!(g.state.objects[t as usize].zone, Zone::Outside);
                        }
                    }
                    (7, 1) => {
                        for t in [own, enemy] {
                            assert_eq!(g.object_view(t).zone, Zone::Graveyard);
                        }
                    }
                    (7, 2) => assert_eq!(
                        g.state
                            .objects
                            .iter()
                            .filter(|o| o.card.name() == NAMES[6] && o.zone == Zone::Graveyard)
                            .count(),
                        2
                    ),
                    (10, 0) => {
                        let rats: Vec<_> = g
                            .state
                            .objects
                            .iter()
                            .enumerate()
                            .filter(|(_, o)| {
                                o.owner == p
                                    && o.zone == Zone::Battlefield
                                    && o.card.token_info().is_some_and(|t| t.subtypes == "Rat")
                            })
                            .collect();
                        assert_eq!(rats.len(), 1);
                        assert!(
                            g.keywords(rats[0].0 as CardId)
                                .contains(Keywords::CANT_BLOCK)
                        );
                    }
                    (10, 1) => {
                        assert_eq!(g.object_view(own).zone, Zone::Graveyard);
                        assert_eq!(g.state.players[p.index()].library.len(), library - 1);
                    }
                    (10, 2) => assert!(g.keywords(own).contains(Keywords::MENACE)),
                    (11, 0) => assert_eq!(g.state.players[p.index()].life, 22),
                    (11, 1) => {
                        assert_eq!(g.state.players[p.index()].life, 19);
                        assert_eq!(g.state.players[p.index()].library.len(), library - 1);
                    }
                    _ => panic!("unchecked mode"),
                }
                assert_eq!(
                    g.keywords(id).contains(Keywords::FLYING),
                    [8, 9].contains(&i)
                );
                assert_eq!(
                    g.keywords(id).contains(Keywords::FLASH),
                    [3, 4, 8, 9].contains(&i)
                );
                assert_eq!(g.keywords(id).contains(Keywords::VIGILANCE), i == 0);
                assert_eq!(g.keywords(id).contains(Keywords::TRAMPLE), i == 1);
            }
        }
    }
}
#[test]
fn modal_trigger_announces_mode_before_targets_without_mana_or_extra_cast_and_replays_every_boundary()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let id = named(&mut g, p, NAMES[0], Zone::Hand);
        replay(&mut g, Action::AnnounceCast { card: id });
        pay(&mut g);
        mode_window(&mut g);
        let snapshot = g.snapshot();
        let mana = g.state.players[p.index()].mana;
        let casts = g.state.players[p.index()].spells_cast_this_turn;
        let o = g.observe(p);
        assert_eq!(o.mode_choices.len(), 2);
        assert_eq!(o.decision, "spell_mode");
        assert_eq!(o.announcing.unwrap().mode, None);
        assert!(g.observe(p.other()).mode_choices.is_empty());
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        reject(&mut g, Action::ChooseMode { mode: 2 });
        reject(&mut g, Action::PayCosts { payment: [0; 6] });
        replay(&mut g, Action::ChooseMode { mode: 0 });
        assert!(g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(own)
        }));
        reject(&mut g, Action::ChooseMode { mode: 1 });
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        assert_eq!(g.state.stack.last().unwrap().mode, Some(0));
        settle(&mut g, 0, Some(own));
        assert_eq!(g.object_view(own).counters.plus_one, 2);
        assert_eq!(g.state.players[p.index()].mana, mana);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, casts);
        g.restore(&snapshot).unwrap();
        replay(&mut g, Action::ChooseMode { mode: 1 });
        settle(&mut g, 1, None);
        assert_eq!(g.state.players[p.index()].life, 24);
        assert_eq!(g.object_view(own).counters.plus_one, 0);
    }
}
#[test]
fn modes_without_legal_required_targets_are_filtered_and_all_unavailable_trigger_is_removed() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, NAMES[1], Zone::Hand);
        replay(&mut g, Action::AnnounceCast { card: id });
        pay(&mut g);
        mode_window(&mut g);
        let modes: Vec<_> = g.observe(p).mode_choices.iter().map(|m| m.mode).collect();
        assert_eq!(modes, vec![1]);
        reject(&mut g, Action::ChooseMode { mode: 0 });
        replay(&mut g, Action::ChooseMode { mode: 1 });
        settle(&mut g, 1, None);
        let id = named(&mut g, p, NAMES[4], Zone::Hand);
        cast_body(&mut g, id, 0, None);
        assert!(g.state.stack.is_empty() && g.state.announcing.is_none());
        assert_eq!(g.object_view(id).zone, Zone::Battlefield);
    }
}
#[test]
fn modal_trigger_ordering_remains_apnap_with_multiple_owned_items_and_no_priority_between_choices()
{
    for active in [Player::P0, Player::P1] {
        let mut g = setup(active);
        let a = named(&mut g, active, NAMES[0], Zone::Battlefield);
        let b = named(&mut g, active, NAMES[0], Zone::Battlefield);
        let other = named(&mut g, active.other(), NAMES[0], Zone::Battlefield);
        // Isolate simultaneous trigger placement using authenticated source views.
        for id in [other, a, b] {
            assert!(g.queue_trigger(&g.object_view(id), 0));
        }
        g.prepare_triggers();
        assert_eq!(g.actor(), Some(active));
        let id = g
            .observe(active)
            .waiting_triggers
            .iter()
            .find(|t| t.card == b)
            .unwrap()
            .id;
        replay(&mut g, Action::PutTrigger { id });
        assert_eq!(g.actor(), Some(active));
        reject(&mut g, Action::Pass);
        replay(&mut g, Action::ChooseMode { mode: 1 });
        assert_eq!(g.actor(), Some(active));
        replay(&mut g, Action::ChooseMode { mode: 1 });
        assert_eq!(g.actor(), Some(active.other()));
        replay(&mut g, Action::ChooseMode { mode: 1 });
        assert_eq!(
            g.state.stack.iter().map(|s| s.card).collect::<Vec<_>>(),
            vec![b, a, other]
        );
        assert!(g.state.stack.iter().all(|s| s.mode == Some(1)));
        settle(&mut g, 1, None);
        assert_eq!(g.state.players[active.index()].life, 28);
        assert_eq!(g.state.players[active.other().index()].life, 24);
    }
}
#[test]
fn selected_trigger_program_survives_source_departure_and_does_not_rebind_a_new_source_incarnation()
{
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let id = named(&mut g, p, NAMES[0], Zone::Hand);
        replay(&mut g, Action::AnnounceCast { card: id });
        pay(&mut g);
        mode_window(&mut g);
        let old = g.object_ref(id);
        g.move_object(id, Zone::Exile);
        g.move_object(id, Zone::Battlefield);
        replay(&mut g, Action::ChooseMode { mode: 0 });
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        assert_eq!(g.state.stack.last().unwrap().object, old);
        settle(&mut g, 1, None);
        assert_eq!(g.object_view(bear).counters.plus_one, 2);
        assert_eq!(g.object_view(id).counters.plus_one, 0);
        assert_eq!(g.state.players[p.index()].life, 24);
        assert_ne!(g.object_ref(id), old);
    }
}
#[test]
fn selected_mode_with_illegal_target_fizzles_without_falling_back_to_a_legal_life_mode() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let id = named(&mut g, p, NAMES[0], Zone::Hand);
        replay(&mut g, Action::AnnounceCast { card: id });
        pay(&mut g);
        mode_window(&mut g);
        replay(&mut g, Action::ChooseMode { mode: 0 });
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        g.move_object(bear, Zone::Hand);
        g.move_object(bear, Zone::Battlefield);
        reject(&mut g, Action::ChooseMode { mode: 1 });
        settle(&mut g, 0, None);
        assert_eq!(g.object_view(bear).counters.plus_one, 0);
        assert_eq!(g.state.players[p.index()].life, 20);
    }
}
#[test]
fn chosen_counter_and_life_modes_rollback_atomically_on_real_resolution_capacity_failure() {
    for p in [Player::P0, Player::P1] {
        for mode in [0, 1] {
            let mut g = setup(p);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let id = named(&mut g, p, NAMES[0], Zone::Hand);
            replay(&mut g, Action::AnnounceCast { card: id });
            pay(&mut g);
            mode_window(&mut g);
            replay(&mut g, Action::ChooseMode { mode });
            if mode == 0 {
                replay(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(bear),
                    },
                );
                g.state.objects[bear as usize].counters.plus_one = u64::MAX;
            } else {
                g.state.players[p.index()].life = i128::MAX;
            }
            replay(&mut g, Action::Pass);
            let saved = g.snapshot();
            let revision = g.revision();
            assert!(matches!(
                g.apply(g.actor().unwrap(), Action::Pass),
                Err(Error::CapacityExceeded(_))
            ));
            assert_eq!(g.snapshot(), saved);
            assert_eq!(g.revision(), revision);
            assert_eq!(g.state.stack.last().unwrap().mode, Some(mode));
        }
    }
}
#[test]
fn a_mode_target_triggers_ward_and_declining_counters_only_the_selected_ability() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let spider = named(&mut g, p.other(), "Skyward Spider", Zone::Battlefield);
        let id = named(&mut g, p, NAMES[4], Zone::Hand);
        replay(&mut g, Action::AnnounceCast { card: id });
        pay(&mut g);
        mode_window(&mut g);
        replay(&mut g, Action::ChooseMode { mode: 0 });
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(spider),
            },
        );
        for _ in 0..12 {
            if g.observe(p).resolution_payment.is_some() {
                break;
            }
            choose(&mut g, 0, None);
        }
        assert_eq!(g.observe(p).resolution_payment.unwrap().cost.generic, 2);
        replay(&mut g, Action::DeclineResolutionMana);
        settle(&mut g, 0, None);
        assert_eq!(pt(&g, spider), (2, 2));
        assert_eq!(g.object_view(id).zone, Zone::Battlefield);
    }
}
#[test]
fn private_sacrifice_scry_draw_preserves_mode_and_program_across_source_departure_and_choices() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let id = named(&mut g, p, NAMES[10], Zone::Hand);
        replay(&mut g, Action::AnnounceCast { card: id });
        pay(&mut g);
        mode_window(&mut g);
        replay(&mut g, Action::ChooseMode { mode: 1 });
        g.move_object(id, Zone::Exile);
        replay(&mut g, Action::Pass);
        replay(&mut g, Action::Pass);
        for _ in 0..12 {
            if g.observe(p).library_choice.is_some() {
                break;
            }
            choose(&mut g, 1, Some(bear));
        }
        assert!(g.observe(p).library_choice.is_some());
        assert_eq!(g.state.resolving.as_ref().unwrap().item.mode, Some(1));
        let other = g.observe(p.other());
        assert!(other.library_choice.is_none() && other.actions.is_empty());
        let s = g.snapshot();
        let library = g.state.players[p.index()].library.len();
        settle(&mut g, 1, None);
        assert_eq!(g.state.players[p.index()].library.len(), library - 1);
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
        let after = g.snapshot();
        g.restore(&s).unwrap();
        settle(&mut g, 1, None);
        assert_eq!(g.snapshot(), after);
    }
}
#[test]
fn temporary_modal_growth_and_keywords_expire_while_permanent_counters_do_not() {
    for p in [Player::P0, Player::P1] {
        for mode in [0, 1] {
            let mut g = setup(p);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.state.objects[bear as usize].counters.plus_one = 1;
            let id = named(&mut g, p, NAMES[6], Zone::Hand);
            cast_body(&mut g, id, mode, None);
            g.begin_step(Step::BeginCombat);
            settle(&mut g, mode, Some(bear));
            if mode == 0 {
                assert_eq!(pt(&g, bear), (5, 3));
            } else {
                assert!(
                    g.keywords(bear)
                        .contains(Keywords::MENACE.union(Keywords::HASTE))
                );
            }
            let saved = g.snapshot();
            g.state.step = Step::Cleanup;
            g.finish_cleanup();
            assert_eq!(pt(&g, bear), (3, 3));
            assert!(
                !g.keywords(bear)
                    .contains(Keywords::MENACE.union(Keywords::HASTE))
            );
            g.restore(&saved).unwrap();
            assert_eq!(g.object_view(bear).counters.plus_one, 1);
        }
    }
}
#[test]
fn owned_landfall_and_morbid_step_guards_do_not_create_modes_for_opponents_or_false_history() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p.other());
        named(&mut g, p, NAMES[5], Zone::Battlefield);
        let land = named(&mut g, p.other(), "Forest", Zone::Hand);
        replay(&mut g, Action::PlayLand { card: land });
        assert!(g.state.pending_triggers.is_empty());
        named(&mut g, p, NAMES[11], Zone::Battlefield);
        g.begin_step(Step::End);
        assert!(g.state.pending_triggers.is_empty());
        g.state.active = p;
        g.begin_step(Step::End);
        assert!(g.state.pending_triggers.is_empty());
        g.state.creatures_died_this_turn = 1;
        g.begin_step(Step::End);
        mode_window(&mut g);
        assert_eq!(g.observe(p).mode_choices.len(), 2);
        replay(&mut g, Action::ChooseMode { mode: 0 });
        settle(&mut g, 0, None);
        assert_eq!(g.state.players[p.index()].life, 22);
    }
}
#[test]
fn second_spell_modal_trigger_is_owned_exactly_second_and_resets_each_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, NAMES[2], Zone::Battlefield);
        let count = |g: &Game| {
            g.state
                .objects
                .iter()
                .filter(|o| {
                    o.zone == Zone::Battlefield
                        && o.owner == p
                        && o.card
                            .token_info()
                            .is_some_and(|t| t.subtypes == "Human Soldier")
                })
                .count()
        };
        for want in [0, 2, 2] {
            let opt = named(&mut g, p, "Opt", Zone::Hand);
            cast_body(&mut g, opt, 0, None);
            assert_eq!(count(&g), want);
        }
        g.state.active = p.other();
        g.state.priority = p.other();
        for _ in 0..2 {
            let opt = named(&mut g, p.other(), "Opt", Zone::Hand);
            cast_body(&mut g, opt, 0, None);
        }
        assert_eq!(count(&g), 2);
        g.state.active = p;
        g.begin_turn();
        g.state.step = Step::Main1;
        g.state.players[p.index()].mana = [20; 6];
        for want in [2, 4] {
            let opt = named(&mut g, p, "Opt", Zone::Hand);
            cast_body(&mut g, opt, 0, None);
            assert_eq!(count(&g), want);
        }
    }
}
