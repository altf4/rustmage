//! Whole-card BIG library rules, paid actions and owned branch/restore checks.
use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::*;
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
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                )
            })
            .unwrap_or_else(|| panic!("unexpected {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("stalled {:?}", g.state.pending);
}
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let rev = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), rev);
}
fn activate(g: &mut Game, card: CardId, ability: u16, target: Option<Target>) {
    checked(
        g,
        Action::AnnounceActivate {
            card,
            ability,
            counter_payment: Counters::default(),
        },
    );
    if let Some(target) = target {
        checked(g, Action::ChooseTarget { target });
    }
    until(g, |g| g.state.announcing.is_none());
}
fn select_library(g: &mut Game, id: CardId) {
    checked(g, Action::SelectLibrary { card: id });
    checked(g, Action::FinishLibrarySelection);
}
fn start_step(g: &mut Game, p: Player, step: Step) {
    g.state.active = p;
    g.state.priority = p;
    g.state.step = step;
    g.begin_step(step);
    g.prepare_triggers();
}
fn tokens(g: &Game, p: Player, subtype: &str) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(id, o)| {
            o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && g.has_subtype(*id as CardId, subtype)
        })
        .map(|(id, _)| id as CardId)
        .collect()
}
#[test]
fn big_farmer_entry_attack_food_cost_exact_milled_recovery_and_decline_both_seats() {
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            let mut g = setup(p);
            let farmer = named(&mut g, p, "Bristlebud Farmer", Zone::Hand);
            cast(&mut g, farmer, None, &[]);
            settle(&mut g);
            assert_eq!(tokens(&g, p, "Food").len(), 2);
            assert_eq!((g.stats(farmer).power, g.stats(farmer).toughness), (5, 5));
            assert!(g.keywords(farmer).contains(Keywords::TRAMPLE));
            let prior = named(&mut g, p, "Forest", Zone::Graveyard);
            let recovered = named(&mut g, p, "Forest", Zone::Library);
            let spell = named(&mut g, p, "Giant Growth", Zone::Library);
            named(&mut g, p, "Grizzly Bears", Zone::Library);
            let count = g.state.players[p.index()].library.len();
            g.state.objects[farmer as usize].summoning_sick = false;
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            checked(&mut g, Action::Attack { card: farmer });
            checked(&mut g, Action::FinishAttackers);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectChoice { .. })
            });
            let opponent_food =
                super::tests::spawn(&mut g, p.other(), Card(5246), Zone::Battlefield);
            let bad = g.object_ref(opponent_food);
            reject(&mut g, Action::Sacrifice { object: bad });
            if pay {
                let food = g.object_ref(tokens(&g, p, "Food")[0]);
                // A queued attack trigger survives the source's departure.
                g.move_object(farmer, Zone::Graveyard);
                checked(&mut g, Action::Sacrifice { object: food });
                assert_eq!(g.state.players[p.index()].library.len(), count - 3);
                assert_eq!(tokens(&g, p, "Food").len(), 1);
                let old = g.object_ref(prior);
                reject(&mut g, Action::RecoverMilled { object: Some(old) });
                let instant = g.object_ref(spell);
                reject(
                    &mut g,
                    Action::RecoverMilled {
                        object: Some(instant),
                    },
                );
                let object = g.object_ref(recovered);
                checked(
                    &mut g,
                    Action::RecoverMilled {
                        object: Some(object),
                    },
                );
                assert_eq!(g.object_view(recovered).zone, Zone::Hand);
            } else {
                checked(&mut g, Action::DeclineEffectPayment);
                assert_eq!(g.state.players[p.index()].library.len(), count);
                assert_eq!(tokens(&g, p, "Food").len(), 2);
            }
        }
    }
}
#[test]
fn big_fomori_external_discard_dynamic_artifacts_private_pick_and_random_bottom() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let vault = named(&mut g, p, "Fomori Vault", Zone::Battlefield);
        let discard = named(&mut g, p, "Forest", Zone::Hand);
        super::tests::spawn(&mut g, p, Card(5246), Zone::Battlefield);
        let second = super::tests::spawn(&mut g, p, Card(5246), Zone::Battlefield);
        let chosen = named(&mut g, p, "Grizzly Bears", Zone::Library);
        let rest = named(&mut g, p, "Giant Growth", Zone::Library);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: vault,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        assert!(g.observe(p.other()).additional_cost.is_none());
        let object = g.object_ref(discard);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        until(&mut g, |g| g.state.announcing.is_none());
        assert_eq!(g.object_view(discard).zone, Zone::Graveyard);
        assert_eq!(g.object_view(vault).zone, Zone::Battlefield);
        assert!(g.object_view(vault).tapped);
        // X is evaluated at resolution, after a third artifact appears.
        super::tests::spawn(&mut g, p, Card(5246), Zone::Battlefield);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectLibrarySelection { .. })
        });
        assert_eq!(g.observe(p).library_selection.unwrap().remaining.len(), 3);
        assert!(g.observe(p.other()).library_selection.is_none());
        select_library(&mut g, chosen);
        assert_eq!(g.object_view(chosen).zone, Zone::Hand);
        assert_eq!(g.object_view(rest).zone, Zone::Library);
        assert_eq!(g.object_view(second).zone, Zone::Battlefield);
    }
}
#[test]
fn big_loot_counts_other_nonland_types_and_permission_survives_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let loot = named(&mut g, p, "Loot, the Key to Everything", Zone::Battlefield);
        named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        named(&mut g, p, "Omenpath Journey", Zone::Battlefield);
        super::tests::spawn(&mut g, p, Card(5246), Zone::Battlefield);
        named(&mut g, p, "Forest", Zone::Battlefield);
        let count = g.state.players[p.index()].library.len();
        start_step(&mut g, p, Step::Upkeep);
        g.move_object(loot, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), count - 3);
        let permissions = &g.state.exile_permissions;
        assert_eq!(permissions.len(), 3);
        assert!(
            permissions
                .iter()
                .all(|x| x.player == p && x.expires_turn == g.state.turn)
        );
    }
}
#[test]
fn big_omenpath_distinct_names_private_search_linked_random_tapped_and_new_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let first = named(&mut g, p, "Forest", Zone::Library);
        let duplicate = named(&mut g, p, "Forest", Zone::Library);
        let island = named(&mut g, p, "Island", Zone::Library);
        let journey = named(&mut g, p, "Omenpath Journey", Zone::Hand);
        cast(&mut g, journey, None, &[]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectSearch { .. })
        });
        assert!(g.observe(p.other()).search_choice.is_none());
        checked(&mut g, Action::ChooseSearch { card: first });
        reject(&mut g, Action::ChooseSearch { card: duplicate });
        checked(&mut g, Action::ChooseSearch { card: island });
        checked(&mut g, Action::FinishSearch);
        settle(&mut g);
        assert_eq!(g.observe(p).linked_exile.len(), 2);
        start_step(&mut g, p, Step::End);
        settle(&mut g);
        let entered = [first, island]
            .into_iter()
            .find(|&id| g.object_view(id).zone == Zone::Battlefield)
            .unwrap();
        assert!(g.object_view(entered).tapped);
        assert_eq!(g.observe(p).linked_exile.len(), 1);
        g.move_object(journey, Zone::Graveyard);
        g.move_object(journey, Zone::Battlefield);
        g.prepare_triggers();
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectSearch { .. })
        });
        checked(&mut g, Action::FinishSearch);
        settle(&mut g);
        let remaining = [first, island]
            .into_iter()
            .find(|&id| g.object_view(id).zone == Zone::Exile)
            .unwrap();
        start_step(&mut g, p, Step::End);
        settle(&mut g);
        assert_eq!(g.object_view(remaining).zone, Zone::Exile);
    }
}
#[test]
fn big_nexus_private_optional_hand_exile_and_copiable_golem_exceptions() {
    for p in [Player::P0, Player::P1] {
        for accept in [false, true] {
            let mut g = setup(p);
            let nexus = named(&mut g, p, "Nexus of Becoming", Zone::Battlefield);
            let card = named(&mut g, p, "Helpful Hunter", Zone::Hand);
            let invalid = named(&mut g, p, "Forest", Zone::Hand);
            let hand = g.hand_count(p);
            start_step(&mut g, p, Step::BeginCombat);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectChoice { .. })
            });
            assert_eq!(g.hand_count(p), hand + 1);
            assert!(g.observe(p.other()).effect_choice.is_none());
            let object = g.object_ref(invalid);
            reject(&mut g, Action::PutHandCreature { object });
            g.move_object(nexus, Zone::Graveyard);
            if accept {
                let object = g.object_ref(card);
                checked(&mut g, Action::PutHandCreature { object });
                settle(&mut g);
                assert_eq!(g.object_view(card).zone, Zone::Exile);
                let token = g
                    .state
                    .objects
                    .iter()
                    .enumerate()
                    .find(|(id, o)| {
                        o.zone == Zone::Battlefield
                            && o.is_copy_token()
                            && g.object_view(*id as CardId).name() == "Helpful Hunter"
                    })
                    .map(|(id, _)| id as CardId)
                    .unwrap();
                assert_eq!((g.stats(token).power, g.stats(token).toughness), (3, 3));
                assert!(g.has_subtype(token, "Golem"));
                assert!(g.current_kinds(token) & 17 == 17);
            } else {
                checked(&mut g, Action::DeclineHandEntry);
                assert_eq!(g.object_view(card).zone, Zone::Hand);
            }
        }
    }
}

