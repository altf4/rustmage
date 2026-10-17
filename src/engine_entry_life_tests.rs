use super::tests::{act, pass_pair, scenario, spawn};
use super::*;

const SHOCKS: &[(&str, &str, &str)] = &[
    ("Blood Crypt", "Swamp", "Mountain"),
    ("Breeding Pool", "Forest", "Island"),
    ("Godless Shrine", "Plains", "Swamp"),
    ("Hallowed Fountain", "Plains", "Island"),
    ("Overgrown Tomb", "Swamp", "Forest"),
    ("Sacred Foundry", "Mountain", "Plains"),
    ("Steam Vents", "Island", "Mountain"),
    ("Stomping Ground", "Mountain", "Forest"),
    ("Temple Garden", "Forest", "Plains"),
    ("Watery Grave", "Island", "Swamp"),
];
fn named(g: &mut Game, owner: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    assert!(card.is_supported(), "{name}");
    spawn(g, owner, card, zone)
}
fn replay(g: &mut Game, action: Action) {
    let saved = g.snapshot();
    let revision = g.revision();
    let actor = g.actor().unwrap();
    let mut fork = g.clone();
    act(&mut fork, action.clone());
    act(g, action.clone());
    let result = g.snapshot();
    assert_eq!(fork.snapshot(), result);
    g.restore(&saved).unwrap();
    assert_eq!(g.apply_index(actor, revision, 0), Err(Error::StaleRevision));
    act(g, action);
    assert_eq!(g.snapshot(), result);
}
fn reject(g: &mut Game, actor: Player, action: Action, error: Error) {
    let saved = g.snapshot();
    let revision = g.revision();
    assert_eq!(g.apply(actor, action), Err(error));
    assert_eq!(g.snapshot(), saved);
    assert_eq!(g.revision(), revision);
}
fn mana_options(g: &Game, id: CardId) -> Vec<Mana> {
    g.legal_actions()
        .into_iter()
        .filter_map(|a| match a {
            Action::ActivateMana {
                card, production, ..
            } if card == id => Some(production),
            _ => None,
        })
        .collect()
}
// Explicit synthetic continuation fixture: no printed search/return card is
// newly admitted by this helper or by the transaction tests below.
fn program(g: &mut Game, code: &'static [Instruction]) {
    let id = named(g, Player::P0, "Shock", Zone::Hand);
    let item = g.cast_item(Player::P0, id, [0; 6]);
    g.state.resolving = Some(Resolution {
        explore: None,
        evidence: None,
        connive: None,
        item,
        program: code,
        pc: 0,
        target_group: 0,
        moved_count: None,
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
    g.advance_entries();
}

#[test]
fn all_shock_lands_offer_a_pre_entry_choice_then_exact_intrinsic_mana() {
    for &(name, a, b) in SHOCKS {
        for pay in [false, true] {
            let mut g = scenario();
            let id = named(&mut g, Player::P0, name, Zone::Hand);
            let reference = g.object_ref(id);
            let card = g.state.objects[id as usize].card;
            assert!(card.has_subtype(a) && card.has_subtype(b));
            assert!(!card.is_basic_land());
            replay(&mut g, Action::PlayLand { card: id });
            let obs = g.observe(Player::P0);
            assert_eq!(obs.schema, crate::engine::OBSERVATION_SCHEMA);
            assert_eq!(obs.decision, "entry_life");
            let choice = obs.entry_life.unwrap();
            assert_eq!(choice.object, reference);
            assert_eq!(choice.card, card);
            assert_eq!(choice.cost, 2);
            assert!(!choice.already_tapped);
            assert!(choice.selected.is_empty() && choice.selected_attachments.is_empty());
            assert!(g.observe(Player::P0).aura_entry.is_none());
            assert_eq!(g.object_view(id).zone, Zone::Hand);
            assert_eq!(g.state.players[0].life, 20);
            assert!(g.state.land_played && g.state.stack.is_empty());
            assert!(
                g.legal_actions()
                    .iter()
                    .all(|a| matches!(a, Action::Concede | Action::ChooseEntryLife { .. }))
            );
            reject(&mut g, Player::P0, Action::Pass, Error::IllegalAction);
            reject(
                &mut g,
                Player::P1,
                Action::ChooseEntryLife { pay },
                Error::WrongActor,
            );
            replay(&mut g, Action::ChooseEntryLife { pay });
            assert!(g.state.entry_batch.is_none());
            assert_eq!(g.object_view(id).zone, Zone::Battlefield);
            assert_ne!(g.object_ref(id), reference);
            assert_eq!(g.object_view(id).tapped, !pay);
            assert_eq!(g.state.players[0].life, if pay { 18 } else { 20 });
            if !pay {
                assert!(mana_options(&g, id).is_empty());
                g.begin_turn();
            }
            let mana = mana_options(&g, id);
            assert_eq!(mana.len(), 2);
            let expected: Vec<_> = [a, b]
                .into_iter()
                .map(|s| match s {
                    "Plains" => 0,
                    "Island" => 1,
                    "Swamp" => 2,
                    "Mountain" => 3,
                    "Forest" => 4,
                    _ => unreachable!(),
                })
                .collect();
            for color in 0..6 {
                assert_eq!(
                    mana.iter().any(|m| m[color] == 1),
                    expected.contains(&color),
                    "{name}"
                );
            }
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a,Action::ActivateMana{card,..} if *card==id))
                .unwrap();
            replay(&mut g, a);
            assert!(g.object_view(id).tapped);
        }
    }
}

