//! Concrete split-search rules and sparse owned choices, not name-based admissions.
use super::tests::{scenario, spawn};
use super::*;
const BLOOM: &str = "Bloomvine Regent // Claim Territory";
fn replay(g: &mut Game, a: Action) {
    let p = g.actor().unwrap();
    let before = g.snapshot();
    let rev = g.revision();
    let i = g.legal_actions().iter().position(|v| v == &a).unwrap();
    assert_eq!(g.apply(p.other(), a.clone()), Err(Error::WrongActor));
    assert_eq!(g.snapshot(), before);
    let parent = g.clone();
    let mut child = parent.clone();
    drop(parent);
    child.apply(p, a.clone()).unwrap();
    g.apply_index(p, rev, i).unwrap();
    assert_eq!(child.snapshot(), g.snapshot());
    let after = g.snapshot();
    g.restore(&before).unwrap();
    assert_eq!(g.apply_index(p, rev, i), Err(Error::StaleRevision));
    g.apply(p, a).unwrap();
    assert_eq!(g.snapshot(), after);
    for viewer in [p, p.other()] {
        let mut a = serde_json::to_value(g.observe(viewer)).unwrap();
        let mut b = serde_json::to_value(child.observe(viewer)).unwrap();
        a["revision"] = 0.into();
        b["revision"] = 0.into();
        assert_eq!(a, b);
    }
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for q in [p, p.other()] {
        g.state.players[q.index()].mana = [50; 6];
    }
    g
}
fn drive(g: &mut Game, search: bool) {
    for _ in 0..300 {
        if search && matches!(g.state.pending, Pending::EffectSearch { .. }) {
            return;
        }
        if !search
            && g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
            && g.state.announcing.is_none()
            && g.state.resolving.is_none()
        {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .max_by_key(|a| match a {
                Action::PayCosts { .. } => 2000,
                Action::PutTrigger { .. } | Action::FinishTargets => 1500,
                Action::Pass => 1000,
                Action::Concede
                | Action::AnnounceCast { .. }
                | Action::AnnounceOmen { .. }
                | Action::AnnounceActivate { .. }
                | Action::Activate { .. } => -1000,
                _ => 0,
            })
            .unwrap();
        replay(g, a);
    }
    panic!("unsettled {:?}", g.state.pending)
}
fn begin(g: &mut Game, p: Player, omen: bool, overflow: bool) -> CardId {
    let card = Card::from_name(if omen { BLOOM } else { "Troop of Ponies" }).unwrap();
    let id = spawn(
        g,
        p,
        card,
        if omen { Zone::Hand } else { Zone::Battlefield },
    );
    if overflow {
        g.state.objects[id as usize].generation = u64::MAX - 1;
    }
    if omen {
        replay(g, Action::AnnounceOmen { card: id });
    } else {
        g.state.objects[id as usize].summoning_sick = false;
        replay(
            g,
            Action::AnnounceActivate {
                card: id,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
    }
    drive(g, true);
    id
}
#[test]
fn split_search_zero_one_two_both_destinations_exact_rng_source_and_public_knowledge() {
    for p in [Player::P0, Player::P1] {
        for omen in [false, true] {
            for count in 0..=2 {
                for chosen in 0..if count == 2 { 2 } else { 1 } {
                    let mut g = setup(p);
                    let source = begin(&mut g, p, omen, false);
                    assert_eq!(
                        g.object_view(source).zone,
                        if omen { Zone::Stack } else { Zone::Graveyard }
                    );
                    let before_rng = g.state.rng;
                    let original = g.state.players[p.index()].library.to_vec();
                    let selected = original[..count].to_vec();
                    for &card in &selected {
                        replay(&mut g, Action::ChooseSearch { card });
                    }
                    assert_eq!(g.state.rng, before_rng);
                    assert_eq!(g.state.players[p.index()].library.to_vec(), original);
                    assert!(g.observe(p.other()).search_choice.is_none());
                    let mut expected = original
                        .iter()
                        .copied()
                        .filter(|id| !selected.contains(id))
                        .collect::<Vec<_>>();
                    let mut rng = before_rng;
                    rng.shuffle(&mut expected);
                    if omen {
                        expected.push(source);
                        rng.shuffle(&mut expected);
                    }
                    replay(&mut g, Action::FinishSearch);
                    if count == 2 {
                        assert_eq!(g.state.rng, before_rng);
                        for viewer in [p, p.other()] {
                            let view = g.observe(viewer);
                            assert!(view.search_choice.is_none());
                            let public = view.search_distribution.unwrap();
                            assert_eq!(public.player, p);
                            assert_eq!(public.candidates.len(), 2);
                            assert_eq!(
                                public.candidates.iter().map(|v| v.id).collect::<Vec<_>>(),
                                selected
                            );
                            assert!(view.objects.iter().all(|v| v.zone != Zone::Library));
                        }
                        assert!(!g.legal_actions().contains(&Action::FinishSearch));
                        replay(
                            &mut g,
                            Action::ChooseSearchBattlefield {
                                card: selected[chosen],
                            },
                        );
                    }
                    drive(&mut g, false);
                    assert_eq!(g.state.rng, rng);
                    assert_eq!(g.state.players[p.index()].library.to_vec(), expected);
                    for (index, &id) in selected.iter().enumerate() {
                        let bf = count == 1 || index == chosen;
                        assert_eq!(
                            g.object_view(id).zone,
                            if bf { Zone::Battlefield } else { Zone::Hand }
                        );
                        if bf {
                            assert!(g.object_view(id).tapped);
                        } else {
                            assert!(
                                g.observe(p.other())
                                    .objects
                                    .iter()
                                    .any(|v| v.id == id && v.zone == Zone::Hand)
                            );
                        }
                    }
                    assert_eq!(
                        g.object_view(source).zone,
                        if omen { Zone::Library } else { Zone::Graveyard }
                    );
                    assert_eq!(
                        g.state.players[p.index()]
                            .library
                            .iter()
                            .filter(|&&id| id == source)
                            .count(),
                        usize::from(omen)
                    );
                    assert!(g.state.adventures.is_none());
                    assert!(g.state.exile_permissions.is_empty());
                }
            }
        }
    }
}
#[test]
fn basic_forest_split_excludes_other_basics_nonbasic_forests_and_unselected_private_library() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let plains = spawn(&mut g, p, Card::Plains, Zone::Library);
        let nonbasic = Card::catalogue()
            .find(|c| {
                c.is_supported() && c.is_land() && !c.is_basic_land() && c.has_subtype("Forest")
            })
            .unwrap();
        let dual = spawn(&mut g, p, nonbasic, Zone::Library);
        begin(&mut g, p, true, false);
        assert!(
            !g.legal_actions()
                .contains(&Action::ChooseSearch { card: plains })
        );
        assert!(
            !g.legal_actions()
                .contains(&Action::ChooseSearch { card: dual })
        );
        let ids = g
            .legal_actions()
            .iter()
            .filter_map(|a| match a {
                Action::ChooseSearch { card } => Some(*card),
                _ => None,
            })
            .take(2)
            .collect::<Vec<_>>();
        for &card in &ids {
            replay(&mut g, Action::ChooseSearch { card });
        }
        replay(&mut g, Action::FinishSearch);
        let public = g.observe(p.other()).search_distribution.unwrap();
        assert_eq!(public.candidates.len(), 2);
        assert!(public.candidates.iter().all(|v| ids.contains(&v.id)));
        let saved = g.snapshot();
        let rev = g.revision();
        assert_eq!(
            g.apply(p, Action::ChooseSearchBattlefield { card: dual }),
            Err(Error::IllegalAction)
        );
        assert_eq!(g.snapshot(), saved);
        assert_eq!(g.revision(), rev);
    }
}
#[test]
fn late_omen_source_generation_exhaustion_rolls_back_split_moves_search_shuffle_and_revealed_choice()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        begin(&mut g, p, true, true);
        let ids = g
            .legal_actions()
            .iter()
            .filter_map(|a| match a {
                Action::ChooseSearch { card } => Some(*card),
                _ => None,
            })
            .take(2)
            .collect::<Vec<_>>();
        for &card in &ids {
            replay(&mut g, Action::ChooseSearch { card });
        }
        replay(&mut g, Action::FinishSearch);
        let saved = g.snapshot();
        let rev = g.revision();
        let views = [
            serde_json::to_value(g.observe(p)).unwrap(),
            serde_json::to_value(g.observe(p.other())).unwrap(),
        ];
        for _ in 0..2 {
            assert!(matches!(
                g.apply(p, Action::ChooseSearchBattlefield { card: ids[1] }),
                Err(Error::CapacityExceeded(_))
            ));
            assert_eq!(g.snapshot(), saved);
            assert_eq!(g.revision(), rev);
            assert_eq!(
                views,
                [
                    serde_json::to_value(g.observe(p)).unwrap(),
                    serde_json::to_value(g.observe(p.other())).unwrap()
                ]
            );
        }
    }
}
fn next_main(g: &mut Game, p: Player) {
    let turn = g.state.turn;
    for _ in 0..300 {
        if g.state.turn > turn && g.state.active == p && g.state.step == Step::Main1 {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass | Action::FinishAttackers | Action::FinishBlockers
                )
            })
            .unwrap();
        replay(g, a);
    }
    panic!("no real next main")
}
fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = spawn(g, p, Card::from_name(name).unwrap(), Zone::Hand);
    replay(g, Action::AnnounceCast { card: id });
    drive(g, false);
    id
}
#[test]
fn complete_bloomvine_normal_paid_self_other_dragon_non_dragon_and_opponent_events() {
    for p in [Player::P0, Player::P1] {
        let q = p.other();
        let mut g = setup(p);
        for _ in 0..8 {
            spawn(&mut g, q, Card::Forest, Zone::Battlefield);
            spawn(&mut g, q, Card::Island, Zone::Battlefield);
        }
        let life = g.state.players[p.index()].life;
        let source = paid(&mut g, p, BLOOM);
        assert_eq!(g.state.players[p.index()].life, life + 3);
        let view = g.object_view(source);
        assert_eq!((view.power, view.toughness), (4, 5));
        assert!(view.keywords.contains(Keywords::FLYING));
        paid(&mut g, p, "Dirgur Island Dragon // Skimming Strike");
        assert_eq!(g.state.players[p.index()].life, life + 6);
        paid(&mut g, p, "Bear Cub");
        assert_eq!(g.state.players[p.index()].life, life + 6);
        next_main(&mut g, q);
        paid(&mut g, q, "Dirgur Island Dragon // Skimming Strike");
        assert_eq!(g.state.players[p.index()].life, life + 6);
    }
}
#[test]
fn simultaneous_bloomvine_sources_each_trigger_for_self_and_other_exactly_once() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let life = g.state.players[p.index()].life;
        let card = Card::from_name(BLOOM).unwrap();
        let a = spawn(&mut g, p, card, Zone::Hand);
        let b = spawn(&mut g, p, card, Zone::Hand);
        let moves = [a, b]
            .into_iter()
            .map(|id| g.entry_move(id, Zone::Battlefield, false))
            .collect();
        g.stage_entries(moves, None);
        g.give_priority();
        drive(&mut g, false);
        assert_eq!(g.state.players[p.index()].life, life + 12);
    }
}
#[test]
fn complete_troop_activation_requires_ready_untapped_body_and_two_mana_then_sacrifices() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let mana_before: u64 = g.state.players[p.index()]
            .mana
            .into_iter()
            .map(u64::from)
            .sum();
        let id = paid(&mut g, p, "Troop of Ponies");
        assert_eq!(g.object_view(id).zone, Zone::Battlefield);
        assert_eq!(
            (g.object_view(id).power, g.object_view(id).toughness),
            (2, 1)
        );
        assert_eq!(
            g.state.players[p.index()]
                .mana
                .into_iter()
                .map(u64::from)
                .sum::<u64>(),
            mana_before - 2
        );
        assert!(g.state.objects[id as usize].summoning_sick);
        let available = |g: &Game| {
            g.legal_actions().iter().any(|a|matches!(a,Action::AnnounceActivate{card,..}|Action::Activate{card,..}if *card==id))
        };
        g.state.objects[id as usize].summoning_sick = true;
        assert!(!available(&g));
        g.state.objects[id as usize].summoning_sick = false;
        g.state.objects[id as usize].tapped = true;
        assert!(!available(&g));
        g.state.objects[id as usize].tapped = false;
        g.state.players[p.index()].mana = [0; 6];
        assert!(!available(&g));
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        assert!(available(&g));
        replay(
            &mut g,
            Action::AnnounceActivate {
                card: id,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        drive(&mut g, true);
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert!(g.observe(p).search_choice.is_some());
    }
}