#[test]
fn big_cage_private_hideaway_coven_after_counter_and_free_cast_both_seats() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let hidden = named(&mut g, p, "Llanowar Elves", Zone::Library);
        let cage = named(&mut g, p, "Collector's Cage", Zone::Hand);
        cast(&mut g, cage, None, &[]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectLibrarySelection { .. })
        });
        assert!(!g.object_view(cage).tapped);
        assert!(g.observe(p.other()).library_selection.is_none());
        select_library(&mut g, hidden);
        assert_eq!(g.object_view(hidden).zone, Zone::Exile);
        assert!(
            g.observe(p)
                .objects
                .iter()
                .any(|o| o.reference.card == hidden)
        );
        assert!(
            g.observe(p.other())
                .objects
                .iter()
                .all(|o| o.reference.card != hidden)
        );
        assert!(g.observe(p.other()).linked_exile.is_empty());
        let one = named(&mut g, p, "Helpful Hunter", Zone::Battlefield);
        let two = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let three = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        activate(&mut g, cage, 0, Some(Target::Card(one)));
        // Counter changes 1 to 2; three creatures now have only two powers.
        settle(&mut g);
        assert_eq!(g.object_view(one).counters.plus_one, 1);
        assert_eq!(g.object_view(hidden).zone, Zone::Exile);
        g.state.objects[cage as usize].tapped = false;
        activate(&mut g, cage, 0, Some(Target::Card(three)));
        // The other 2-power creature must change before the second resolves.
        g.state.objects[two as usize].growth.power = 1;
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::ResolutionCasts { .. })
        });
        let before = g.state.players[p.index()].mana;
        let object = g.object_ref(hidden);
        checked(
            &mut g,
            Action::CastExiledDuringResolution {
                object,
                face: CastFace::Normal,
            },
        );
        until(&mut g, |g| g.state.announcing.is_none());
        checked(&mut g, Action::FinishResolutionCasts);
        settle(&mut g);
        assert_eq!(g.object_view(hidden).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].mana, before);
        assert!(
            g.observe(p.other())
                .objects
                .iter()
                .any(|o| o.reference.card == hidden)
        );
    }
}

