//! Bounded private-opcode regression fixtures; whole paid producers have their own module.
//! Every envelope is a genuinely paid supported Divination; the immutable program
//! is explicitly substituted only here; these fixtures are not whole-card proof.
use super::assurance::big_helpers::{cast, checked, named, settle, setup};
use super::*;
use crate::cards::{
    LibraryLook, LibraryPickDestination, LibraryRemainder, LibrarySelectionCount,
    LibrarySelectionEffect, LibrarySelectionFilter, LibrarySelectionSpec,
};
const ORIGIN: &str = "Origin of the Avengers";
const NICK: &str = "Nick Fury, Agent of S.H.I.E.L.D.";
fn origin_program() -> &'static [Instruction] {
    Card::from_name(ORIGIN).unwrap().triggers()[1].program
}
fn nick_program() -> &'static [Instruction] {
    Card::from_name(NICK).unwrap().activation(0).program
}
fn quiet(g: &Game) -> bool {
    g.state.resolving.is_none()
        && g.state.stack.is_empty()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
        && g.state.pending == Pending::Priority
}
fn views(g: &Game) -> [serde_json::Value; 2] {
    [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap())
}
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    let seen = views(g);
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
    assert_eq!(views(g), seen);
}
fn orphan(g: Game, action: Action) -> Game {
    let actor = g.actor().unwrap();
    let revision = g.revision();
    let before = g.snapshot();
    let seen = views(&g);
    let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
    let (mut raw, mut indexed) = (g.clone(), g.clone());
    drop(g);
    raw.apply(actor, action.clone()).unwrap();
    indexed.apply_index(actor, revision, index).unwrap();
    let after = raw.snapshot();
    assert_eq!(after, indexed.snapshot());
    assert_eq!(views(&raw), views(&indexed));
    raw.restore(&before).unwrap();
    let mut restored = seen;
    for view in &mut restored {
        view["revision"] = serde_json::json!(raw.revision());
    }
    assert_eq!(views(&raw), restored);
    assert_eq!(
        raw.apply_index(actor, revision, index),
        Err(Error::StaleRevision)
    );
    raw.apply(actor, action).unwrap();
    assert_eq!(raw.snapshot(), after);
    indexed
}
fn bounded(g: &mut Game, p: Player, program: &'static [Instruction]) {
    assert!(g.state.resolving.is_none() && g.state.stack.is_empty());
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    let source = named(g, p, "Divination", Zone::Hand);
    g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 2];
    cast(g, source, None, &[]);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    let item = g.state.stack.pop().unwrap();
    assert_eq!(item.card, source);
    g.state.resolving = Some(programs::Resolution {
        linked_copy: None,
        cascade: None,
        target_card_facts: None,
        immediate_blink_return: None,
        explore: None,
        evidence: None,
        connive: None,
        item,
        program,
        pc: 0,
        target_group: 0,
        moved_count: None,
        discard_after: None,
        discards: [0; 2],
        mana_payment: None,
        forage: None,
        selected_discards: [Vec::new(), Vec::new()],
        milled: Vec::new(),
        milled_selection: None,
        library: None,
        selection: None,
        target_library: None,
        search: None,
        optional_searches: None,
        choice: None,
    });
    g.run_program();
}
fn select(g: &mut Game, id: CardId) {
    checked(g, Action::SelectLibrary { card: id });
    checked(g, Action::FinishLibrarySelection);
}
fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    let c = Card::from_name(name).unwrap();
    let id = named(g, p, name, Zone::Hand);
    let (colored, generic) = c.cost();
    g.state.players[p.index()].mana = colored;
    g.state.players[p.index()].mana[5] += generic;
    cast(g, id, None, &[]);
    settle(g);
    id
}
#[test]
fn msh_entry_core_origin_real_program_filter_payment_and_decline_both_seats() {
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            let mut g = setup(p);
            let hero = named(&mut g, p, "Jennifer Walters", Zone::Hand);
            let large = named(&mut g, p, "Pet Avengers", Zone::Hand);
            let other = named(&mut g, p, "Hill Giant", Zone::Hand);
            bounded(&mut g, p, origin_program());
            let n = g.hand_count(p);
            let choice = g.observe(p).effect_choice.unwrap();
            assert_eq!(choice.candidates, vec![g.object_ref(hero)]);
            assert!(g.observe(p.other()).effect_choice.is_none());
            for id in [large, other] {
                let object = g.object_ref(id);
                reject(&mut g, Action::PutHandPermanent { object });
            }
            let action = if pay {
                Action::PutHandPermanent {
                    object: g.object_ref(hero),
                }
            } else {
                Action::DeclineHandEntry
            };
            g = orphan(g, action);
            settle(&mut g);
            assert_eq!(g.hand_count(p), if pay { n - 1 } else { n + 1 });
            assert_eq!(
                g.state.objects[hero as usize].zone,
                if pay { Zone::Battlefield } else { Zone::Hand }
            );
            if pay {
                assert_eq!(g.object_view(hero).card.name(), "Jennifer Walters");
            }
        }
    }
}
#[test]
fn msh_entry_core_origin_private_phase_does_not_disclose_hidden_eligibility() {
    for p in [Player::P0, Player::P1] {
        let mut a = setup(p);
        let mut b = setup(p);
        named(&mut a, p, "Jennifer Walters", Zone::Hand);
        named(&mut b, p, "Hill Giant", Zone::Hand);
        bounded(&mut a, p, origin_program());
        bounded(&mut b, p, origin_program());
        assert_eq!(
            serde_json::to_value(a.observe(p.other())).unwrap(),
            serde_json::to_value(b.observe(p.other())).unwrap()
        );
        assert_eq!(a.actor(), b.actor());
        assert_eq!(
            a.observe(p).effect_choice.unwrap().required,
            b.observe(p).effect_choice.unwrap().required
        );
        assert!(b.observe(p).effect_choice.unwrap().candidates.is_empty());
        checked(&mut b, Action::DeclineHandEntry);
        assert!(quiet(&b));
    }
}
const ANY_HAND: &[Instruction] = &[
    Instruction::PutHandFiltered {
        filter: crate::SearchFilter::Any,
        skip: 1,
    },
    Instruction::Skip(1),
    Instruction::Draw(PlayerSet::Controller, 1),
];
#[test]
fn msh_entry_core_optional_payment_cannot_choose_an_impossible_aura() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let aura = named(&mut g, p, "Witness Protection", Zone::Hand);
        bounded(&mut g, p, ANY_HAND);
        let n = g.hand_count(p);
        let object = g.object_ref(aura);
        assert!(
            !g.legal_actions()
                .contains(&Action::PutHandPermanent { object })
        );
        reject(&mut g, Action::PutHandPermanent { object });
        checked(&mut g, Action::DeclineHandEntry);
        assert_eq!(g.hand_count(p), n + 1);
        assert_eq!(g.state.objects[aura as usize].zone, Zone::Hand);
    }
}
#[test]
fn msh_entry_core_selected_legal_resolution_payment_stays_paid_when_redirected() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let hero = named(&mut g, p, "Adaptive Automaton", Zone::Hand);
        bounded(&mut g, p, ANY_HAND);
        let n = g.hand_count(p);
        let object = g.object_ref(hero);
        assert!(
            g.legal_actions()
                .contains(&Action::PutHandPermanent { object })
        );
        // Bounded ingress-replacement seam: the current admitted pool has no
        // hand-to-BF exile replacement. Select legally, redirect its owned staged
        // EntryMove before commit, and execute the ordinary continuation.
        g.choose_effect_object(p, object);
        assert!(g.advance_entries());
        g.state.entry_batch.as_mut().unwrap().moves[0].destination = Zone::Exile;
        let choice = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseEntryValue { .. }))
            .unwrap();
        checked(&mut g, choice);
        assert!(quiet(&g));
        assert_eq!(g.hand_count(p), n - 1);
        assert_eq!(g.state.objects[hero as usize].zone, Zone::Exile);
    }
}
#[test]
fn msh_entry_core_nick_front_entry_freezes_two_mentor_packets_before_transform() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = paid(&mut g, p, "Mentor of the Meek");
        let b = paid(&mut g, p, "Mentor of the Meek");
        let uprising = paid(&mut g, p, "Garruk's Uprising");
        let eight = named(&mut g, p, "Island", Zone::Library);
        for _ in 0..6 {
            named(&mut g, p, "Forest", Zone::Library);
        }
        let hero = named(&mut g, p, "Jennifer Walters", Zone::Library);
        bounded(&mut g, p, nick_program());
        select(&mut g, hero);
        assert_eq!(g.object_view(hero).power, 2);
        assert!(g.state.stack.is_empty());
        assert_eq!(
            g.state
                .pending_triggers
                .iter()
                .filter(|s| s.card == a || s.card == b)
                .count(),
            2
        );
        assert!(!g.state.pending_triggers.iter().any(|s| s.card == uprising));
        for who in [p, p.other()] {
            let v = g.observe(who);
            assert_eq!(
                v.entry_transform.unwrap().object.reference,
                g.object_ref(hero)
            );
        }
        let hand = g.hand_count(p);
        g = orphan(g, Action::ChooseOptional { accept: true });
        assert_eq!(g.object_view(hero).power, 6);
        assert_eq!(g.object_view(hero).card.name(), "The Sensational She-Hulk");
        assert!(
            !g.state
                .pending_triggers
                .iter()
                .chain(&g.state.stack)
                .any(|s| s.card == uprising)
        );
        assert_eq!(g.state.players[p.index()].library.last(), Some(&eight));
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        for _ in 0..25 {
            if quiet(&g) {
                break;
            }
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::Pass
                            | Action::PutTrigger { .. }
                            | Action::PayResolutionMana { .. }
                            | Action::FinishTargets
                    )
                })
                .unwrap();
            checked(&mut g, action);
        }
        assert!(quiet(&g));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.hand_count(p), hand + 2);
    }
}
#[test]
fn msh_entry_core_nick_optional_decline_keeps_front_and_finishes_remainder_once() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let hero = named(&mut g, p, "Monica Rambeau", Zone::Library);
        bounded(&mut g, p, nick_program());
        select(&mut g, hero);
        let snapshot = g.snapshot();
        let n = g.state.players[p.index()].library.len();
        g = orphan(g, Action::ChooseOptional { accept: false });
        assert!(quiet(&g));
        assert_eq!(g.object_view(hero).card.name(), "Monica Rambeau");
        assert_eq!(g.state.players[p.index()].library.len(), n);
        assert!(g.observe(p).entry_transform.is_none());
        reject(&mut g, Action::ChooseOptional { accept: true });
        g.restore(&snapshot).unwrap();
        checked(&mut g, Action::ChooseOptional { accept: true });
        assert_eq!(g.object_view(hero).card.name(), "Photon, Living Light");
    }
}
#[test]
fn msh_entry_core_nick_does_not_run_legend_sba_between_entry_and_transform() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let old = paid(&mut g, p, "Jennifer Walters");
        let new = named(&mut g, p, "Jennifer Walters", Zone::Library);
        bounded(&mut g, p, nick_program());
        select(&mut g, new);
        assert_eq!(g.state.objects[old as usize].zone, Zone::Battlefield);
        assert_eq!(g.state.objects[new as usize].zone, Zone::Battlefield);
        checked(&mut g, Action::ChooseOptional { accept: true });
        assert!(quiet(&g));
        assert_eq!(g.object_view(old).card.name(), "Jennifer Walters");
        assert_eq!(g.object_view(new).card.name(), "The Sensational She-Hulk");
        assert_eq!(g.state.objects[old as usize].zone, Zone::Battlefield);
    }
}
#[test]
fn msh_entry_core_nick_exact_actual_recipient_and_controller_independence() {
    for p in [Player::P0, Player::P1] {
        for redirected in [false, true] {
            let mut g = setup(p);
            let hero = named(&mut g, p, "Jennifer Walters", Zone::Library);
            bounded(&mut g, p, nick_program());
            checked(&mut g, Action::SelectLibrary { card: hero });
            assert!(g.legal_actions().contains(&Action::FinishLibrarySelection));
            // Bounded replacement boundary for currently unavailable ingress changes.
            stage_selected_fixture(
                &mut g,
                p,
                hero,
                if redirected {
                    Zone::Exile
                } else {
                    Zone::Battlefield
                },
                p.other(),
                false,
            );
            if redirected {
                assert!(quiet(&g));
                assert!(g.observe(p).entry_transform.is_none());
                assert_eq!(g.state.objects[hero as usize].zone, Zone::Exile);
            } else {
                assert_eq!(g.actor(), Some(p));
                assert_eq!(g.state.objects[hero as usize].controller, p.other());
                checked(&mut g, Action::ChooseOptional { accept: true });
                assert_eq!(g.object_view(hero).card.name(), "The Sensational She-Hulk");
                assert_eq!(g.state.objects[hero as usize].controller, p.other());
            }
        }
    }
}
#[test]
fn msh_entry_core_nick_stale_entered_incarnation_cannot_transform_a_new_object() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let hero = named(&mut g, p, "Jennifer Walters", Zone::Library);
        bounded(&mut g, p, nick_program());
        select(&mut g, hero);
        let before = g.object_ref(hero);
        // Explicit identity-boundary fixture: no priority is offered during this choice.
        g.move_object(hero, Zone::Exile);
        g.move_object(hero, Zone::Battlefield);
        assert_ne!(g.object_ref(hero), before);
        checked(&mut g, Action::ChooseOptional { accept: true });
        assert_eq!(g.object_view(hero).card.name(), "Jennifer Walters");
        assert_eq!(g.object_view(hero).face.unwrap().transforms, 0);
    }
}
const ANY_LIBRARY: &[Instruction] = &[Instruction::SelectLibraryThenMayTransform(
    LibrarySelectionEffect {
        spec: LibrarySelectionSpec {
            look: 7,
            count: LibrarySelectionCount::Fixed(1),
            filter: LibrarySelectionFilter::Any,
            optional: true,
            reveal: false,
            remainder: LibraryRemainder::BottomRandom,
        },
        look: LibraryLook::Fixed(7),
        destination: LibraryPickDestination::Battlefield,
        public: false,
        maximum_mana_value: None,
    },
)];
#[test]
fn msh_entry_core_private_library_remainder_stays_with_look_owner_during_foreign_entry_choice() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let entrant = named(&mut g, p, "Adaptive Automaton", Zone::Library);
        bounded(&mut g, p, ANY_LIBRARY);
        checked(&mut g, Action::SelectLibrary { card: entrant });
        g.finish_library_selection(p);
        g.state.entry_batch.as_mut().unwrap().moves[0].controller = p.other();
        assert!(g.advance_entries());
        assert_eq!(g.actor(), Some(p.other()));
        assert!(g.observe(p.other()).library_selection.is_none());
        assert!(g.observe(p).library_selection.is_some());
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseEntryValue { .. }))
            .unwrap();
        g = orphan(g, action);
        assert!(quiet(&g));
        assert_eq!(g.state.objects[entrant as usize].controller, p.other());
    }
}
#[test]
fn msh_entry_core_nick_filter_optional_empty_short_libraries_and_atomic_choices() {
    for p in [Player::P0, Player::P1] {
        for count in [0, 1, 6, 7, 8] {
            let mut g = setup(p);
            while let Some(&id) = g.state.players[p.index()].library.last() {
                g.move_object(id, Zone::Outside);
            }
            let mut ids = Vec::new();
            for i in 0..count {
                ids.push(named(
                    &mut g,
                    p,
                    if i == 0 {
                        "Captain America's Shield"
                    } else {
                        "Island"
                    },
                    Zone::Library,
                ));
            }
            bounded(&mut g, p, nick_program());
            if count == 0 {
                assert!(quiet(&g));
                continue;
            }
            assert!(g.observe(p.other()).library_selection.is_none());
            let v = g.observe(p).library_selection.unwrap();
            assert_eq!(v.remaining.len(), count.min(7));
            if count == 8 {
                reject(&mut g, Action::SelectLibrary { card: ids[0] });
            }
            let wrong = ids
                .iter()
                .copied()
                .find(|&id| g.state.objects[id as usize].card.is_land());
            if let Some(card) = wrong {
                reject(&mut g, Action::SelectLibrary { card });
            }
            g = orphan(g, Action::FinishLibrarySelection);
            assert!(quiet(&g));
            assert_eq!(g.state.players[p.index()].library.len(), count);
        }
    }
}

