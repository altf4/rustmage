use super::tests::{act, scenario, spawn};
use super::*;
use crate::cards::{
    LibraryRemainder, LibrarySelectionCount, LibrarySelectionFilter as F, LibrarySelectionSpec,
};
const NAMES: [&str; 12] = [
    "Ashe, Princess of Dalmasca",
    "Casey Jones, Jury-Rig Justiciar",
    "Cowabunga!",
    "Eclipsed Boggart",
    "Eclipsed Elf",
    "Eclipsed Flamekin",
    "Eclipsed Kithkin",
    "Eclipsed Merrow",
    "Frontier Seeker",
    "Master Piandao",
    "Staunch Crewmate",
    "Thundertrap Trainer",
];
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    assert!(card.is_supported(), "{name}");
    spawn(g, p, card, zone)
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for s in &mut g.state.players {
        s.mana = [30; 6];
    }
    g
}
fn top(g: &mut Game, p: Player, names: &[&str]) -> Vec<CardId> {
    names
        .iter()
        .rev()
        .map(|n| named(g, p, n, Zone::Library))
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}
fn replay(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    let p = g.actor().unwrap();
    let mut f = g.clone();
    act(&mut f, a.clone());
    act(g, a.clone());
    let expected = g.snapshot();
    assert_eq!(expected, f.snapshot());
    g.restore(&s).unwrap();
    assert_eq!(g.apply_index(p, r, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), expected);
}
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(s, g.snapshot());
    assert_eq!(r, g.revision());
}
fn choose(g: &mut Game) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::SelectLibrary { .. }))
        .or_else(|| {
            g.legal_actions().into_iter().find(|a| {
                matches!(
                    a,
                    Action::FinishLibrarySelection
                        | Action::OrderLibraryBottom { .. }
                        | Action::ChooseOffspring { pay: false }
                        | Action::ChooseSpellCost { .. }
                        | Action::PayCosts { .. }
                        | Action::PutTrigger { .. }
                        | Action::Pass
                )
            })
        })
        .expect("fixture continuation");
    replay(g, a);
}
fn idle(g: &Game) -> bool {
    g.state.announcing.is_none()
        && g.state.resolving.is_none()
        && g.state.stack.is_empty()
        && g.state.pending == Pending::Priority
}
fn settle(g: &mut Game) {
    for _ in 0..200 {
        if idle(g) {
            return;
        }
        choose(g);
    }
    panic!("unsettled fixture");
}
fn announce(g: &mut Game, id: CardId, pay: bool, cost: u16) {
    replay(g, Action::AnnounceCast { card: id });
    if matches!(g.state.pending, Pending::Offspring { .. }) {
        replay(g, Action::ChooseOffspring { pay });
    }
    if matches!(g.state.pending, Pending::SpellCost { .. }) {
        replay(g, Action::ChooseSpellCost { cost });
    }
    while g.state.announcing.is_some() {
        choose(g);
    }
}
fn attack(g: &mut Game, p: Player, id: CardId) {
    g.state.objects[id as usize].summoning_sick = false;
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    replay(g, Action::Attack { card: id });
    replay(g, Action::FinishAttackers);
}
fn seek(g: &mut Game) {
    for _ in 0..100 {
        if g.observe(g.actor().unwrap()).library_selection.is_some() {
            return;
        }
        choose(g);
    }
    panic!("missing window");
}
fn window(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    announce(g, id, false, 0);
    if matches!(name, "Ashe, Princess of Dalmasca" | "Master Piandao") {
        settle(g);
        assert!(g.state.resolving.is_none());
        attack(g, p, id);
    }
    seek(g);
    id
}
fn candidates(g: &Game) -> Vec<CardId> {
    g.observe(g.actor().unwrap())
        .library_selection
        .unwrap()
        .candidates
        .iter()
        .map(|r| r.card)
        .collect()
}
#[test]
fn all_twelve_complete_bodies_execute_in_both_seats_with_real_attack_or_entry_events() {
    for p in [Player::P0, Player::P1] {
        for (i, name) in NAMES.iter().enumerate() {
            let mut g = setup(p);
            top(
                &mut g,
                p,
                &[
                    "Prophetic Prism",
                    "Chitinous Graspling",
                    "Plains",
                    "Forest",
                    "Opt",
                ],
            );
            let source = window(&mut g, p, name);
            let view = g.observe(p).library_selection.unwrap();
            assert_eq!(view.remaining.len(), if i == 0 || i == 8 { 5 } else { 4 });
            assert_eq!(
                (view.minimum, view.maximum, view.reveal, view.remainder),
                (0, 1, true, "bottom_random")
            );
            assert!(g.observe(p.other()).library_selection.is_none());
            assert!(g.observe(p.other()).actions.is_empty());
            let id = view.candidates[0].card;
            let before = g.object_ref(id);
            let hand = g.hand_count(p);
            replay(&mut g, Action::SelectLibrary { card: id });
            assert_eq!(g.object_view(id).zone, Zone::Library);
            replay(&mut g, Action::FinishLibrarySelection);
            settle(&mut g);
            assert_eq!(g.hand_count(p), hand + 1);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
            assert_eq!(g.object_ref(id).generation, before.generation + 1);
            assert!(g.state.known_hand.contains(&g.object_ref(id)));
            assert!(
                g.observe(p.other())
                    .objects
                    .iter()
                    .any(|v| v.reference == g.object_ref(id))
            );
            if i == 2 {
                assert_eq!(g.object_view(source).zone, Zone::Graveyard);
            } else {
                let expected = [
                    (3, 2),
                    (2, 1),
                    (0, 0),
                    (2, 3),
                    (3, 2),
                    (1, 4),
                    (2, 1),
                    (2, 3),
                    (2, 1),
                    (4, 4),
                    (2, 1),
                    (1, 2),
                ][i];
                let v = g.object_view(source);
                assert_eq!((v.power, v.toughness), expected);
                assert_eq!(
                    v.keywords,
                    if i == 1 {
                        Keywords::HASTE
                    } else if i == 9 {
                        Keywords::FIRST_STRIKE
                    } else {
                        Keywords::NONE
                    }
                );
            }
        }
    }
}
#[test]
fn each_union_and_exclusion_selects_the_exact_looked_candidates() {
    let cases: [(&str, [&str; 5], [bool; 5]); 12] = [
        (
            NAMES[0],
            [
                "Prophetic Prism",
                "Living Phone",
                "Forest",
                "Opt",
                "Bear Cub",
            ],
            [true, true, false, false, false],
        ),
        (
            NAMES[1],
            [
                "Prophetic Prism",
                "Living Phone",
                "Forest",
                "Opt",
                "Bear Cub",
            ],
            [true, true, false, false, false],
        ),
        (
            NAMES[2],
            [
                "Chitinous Graspling",
                "Swiftfoot Boots",
                "Forest",
                "Opt",
                "Bear Cub",
            ],
            [true, false, true, false, false],
        ),
        (
            NAMES[3],
            ["Chitinous Graspling", "Swamp", "Mountain", "Forest", "Opt"],
            [true, true, true, false, false],
        ),
        (
            NAMES[4],
            ["Chitinous Graspling", "Swamp", "Forest", "Island", "Opt"],
            [true, true, true, false, false],
        ),
        (
            NAMES[5],
            ["Chitinous Graspling", "Island", "Mountain", "Swamp", "Opt"],
            [true, true, true, false, false],
        ),
        (
            NAMES[6],
            ["Chitinous Graspling", "Forest", "Plains", "Swamp", "Opt"],
            [true, true, true, false, false],
        ),
        (
            NAMES[7],
            ["Chitinous Graspling", "Plains", "Island", "Mountain", "Opt"],
            [true, true, true, false, false],
        ),
        (
            NAMES[8],
            [
                "Chitinous Graspling",
                "Plains",
                "Hallowed Fountain",
                "Forest",
                "Bear Cub",
            ],
            [true, true, true, false, false],
        ),
        (
            NAMES[9],
            [
                "Chitinous Graspling",
                "Swiftfoot Boots",
                "Accumulate Wisdom",
                "Bear Cub",
                "Forest",
            ],
            [true, true, true, false, false],
        ),
        (
            NAMES[10],
            [
                "Prophetic Prism",
                "Chitinous Graspling",
                "Bear Cub",
                "Island",
                "Opt",
            ],
            [true, true, false, false, false],
        ),
        (
            NAMES[11],
            [
                "Prophetic Prism",
                "Opt",
                "Living Phone",
                "Forest",
                "Bear Cub",
            ],
            [true, true, false, false, false],
        ),
    ];
    for (name, names, yes) in cases {
        let mut g = setup(Player::P0);
        let ids = top(&mut g, Player::P0, &names);
        window(&mut g, Player::P0, name);
        let n = if name == NAMES[0] || name == NAMES[8] {
            5
        } else {
            4
        };
        assert_eq!(
            candidates(&g),
            ids[..n]
                .iter()
                .zip(yes)
                .filter_map(|(&id, ok)| ok.then_some(id))
                .collect::<Vec<_>>(),
            "{name}"
        );
        for (&id, ok) in ids[..n].iter().zip(yes) {
            if !ok {
                reject(&mut g, Action::SelectLibrary { card: id });
            }
        }
        settle(&mut g);
    }
}
#[test]
fn standalone_subtypes_and_boolean_predicates_keep_changeling_and_noncreature_types_separate() {
    let filters: [(F, &[usize]); 6] = [
        (F::Subtype("Ally"), &[0]),
        (F::Subtype("Plains"), &[1]),
        (F::Subtype("Equipment"), &[2]),
        (F::Subtype("Lesson"), &[3]),
        (
            F::AnyOf(&[F::Subtype("Equipment"), F::Subtype("Lesson")]),
            &[2, 3],
        ),
        (F::ExcludingKinds(3), &[2, 3]),
    ];
    for (filter, expected) in filters {
        let mut g = setup(Player::P0);
        let ids = top(
            &mut g,
            Player::P0,
            &[
                "Chitinous Graspling",
                "Plains",
                "Swiftfoot Boots",
                "Accumulate Wisdom",
            ],
        );
        window(&mut g, Player::P0, NAMES[2]);
        let item = g.state.resolving.as_ref().unwrap().item.clone();
        assert!(g.start_library_selection(
            &item,
            LibrarySelectionSpec {
                look: 4,
                count: LibrarySelectionCount::Fixed(1),
                filter,
                optional: true,
                reveal: true,
                remainder: LibraryRemainder::BottomRandom
            }
        ));
        assert_eq!(
            candidates(&g),
            expected.iter().map(|&i| ids[i]).collect::<Vec<_>>()
        );
        settle(&mut g);
    }
    for (filter, expected) in [
        (F::AllOf(&[F::Kinds(16), F::Subtype("Pirate")]), vec![0]),
        (
            F::AnyOf(&[F::Kinds(16), F::Subtype("Pirate")]),
            vec![0, 1, 2],
        ),
    ] {
        let mut g = setup(Player::P0);
        let ids = top(
            &mut g,
            Player::P0,
            &[
                "Three Tree Mascot",
                "Prophetic Prism",
                "Chitinous Graspling",
                "Plains",
            ],
        );
        window(&mut g, Player::P0, NAMES[2]);
        let item = g.state.resolving.as_ref().unwrap().item.clone();
        g.start_library_selection(
            &item,
            LibrarySelectionSpec {
                look: 4,
                count: LibrarySelectionCount::Fixed(1),
                filter,
                optional: true,
                reveal: true,
                remainder: LibraryRemainder::BottomRandom,
            },
        );
        assert_eq!(
            candidates(&g),
            expected.iter().map(|&i| ids[i]).collect::<Vec<_>>()
        );
        settle(&mut g);
    }
}
#[test]
fn all_eclipsed_hybrid_routes_preserve_printed_values_and_intrinsic_colors() {
    for (i, options, colors, mv) in [
        (3, 4, 12, 3),
        (4, 4, 20, 3),
        (5, 3, 10, 3),
        (6, 3, 17, 2),
        (7, 4, 3, 3),
    ] {
        let card = Card::from_name(NAMES[i]).unwrap();
        assert_eq!(card.cost_options().count(), options);
        for choice in 0..options {
            let mut g = setup(Player::P0);
            top(
                &mut g,
                Player::P0,
                &["Chitinous Graspling", "Forest", "Plains", "Opt"],
            );
            let id = named(&mut g, Player::P0, NAMES[i], Zone::Hand);
            replay(&mut g, Action::AnnounceCast { card: id });
            replay(
                &mut g,
                Action::ChooseSpellCost {
                    cost: choice as u16,
                },
            );
            let cost = card.cost_options().nth(choice).unwrap();
            let request = g.observe(Player::P0).payment_request.unwrap();
            assert_eq!(
                (request.required, request.generic),
                (cost.colored, cost.generic)
            );
            let mana = g.state.players[0].mana;
            while g.state.announcing.is_some() {
                choose(&mut g);
            }
            let spent: Vec<_> = mana
                .iter()
                .zip(g.state.players[0].mana)
                .map(|(&before, after)| before - after)
                .collect();
            assert_eq!(
                spent.iter().sum::<u32>(),
                cost.colored.iter().sum::<u32>() + cost.generic
            );
            assert!(
                spent
                    .iter()
                    .zip(cost.colored)
                    .all(|(&paid, required)| paid >= required)
            );
            assert_eq!(g.state.stack[0].source_card.fixed_mana_value(), mv);
            assert_eq!(card.colors(), colors);
            seek(&mut g);
            settle(&mut g);
            assert_eq!(g.hand_count(Player::P0), 1);
            assert_eq!(g.object_view(id).zone, Zone::Battlefield);
        }
    }
}
#[test]
fn trainer_paid_parent_and_copy_each_get_their_full_independent_entry_selection() {
    for p in [Player::P0, Player::P1] {
        for paid in [false, true] {
            let mut g = setup(p);
            top(
                &mut g,
                p,
                &[
                    "Prophetic Prism",
                    "Opt",
                    "Accumulate Wisdom",
                    "Swiftfoot Boots",
                    "Stock Up",
                    "Commune with Evil",
                ],
            );
            let source = named(&mut g, p, NAMES[11], Zone::Hand);
            announce(&mut g, source, paid, 0);
            let mut counts = [0; 2];
            for _ in 0..200 {
                if idle(&g) {
                    break;
                }
                if g.observe(g.actor().unwrap()).library_selection.is_some() {
                    let item = g.state.resolving.as_ref().unwrap().item.clone();
                    counts[usize::from(item.source_card.is_token())] += 1;
                    let id = candidates(&g)[0];
                    replay(&mut g, Action::SelectLibrary { card: id });
                    replay(&mut g, Action::FinishLibrarySelection);
                } else {
                    choose(&mut g);
                }
            }
            assert!(idle(&g));
            assert_eq!(counts, [1, usize::from(paid)]);
            assert_eq!(g.hand_count(p), 1 + usize::from(paid));
            let copies: Vec<_> = g
                .observe(p)
                .objects
                .into_iter()
                .filter(|o| {
                    o.zone == Zone::Battlefield && o.token.is_some() && o.card.name() == NAMES[11]
                })
                .collect();
            assert_eq!(copies.len(), usize::from(paid));
            if paid {
                let v = &copies[0];
                assert_eq!((v.power, v.toughness), (1, 1));
                assert_eq!(v.card.info().mana_cost, "{1}{U}");
                assert_eq!(v.card.fixed_mana_value(), 2);
                assert_eq!(
                    v.card.triggers(),
                    g.state.objects[source as usize].card.triggers()
                );
                assert_eq!(v.card.colors(), 2);
            }
            assert_eq!(g.object_view(source).zone, Zone::Battlefield);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        }
    }
}
#[test]
fn attack_selection_survives_source_departure_and_reveals_only_the_chosen_incarnation() {
    for name in [NAMES[0], NAMES[9]] {
        let mut g = setup(Player::P0);
        let ids = top(
            &mut g,
            Player::P0,
            &["Swiftfoot Boots", "Opt", "Forest", "Bear Cub", "Plains"],
        );
        let source = window(&mut g, Player::P0, name);
        let old = g.object_ref(ids[0]);
        g.move_object(source, Zone::Exile);
        replay(&mut g, Action::SelectLibrary { card: ids[0] });
        assert!(g.state.known_hand.is_empty());
        assert!(g.observe(Player::P1).objects.iter().all(|o| o.id != ids[0]));
        replay(&mut g, Action::FinishLibrarySelection);
        settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Exile);
        assert_eq!(g.object_ref(ids[0]).generation, old.generation + 1);
        assert!(
            g.observe(Player::P1)
                .objects
                .iter()
                .any(|o| o.reference == g.object_ref(ids[0]))
        );
    }
}
#[test]
fn no_eligible_random_windows_still_reveal_the_private_look_and_allow_declining() {
    for name in [NAMES[0], NAMES[11]] {
        let mut g = setup(Player::P0);
        let ids = top(
            &mut g,
            Player::P0,
            &["Bear Cub", "Forest", "Plains", "Bear Cub", "Mountain"],
        );
        window(&mut g, Player::P0, name);
        assert!(candidates(&g).is_empty());
        let f = g.observe(Player::P0).library_selection.unwrap();
        assert_eq!((f.minimum, f.maximum), (0, 0));
        assert!(g.observe(Player::P1).library_selection.is_none());
        reject(&mut g, Action::SelectLibrary { card: ids[0] });
        let rng = g.state.rng;
        replay(&mut g, Action::FinishLibrarySelection);
        settle(&mut g);
        assert_ne!(g.state.rng, rng);
        assert_eq!(g.hand_count(Player::P0), 0);
        assert!(
            ids.iter()
                .all(|&id| g.object_view(id).zone == Zone::Library)
        );
    }
}
#[test]
fn trainer_suppression_blocks_both_entries_without_refunding_offspring_payment() {
    for orb in ["Torpor Orb", "Doorkeeper Thrull"] {
        let mut g = setup(Player::P0);
        named(&mut g, Player::P1, orb, Zone::Battlefield);
        let id = named(&mut g, Player::P0, NAMES[11], Zone::Hand);
        let mana = g.state.players[0].mana.iter().sum::<u32>();
        announce(&mut g, id, true, 0);
        settle(&mut g);
        assert_eq!(g.state.players[0].mana.iter().sum::<u32>(), mana - 6);
        assert_eq!(g.hand_count(Player::P0), 0);
        assert!(
            !g.state
                .objects
                .iter()
                .any(|o| o.card.is_token() && o.card.name() == NAMES[11])
        );
        assert_eq!(g.object_view(id).zone, Zone::Battlefield);
    }
}
#[test]
fn filtered_reveal_failure_restores_window_rng_knowledge_and_exact_references() {
    let mut g = setup(Player::P0);
    let ids = top(
        &mut g,
        Player::P0,
        &["Prophetic Prism", "Opt", "Bear Cub", "Forest"],
    );
    g.state.objects[ids[0] as usize].generation = u64::MAX;
    window(&mut g, Player::P0, NAMES[11]);
    replay(&mut g, Action::SelectLibrary { card: ids[0] });
    let saved = g.snapshot();
    let revision = g.revision();
    assert!(matches!(
        g.apply(Player::P0, Action::FinishLibrarySelection),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), saved);
    assert_eq!(g.revision(), revision);
    assert_eq!(g.object_view(ids[0]).zone, Zone::Library);
    assert!(g.state.known_hand.is_empty());
}