#[test]
fn zero_qualifying_forests_still_search_shuffles_then_shuffles_the_actual_omen_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        for id in g.state.players[p.index()].library.iter().copied() {
            g.state.objects[id as usize].card = Card::Plains;
        }
        let source = begin(&mut g, p, true, false);
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseSearch { .. }))
        );
        let mut expected = g.state.players[p.index()].library.to_vec();
        let mut rng = g.state.rng;
        rng.shuffle(&mut expected);
        expected.push(source);
        rng.shuffle(&mut expected);
        replay(&mut g, Action::FinishSearch);
        drive(&mut g, false);
        assert_eq!(g.state.rng, rng);
        assert_eq!(g.state.players[p.index()].library.to_vec(), expected);
    }
}
#[test]
fn actually_paid_counterspell_prevents_all_search_and_completion_shuffles() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let q = p.other();
        let source = spawn(&mut g, p, Card::from_name(BLOOM).unwrap(), Zone::Hand);
        replay(&mut g, Action::AnnounceOmen { card: source });
        for _ in 0..30 {
            if g.state.pending == Pending::Priority
                && g.state.stack.iter().any(|s| s.card == source)
            {
                break;
            }
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
                .unwrap();
            replay(&mut g, a);
        }
        let library = g.state.players[p.index()].library.clone();
        let rng = g.state.rng;
        let counter = spawn(&mut g, q, Card::Counterspell, Zone::Hand);
        replay(&mut g, Action::Pass);
        replay(&mut g, Action::AnnounceCast { card: counter });
        drive(&mut g, false);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert_eq!(g.object_view(counter).zone, Zone::Graveyard);
        assert_eq!(g.state.rng, rng);
        assert_eq!(g.state.players[p.index()].library, library);
        assert!(g.observe(p).search_choice.is_none());
        assert!(g.observe(q).search_distribution.is_none());
        assert!(g.state.adventures.is_none());
    }
}

