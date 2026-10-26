//! Current Oracle outcome proofs for three inherited complete HOB Adventure pairs.
use super::*;

fn ready(g: &mut Game, predicate: impl Fn(&Game) -> bool) {
    for _ in 0..24 {
        if predicate(g) {
            return;
        }
        assert_eq!(g.state.pending, Pending::Priority);
        act(g, Action::Pass);
    }
    panic!("inherited Adventure resolution budget");
}
fn paid(g: &mut Game, card: CardId, adventure: bool, target: Option<Target>, mana: Mana) {
    g.state.players[g.actor().unwrap().index()].mana = mana;
    act(
        g,
        if adventure {
            Action::AnnounceAdventure { card }
        } else if g.object_view(card).zone == Zone::Exile {
            Action::AnnounceExile { card }
        } else {
            Action::AnnounceCast { card }
        },
    );
    if matches!(g.state.pending, Pending::SpellCost { .. }) {
        choose(g, |a| matches!(a, Action::ChooseSpellCost { .. }));
    }
    if let Some(target) = target {
        act(g, Action::ChooseTarget { target });
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
    let payer = g.state.stack.last().unwrap().controller;
    assert_eq!(g.state.players[payer.index()].mana, [0; 6]);
}
fn no_permission(g: &mut Game, p: Player, card: CardId) {
    assert!(!g.can_play_adventure(p, card));
    assert!(!g.can_play_adventure(p.other(), card));
    reject(g, Action::AnnounceExile { card });
}

#[test]
fn hob_inherited_bilbo_paid_scry_privacy_order_and_same_creature_etb_draw() {
    for p in [Player::P0, Player::P1] {
        for bottom_one in [false, true] {
            let mut g = scenario(p);
            let below = put(&mut g, p, "Forest", Zone::Library);
            let second = put(&mut g, p, "Island", Zone::Library);
            let first = put(&mut g, p, "Grizzly Bears", Zone::Library);
            let bilbo = put(&mut g, p, "Bilbo Baggins, Burglar", Zone::Hand);
            let original = g.object_ref(bilbo);
            paid(&mut g, bilbo, true, None, [0, 1, 0, 0, 0, 0]);
            assert_eq!(
                g.spell_characteristics().last().unwrap().name,
                "Take a Glance"
            );
            ready(&mut g, |g| g.observe(p).library_choice.is_some());
            assert!(g.observe(p.other()).library_choice.is_none());
            assert_eq!(
                g.observe(p)
                    .library_choice
                    .unwrap()
                    .remaining
                    .iter()
                    .map(|v| v.id)
                    .collect::<Vec<_>>(),
                vec![first, second]
            );
            for hidden in [first, second, below] {
                assert!(g.observe(p.other()).objects.iter().all(|v| v.id != hidden));
            }
            reject(
                &mut g,
                Action::ArrangeLibrary {
                    card: below,
                    destination: LibraryDestination::Top,
                },
            );
            if bottom_one {
                act(
                    &mut g,
                    Action::ArrangeLibrary {
                        card: first,
                        destination: LibraryDestination::Bottom,
                    },
                );
                act(
                    &mut g,
                    Action::ArrangeLibrary {
                        card: second,
                        destination: LibraryDestination::Top,
                    },
                );
            } else {
                // The first selected top card will be drawn first: reverse both.
                act(
                    &mut g,
                    Action::ArrangeLibrary {
                        card: second,
                        destination: LibraryDestination::Top,
                    },
                );
                act(
                    &mut g,
                    Action::ArrangeLibrary {
                        card: first,
                        destination: LibraryDestination::Top,
                    },
                );
            }
            settle(&mut g);
            let library = &g.state.players[p.index()].library;
            let expected = second;
            assert_eq!(library.last(), Some(&expected));
            if bottom_one {
                assert_eq!(library.first(), Some(&first));
            } else {
                assert_eq!(library[library.len() - 2], first);
            }
            assert_eq!(g.object_view(bilbo).zone, Zone::Exile);
            assert!(g.can_play_adventure(p, bilbo));
            assert!(!g.can_play_adventure(p.other(), bilbo));
            assert_ne!(g.object_ref(bilbo), original);
            reject(&mut g, Action::AnnounceAdventure { card: bilbo });
            let before_draw = g.state.players[p.index()].cards_drawn_this_turn;
            paid(&mut g, bilbo, false, None, [0, 1, 0, 0, 0, 2]);
            settle(&mut g);
            assert_eq!(g.object_view(bilbo).zone, Zone::Battlefield);
            assert_eq!(stats(&g, bilbo), (2, 1));
            assert_eq!(g.object_view(expected).zone, Zone::Hand);
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                before_draw + 1
            );
            assert!(!g.can_play_adventure(p, bilbo));
        }
    }
}

