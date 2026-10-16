use super::tests::{act, scenario, spawn};
use super::*;
const NAMES: [&str; 19] = [
    "Circuitous Route",
    "Map the Frontier",
    "Dragonstorm Forecaster",
    "Magitek Infantry",
    "Honored Knight-Captain",
    "Mystical Teachings",
    "Glimpse the Core",
    "Radioactive Spider",
    "Spinewoods Armadillo",
    "Call the Mountain Chocobo",
    "Chocobo Racetrack",
    "Gysahl Greens",
    "Harvester of Misery",
    "Outcaster Greenblade",
    "Pia, Aether Ascetic",
    "Simulacrum Shaper",
    "Solemn Simulacrum",
    "Steel Wrecking Ball",
    "Sureshot Sower",
];
fn named(g: &mut Game, p: Player, n: &str, z: Zone) -> CardId {
    spawn(g, p, Card::from_name(n).unwrap(), z)
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for s in &mut g.state.players {
        s.mana = [40; 6];
    }
    g
}
fn replay(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    let p = g.actor().unwrap();
    assert!(
        g.legal_actions().contains(&a),
        "{a:?} {:?}",
        g.state.pending
    );
    let mut f = g.clone();
    act(&mut f, a.clone());
    act(g, a.clone());
    let end = g.snapshot();
    assert_eq!(f.snapshot(), end);
    g.restore(&s).unwrap();
    assert_eq!(g.apply_index(p, r, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), end);
}
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), r);
}
fn continuation(g: &mut Game, target: Option<CardId>, mode: u16) {
    let aa = g.legal_actions();
    let a = target
        .and_then(|id| {
            aa.iter().find(|a| {
                **a == Action::ChooseTarget {
                    target: Target::Card(id),
                }
            })
        })
        .or_else(|| {
            aa.iter()
                .find(|a| matches!(a,Action::ChooseMode{mode:m} if *m==mode))
        })
        .or_else(|| {
            aa.iter().find(|a| {
                matches!(
                    a,
                    Action::PayCosts { .. }
                        | Action::ChooseSpellCost { cost: 0 }
                        | Action::ChooseOptional { accept: true }
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::FinishSearch
                        | Action::DeclineEffectPayment
                        | Action::Pass
                )
            })
        })
        .unwrap_or_else(|| panic!("no continuation: {aa:?}"));
    replay(g, a.clone());
}
fn idle(g: &Game) -> bool {
    g.state.announcing.is_none()
        && g.state.resolving.is_none()
        && g.state.stack.is_empty()
        && g.state.pending == Pending::Priority
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
}
fn settle(g: &mut Game, target: Option<CardId>) {
    g.prepare_triggers();
    for _ in 0..100 {
        if idle(g) {
            return;
        }
        continuation(g, target, 0);
    }
    panic!("unsettled {:?}", g.state.pending);
}
fn search(g: &mut Game, target: Option<CardId>) {
    g.prepare_triggers();
    for _ in 0..100 {
        if matches!(g.state.pending, Pending::EffectSearch { .. }) {
            return;
        }
        assert!(!idle(g), "no search");
        continuation(g, target, 0);
    }
    panic!("no search");
}
fn cast(g: &mut Game, id: CardId, target: Option<CardId>, mode: u16) {
    replay(g, Action::AnnounceCast { card: id });
    while g.state.announcing.is_some() {
        continuation(g, target, mode);
    }
}
fn activate(g: &mut Game, id: CardId, target: Option<CardId>) {
    replay(
        g,
        Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    while g.state.announcing.is_some() {
        continuation(g, target, 0);
    }
}
fn options(g: &Game) -> Vec<CardId> {
    g.legal_actions()
        .iter()
        .filter_map(|a| {
            if let Action::ChooseSearch { card } = a {
                Some(*card)
            } else {
                None
            }
        })
        .collect()
}
#[test]
fn all_nineteen_complete_bodies_cast_in_both_seats() {
    for p in [Player::P0, Player::P1] {
        for n in NAMES {
            let mut g = setup(p);
            let target = named(&mut g, p.other(), "Three Tree Mascot", Zone::Battlefield);
            g.state.objects[target as usize].growth = Stats {
                power: 10,
                toughness: 10,
            };
            let source = named(&mut g, p, n, Zone::Hand);
            assert!(Card::from_name(n).unwrap().is_supported(), "{n}");
            cast(&mut g, source, Some(target), 0);
            settle(&mut g, Some(target));
            assert_eq!(
                g.object_view(source).zone,
                if (Card::from_name(n).unwrap().kind_mask() & (4 | 8)) == 0 {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                },
                "{n}"
            );
            assert_eq!(g.observe(p).schema, OBSERVATION_SCHEMA);
        }
    }
}
#[test]
fn immutable_predicate_matrix_uses_intersection_union_name_and_actual_flash() {
    use crate::SearchFilter as F;
    fn c(n: &str) -> Card {
        Card::from_name(n).unwrap()
    }
    let filter = |n: &str| {
        let card = c(n);
        if let Some(Instruction::Search(s)) = card.spell_program(None).unwrap().1.first() {
            s.filter
        } else {
            panic!("{n}")
        }
    };
    for (name, yes, no) in [
        (
            "Circuitous Route",
            vec!["Forest", "Azorius Guildgate"],
            vec!["Commercial District", "Three Tree Mascot"],
        ),
        (
            "Map the Frontier",
            vec!["Wastes", "Bristling Backwoods"],
            vec!["Commercial District", "Three Tree Mascot"],
        ),
        (
            "Mystical Teachings",
            vec!["Shock", "Little Bear", "Celestial Armor"],
            vec!["Gysahl Greens", "Call the Mountain Chocobo"],
        ),
        (
            "Glimpse the Core",
            vec!["Forest"],
            vec!["Wastes", "Commercial District"],
        ),
    ] {
        let f = if name == "Glimpse the Core" {
            match c(name).spell_program(Some(0)).unwrap().1[0] {
                Instruction::Search(s) => s.filter,
                _ => panic!(),
            }
        } else {
            filter(name)
        };
        for n in yes {
            assert!(f.matches(c(n)), "{name}: {n}")
        }
        for n in no {
            assert!(!f.matches(c(n)), "{name}: {n}")
        }
    }
    let radio = c("Radioactive Spider");
    let Instruction::Search(s) = radio.activations()[0].program[0] else {
        panic!()
    };
    for n in ["Three Tree Mascot", "Crib Swap"] {
        assert!(s.filter.matches(c(n)), "{n}")
    }
    for n in ["Attercop", "Savannah Lions"] {
        assert!(!s.filter.matches(c(n)), "{n}")
    }
    assert!(F::Name("Magitek Infantry").matches(c("Magitek Infantry")));
    assert!(!F::Name("Magitek Infantry").matches(c("Steel Wrecking Ball")));
    assert!(!F::Subtype("Mountain").matches(c("Crib Swap")));
    assert!(!F::Keyword(Keywords::FLASH).matches(c("Think Twice")));
}
#[test]
fn two_same_name_search_objects_are_distinct_private_deferred_and_simultaneous() {
    for p in [Player::P0, Player::P1] {
        for n in ["Circuitous Route", "Map the Frontier"] {
            let mut g = setup(p);
            let source = named(&mut g, p, n, Zone::Hand);
            cast(&mut g, source, None, 0);
            search(&mut g, None);
            let aa = options(&g);
            let ids = [aa[0], aa[1]];
            let refs = [g.object_ref(ids[0]), g.object_ref(ids[1])];
            let rng = g.state.rng;
            let lib = g.state.players[p.index()].library.clone();
            for id in ids {
                replay(&mut g, Action::ChooseSearch { card: id });
            }
            assert_eq!(g.state.rng, rng);
            assert_eq!(g.state.players[p.index()].library, lib);
            assert!(options(&g).is_empty());
            assert!(g.observe(p.other()).search_choice.is_none());
            assert!(g.observe(p.other()).actions.is_empty());
            reject(&mut g, Action::ChooseSearch { card: ids[0] });
            replay(&mut g, Action::FinishSearch);
            for (id, r) in ids.into_iter().zip(refs) {
                let v = g.object_view(id);
                assert_eq!(v.zone, Zone::Battlefield);
                assert!(v.tapped);
                assert_eq!(v.reference.generation, r.generation + 1);
            }
            assert_ne!(g.state.rng, rng);
        }
    }
}
#[test]
fn named_searches_capture_source_and_do_not_require_source_to_survive() {
    for p in [Player::P0, Player::P1] {
        for n in [
            "Dragonstorm Forecaster",
            "Magitek Infantry",
            "Honored Knight-Captain",
            "Radioactive Spider",
        ] {
            let mut g = setup(p);
            let wanted = match n {
                "Dragonstorm Forecaster" => "Dragonstorm Globe",
                "Magitek Infantry" => n,
                "Honored Knight-Captain" => "Captain America's Shield",
                _ => "Three Tree Mascot",
            };
            let found = named(&mut g, p, wanted, Zone::Library);
            let source = named(&mut g, p, n, Zone::Battlefield);
            activate(&mut g, source, None);
            if matches!(n, "Honored Knight-Captain" | "Radioactive Spider") {
                assert_eq!(g.object_view(source).zone, Zone::Graveyard);
            } else {
                g.move_object(source, Zone::Graveyard);
            }
            search(&mut g, None);
            assert!(options(&g).contains(&found));
            replay(&mut g, Action::ChooseSearch { card: found });
            replay(&mut g, Action::FinishSearch);
            assert_eq!(
                g.object_view(found).zone,
                if matches!(n, "Magitek Infantry" | "Honored Knight-Captain") {
                    Zone::Battlefield
                } else {
                    Zone::Hand
                }
            );
            if n == "Magitek Infantry" {
                assert!(g.object_view(found).tapped);
            } else if !matches!(n, "Honored Knight-Captain") {
                assert!(
                    g.observe(p.other())
                        .known_hand
                        .contains(&g.object_ref(found))
                );
            }
            settle(&mut g, None);
        }
    }
}
#[test]
fn hand_discard_self_is_not_cycling_and_target_choices_precede_payment() {
    for p in [Player::P0, Player::P1] {
        for n in [
            "Spinewoods Armadillo",
            "Harvester of Misery",
            "Steel Wrecking Ball",
            "Sureshot Sower",
        ] {
            let mut g = setup(p);
            let target = named(
                &mut g,
                p.other(),
                if n == "Steel Wrecking Ball" {
                    "Captain America's Shield"
                } else {
                    "Three Tree Mascot"
                },
                Zone::Battlefield,
            );
            if n == "Sureshot Sower" {
                g.state.objects[target as usize].gained_keywords = Keywords::FLYING;
            }
            let source = named(&mut g, p, n, Zone::Hand);
            assert!(!g.state.objects[source as usize].card.activations()[0].cycling);
            let old = g.object_ref(source);
            replay(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            if n != "Spinewoods Armadillo" {
                assert_eq!(g.object_ref(source), old);
                assert_eq!(g.object_view(source).zone, Zone::Hand);
                replay(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(target),
                    },
                );
                assert_eq!(g.object_view(source).zone, Zone::Hand);
            }
            while g.state.announcing.is_some() {
                continuation(&mut g, None, 0);
            }
            assert_eq!(g.object_view(source).zone, Zone::Graveyard);
            let packet = g.state.stack.last().unwrap();
            assert_eq!(packet.object, old);
            assert_eq!(packet.kind, StackKind::ActivatedAbility { ability: 0 });
            let life = g.state.players[p.index()].life;
            if n == "Spinewoods Armadillo" {
                search(&mut g, None);
                assert!(!options(&g).is_empty());
                replay(&mut g, Action::FinishSearch);
                assert_eq!(g.state.players[p.index()].life, life + 3);
            } else {
                settle(&mut g, None);
                assert_eq!(
                    g.object_view(target).zone,
                    if n == "Steel Wrecking Ball" {
                        Zone::Battlefield
                    } else {
                        Zone::Graveyard
                    }
                );
            }
            g.state.priority = p;
            assert!(
                !g.legal_actions()
                    .iter()
                    .any(|a| matches!(a,Action::AnnounceActivate{card,..}if *card==source))
            );
        }
    }
}

fn birds(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(i, o)| {
            (o.owner == p
                && o.zone == Zone::Battlefield
                && o.card
                    .token_info()
                    .is_some_and(|t| t.factory_identity == "@token/607bf09599a16caa64c6fee1"))
            .then_some(i as CardId)
        })
        .collect()
}
#[test]
fn call_search_tail_is_deferred_and_full_chocobo_landfall_has_no_retroactive_entry() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let mountain = named(&mut g, p, "Mountain", Zone::Library);
        let call = named(&mut g, p, "Call the Mountain Chocobo", Zone::Hand);
        cast(&mut g, call, None, 0);
        search(&mut g, None);
        assert!(birds(&g, p).is_empty());
        replay(&mut g, Action::ChooseSearch { card: mountain });
        assert!(birds(&g, p).is_empty());
        assert_eq!(g.object_view(mountain).zone, Zone::Library);
        replay(&mut g, Action::FinishSearch);
        settle(&mut g, None);
        let old = birds(&g, p)[0];
        assert_eq!(
            (g.object_view(old).power, g.object_view(old).toughness),
            (2, 2)
        );
        assert_eq!(g.object_view(mountain).zone, Zone::Hand);
        let race = named(&mut g, p, "Chocobo Racetrack", Zone::Battlefield);
        g.state.priority = p;
        replay(&mut g, Action::PlayLand { card: mountain });
        settle(&mut g, None);
        let all = birds(&g, p);
        assert_eq!(all.len(), 2);
        assert_eq!(g.object_view(old).power, 3);
        let new = *all.iter().find(|&&i| i != old).unwrap();
        assert_eq!(g.object_view(new).power, 2);
        g.move_object(race, Zone::Graveyard);
        g.finish_cleanup();
        assert_eq!(g.object_view(old).power, 2);
        assert_eq!(g.object_view(new).power, 2);
    }
}
#[test]
fn simultaneous_search_landfall_only_existing_chocobos_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let greens = named(&mut g, p, "Gysahl Greens", Zone::Hand);
        cast(&mut g, greens, None, 0);
        settle(&mut g, None);
        let old = birds(&g, p)[0];
        named(&mut g, p, "Chocobo Racetrack", Zone::Battlefield);
        let route = named(&mut g, p, "Circuitous Route", Zone::Hand);
        cast(&mut g, route, None, 0);
        search(&mut g, None);
        let ids = options(&g);
        for &id in &ids[..2] {
            replay(&mut g, Action::ChooseSearch { card: id });
        }
        replay(&mut g, Action::FinishSearch);
        settle(&mut g, None);
        let all = birds(&g, p);
        assert_eq!(all.len(), 3);
        assert_eq!(g.object_view(old).power, 4);
        for id in all.into_iter().filter(|&i| i != old) {
            assert_eq!(g.object_view(id).power, 2);
        }
    }
}
#[test]
fn optional_search_decline_differs_from_accept_fail_and_dies_draws_remain_distinct() {
    for p in [Player::P0, Player::P1] {
        for n in ["Simulacrum Shaper", "Solemn Simulacrum"] {
            let mut g = setup(p);
            let source = named(&mut g, p, n, Zone::Hand);
            cast(&mut g, source, None, 0);
            while !matches!(g.state.pending, Pending::EffectOptional { .. }) {
                continuation(&mut g, None, 0);
            }
            let s = g.snapshot();
            let rng = g.state.rng;
            let library = g.state.players[p.index()].library.clone();
            assert!(g.observe(p).search_choice.is_none());
            replay(&mut g, Action::ChooseOptional { accept: false });
            settle(&mut g, None);
            assert_eq!(g.state.rng, rng);
            assert_eq!(g.state.players[p.index()].library, library);
            g.restore(&s).unwrap();
            replay(&mut g, Action::ChooseOptional { accept: true });
            assert!(!options(&g).is_empty());
            replay(&mut g, Action::FinishSearch);
            settle(&mut g, None);
            assert_ne!(g.state.rng, rng);
            let hand = g
                .state
                .objects
                .iter()
                .filter(|o| o.owner == p && o.zone == Zone::Hand)
                .count();
            g.move_object(source, Zone::Graveyard);
            g.prepare_triggers();
            if n == "Solemn Simulacrum" {
                while !matches!(g.state.pending, Pending::EffectOptional { .. }) {
                    continuation(&mut g, None, 0);
                }
                let death = g.snapshot();
                replay(&mut g, Action::ChooseOptional { accept: false });
                settle(&mut g, None);
                assert_eq!(
                    g.state
                        .objects
                        .iter()
                        .filter(|o| o.owner == p && o.zone == Zone::Hand)
                        .count(),
                    hand
                );
                g.restore(&death).unwrap();
                replay(&mut g, Action::ChooseOptional { accept: true });
            }
            settle(&mut g, None);
            assert_eq!(
                g.state
                    .objects
                    .iter()
                    .filter(|o| o.owner == p && o.zone == Zone::Hand)
                    .count(),
                hand + 1
            );
        }
    }
}
#[test]
fn pia_committed_optional_discard_precedes_search_and_later_rollback_preserves_it() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let enchantment = named(&mut g, p, "Turtle Power!", Zone::Library);
        let discard = named(&mut g, p, "Forest", Zone::Hand);
        let source = named(&mut g, p, "Pia, Aether Ascetic", Zone::Hand);
        cast(&mut g, source, None, 0);
        while g.observe(p).effect_choice.is_none() {
            continuation(&mut g, None, 0);
        }
        let before = g.snapshot();
        let rng = g.state.rng;
        replay(&mut g, Action::DeclineEffectPayment);
        settle(&mut g, None);
        assert_eq!(g.state.rng, rng);
        assert_eq!(g.object_view(discard).zone, Zone::Hand);
        g.restore(&before).unwrap();
        replay(&mut g, Action::Discard { card: discard });
        assert_eq!(g.object_view(discard).zone, Zone::Graveyard);
        assert!(options(&g).contains(&enchantment));
        g.state.objects[enchantment as usize].generation = u64::MAX;
        replay(&mut g, Action::ChooseSearch { card: enchantment });
        let s = g.snapshot();
        let rev = g.revision();
        assert!(matches!(
            g.apply(p, Action::FinishSearch),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), s);
        assert_eq!(g.revision(), rev);
        assert_eq!(g.object_view(discard).zone, Zone::Graveyard);
        assert_eq!(g.state.rng, rng);
    }
}