#[test]
fn actual_opponent_turn_whirl_entry_gains_bloom_life_and_enables_split_omen_not_horse() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.active = p.other();
        g.state.priority = p;
        let bloom = spawn(
            &mut g,
            p,
            Card::from_name(BLOOM).unwrap(),
            Zone::Battlefield,
        );
        let troop = spawn(
            &mut g,
            p,
            Card::from_name("Troop of Ponies").unwrap(),
            Zone::Hand,
        );
        let omen = spawn(&mut g, p, Card::from_name(BLOOM).unwrap(), Zone::Hand);
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceOmen { card: omen })
        );
        let whirl = spawn(
            &mut g,
            p,
            Card::from_name("Whirlwing Stormbrood // Dynamic Soar").unwrap(),
            Zone::Hand,
        );
        let life = g.state.players[p.index()].life;
        replay(&mut g, Action::AnnounceCast { card: whirl });
        drive(&mut g, false);
        assert_eq!(g.state.objects[whirl as usize].zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].life, life + 3);
        assert_eq!(g.state.objects[bloom as usize].zone, Zone::Battlefield);
        assert_eq!(g.state.active, p.other());
        if g.actor() == Some(p.other()) {
            replay(&mut g, Action::Pass);
        }
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: troop })
        );
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceOmen { card: omen })
        );
        replay(&mut g, Action::AnnounceOmen { card: omen });
        drive(&mut g, true);
        let selected = g.state.players[p.index()].library[..2].to_vec();
        for &card in &selected {
            replay(&mut g, Action::ChooseSearch { card });
        }
        replay(&mut g, Action::FinishSearch);
        assert_eq!(
            g.observe(p)
                .search_distribution
                .as_ref()
                .unwrap()
                .candidates
                .len(),
            2
        );
        assert!(g.observe(p.other()).search_choice.is_none());
        replay(
            &mut g,
            Action::ChooseSearchBattlefield { card: selected[1] },
        );
        drive(&mut g, false);
        assert_eq!(g.state.active, p.other());
        assert_eq!(g.state.objects[omen as usize].zone, Zone::Library);
        assert_eq!(
            g.state.objects[selected[1] as usize].zone,
            Zone::Battlefield
        );
        assert!(g.state.objects[selected[1] as usize].tapped);
        assert_eq!(g.state.objects[selected[0] as usize].zone, Zone::Hand);
        assert!(g.state.adventures.is_none());
    }
}