#[test]
fn noncreature_changeling_metadata_matches_subtype_unions_but_not_frontier_creature_conjunction() {
    // This supported kindred instant exposes a grouping error that a
    // creature-only changeling could not catch.
    let crib = Card::from_name("Crib Swap").unwrap();
    assert!(crib.is_supported());
    assert!(crib.keywords().contains(Keywords::CHANGELING));
    assert!(!crib.is_creature());
    assert!(crib.has_subtype("Mount"));
    for name in [NAMES[2], NAMES[3], NAMES[8], NAMES[9], NAMES[10], NAMES[11]] {
        let mut g = setup(Player::P0);
        let ordinary = top(
            &mut g,
            Player::P0,
            &["Plains", "Swiftfoot Boots", "Accumulate Wisdom", "Bear Cub"],
        );
        let id = spawn(&mut g, Player::P0, crib, Zone::Library);
        window(&mut g, Player::P0, name);
        assert_eq!(candidates(&g).contains(&id), name != NAMES[8], "{name}");
        assert!(!crib.has_subtype("Plains"));
        assert!(!crib.has_subtype("Equipment"));
        assert!(!crib.has_subtype("Lesson"));
        assert!(
            ordinary
                .iter()
                .all(|&n| g.object_view(n).zone == Zone::Library)
        );
        // This scenario tests filtering rather than the spell effect; decline.
        replay(&mut g, Action::FinishLibrarySelection);
        settle(&mut g);
    }
}
#[test]
fn paid_trainer_short_and_empty_libraries_still_create_exactly_one_nonrecursive_copy() {
    for n in 0..=2 {
        let mut g = setup(Player::P0);
        for id in g.state.players[0]
            .library
            .clone()
            .iter()
            .copied()
            .collect::<Vec<_>>()
        {
            g.move_object(id, Zone::Exile);
        }
        for _ in 0..n {
            named(&mut g, Player::P0, "Opt", Zone::Library);
        }
        let source = named(&mut g, Player::P0, NAMES[11], Zone::Hand);
        announce(&mut g, source, true, 0);
        settle(&mut g);
        assert_eq!(g.hand_count(Player::P0), n);
        assert_eq!(g.state.players[0].library.len(), 0);
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Battlefield
                    && o.card.is_token()
                    && o.card.name() == NAMES[11])
                .count(),
            1
        );
        assert_eq!(g.state.players[0].cards_drawn_this_turn, 0);
        assert_eq!(g.outcome(), None);
    }
}
#[test]
fn every_new_filtered_selection_can_decline_even_when_an_eligible_card_is_available() {
    for name in NAMES {
        let mut g = setup(Player::P0);
        let ids = top(
            &mut g,
            Player::P0,
            &[
                "Prophetic Prism",
                "Chitinous Graspling",
                "Plains",
                "Forest",
                "Opt",
            ],
        );
        window(&mut g, Player::P0, name);
        assert!(!candidates(&g).is_empty());
        let hand = g.hand_count(Player::P0);
        replay(&mut g, Action::FinishLibrarySelection);
        settle(&mut g);
        assert_eq!(g.hand_count(Player::P0), hand);
        assert!(
            ids.iter()
                .all(|&id| g.object_view(id).zone == Zone::Library)
        );
        assert!(g.state.known_hand.is_empty());
    }
}
