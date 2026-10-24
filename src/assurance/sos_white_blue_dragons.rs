//! Whole-card dragon mechanics, paid casts and owned continuations.
use super::*;
fn pay(g: &mut Game) {
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    big_helpers::checked(g, action);
}
fn offer(g: &mut Game) {
    for _ in 0..32 {
        if matches!(g.state.pending, Pending::ResolutionCasts { .. }) {
            return;
        }
        big_helpers::checked(g, Action::Pass);
    }
    panic!("cascade did not offer a cast");
}
fn free_card(g: &mut Game, face: CastFace) -> ObjectRef {
    let object = g.state.resolution_free_casts.as_ref().unwrap().remaining[0];
    big_helpers::checked(g, Action::CastExiledDuringResolution { object, face });
    object
}
fn choose_extra(g: &mut Game, id: CardId) {
    let object = g.object_ref(id);
    big_helpers::checked(g, Action::ChooseAdditionalCost { object });
}

#[test]
fn sos_dragon_quandrix_cast_cascade_public_sequence_decline_and_random_bottom() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let untouched = g.state.players[p.index()].library.clone();
        let hit = put(&mut g, p, "Bear Cub", Zone::Library);
        let expensive = put(&mut g, p, "Koma, World-Eater", Zone::Library);
        let land = put(&mut g, p, "Forest", Zone::Library);
        let dragon = put(&mut g, p, "Quandrix, the Proof", Zone::Hand);
        big_helpers::cast(&mut g, dragon, None, &[]);
        offer(&mut g);
        let refs = [
            g.object_ref(land),
            g.object_ref(expensive),
            g.object_ref(hit),
        ];
        let view = g.observe(p.other());
        let cascade = view.cascade.unwrap();
        assert_eq!(cascade.exiled, refs);
        assert_eq!(cascade.maximum_mana_value, 6);
        assert!(refs.iter().all(|r| {
            view.objects
                .iter()
                .any(|o| o.reference == *r && o.zone == Zone::Exile)
        }));
        assert_eq!(g.object_view(dragon).zone, Zone::Stack);
        big_helpers::checked(&mut g, Action::FinishResolutionCasts);
        assert!(g.observe(p).cascade.is_none());
        let library = &g.state.players[p.index()].library;
        assert_eq!(&library[3..], untouched.as_slice());
        let mut bottom = library[..3].to_vec();
        bottom.sort_unstable();
        let mut expected = vec![hit, expensive, land];
        expected.sort_unstable();
        assert_eq!(bottom, expected);
        big_helpers::settle(&mut g);
        assert_eq!(stats(&g, dragon), (6, 6));
        assert!(
            g.keywords(dragon)
                .contains(Keywords::FLYING.union(Keywords::TRAMPLE))
        );
    }
}

#[test]
fn sos_dragon_cascade_survives_countered_source_and_casts_before_original() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let hit = put(&mut g, p, "Bear Cub", Zone::Library);
        let dragon = put(&mut g, p, "Quandrix, the Proof", Zone::Hand);
        big_helpers::cast(&mut g, dragon, None, &[]);
        let counter = put(&mut g, p, "Essence Scatter", Zone::Hand);
        big_helpers::cast(&mut g, counter, None, &[Target::Spell(dragon)]);
        offer(&mut g);
        assert_eq!(g.object_view(dragon).zone, Zone::Graveyard);
        assert_eq!(g.observe(p).cascade.unwrap().maximum_mana_value, 6);
        let before = g.state.players[p.index()].mana;
        let object = free_card(&mut g, CastFace::Normal);
        assert_eq!(object.card, hit);
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].mana, before);
        assert_eq!(g.object_view(hit).zone, Zone::Stack);
        big_helpers::checked(&mut g, Action::FinishResolutionCasts);
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(hit).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 3);
    }
}

#[test]
fn sos_dragon_hand_grant_frozen_x_and_strict_resulting_adventure_mana_value() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let dragon = put(&mut g, p, "Quandrix, the Proof", Zone::Battlefield);
        put(&mut g, p, "Unsummon", Zone::Graveyard);
        let hit = put(&mut g, p, "Aquatic Alchemist", Zone::Library);
        let spell = put(&mut g, p, "Fractalize", Zone::Hand);
        big_helpers::checked(&mut g, Action::AnnounceCast { card: spell });
        big_helpers::checked(&mut g, Action::ChooseX { value: 2 });
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(dragon),
            },
        );
        pay(&mut g);
        offer(&mut g);
        assert_eq!(g.observe(p).cascade.unwrap().maximum_mana_value, 3);
        let object = g.object_ref(hit);
        reject(
            &mut g,
            Action::CastExiledDuringResolution {
                object,
                face: CastFace::Adventure,
            },
        );
        free_card(&mut g, CastFace::Normal);
        pay(&mut g);
        big_helpers::checked(&mut g, Action::FinishResolutionCasts);
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(hit).zone, Zone::Battlefield);
        assert_eq!(stats(&g, dragon), (3, 3));
    }
}