#[test]
fn payments_obey_exact_life_and_lethal_payment_waits_for_entry() {
    for life in [-1, 0, 1, 2, 3] {
        let mut g = scenario();
        g.state.players[0].life = life;
        let id = named(&mut g, Player::P0, "Blood Crypt", Zone::Hand);
        act(&mut g, Action::PlayLand { card: id });
        if life < 2 {
            assert!(g.state.entry_batch.is_none());
            assert_eq!(g.state.players[0].life, life);
            assert!(g.object_view(id).tapped);
            if life > 0 {
                reject(
                    &mut g,
                    Player::P0,
                    Action::ChooseEntryLife { pay: true },
                    Error::IllegalAction,
                );
            }
        } else {
            assert!(g.state.outcome.is_none());
            assert_eq!(g.object_view(id).zone, Zone::Hand);
            replay(&mut g, Action::ChooseEntryLife { pay: true });
            assert_eq!(g.object_view(id).zone, Zone::Battlefield);
            assert!(!g.object_view(id).tapped);
            assert_eq!(g.state.players[0].life, life - 2);
        }
        assert_eq!(
            g.outcome(),
            if life <= 0 || life == 2 {
                Some(Outcome::Winner(Player::P1))
            } else {
                None
            }
        );
    }
}

#[test]
fn payment_does_not_remove_an_external_tapped_requirement() {
    for pay in [false, true] {
        let mut g = scenario();
        let id = named(&mut g, Player::P0, "Temple Garden", Zone::Exile);
        g.stage_entries(vec![g.entry_move(id, Zone::Battlefield, true)], None);
        assert!(g.advance_entries());
        assert!(g.observe(Player::P0).entry_life.unwrap().already_tapped);
        replay(&mut g, Action::ChooseEntryLife { pay });
        assert!(g.object_view(id).tapped);
        assert_eq!(g.state.players[0].life, 20 - if pay { 2 } else { 0 });
    }
}

#[test]
fn simultaneous_public_entries_choose_apnap_and_commit_together() {
    let mut g = scenario();
    g.state.active = Player::P1;
    let a = named(&mut g, Player::P0, "Temple Garden", Zone::Exile);
    let b = named(&mut g, Player::P1, "Watery Grave", Zone::Exile);
    g.stage_entries(
        vec![
            g.entry_move(a, Zone::Battlefield, false),
            g.entry_move(b, Zone::Battlefield, false),
        ],
        None,
    );
    g.advance_entries();
    assert_eq!(g.actor(), Some(Player::P1));
    assert_eq!(g.observe(Player::P0).entry_life.unwrap().object.card, b);
    replay(&mut g, Action::ChooseEntryLife { pay: true });
    assert_eq!(g.actor(), Some(Player::P0));
    assert_eq!(g.object_view(a).zone, Zone::Exile);
    assert_eq!(g.object_view(b).zone, Zone::Exile);
    let choice = g.observe(Player::P0).entry_life.unwrap();
    assert_eq!(choice.selected.len(), 1);
    assert_eq!(choice.selected[0].object.card, b);
    assert!(choice.selected[0].paid);
    replay(&mut g, Action::ChooseEntryLife { pay: false });
    assert!(g.object_view(a).tapped);
    assert!(!g.object_view(b).tapped);
    assert_eq!(g.state.players[1].life, 18);
    assert_eq!(g.state.players[0].life, 20);
}