#[test]
fn big_cage_historical_controllers_can_look_after_control_and_source_departure() {
    let mut g = setup(Player::P0);
    let hidden = named(&mut g, Player::P0, "Forest", Zone::Library);
    let cage = named(&mut g, Player::P0, "Collector's Cage", Zone::Hand);
    cast(&mut g, cage, None, &[]);
    // Record both controllers before the hideaway selection has resolved.
    until(&mut g, |g| {
        matches!(g.state.pending, Pending::EffectLibrarySelection { .. })
    });
    let reference = g.object_ref(cage);
    g.change_controller(reference, Player::P1);
    g.change_controller(reference, Player::P0);
    select_library(&mut g, hidden);
    g.move_object(cage, Zone::Graveyard);
    for p in [Player::P0, Player::P1] {
        assert!(
            g.observe(p)
                .objects
                .iter()
                .any(|o| o.reference.card == hidden)
        );
    }
}

#[test]
fn big_font_chooses_token_on_resolution_and_pays_three_distinct_token_names_atomically() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let font = named(&mut g, p, "Transmutation Font", Zone::Battlefield);
        let found = named(&mut g, p, "Solemn Simulacrum", Zone::Library);
        for (card, subtype) in [
            (Card(5346), "Blood"),
            (Card(5296), "Clue"),
            (Card(5246), "Food"),
        ] {
            g.state.objects[font as usize].tapped = false;
            activate(&mut g, font, 0, None);
            assert!(g.state.stack.iter().any(|s| s.card == font));
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::TokenChoice { .. })
            });
            assert_eq!(tokens(&g, p, subtype).len(), 0);
            checked(&mut g, Action::ChooseToken { token: card });
            settle(&mut g);
            assert_eq!(tokens(&g, p, subtype).len(), 1);
        }
        let duplicate = super::tests::spawn(&mut g, p, Card(5246), Zone::Battlefield);
        g.state.objects[font as usize].tapped = false;
        let chosen = [
            tokens(&g, p, "Blood")[0],
            tokens(&g, p, "Clue")[0],
            tokens(&g, p, "Food")[0],
        ];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: font,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        let first = g.object_ref(chosen[2]);
        checked(&mut g, Action::ChooseAdditionalCost { object: first });
        let same = g.object_ref(duplicate);
        reject(&mut g, Action::ChooseAdditionalCost { object: same });
        for id in &chosen[..2] {
            let object = g.object_ref(*id);
            checked(&mut g, Action::ChooseAdditionalCost { object });
        }
        assert!(
            chosen
                .iter()
                .all(|&id| g.object_view(id).zone == Zone::Battlefield)
        );
        until(&mut g, |g| g.state.announcing.is_none());
        assert!(
            chosen
                .iter()
                .all(|&id| g.object_view(id).zone == Zone::Outside)
        );
        assert!(g.object_view(font).tapped);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectSearch { .. })
        });
        assert!(g.observe(p.other()).search_choice.is_none());
        checked(&mut g, Action::ChooseSearch { card: found });
        checked(&mut g, Action::FinishSearch);
        settle(&mut g);
        assert_eq!(g.object_view(found).zone, Zone::Battlefield);
        assert_eq!(g.object_view(duplicate).zone, Zone::Battlefield);
    }
}