#[test]
fn sos_dragon_granted_cascade_survives_grantor_departure_and_excludes_other_origins() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let dragon = put(&mut g, p, "Quandrix, the Proof", Zone::Battlefield);
        put(&mut g, p, "Group Project", Zone::Library);
        let spell = put(&mut g, p, "Divination", Zone::Hand);
        big_helpers::cast(&mut g, spell, None, &[]);
        big_helpers::checked(&mut g, Action::Pass);
        let bounce = put(&mut g, p.other(), "Unsummon", Zone::Hand);
        big_helpers::cast(&mut g, bounce, None, &[Target::Card(dragon)]);
        offer(&mut g);
        assert_eq!(g.object_view(dragon).zone, Zone::Hand);
        assert_eq!(g.observe(p).cascade.unwrap().maximum_mana_value, 3);
        free_card(&mut g, CastFace::Normal);
        pay(&mut g);
        big_helpers::checked(&mut g, Action::FinishResolutionCasts);
        big_helpers::settle(&mut g);
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Battlefield && o.is_token())
                .count(),
            1
        );
        for zone in [Zone::Graveyard, Zone::Exile] {
            let mut g = scenario(p);
            put(&mut g, p, "Quandrix, the Proof", Zone::Battlefield);
            let spell = put(&mut g, p, "Divination", zone);
            if zone == Zone::Graveyard {
                g.grant_graveyard_permission(p, g.object_ref(spell), GraveyardCastKind::Cast);
            } else {
                g.grant_exile_play_permission(p, g.object_ref(spell), false);
            }
            big_helpers::checked(
                &mut g,
                if zone == Zone::Graveyard {
                    Action::AnnounceCast { card: spell }
                } else {
                    Action::AnnounceExile { card: spell }
                },
            );
            pay(&mut g);
            assert_eq!(g.state.stack.len(), 1);
            big_helpers::settle(&mut g);
            assert!(g.observe(p).cascade.is_none());
        }
    }
}

#[test]
fn sos_dragon_cascade_additional_costs_remain_mandatory_and_transactional() {
    for p in [Player::P0, Player::P1] {
        for enough in [false, true] {
            let mut g = scenario(p);
            let hit = put(&mut g, p, "Soaring Stoneglider", Zone::Library);
            let costs = if enough {
                Some([
                    put(&mut g, p, "Forest", Zone::Graveyard),
                    put(&mut g, p, "Unsummon", Zone::Graveyard),
                ])
            } else {
                None
            };
            g.state.players[p.index()].mana = [0, 1, 0, 0, 1, 4];
            let dragon = put(&mut g, p, "Quandrix, the Proof", Zone::Hand);
            big_helpers::cast(&mut g, dragon, None, &[]);
            offer(&mut g);
            let object = g.object_ref(hit);
            if let Some(costs) = costs {
                free_card(&mut g, CastFace::Normal);
                big_helpers::checked(&mut g, Action::ChooseSpellCost { cost: 0 });
                choose_extra(&mut g, costs[0]);
                let first = g.object_ref(costs[0]);
                reject(&mut g, Action::ChooseAdditionalCost { object: first });
                choose_extra(&mut g, costs[1]);
                pay(&mut g);
                assert!(
                    costs
                        .into_iter()
                        .all(|id| g.object_view(id).zone == Zone::Exile)
                );
            } else {
                reject(
                    &mut g,
                    Action::CastExiledDuringResolution {
                        object,
                        face: CastFace::Normal,
                    },
                );
            }
            big_helpers::checked(&mut g, Action::FinishResolutionCasts);
            big_helpers::settle(&mut g);
            assert_eq!(
                g.object_view(hit).zone,
                if enough {
                    Zone::Battlefield
                } else {
                    Zone::Library
                }
            );
        }
    }
}

#[test]
fn sos_dragon_cascade_no_hit_returns_entire_library_without_failed_draw() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Quandrix, the Proof", Zone::Battlefield);
        put(&mut g, p, "Divination", Zone::Library);
        let before = g.state.players[p.index()].library.len();
        let spell = put(&mut g, p, "Unsummon", Zone::Hand);
        big_helpers::cast(&mut g, spell, None, &[Target::Card(source)]);
        big_helpers::settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), before);
        assert!(!g.state.players[p.index()].attempted_empty_draw);
        assert!(g.observe(p).cascade.is_none());
        assert_eq!(g.object_view(source).zone, Zone::Hand);
    }
}

#[test]
fn sos_dragon_cascade_chained_casts_keep_both_cleanup_continuations_owned() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let grantor = put(&mut g, p, "Quandrix, the Proof", Zone::Battlefield);
        let bear = put(&mut g, p, "Bear Cub", Zone::Library);
        let child = put(&mut g, p, "Quandrix, the Proof", Zone::Library);
        let spell = put(&mut g, p, "Fractalize", Zone::Hand);
        big_helpers::checked(&mut g, Action::AnnounceCast { card: spell });
        big_helpers::checked(&mut g, Action::ChooseX { value: 6 });
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(grantor),
            },
        );
        pay(&mut g);
        big_helpers::checked(&mut g, Action::Pass);
        let bounce = put(&mut g, p.other(), "Unsummon", Zone::Hand);
        big_helpers::cast(&mut g, bounce, None, &[Target::Card(grantor)]);
        offer(&mut g);
        assert_eq!(g.observe(p).cascade.unwrap().maximum_mana_value, 7);
        assert_eq!(free_card(&mut g, CastFace::Normal).card, child);
        pay(&mut g);
        big_helpers::checked(&mut g, Action::FinishResolutionCasts);
        offer(&mut g);
        let cascade = g.observe(p).cascade.unwrap();
        assert_eq!(cascade.maximum_mana_value, 6);
        assert_eq!(cascade.exiled, vec![g.object_ref(bear)]);
        assert_eq!(free_card(&mut g, CastFace::Normal).card, bear);
        pay(&mut g);
        big_helpers::checked(&mut g, Action::FinishResolutionCasts);
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(child).zone, Zone::Battlefield);
        assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
        assert_eq!(g.object_view(grantor).zone, Zone::Hand);
        assert!(g.observe(p).cascade.is_none());
        assert!(g.state.resolution_free_casts.is_none());
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 3);
    }
}
