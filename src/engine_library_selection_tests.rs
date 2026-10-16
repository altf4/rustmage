use super::tests::{act, scenario, spawn};
use super::*;
const NAMES: [&str; 15] = [
    "Accumulate Wisdom",
    "Beastrider Vanguard",
    "Boughside Wanderers",
    "Commune with Beavers",
    "Commune with Evil",
    "Commune with Nature",
    "Confounding Riddle",
    "Flow State",
    "Follow the Lumarets",
    "Living Phone",
    "Morbius the Living Vampire",
    "Rakshasa's Bargain",
    "Resentful Revelation",
    "Squad Rallier",
    "Stock Up",
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
        s.mana = [30; 6];
    }
    g
}
fn replay(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let r = g.revision();
    let p = g.actor().unwrap();
    let mut f = g.clone();
    act(&mut f, a.clone());
    act(g, a.clone());
    let expected = g.snapshot();
    assert_eq!(f.snapshot(), expected);
    g.restore(&saved).unwrap();
    assert_eq!(g.apply_index(p, r, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), expected);
}
fn reject(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let r = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), saved);
    assert_eq!(g.revision(), r);
}
fn choose(g: &mut Game) {
    let actor = g.actor().unwrap();
    if let Some(view) = g.observe(actor).library_selection {
        assert!(g.observe(actor.other()).library_selection.is_none());
        assert!(
            view.remaining
                .iter()
                .chain(&view.selected)
                .chain(&view.bottom)
                .all(|v| v.zone == Zone::Library)
        );
    }
    let actions = g.legal_actions();
    let a = actions
        .iter()
        .find(|a| matches!(a, Action::SelectLibrary { .. }))
        .or_else(|| {
            actions.iter().find(|a| {
                matches!(
                    a,
                    Action::FinishLibrarySelection
                        | Action::OrderLibraryBottom { .. }
                        | Action::ChooseSpellCost { .. }
                        | Action::ChooseMode { .. }
                        | Action::ChooseTarget { .. }
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::PutTrigger { .. }
                        | Action::ArrangeLibrary { .. }
                        | Action::Pass
                        | Action::DeclineResolutionMana
                )
            })
        })
        .expect("fixture continuation")
        .clone();
    replay(g, a);
}
fn settle(g: &mut Game) {
    g.prepare_triggers();
    for _ in 0..150 {
        if g.outcome().is_some()
            || g.state.stack.is_empty()
                && g.state.announcing.is_none()
                && g.state.pending == Pending::Priority
        {
            return;
        }
        choose(g);
    }
    panic!("unsettled");
}
fn cast(g: &mut Game, id: CardId) {
    replay(g, Action::AnnounceCast { card: id });
    while g.state.announcing.is_some() {
        choose(g);
    }
    settle(g);
}
fn window(g: &mut Game, id: CardId) {
    replay(g, Action::AnnounceCast { card: id });
    for _ in 0..90 {
        if g.observe(g.actor().unwrap()).library_selection.is_some() {
            return;
        }
        choose(g);
    }
    panic!("no window");
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
fn finish(g: &mut Game) {
    replay(g, Action::FinishLibrarySelection);
    while g.observe(g.actor().unwrap()).library_selection.is_some() {
        choose(g);
    }
    settle(g);
}
#[test]
fn all_fifteen_complete_bodies_cast_for_both_seats_and_execute_their_selection() {
    for p in [Player::P0, Player::P1] {
        for n in NAMES {
            let mut g = setup(p);
            top(
                &mut g,
                p,
                &["Bear Cub", "Opt", "Forest", "Shivan Dragon", "Bear Cub"],
            );
            let source = named(&mut g, p, n, Zone::Hand);
            let before = g.state.players[p.index()].cards_drawn_this_turn;
            cast(&mut g, source);
            if [
                "Beastrider Vanguard",
                "Squad Rallier",
                "Morbius the Living Vampire",
            ]
            .contains(&n)
            {
                if n == "Morbius the Living Vampire" {
                    g.move_object(source, Zone::Graveyard);
                }
                replay(
                    &mut g,
                    Action::AnnounceActivate {
                        card: source,
                        ability: 0,
                        counter_payment: Counters::default(),
                    },
                );
                while g.state.announcing.is_some() {
                    choose(&mut g);
                }
                settle(&mut g);
            }
            if n == "Living Phone" {
                g.move_object(source, Zone::Graveyard);
                settle(&mut g);
            }
            assert_eq!(
                g.hand_count(p),
                if matches!(n, "Stock Up" | "Rakshasa's Bargain") {
                    2
                } else {
                    1
                },
                "{n}"
            );
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                before,
                "{n}"
            );
            if n == "Commune with Evil" {
                assert_eq!(g.state.players[p.index()].life, 23);
            }
            if n == "Morbius the Living Vampire" {
                assert_eq!(g.object_view(source).zone, Zone::Exile);
            }
            if n == "Resentful Revelation" {
                assert_eq!(g.object_view(source).zone, Zone::Graveyard);
            }
        }
    }
}
#[test]
fn mandatory_counts_window_membership_order_and_private_transactions_are_enforced() {
    let mut g = setup(Player::P0);
    let ids = top(
        &mut g,
        Player::P0,
        &["Bear Cub", "Opt", "Plains", "Forest", "Shivan Dragon"],
    );
    let outside = named(&mut g, Player::P0, "Forest", Zone::Hand);
    let spell = named(&mut g, Player::P0, "Stock Up", Zone::Hand);
    window(&mut g, spell);
    let original = g.state.players[0].library.clone();
    let hand = g.hand_count(Player::P0);
    let saved = g.snapshot();
    assert!(g.observe(Player::P1).library_selection.is_none());
    reject(&mut g, Action::FinishLibrarySelection);
    reject(&mut g, Action::SelectLibrary { card: outside });
    reject(&mut g, Action::OrderLibraryBottom { card: ids[2] });
    assert!(
        g.apply(Player::P1, Action::SelectLibrary { card: ids[0] })
            .is_err()
    );
    assert_eq!(g.snapshot(), saved);
    replay(&mut g, Action::SelectLibrary { card: ids[0] });
    reject(&mut g, Action::SelectLibrary { card: ids[0] });
    reject(&mut g, Action::FinishLibrarySelection);
    replay(&mut g, Action::SelectLibrary { card: ids[1] });
    reject(&mut g, Action::SelectLibrary { card: ids[2] });
    replay(&mut g, Action::FinishLibrarySelection);
    assert!(g.observe(Player::P0).library_selection.unwrap().ordering);
    assert_eq!(g.hand_count(Player::P0), hand);
    assert_eq!(g.state.players[0].library, original);
    reject(&mut g, Action::FinishLibrarySelection);
    replay(&mut g, Action::OrderLibraryBottom { card: ids[3] });
    replay(&mut g, Action::OrderLibraryBottom { card: ids[4] });
    assert_eq!(g.state.players[0].library, original);
    replay(&mut g, Action::OrderLibraryBottom { card: ids[2] });
    assert_eq!(&g.state.players[0].library[..3], &[ids[2], ids[4], ids[3]]);
    assert_eq!(g.hand_count(Player::P0), hand + 2);
    assert!(
        !g.observe(Player::P1)
            .objects
            .iter()
            .any(|v| ids[..2].contains(&v.id))
    );
    settle(&mut g);
}
#[test]
fn optional_filtered_selection_may_decline_and_reveals_only_selected_new_hand_incarnations() {
    for take in [false, true] {
        let mut g = setup(Player::P0);
        let ids = top(
            &mut g,
            Player::P0,
            &["Bear Cub", "Opt", "Forest", "Shivan Dragon", "Plains"],
        );
        let spell = named(&mut g, Player::P0, "Commune with Nature", Zone::Hand);
        window(&mut g, spell);
        let view = g.observe(Player::P0).library_selection.unwrap();
        assert_eq!((view.minimum, view.maximum), (0, 1));
        assert_eq!(view.candidates.len(), 2);
        reject(&mut g, Action::SelectLibrary { card: ids[1] });
        if take {
            replay(&mut g, Action::SelectLibrary { card: ids[0] });
        }
        finish(&mut g);
        assert_eq!(g.hand_count(Player::P0), usize::from(take));
        let public = g.observe(Player::P1);
        assert_eq!(
            public
                .objects
                .iter()
                .filter(|v| v.zone == Zone::Hand && v.owner == Player::P0)
                .count(),
            usize::from(take)
        );
        if take {
            assert!(g.state.known_hand.contains(&g.object_ref(ids[0])));
            assert_eq!(g.object_ref(ids[0]).generation, 1);
            g.move_object(ids[0], Zone::Graveyard);
            assert!(!g.state.known_hand.iter().any(|r| r.card == ids[0]));
        }
    }
}
#[test]
fn empty_and_short_libraries_put_as_many_as_possible_without_drawing_or_losing() {
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
        let ids: Vec<_> = (0..n)
            .map(|_| named(&mut g, Player::P0, "Forest", Zone::Library))
            .collect();
        let spell = named(&mut g, Player::P0, "Stock Up", Zone::Hand);
        cast(&mut g, spell);
        assert_eq!(g.hand_count(Player::P0), n);
        assert!(ids.iter().all(|&id| g.object_view(id).zone == Zone::Hand));
        assert_eq!(g.state.players[0].cards_drawn_this_turn, 0);
        assert_eq!(g.outcome(), None);
    }
}
#[test]
fn conditions_use_own_graveyard_and_freeze_at_resolution() {
    for mode in 0..3 {
        for enabled in [false, true] {
            let mut g = setup(Player::P0);
            let name = ["Accumulate Wisdom", "Flow State", "Follow the Lumarets"][mode];
            top(&mut g, Player::P0, &["Bear Cub", "Forest", "Plains", "Opt"]);
            if enabled {
                match mode {
                    0 => {
                        for _ in 0..3 {
                            named(&mut g, Player::P0, "Accumulate Wisdom", Zone::Graveyard);
                        }
                    }
                    1 => {
                        named(&mut g, Player::P0, "Opt", Zone::Graveyard);
                        named(&mut g, Player::P0, "Stock Up", Zone::Graveyard);
                    }
                    _ => g.state.players[0].life_gained_this_turn = 1,
                }
            } else {
                for _ in 0..3 {
                    named(&mut g, Player::P1, "Accumulate Wisdom", Zone::Graveyard);
                }
                named(&mut g, Player::P1, "Opt", Zone::Graveyard);
                named(&mut g, Player::P1, "Stock Up", Zone::Graveyard);
                g.state.players[1].life_gained_this_turn = 1;
            }
            let source = named(&mut g, Player::P0, name, Zone::Hand);
            replay(&mut g, Action::AnnounceCast { card: source });
            while g.state.announcing.is_some() {
                choose(&mut g);
            }
            replay(&mut g, Action::Pass);
            replay(&mut g, Action::Pass);
            if mode == 0 && enabled {
                assert_eq!(g.hand_count(Player::P0), 3);
                assert!(g.state.resolving.is_none());
            } else {
                let view = g.observe(Player::P0).library_selection.unwrap();
                let expected = if enabled { 2 } else { 1 };
                assert_eq!(view.maximum, expected);
                if mode == 1 {
                    named(&mut g, Player::P0, "Opt", Zone::Graveyard);
                    named(&mut g, Player::P0, "Stock Up", Zone::Graveyard);
                }
                g.state.players[0].life_gained_this_turn = 100;
                assert_eq!(
                    g.observe(Player::P0).library_selection.unwrap().maximum,
                    expected
                );
                settle(&mut g);
                assert_eq!(g.hand_count(Player::P0), expected);
            }
        }
    }
}
#[test]
fn power_filter_uses_characteristic_defining_library_power_not_printed_star_or_battlefield_bonuses()
{
    for creatures in [0, 3] {
        let mut g = setup(Player::P0);
        for _ in 0..creatures {
            named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
        }
        named(&mut g, Player::P0, "Warleader's Call", Zone::Battlefield);
        let star = top(
            &mut g,
            Player::P0,
            &["Crusader of Odric", "Bear Cub", "Opt", "Forest"],
        )[0];
        let source = named(&mut g, Player::P0, "Squad Rallier", Zone::Battlefield);
        replay(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        while g.state.announcing.is_some() {
            choose(&mut g);
        }
        replay(&mut g, Action::Pass);
        replay(&mut g, Action::Pass);
        let view = g.observe(Player::P0).library_selection.unwrap();
        assert_eq!(
            view.candidates.iter().any(|r| r.card == star),
            creatures == 0
        );
        assert!(
            view.candidates
                .iter()
                .any(|r| g.state.objects[r.card as usize].card.name() == "Bear Cub")
        );
        settle(&mut g);
    }
}
#[test]
fn random_remainders_are_branch_local_and_advance_owned_rng_only_at_commit() {
    let mut g = setup(Player::P0);
    top(&mut g, Player::P0, &["Bear Cub", "Plains", "Opt", "Forest"]);
    let source = named(&mut g, Player::P0, "Follow the Lumarets", Zone::Hand);
    window(&mut g, source);
    let oldrng = g.state.rng;
    let selected = g.observe(Player::P0).library_selection.unwrap().candidates[0].card;
    replay(&mut g, Action::SelectLibrary { card: selected });
    assert_eq!(g.state.rng, oldrng);
    let saved = g.snapshot();
    let oldlib = g.state.players[0].library.clone();
    let mut f = g.clone();
    replay(&mut g, Action::FinishLibrarySelection);
    replay(&mut f, Action::FinishLibrarySelection);
    assert_eq!(g.snapshot(), f.snapshot());
    assert_ne!(g.state.rng, oldrng);
    assert_ne!(g.state.players[0].library, oldlib);
    g.restore(&saved).unwrap();
    assert_eq!(g.state.rng, oldrng);
    assert_eq!(g.state.players[0].library, oldlib);
}
#[test]
fn final_commit_generation_overflow_restores_frame_library_hand_rng_and_revision() {
    let mut g = setup(Player::P0);
    let ids = top(&mut g, Player::P0, &["Bear Cub", "Plains", "Forest", "Opt"]);
    g.state.objects[ids[0] as usize].generation = u64::MAX;
    let source = named(&mut g, Player::P0, "Follow the Lumarets", Zone::Hand);
    window(&mut g, source);
    replay(&mut g, Action::SelectLibrary { card: ids[0] });
    let saved = g.snapshot();
    let rev = g.revision();
    assert!(matches!(
        g.apply(Player::P0, Action::FinishLibrarySelection),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), saved);
    assert_eq!(g.revision(), rev);
}
#[test]
fn source_departure_and_death_trigger_do_not_cancel_a_private_selection() {
    let mut g = setup(Player::P0);
    top(
        &mut g,
        Player::P0,
        &["Bear Cub", "Opt", "Forest", "Plains", "Shivan Dragon"],
    );
    let source = named(&mut g, Player::P0, "Living Phone", Zone::Battlefield);
    g.move_object(source, Zone::Graveyard);
    g.prepare_triggers();
    replay(&mut g, Action::Pass);
    replay(&mut g, Action::Pass);
    assert!(g.observe(Player::P0).library_selection.is_some());
    g.move_object(source, Zone::Exile);
    settle(&mut g);
    assert_eq!(g.hand_count(Player::P0), 1);
    assert_eq!(g.object_view(source).zone, Zone::Exile);
}
#[test]
fn boughside_keeps_entry_selection_and_later_landfall_boost() {
    let mut g = setup(Player::P0);
    let source = named(&mut g, Player::P0, "Boughside Wanderers", Zone::Hand);
    cast(&mut g, source);
    assert_eq!(g.hand_count(Player::P0), 1);
    let land = g
        .state
        .objects
        .iter()
        .enumerate()
        .find(|(_, o)| o.owner == Player::P0 && o.zone == Zone::Hand && o.card.is_land())
        .unwrap()
        .0 as CardId;
    replay(&mut g, Action::PlayLand { card: land });
    settle(&mut g);
    assert_eq!(
        (g.object_view(source).power, g.object_view(source).toughness),
        (6, 6)
    );
    g.finish_cleanup();
    assert_eq!(
        (g.object_view(source).power, g.object_view(source).toughness),
        (4, 4)
    );
}
#[test]
fn resentful_revelation_works_normally_and_as_flashback_with_exact_exile() {
    let mut g = setup(Player::P0);
    let source = named(&mut g, Player::P0, "Resentful Revelation", Zone::Hand);
    cast(&mut g, source);
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    assert_eq!(g.hand_count(Player::P0), 1);
    replay(&mut g, Action::AnnounceFlashback { card: source });
    while g.state.announcing.is_some() {
        choose(&mut g);
    }
    settle(&mut g);
    assert_eq!(g.object_view(source).zone, Zone::Exile);
    assert_eq!(g.hand_count(Player::P0), 2);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 0);
}
#[test]
fn confounding_riddle_retains_counter_unless_four_mana_mode() {
    for pay in [false, true] {
        let mut g = setup(Player::P0);
        let bear = named(&mut g, Player::P0, "Bear Cub", Zone::Hand);
        replay(&mut g, Action::AnnounceCast { card: bear });
        while g.state.announcing.is_some() {
            choose(&mut g);
        }
        replay(&mut g, Action::Pass);
        let riddle = named(&mut g, Player::P1, "Confounding Riddle", Zone::Hand);
        replay(&mut g, Action::AnnounceCast { card: riddle });
        replay(&mut g, Action::ChooseMode { mode: 1 });
        while g.state.announcing.is_some() {
            choose(&mut g);
        }
        replay(&mut g, Action::Pass);
        replay(&mut g, Action::Pass);
        assert!(g.observe(Player::P0).resolution_payment.is_some());
        let action = if pay {
            g.legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayResolutionMana { .. }))
                .unwrap()
        } else {
            Action::DeclineResolutionMana
        };
        replay(&mut g, action);
        settle(&mut g);
        assert_eq!(
            g.object_view(bear).zone,
            if pay {
                Zone::Battlefield
            } else {
                Zone::Graveyard
            }
        );
        assert!(g.observe(Player::P0).library_selection.is_none());
    }
}