#[test]
fn big_font_sorcery_timing_and_funding_cannot_spend_a_reserved_sacrifice_token() {
    let mut g = setup(Player::P0);
    let font = named(&mut g, Player::P0, "Transmutation Font", Zone::Battlefield);
    super::tests::spawn(&mut g, Player::P0, Card(5346), Zone::Battlefield);
    super::tests::spawn(&mut g, Player::P0, Card(5246), Zone::Battlefield);
    let treasure = Card::catalogue()
        .find(|c| c.token_info().is_some_and(|t| t.subtypes == "Treasure"))
        .unwrap();
    super::tests::spawn(&mut g, Player::P0, treasure, Zone::Battlefield);
    named(&mut g, Player::P0, "Forest", Zone::Battlefield);
    named(&mut g, Player::P0, "Island", Zone::Battlefield);
    g.state.players[0].mana = [0; 6];
    assert!(
        !g.legal_actions()
            .iter()
            .any(|a| matches!(a,Action::AnnounceActivate{card,ability:1,..}if *card==font))
    );
    g.state.players[0].mana = [0, 0, 0, 0, 0, 3];
    assert!(
        g.legal_actions()
            .iter()
            .any(|a| matches!(a,Action::AnnounceActivate{card,ability:1,..}if *card==font))
    );
    g.state.step = Step::BeginCombat;
    assert!(
        !g.legal_actions()
            .iter()
            .any(|a| matches!(a,Action::AnnounceActivate{card,ability:1,..}if *card==font))
    );
}