#[test]
fn hob_inherited_countered_take_a_glance_never_scries_or_grants_permission() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let bilbo = put(&mut g, p, "Bilbo Baggins, Burglar", Zone::Hand);
        let counter = put(&mut g, p.other(), "Negate", Zone::Hand);
        let library = g.state.players[p.index()].library.clone();
        paid(&mut g, bilbo, true, None, [0, 1, 0, 0, 0, 0]);

        act(&mut g, Action::Pass);
        assert_eq!(g.actor(), Some(p.other()));
        paid(
            &mut g,
            counter,
            false,
            Some(Target::Spell(bilbo)),
            [0, 1, 0, 0, 0, 1],
        );
        settle(&mut g);
        assert_eq!(g.object_view(bilbo).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].library, library);
        assert!(g.observe(p).library_choice.is_none());
        no_permission(&mut g, p, bilbo);
    }
}

#[test]
fn hob_inherited_smaug_paid_creature_only_five_damage_then_flying_creature() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let target = put(&mut g, p.other(), "Galewind Moose", Zone::Battlefield);
        let smaug = put(&mut g, p, "Smaug, the Great Calamity", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 4];
        act(&mut g, Action::AnnounceAdventure { card: smaug });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.spell_characteristics().last().unwrap().name, "Spew Flame");
        settle(&mut g);
        assert_eq!(g.object_view(target).damage, 5);
        assert_eq!(g.object_view(target).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.other().index()].life, 20);
        assert!(g.can_play_adventure(p, smaug));
        paid(&mut g, smaug, false, None, [0, 0, 0, 2, 0, 5]);
        settle(&mut g);
        assert_eq!(stats(&g, smaug), (5, 5));
        assert!(g.keywords(smaug).contains(Keywords::FLYING));
        assert_eq!(g.object_view(smaug).zone, Zone::Battlefield);
        assert!(!g.can_play_adventure(p, smaug));
    }
}

#[test]
fn hob_inherited_stale_spew_flame_fizzles_without_damage_or_adventure_right() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let target = put(&mut g, p.other(), "Galewind Moose", Zone::Battlefield);
        let smaug = put(&mut g, p, "Smaug, the Great Calamity", Zone::Hand);
        paid(
            &mut g,
            smaug,
            true,
            Some(Target::Card(target)),
            [0, 0, 0, 1, 0, 4],
        );
        let old = g.object_ref(target);
        g.move_object(target, Zone::Hand);
        g.move_object(target, Zone::Battlefield);
        assert_ne!(g.object_ref(target), old);
        settle(&mut g);
        assert_eq!(g.object_view(target).damage, 0);
        assert_eq!(g.object_view(smaug).zone, Zone::Graveyard);
        no_permission(&mut g, p, smaug);
    }
}