#[test]
fn multiple_payments_share_life_and_skip_impossible_later_payments_without_sbas() {
    for life in [2, 3, 4] {
        let mut g = scenario();
        g.state.players[0].life = life;
        let a = named(&mut g, Player::P0, "Blood Crypt", Zone::Exile);
        let b = named(&mut g, Player::P0, "Watery Grave", Zone::Exile);
        g.stage_entries(
            vec![
                g.entry_move(a, Zone::Battlefield, false),
                g.entry_move(b, Zone::Battlefield, false),
            ],
            None,
        );
        g.advance_entries();
        replay(&mut g, Action::ChooseEntryLife { pay: true });
        if life == 4 {
            assert_eq!(g.object_view(a).zone, Zone::Exile);
            assert_eq!(g.state.players[0].life, 2);
            replay(&mut g, Action::ChooseEntryLife { pay: true });
            assert!(!g.object_view(b).tapped);
        } else {
            assert!(g.object_view(b).tapped);
        }
        assert!(!g.object_view(a).tapped);
        assert_eq!(g.object_view(a).zone, Zone::Battlefield);
        assert_eq!(g.object_view(b).zone, Zone::Battlefield);
        assert_eq!(
            g.outcome(),
            if life == 3 {
                None
            } else {
                Some(Outcome::Winner(Player::P1))
            }
        );
    }
}

#[test]
fn search_holds_library_and_rng_then_commits_entry_shuffles_and_resumes_at_zero_life() {
    const CODE: &[Instruction] = &[
        Instruction::Search(SearchSpec {
            filter: crate::SearchFilter::Land,
            maximum: 1,
            destination: crate::SearchDestination::Battlefield,
            tapped: false,
            reveal: false,
        }),
        Instruction::GainLife(PlayerSet::Controller, 3),
        Instruction::Draw(PlayerSet::Controller, 1),
    ];
    let mut g = scenario();
    g.state.players[0].life = 2;
    let lagac = named(&mut g, Player::P0, "Spitfire Lagac", Zone::Battlefield);
    let id = named(&mut g, Player::P0, "Stomping Ground", Zone::Library);
    program(&mut g, CODE);
    replay(&mut g, Action::ChooseSearch { card: id });
    let library = g.state.players[0].library.clone();
    let rng = g.state.rng;
    replay(&mut g, Action::FinishSearch);
    assert_eq!(g.state.players[0].library, library);
    assert_eq!(g.state.rng, rng);
    assert!(g.observe(Player::P1).entry_life.is_none());
    assert!(g.observe(Player::P1).actions.is_empty());
    assert!(!g.observe(Player::P1).objects.iter().any(|o| o.id == id));
    let mut expected: Vec<_> = library.into_iter().filter(|&i| i != id).collect();
    let mut expected_rng = rng;
    expected_rng.shuffle(&mut expected);
    let drawn = expected.pop().unwrap();
    replay(&mut g, Action::ChooseEntryLife { pay: true });
    assert!(g.outcome().is_none());
    assert_eq!(g.state.players[0].life, 3);
    assert_eq!(g.state.players[0].library.as_slice(), expected.as_slice());
    assert_eq!(g.state.rng, expected_rng);
    assert_eq!(g.object_view(drawn).zone, Zone::Hand);
    assert_eq!(g.object_view(id).zone, Zone::Battlefield);
    assert!(!g.object_view(id).tapped);
    assert!(g.state.resolving.is_none());
    assert!(g.state.stack.iter().any(|s| s.card == lagac));
    pass_pair(&mut g);
    assert_eq!(g.state.players[1].life, 19);
}

#[test]
fn announced_land_is_public_during_choice_then_reveal_record_is_removed_on_entry() {
    let mut g = scenario();
    let id = named(&mut g, Player::P0, "Hallowed Fountain", Zone::Hand);
    let other = named(&mut g, Player::P0, "Watery Grave", Zone::Hand);
    assert!(!g.observe(Player::P1).objects.iter().any(|o| o.id == id));
    replay(&mut g, Action::PlayLand { card: id });
    let opponent = g.observe(Player::P1);
    assert_eq!(opponent.entry_life.unwrap().object, g.object_ref(id));
    assert!(
        opponent
            .objects
            .iter()
            .any(|o| o.id == id && o.zone == Zone::Hand)
    );
    assert!(!opponent.objects.iter().any(|o| o.id == other));
    assert!(opponent.actions.is_empty());
    assert!(g.state.known_hand.contains(&g.object_ref(id)));
    replay(&mut g, Action::ChooseEntryLife { pay: false });
    assert!(g.state.known_hand.is_empty());
    reject(
        &mut g,
        Player::P0,
        Action::PlayLand { card: other },
        Error::IllegalAction,
    );
}