// Explicit replacement boundary: the production selector and exact selected
// references are real. Only the ingress plan is supplied by this bounded seam.
fn stage_selected_fixture(
    g: &mut Game,
    p: Player,
    id: CardId,
    destination: Zone,
    controller: Player,
    skipped: bool,
) {
    assert!(g.legal_actions().contains(&Action::FinishLibrarySelection));
    g.forget_library_top(p);
    g.state
        .resolving
        .as_mut()
        .unwrap()
        .selection
        .as_mut()
        .unwrap()
        .entering = true;
    let mut movement = g.entry_move(id, destination, false);
    movement.controller = controller;
    movement.skipped = skipped;
    g.stage_entries(vec![movement], None);
    if g.state.entry_batch.is_some() {
        assert!(!g.advance_entries());
    } else {
        g.run_program();
    }
}
#[test]
fn msh_entry_core_nick_random_remainder_excludes_actual_departures_and_includes_blocked_pick() {
    for p in [Player::P0, Player::P1] {
        for skipped in [false, true] {
            let mut g = setup(p);
            let hero = named(&mut g, p, "Jennifer Walters", Zone::Library);
            let count = g.state.players[p.index()].library.len();
            bounded(&mut g, p, nick_program());
            checked(&mut g, Action::SelectLibrary { card: hero });
            if skipped {
                stage_selected_fixture(&mut g, p, hero, Zone::Battlefield, p, true);
                assert!(quiet(&g));
                assert_eq!(g.state.objects[hero as usize].zone, Zone::Library);
                assert_eq!(g.state.players[p.index()].library.len(), count);
            } else {
                checked(&mut g, Action::FinishLibrarySelection);
                let other = g
                    .state
                    .resolving
                    .as_ref()
                    .unwrap()
                    .selection
                    .as_ref()
                    .unwrap()
                    .remaining[0]
                    .card;
                // A bounded replacement-side move, not a legal priority window.
                g.move_object(other, Zone::Exile);
                checked(&mut g, Action::ChooseOptional { accept: true });
                assert!(!g.state.players[p.index()].library.contains(&other));
                assert_eq!(g.state.objects[other as usize].zone, Zone::Exile);
                assert_eq!(g.state.players[p.index()].library.len(), count - 2);
            }
        }
    }
}