#[test]
fn hob_inherited_thranduil_each_hybrid_face_exact_mill_cardinality_and_owned_choice() {
    for p in [Player::P0, Player::P1] {
        for (count, colors) in [[0, 0, 0, 0, 2, 0], [0, 1, 0, 0, 1, 0], [0, 2, 0, 0, 0, 0]]
            .into_iter()
            .enumerate()
        {
            let mut g = scenario(p);
            let old_land = put(&mut g, p, "Forest", Zone::Graveyard);
            let one = put(&mut g, p, "Forest", Zone::Library);
            let spell = put(&mut g, p, "Shock", Zone::Library);
            let two = put(&mut g, p, "Island", Zone::Library);
            let creature = put(&mut g, p, "Grizzly Bears", Zone::Library);
            let thranduil = put(&mut g, p, "Thranduil, Sindarin Liege", Zone::Hand);
            let mut mana = colors;
            mana[5] = 1;
            paid(&mut g, thranduil, true, None, mana);
            ready(&mut g, |g| g.observe(p).milled_choice.is_some());
            let choice = g.observe(p).milled_choice.unwrap();
            assert_eq!(choice.milled.len(), 4);
            assert_eq!(
                choice
                    .milled
                    .iter()
                    .map(|r| r.card)
                    .collect::<std::collections::BTreeSet<_>>(),
                [one, spell, two, creature].into_iter().collect()
            );
            assert_eq!(
                choice
                    .candidates
                    .iter()
                    .map(|r| r.card)
                    .collect::<std::collections::BTreeSet<_>>(),
                [one, two].into_iter().collect()
            );
            assert_eq!((choice.minimum, choice.maximum), (0, 2));
            assert!(g.observe(p.other()).milled_choice.is_none());
            for id in [old_land, spell, creature] {
                let object = g.object_ref(id);
                reject(&mut g, Action::SelectMilled { object });
            }
            if count == 1 && p == Player::P1 {
                contracts::branch_contract(g.clone());
            }
            for id in [one, two].into_iter().take(count) {
                let object = g.object_ref(id);
                act(&mut g, Action::SelectMilled { object });
                reject(&mut g, Action::SelectMilled { object });
            }
            if g.legal_actions().contains(&Action::FinishMilledSelection) {
                act(&mut g, Action::FinishMilledSelection);
            }
            settle(&mut g);
            for (index, id) in [one, two].into_iter().enumerate() {
                assert_eq!(
                    g.object_view(id).zone,
                    if index < count {
                        Zone::Hand
                    } else {
                        Zone::Graveyard
                    }
                );
            }
            for id in [old_land, spell, creature] {
                assert_eq!(g.object_view(id).zone, Zone::Graveyard);
            }
            assert!(g.can_play_adventure(p, thranduil));
            let mut mana = colors;
            mana[5] = 2;
            paid(&mut g, thranduil, false, None, mana);
            settle(&mut g);
            assert_eq!(stats(&g, thranduil), (2, 3));
            assert_eq!(g.object_view(thranduil).zone, Zone::Battlefield);
            assert!(!g.can_play_adventure(p, thranduil));
        }
    }
}

#[test]
fn hob_inherited_thranduil_other_own_elves_landfall_token_and_source_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let elf = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
        let bear = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let thranduil = put(&mut g, p, "Thranduil, Sindarin Liege", Zone::Hand);
        paid(&mut g, thranduil, false, None, [0, 0, 0, 0, 2, 2]);
        settle(&mut g);
        assert_eq!(stats(&g, thranduil), (2, 3));
        assert_eq!(stats(&g, elf), (2, 2));
        assert_eq!(stats(&g, enemy), (1, 1));
        assert_eq!(stats(&g, bear), (2, 2));
        let land = put(&mut g, p, "Forest", Zone::Hand);
        act(&mut g, Action::PlayLand { card: land });
        settle(&mut g);
        let tokens: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, o)| {
                (o.zone == Zone::Battlefield && o.is_token()).then_some(id as CardId)
            })
            .collect();
        assert_eq!(tokens.len(), 1);
        let token = tokens[0];
        let v = g.object_view(token);
        assert_eq!((v.owner, v.controller, v.colors, v.kinds), (p, p, 16, 1));
        assert!(g.has_subtype(token, "Elf"));
        assert_eq!(stats(&g, token), (2, 2));
        let foreign_land = put(&mut g, p.other(), "Island", Zone::Hand);
        g.move_object(foreign_land, Zone::Battlefield);
        g.prepare_triggers();
        settle(&mut g);
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Battlefield && o.is_token())
                .count(),
            1
        );
        let bounce = put(&mut g, p, "Unsummon", Zone::Hand);
        paid(
            &mut g,
            bounce,
            false,
            Some(Target::Card(thranduil)),
            [0, 1, 0, 0, 0, 0],
        );
        settle(&mut g);
        assert_eq!(g.object_view(thranduil).zone, Zone::Hand);
        assert_eq!(stats(&g, elf), (1, 1));
        assert_eq!(stats(&g, token), (1, 1));
    }
}