#[test]
fn mixed_attachment_and_life_choices_report_previous_public_selections() {
    for aura_first in [false, true] {
        let mut g = scenario();
        let bear = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        let aura = named(&mut g, Player::P0, "Frantic Strength", Zone::Exile);
        let land = named(&mut g, Player::P0, "Stomping Ground", Zone::Exile);
        let moves = if aura_first {
            vec![
                g.entry_move(aura, Zone::Battlefield, false),
                g.entry_move(land, Zone::Battlefield, false),
            ]
        } else {
            vec![
                g.entry_move(land, Zone::Battlefield, false),
                g.entry_move(aura, Zone::Battlefield, false),
            ]
        };
        g.stage_entries(moves, None);
        g.advance_entries();
        let attachment = Action::ChooseAuraAttachment {
            object: g.object_ref(bear),
        };
        if aura_first {
            replay(&mut g, attachment.clone());
            let choice = g.observe(Player::P1).entry_life.unwrap();
            assert_eq!(choice.selected_attachments.len(), 1);
            assert_eq!(choice.selected_attachments[0].aura.card, aura);
            replay(&mut g, Action::ChooseEntryLife { pay: true });
        } else {
            replay(&mut g, Action::ChooseEntryLife { pay: true });
            let choice = g.observe(Player::P1).aura_entry.unwrap();
            assert_eq!(choice.selected_life.len(), 1);
            assert!(choice.selected_life[0].paid);
            assert_eq!(choice.selected_life[0].object.card, land);
            replay(&mut g, attachment);
        }
        assert_eq!(g.object_view(aura).attached_to, Some(g.object_ref(bear)));
        assert_eq!(g.object_view(land).zone, Zone::Battlefield);
        assert!(!g.object_view(land).tapped);
    }
}

#[test]
fn linked_exile_return_pays_again_for_a_new_incarnation_and_defers_entry_triggers() {
    let mut g = scenario();
    let id = named(&mut g, Player::P0, "Temple Garden", Zone::Hand);
    replay(&mut g, Action::PlayLand { card: id });
    replay(&mut g, Action::ChooseEntryLife { pay: true });
    let first = g.object_ref(id);
    g.move_object(id, Zone::Exile);
    // Synthetic CR610.3 link; no newly admitted exile spell targets a land.
    let source = named(&mut g, Player::P1, "Banishing Light", Zone::Battlefield);
    g.state.exile_links.push(ExileLink {
        source: g.object_ref(source),
        exiled: g.object_ref(id),
        return_zone: Zone::Battlefield,
    });
    let lagac = named(&mut g, Player::P0, "Spitfire Lagac", Zone::Battlefield);
    g.move_object(source, Zone::Hand);
    g.advance_entries();
    assert_eq!(g.observe(Player::P0).entry_life.unwrap().object.card, id);
    assert_eq!(g.object_view(id).zone, Zone::Exile);
    assert!(!g.state.pending_triggers.iter().any(|s| s.card == lagac));
    replay(&mut g, Action::ChooseEntryLife { pay: false });
    assert!(g.object_view(id).tapped);
    assert_ne!(first, g.object_ref(id));
    assert_eq!(g.state.players[0].life, 18);
    assert!(g.state.stack.iter().any(|s| s.card == lagac));
    pass_pair(&mut g);
    assert_eq!(g.state.players[1].life, 19);
}

#[test]
fn planeswalker_entry_condition_uses_existing_own_battlefield_only() {
    let planeswalker = Card::catalogue().find(|c| c.kind_mask() & 64 != 0).unwrap();
    let cycle: Vec<_> = Card::catalogue()
        .filter(|c| c.is_supported() && c.entry_tapped() == EntryTapped::UnlessPlaneswalker)
        .collect();
    assert_eq!(cycle.len(), 10);
    for land in cycle {
        for owner in [Player::P0, Player::P1] {
            for zone in [Zone::Battlefield, Zone::Hand, Zone::Exile] {
                let mut g = scenario();
                // Type-only structural fixture. This does not execute or admit
                // the unsupported planeswalker's loyalty or other abilities.
                spawn(&mut g, owner, planeswalker, zone);
                let id = spawn(&mut g, Player::P0, land, Zone::Hand);
                replay(&mut g, Action::PlayLand { card: id });
                assert_eq!(
                    g.object_view(id).tapped,
                    owner != Player::P0 || zone != Zone::Battlefield
                );
                assert!(g.state.entry_batch.is_none());
            }
        }
        let mut g = scenario();
        let id = spawn(&mut g, Player::P0, land, Zone::Exile);
        let pw = spawn(&mut g, Player::P0, planeswalker, Zone::Exile);
        let tapped = g.enters_tapped(id);
        g.stage_entries(
            vec![
                g.entry_move(pw, Zone::Battlefield, false),
                g.entry_move(id, Zone::Battlefield, tapped),
            ],
            None,
        );
        assert!(g.object_view(id).tapped); // A simultaneous newcomer does not count.
    }
}