#[test]
fn rakshasas_bargain_accepts_all_eight_two_brid_payments_with_printed_mana_value_six() {
    let card = Card::from_name("Rakshasa's Bargain").unwrap();
    assert_eq!(card.cost_options().count(), 8);
    for cost in 0..8 {
        let mut g = setup(Player::P0);
        let ids = top(&mut g, Player::P0, &["Bear Cub", "Opt", "Forest", "Plains"]);
        let source = named(&mut g, Player::P0, "Rakshasa's Bargain", Zone::Hand);
        replay(&mut g, Action::AnnounceCast { card: source });
        replay(&mut g, Action::ChooseSpellCost { cost });
        assert_eq!(
            g.state
                .announcing
                .as_ref()
                .unwrap()
                .item
                .source_card
                .fixed_mana_value(),
            6
        );
        while g.state.announcing.is_some() {
            choose(&mut g);
        }
        settle(&mut g);
        assert_eq!(g.hand_count(Player::P0), 2);
        assert_eq!(
            ids.iter()
                .filter(|&&id| g.object_view(id).zone == Zone::Graveyard)
                .count(),
            2
        );
        assert_eq!(g.state.players[0].cards_drawn_this_turn, 0);
    }
}
#[test]
fn follow_may_select_zero_one_or_two_and_empty_filters_still_order_all_remainders() {
    for count in 0..=2 {
        let mut g = setup(Player::P0);
        g.state.players[0].life_gained_this_turn = 1;
        let ids = top(&mut g, Player::P0, &["Bear Cub", "Forest", "Plains", "Opt"]);
        let source = named(&mut g, Player::P0, "Follow the Lumarets", Zone::Hand);
        window(&mut g, source);
        reject(&mut g, Action::SelectLibrary { card: CardId::MAX });
        for &id in ids.iter().take(count) {
            replay(&mut g, Action::SelectLibrary { card: id });
        }
        finish(&mut g);
        assert_eq!(g.hand_count(Player::P0), count);
        assert_eq!(
            g.observe(Player::P1)
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Hand && o.owner == Player::P0)
                .count(),
            count
        );
    }
    let mut g = setup(Player::P0);
    let ids = top(&mut g, Player::P0, &["Opt", "Opt", "Opt", "Opt", "Opt"]);
    let source = named(&mut g, Player::P0, "Commune with Nature", Zone::Hand);
    window(&mut g, source);
    let view = g.observe(Player::P0).library_selection.unwrap();
    assert!(view.ordering && view.candidates.is_empty());
    assert_eq!((view.minimum, view.maximum), (0, 0));
    settle(&mut g);
    assert_eq!(g.hand_count(Player::P0), 0);
    assert!(
        ids.iter()
            .all(|&id| g.object_view(id).zone == Zone::Library)
    );
}
#[test]
fn graveyard_remainder_overflow_rolls_back_prior_hand_put_and_puts_do_not_emit_draws() {
    let mut g = setup(Player::P0);
    let watcher = named(&mut g, Player::P0, "Clinquant Skymage", Zone::Battlefield);
    let ids = top(&mut g, Player::P0, &["Bear Cub", "Opt", "Forest"]);
    g.state.objects[ids[1] as usize].generation = u64::MAX;
    let source = named(&mut g, Player::P0, "Commune with Evil", Zone::Hand);
    window(&mut g, source);
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
    // Remove the synthetic capacity boundary, update only its frozen exact ref,
    // and exercise the complete real effect after restoring the transaction.
    g.state.objects[ids[1] as usize].generation = 0;
    let reference = g.object_ref(ids[1]);
    let frame = g
        .state
        .resolving
        .as_mut()
        .unwrap()
        .selection
        .as_mut()
        .unwrap();
    for r in frame.remaining.iter_mut().chain(&mut frame.eligible) {
        if r.card == ids[1] {
            *r = reference;
        }
    }
    finish(&mut g);
    assert_eq!(g.object_view(ids[0]).zone, Zone::Hand);
    assert!(
        ids[1..]
            .iter()
            .all(|&id| g.object_view(id).zone == Zone::Graveyard)
    );
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 0);
    assert_eq!(g.object_view(watcher).counters.plus_one, 0);
    assert_eq!(g.state.players[0].life, 23);
}