#[test]
fn all_three_new_flashback_bodies_use_real_graveyard_costs_and_exile_on_completion() {
    for p in [Player::P0, Player::P1] {
        for n in [
            "Mystical Teachings",
            "Call the Mountain Chocobo",
            "Gysahl Greens",
        ] {
            let mut g = setup(p);
            let source = named(&mut g, p, n, Zone::Graveyard);
            replay(&mut g, Action::AnnounceFlashback { card: source });
            while g.state.announcing.is_some() {
                continuation(&mut g, None, 0);
            }
            let item = g.state.stack.last().unwrap();
            assert_eq!(
                item.cast,
                Some(CastInfo {
                    origin: Zone::Graveyard,
                    method: CastMethod::Flashback,
                    offspring: None,
                    kicker: None,
                    bargain: None,
                    evidence: None,
                })
            );
            settle(&mut g, None);
            assert_eq!(g.object_view(source).zone, Zone::Exile);
            assert_eq!(birds(&g, p).len(), usize::from(n != "Mystical Teachings"));
        }
    }
}
#[test]
fn glimpse_cave_mode_is_owned_exact_generation_tapped_and_cannot_follow_reentry() {
    for p in [Player::P0, Player::P1] {
        for leave in [false, true] {
            let mut g = setup(p);
            let cave = named(&mut g, p, "Captivating Cave", Zone::Graveyard);
            let wrong = named(&mut g, p.other(), "Captivating Cave", Zone::Graveyard);
            let source = named(&mut g, p, "Glimpse the Core", Zone::Hand);
            replay(&mut g, Action::AnnounceCast { card: source });
            replay(&mut g, Action::ChooseMode { mode: 1 });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(wrong),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(0),
                },
            );
            replay(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(cave),
                },
            );
            while g.state.announcing.is_some() {
                continuation(&mut g, None, 1);
            }
            let before = g.object_ref(cave);
            if leave {
                g.move_object(cave, Zone::Hand);
                g.move_object(cave, Zone::Graveyard);
            }
            settle(&mut g, None);
            assert_eq!(
                g.object_view(cave).zone,
                if leave {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(
                g.object_ref(cave).generation,
                before.generation + if leave { 2 } else { 1 }
            );
            if !leave {
                assert!(g.object_view(cave).tapped);
            }
        }
    }
}
#[test]
fn hand_abilities_enforce_origin_live_flying_timing_funding_and_atomic_cost_failure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ground = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let sower = named(&mut g, p, "Sureshot Sower", Zone::Hand);
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::AnnounceActivate{card,..}if *card==sower))
        );
        g.state.objects[ground as usize].gained_keywords = Keywords::FLYING;
        g.state.players[p.index()].mana = [0; 6];
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::AnnounceActivate{card,..}if *card==sower))
        );
        g.state.players[p.index()].mana = [40; 6];
        activate(&mut g, sower, Some(ground));
        g.state.objects[ground as usize].gained_keywords = Keywords::NONE;
        settle(&mut g, None);
        assert_eq!(g.object_view(ground).zone, Zone::Battlefield);
        let spider = named(&mut g, p, "Radioactive Spider", Zone::Battlefield);
        g.state.step = Step::Upkeep;
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::AnnounceActivate{card,..}if *card==spider))
        );
        g.state.step = Step::Main1;
        let arm = named(&mut g, p, "Spinewoods Armadillo", Zone::Battlefield);
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::AnnounceActivate{card,..}if *card==arm))
        );
        g.move_object(arm, Zone::Hand);
        g.state.objects[arm as usize].generation = u64::MAX;
        replay(
            &mut g,
            Action::AnnounceActivate {
                card: arm,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        let old = g.snapshot();
        let rev = g.revision();
        assert!(matches!(g.apply(p, pay), Err(Error::CapacityExceeded(_))));
        assert_eq!(g.snapshot(), old);
        assert_eq!(g.revision(), rev);
    }
}
#[test]
fn scaling_other_artifact_and_desert_sources_are_live_owner_scoped_and_indexed() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let infantry = named(&mut g, p, "Magitek Infantry", Zone::Battlefield);
        let blade = named(&mut g, p, "Outcaster Greenblade", Zone::Battlefield);
        assert_eq!(g.object_view(infantry).power, 1);
        assert_eq!(g.object_view(blade).power, 1);
        named(&mut g, p.other(), "Sandstorm Verge", Zone::Battlefield);
        named(&mut g, p.other(), "Steel Wrecking Ball", Zone::Battlefield);
        assert_eq!(g.object_view(infantry).power, 1);
        assert_eq!(g.object_view(blade).power, 1);
        let artifact = named(&mut g, p, "Steel Wrecking Ball", Zone::Battlefield);
        let desert = named(&mut g, p, "Sandstorm Verge", Zone::Battlefield);
        assert_eq!(g.object_view(infantry).power, 2);
        assert_eq!(
            (g.object_view(blade).power, g.object_view(blade).toughness),
            (2, 3)
        );
        g.move_object(artifact, Zone::Graveyard);
        g.move_object(desert, Zone::Hand);
        assert_eq!(g.object_view(infantry).power, 1);
        assert_eq!(g.object_view(blade).power, 1);
    }
}
#[test]
fn residual_entry_stats_tokens_damage_other_creatures_and_cleanup() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = named(&mut g, p, "Three Tree Mascot", Zone::Battlefield);
        let foe = named(&mut g, p.other(), "Three Tree Mascot", Zone::Battlefield);
        for id in [own, foe] {
            g.state.objects[id as usize].counters.plus_one = 5;
        }
        let harvester = named(&mut g, p, "Harvester of Misery", Zone::Hand);
        cast(&mut g, harvester, None, 0);
        settle(&mut g, None);
        assert_eq!(
            (
                g.object_view(harvester).power,
                g.object_view(harvester).toughness
            ),
            (5, 4)
        );
        assert!(g.object_view(harvester).keywords.contains(Keywords::MENACE));
        for id in [own, foe] {
            assert_eq!(
                (g.object_view(id).power, g.object_view(id).toughness),
                (5, 4)
            );
        }
        g.finish_cleanup();
        for id in [own, foe] {
            assert_eq!(
                (g.object_view(id).power, g.object_view(id).toughness),
                (7, 6)
            );
        }
        g.state.step = Step::Main1;
        g.state.active = p;
        g.state.priority = p;
        g.state.players[p.index()].mana = [40; 6];
        let knight = named(&mut g, p, "Honored Knight-Captain", Zone::Hand);
        cast(&mut g, knight, None, 0);
        settle(&mut g, None);
        let soldier = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| {
                o.owner == p
                    && o.zone == Zone::Battlefield
                    && o.card
                        .token_info()
                        .is_some_and(|t| t.subtypes == "Human Soldier")
            })
            .map(|(i, _)| i as CardId)
            .unwrap();
        let v = g.object_view(soldier);
        assert_eq!((v.power, v.toughness), (1, 1));
        assert_eq!(v.token.unwrap().colors, 1);
        let steel = named(&mut g, p, "Steel Wrecking Ball", Zone::Hand);
        cast(&mut g, steel, Some(foe), 0);
        settle(&mut g, Some(foe));
        assert_eq!(g.object_view(foe).damage, 5);
        for (n, pt, kw) in [
            (
                "Radioactive Spider",
                (1, 1),
                Keywords::REACH.union(Keywords::DEATHTOUCH),
            ),
            ("Spinewoods Armadillo", (7, 7), Keywords::REACH),
            ("Sureshot Sower", (3, 1), Keywords::REACH),
        ] {
            let id = named(&mut g, p, n, Zone::Battlefield);
            let v = g.object_view(id);
            assert_eq!((v.power, v.toughness), pt);
            assert!(v.keywords.contains(kw));
        }
    }
}