#[test]
fn simultaneous_lethal_payments_wait_for_both_players_then_draw_the_game() {
    let mut g = scenario();
    g.state.players[0].life = 2;
    g.state.players[1].life = 2;
    let a = named(&mut g, Player::P0, "Temple Garden", Zone::Exile);
    let b = named(&mut g, Player::P1, "Watery Grave", Zone::Exile);
    g.stage_entries(
        vec![
            g.entry_move(a, Zone::Battlefield, false),
            g.entry_move(b, Zone::Battlefield, false),
        ],
        None,
    );
    g.advance_entries();
    replay(&mut g, Action::ChooseEntryLife { pay: true });
    assert_eq!(g.state.players[0].life, 0);
    assert!(g.outcome().is_none());
    assert_eq!(g.actor(), Some(Player::P1));
    assert_eq!(g.object_view(a).zone, Zone::Exile);
    replay(&mut g, Action::ChooseEntryLife { pay: true });
    assert_eq!(g.outcome(), Some(Outcome::Draw));
    assert_eq!(g.object_view(a).zone, Zone::Battlefield);
    assert_eq!(g.object_view(b).zone, Zone::Battlefield);
}

#[test]
fn real_swampcycling_finds_a_nonbasic_shock_then_its_land_play_offers_payment() {
    let mut g = scenario();
    let cycling = named(&mut g, Player::P0, "Canyon Crawler", Zone::Hand);
    let land = named(&mut g, Player::P0, "Watery Grave", Zone::Library);
    let invalid = named(&mut g, Player::P0, "Hallowed Fountain", Zone::Library);
    g.state.players[0].mana[5] = 2;
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::Activate{card,..} if *card==cycling))
        .unwrap();
    replay(&mut g, a);
    pass_pair(&mut g);
    assert!(
        g.legal_actions()
            .contains(&Action::ChooseSearch { card: land })
    );
    reject(
        &mut g,
        Player::P0,
        Action::ChooseSearch { card: invalid },
        Error::IllegalAction,
    );
    replay(&mut g, Action::ChooseSearch { card: land });
    replay(&mut g, Action::FinishSearch);
    assert!(
        g.observe(Player::P1)
            .objects
            .iter()
            .any(|o| o.id == land && o.zone == Zone::Hand)
    );
    assert_eq!(g.state.known_hand.len(), 1);
    replay(&mut g, Action::PlayLand { card: land });
    assert_eq!(g.state.known_hand.len(), 1); // No duplicate knowledge entry.
    replay(&mut g, Action::ChooseEntryLife { pay: true });
    assert!(g.state.known_hand.is_empty());
    assert!(!g.object_view(land).tapped);
    assert_eq!(g.state.players[0].life, 18);
}

#[test]
fn official_shack_and_shock_example_uses_life_before_simultaneous_payment() {
    // Wizards' Duskmourn release notes, New Cycle of Nonbasic Lands:
    // https://magic.wizards.com/en/news/feature/duskmourn-house-of-horror-release-notes
    // Shack + Breeding Pool at 14 life: Shack enters tapped whether or not
    // the player pays for Pool. Replacement ordering does not change this.
    for life in [13, 14] {
        for pay in [false, true] {
            for shock_first in [false, true] {
                let mut g = scenario();
                g.state.players[0].life = life;
                let shack = named(&mut g, Player::P0, "Lakeside Shack", Zone::Exile);
                let pool = named(&mut g, Player::P0, "Breeding Pool", Zone::Exile);
                let m1 = g.entry_move(shack, Zone::Battlefield, g.enters_tapped(shack));
                let m2 = g.entry_move(pool, Zone::Battlefield, g.enters_tapped(pool));
                g.stage_entries(
                    if shock_first {
                        vec![m2, m1]
                    } else {
                        vec![m1, m2]
                    },
                    None,
                );
                g.advance_entries();
                assert_eq!(g.observe(Player::P0).entry_life.unwrap().object.card, pool);
                replay(&mut g, Action::ChooseEntryLife { pay });
                assert_eq!(g.object_view(shack).tapped, life > 13);
                assert_eq!(g.object_view(pool).tapped, !pay);
                assert_eq!(g.state.players[0].life, life - if pay { 2 } else { 0 });
            }
        }
    }
}