#[test]
fn zero_eligible_random_window_still_exposes_private_look_before_forced_commit() {
    let mut g = setup(Player::P0);
    let ids = top(&mut g, Player::P0, &["Opt", "Opt", "Opt", "Opt"]);
    let source = named(&mut g, Player::P0, "Follow the Lumarets", Zone::Hand);
    let rng = g.state.rng;
    window(&mut g, source);
    let view = g.observe(Player::P0).library_selection.unwrap();
    assert_eq!(view.remaining.iter().map(|o| o.id).collect::<Vec<_>>(), ids);
    assert!(view.candidates.is_empty() && view.selected.is_empty() && !view.ordering);
    assert_eq!((view.minimum, view.maximum), (0, 0));
    assert!(g.observe(Player::P1).library_selection.is_none());
    assert_eq!(
        g.legal_actions(),
        vec![Action::Concede, Action::FinishLibrarySelection]
    );
    assert_eq!(g.state.rng, rng);
    assert!(
        ids.iter()
            .all(|&id| g.object_view(id).zone == Zone::Library)
    );
    replay(&mut g, Action::FinishLibrarySelection);
    settle(&mut g);
    assert_ne!(g.state.rng, rng);
    assert_eq!(g.hand_count(Player::P0), 0);
    assert!(
        ids.iter()
            .all(|&id| g.object_view(id).zone == Zone::Library)
    );
}

#[test]
fn spell_departure_overflow_restores_rng_after_random_remainder_was_shuffled() {
    let mut g = setup(Player::P0);
    let ids = top(&mut g, Player::P0, &["Bear Cub", "Plains", "Forest", "Opt"]);
    let source = named(&mut g, Player::P0, "Follow the Lumarets", Zone::Hand);
    window(&mut g, source);
    replay(&mut g, Action::SelectLibrary { card: ids[0] });
    // The selected hand put and remainder shuffle occur before the resolving
    // spell's final departure; fail that departure after RNG has been consumed.
    g.state.objects[source as usize].generation = u64::MAX;
    let reference = g.object_ref(source);
    g.state.resolving.as_mut().unwrap().item.object = reference;
    let saved = g.snapshot();
    let revision = g.revision();
    assert!(matches!(
        g.apply(Player::P0, Action::FinishLibrarySelection),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), saved);
    assert_eq!(g.revision(), revision);
    assert!(
        ids.iter()
            .all(|&id| g.object_view(id).zone == Zone::Library)
    );
    assert_eq!(g.object_view(source).zone, Zone::Stack);
}