#[test]
fn ordinary_hand_discard_emits_exactly_one_noncycling_event_not_a_spell() {
    for p in [Player::P0, Player::P1] {
        for n in [
            "Spinewoods Armadillo",
            "Harvester of Misery",
            "Steel Wrecking Ball",
            "Sureshot Sower",
        ] {
            let mut g = setup(p);
            let observer = named(&mut g, p, "Marauding Mako", Zone::Battlefield);
            let target = named(
                &mut g,
                p.other(),
                if n == "Steel Wrecking Ball" {
                    "Bear Trap"
                } else {
                    "Three Tree Mascot"
                },
                Zone::Battlefield,
            );
            g.state.objects[target as usize].gained_keywords = Keywords::FLYING;
            let source = named(&mut g, p, n, Zone::Hand);
            activate(&mut g, source, Some(target));
            let events: Vec<_> = g
                .state
                .stack
                .iter()
                .chain(&g.state.pending_triggers)
                .chain(&g.state.ordering_triggers)
                .filter(|s| s.card == observer)
                .filter_map(|s| s.discard_event)
                .collect();
            assert_eq!(
                events,
                vec![DiscardTriggerView {
                    player: p,
                    amount: 1,
                    cycling: false
                }]
            );
            assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 0);
            assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 1);
            settle(&mut g, None);
            assert_eq!(g.object_view(observer).counters.plus_one, 1);
        }
    }
}
#[test]
fn late_multi_entry_capacity_and_post_search_life_overflow_restore_rng_choices_and_prior_costs() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let route = named(&mut g, p, "Circuitous Route", Zone::Hand);
        cast(&mut g, route, None, 0);
        search(&mut g, None);
        let ids = options(&g);
        g.state.objects[ids[1] as usize].generation = u64::MAX;
        for &id in &ids[..2] {
            replay(&mut g, Action::ChooseSearch { card: id });
        }
        let before = g.snapshot();
        let rev = g.revision();
        assert!(matches!(
            g.apply(p, Action::FinishSearch),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.revision(), rev);
        assert_eq!(g.object_view(ids[0]).zone, Zone::Library);
        let mut g = setup(p);
        let arm = named(&mut g, p, "Spinewoods Armadillo", Zone::Hand);
        activate(&mut g, arm, None);
        search(&mut g, None);
        assert_eq!(g.object_view(arm).zone, Zone::Graveyard);
        g.state.players[p.index()].life = i128::MAX;
        let before = g.snapshot();
        let rev = g.revision();
        assert!(matches!(
            g.apply(p, Action::FinishSearch),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.revision(), rev);
        assert_eq!(g.object_view(arm).zone, Zone::Graveyard);
    }
}

#[test]
fn composite_filters_are_shared_immutable_and_only_selected_ids_add_snapshot_bytes() {
    let mut g = setup(Player::P0);
    let source = named(&mut g, Player::P0, "Circuitous Route", Zone::Hand);
    cast(&mut g, source, None, 0);
    search(&mut g, None);
    let bytes = g.state_bytes();
    let id = options(&g)[0];
    let snapshot = g.snapshot();
    replay(&mut g, Action::ChooseSearch { card: id });
    assert_eq!(g.state_bytes(), bytes + std::mem::size_of::<CardId>());
    g.restore(&snapshot).unwrap();
    assert_eq!(g.state_bytes(), bytes);
    println!(
        "SearchFilter={} SearchSpec={} active_search_bytes={bytes}",
        std::mem::size_of::<crate::SearchFilter>(),
        std::mem::size_of::<SearchSpec>()
    );
    assert_eq!(std::mem::size_of::<crate::SearchFilter>(), 24);
}
