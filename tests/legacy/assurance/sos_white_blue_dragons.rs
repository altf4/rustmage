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

fn first_draw(g: &mut Game) {
    for _ in 0..32 {
        if matches!(g.state.pending, Pending::MiracleReveal { .. }) {
            return;
        }
        big_helpers::checked(g, Action::Pass);
    }
    panic!("no first-draw decision {:?}", g.state.pending);
}
fn lorehold(g: &mut Game, p: Player) -> CardId {
    let dragon = put(g, p, "Lorehold, the Historian", Zone::Hand);
    big_helpers::cast(g, dragon, None, &[]);
    big_helpers::settle(g);
    assert_eq!(stats(g, dragon), (5, 5));
    assert!(
        g.keywords(dragon)
            .contains(Keywords::FLYING.union(Keywords::HASTE))
    );
    dragon
}
fn draw_spell(g: &mut Game, p: Player, name: &str) {
    let spell = put(g, p, name, Zone::Hand);
    big_helpers::cast(g, spell, None, &[]);
    first_draw(g);
}
fn reveal_offer(g: &mut Game) {
    big_helpers::checked(g, Action::RevealMiracle);
    offer(g);
    let w = g.state.resolution_free_casts.as_ref().unwrap();
    assert_eq!(w.origin, Zone::Hand);
    assert_eq!(
        w.alternative_cost,
        Some(ManaCost {
            colored: [0; 6],
            generic: 2
        })
    );
}
#[test]
fn sos_dragon_miracle_first_draw_private_pause_finishes_draws_then_paid_cast() {
    for p in [Player::P0, Player::P1] {
        let mut left = scenario(p);
        lorehold(&mut left, p);
        let land = put(&mut left, p, "Forest", Zone::Library);
        let spell = put(&mut left, p, "Group Project", Zone::Library);
        let mut right = left.clone();
        let n = right.state.players[p.index()].library.len();
        right.state.players[p.index()].library.swap(n - 1, n - 2);
        draw_spell(&mut left, p, "Divination");
        draw_spell(&mut right, p, "Divination");
        assert_eq!(left.state.players[p.index()].cards_drawn_this_turn, 1);
        assert_eq!(right.state.players[p.index()].cards_drawn_this_turn, 1);
        assert_eq!(
            serde_json::to_value(left.observe(p.other())).unwrap(),
            serde_json::to_value(right.observe(p.other())).unwrap()
        );
        assert!(left.observe(p.other()).miracle.is_none());
        assert_eq!(left.observe(p).miracle.unwrap().object.card, spell);
        assert!(left.observe(p).miracle.unwrap().can_reveal);
        assert_eq!(right.observe(p).miracle.unwrap().object.card, land);
        assert!(!right.observe(p).miracle.unwrap().can_reveal);
        reject(&mut right, Action::RevealMiracle);
        big_helpers::checked(&mut right, Action::DeclineMiracle);
        assert_eq!(right.state.players[p.index()].cards_drawn_this_turn, 2);
        assert!(right.observe(p).miracle.is_none());
        assert_eq!(right.object_view(spell).zone, Zone::Hand);
        big_helpers::settle(&mut right);
        reveal_offer(&mut left);
        assert_eq!(left.state.players[p.index()].cards_drawn_this_turn, 2);
        assert!(left.state.known_hand.contains(&left.object_ref(spell)));
        left.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        let object = free_card(&mut left, CastFace::Normal);
        assert_eq!(object.card, spell);
        pay(&mut left);
        let cast = left.state.stack.last().unwrap().cast.unwrap();
        assert_eq!(cast.method, CastMethod::Miracle);
        assert_eq!(cast.origin, Zone::Hand);
        assert_eq!(cast.mana_spent(), 2);
        big_helpers::checked(&mut left, Action::FinishResolutionCasts);
        big_helpers::settle(&mut left);
        assert!(
            left.state
                .objects
                .iter()
                .any(|o| o.zone == Zone::Battlefield && o.is_token())
        );
    }
}
#[test]
fn sos_dragon_miracle_source_departure_exact_hand_and_optional_window() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let dragon = lorehold(&mut g, p);
        let spell = put(&mut g, p, "Group Project", Zone::Library);
        draw_spell(&mut g, p, "Divination");
        big_helpers::checked(&mut g, Action::RevealMiracle);
        let bounce = put(&mut g, p, "Unsummon", Zone::Hand);
        big_helpers::cast(&mut g, bounce, None, &[Target::Card(dragon)]);
        offer(&mut g);
        assert_eq!(g.object_view(dragon).zone, Zone::Hand);
        assert_eq!(
            g.state.resolution_free_casts.as_ref().unwrap().remaining[0],
            g.object_ref(spell)
        );
        big_helpers::checked(&mut g, Action::FinishResolutionCasts);
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Hand);
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::CastExiledDuringResolution { .. }))
        );

        let mut g = scenario(p);
        lorehold(&mut g, p);
        let spell = put(&mut g, p, "Group Project", Zone::Library);
        draw_spell(&mut g, p, "Chart a Course");
        big_helpers::checked(&mut g, Action::RevealMiracle);
        assert!(matches!(g.state.pending, Pending::EffectDiscard { .. }));
        big_helpers::checked(&mut g, Action::Discard { card: spell });
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        assert!(g.state.resolution_free_casts.is_none());
    }
}
#[test]
fn sos_dragon_lorehold_optional_opponent_upkeep_discard_miracle_sorcery() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        lorehold(&mut g, p);
        g.begin_step(Step::Upkeep);
        g.prepare_triggers();
        assert!(g.state.stack.is_empty());
        let discard = put(&mut g, p, "Forest", Zone::Hand);
        g.state.active = p.other();
        g.begin_step(Step::Upkeep);
        g.give_priority();
        for _ in 0..16 {
            if matches!(g.state.pending, Pending::EffectChoice { .. }) {
                break;
            }
            big_helpers::checked(&mut g, Action::Pass);
        }
        let mut decline = g.clone();
        big_helpers::checked(&mut decline, Action::DeclineEffectPayment);
        big_helpers::settle(&mut decline);
        assert_eq!(decline.state.players[p.index()].cards_drawn_this_turn, 0);
        let enemy = put(&mut g, p.other(), "Forest", Zone::Hand);
        let spell = put(&mut g, p, "Group Project", Zone::Library);
        reject(&mut g, Action::Discard { card: enemy });
        big_helpers::checked(&mut g, Action::Discard { card: discard });
        assert_eq!(g.object_view(discard).zone, Zone::Graveyard);
        reveal_offer(&mut g);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        free_card(&mut g, CastFace::Normal);
        pay(&mut g);
        assert_eq!(g.state.active, p.other());
        assert_eq!(g.object_view(spell).zone, Zone::Stack);
        big_helpers::checked(&mut g, Action::FinishResolutionCasts);
        big_helpers::settle(&mut g);
    }
}
#[test]
fn sos_dragon_miracle_mandatory_additional_cost_or_mana_alternative() {
    for p in [Player::P0, Player::P1] {
        for mana in [false, true] {
            let mut g = scenario(p);
            lorehold(&mut g, p);
            let sacrifice = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let spell = put(&mut g, p, "Eaten Alive", Zone::Library);
            draw_spell(&mut g, p, "Divination");
            reveal_offer(&mut g);
            g.state.players[p.index()].mana = if mana {
                [0, 0, 1, 0, 0, 5]
            } else {
                [0, 0, 0, 0, 0, 2]
            };
            free_card(&mut g, CastFace::Normal);
            if mana {
                big_helpers::checked(&mut g, Action::ChooseSpellCost { cost: 1 });
            } else {
                big_helpers::checked(&mut g, Action::ChooseSpellCost { cost: 0 });
            }
            big_helpers::checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            if !mana {
                choose_extra(&mut g, sacrifice);
            }
            pay(&mut g);
            assert_eq!(g.state.stack.last().unwrap().source.unwrap().id, spell);
            big_helpers::checked(&mut g, Action::FinishResolutionCasts);
            big_helpers::settle(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Exile);
            assert_eq!(
                g.object_view(sacrifice).zone,
                if mana {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
        }
    }
}
#[test]
fn sos_dragon_miracle_x_zero_targets_and_natural_draw_step() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        lorehold(&mut g, p);
        let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Fractalize", Zone::Library);
        g.state.step = Step::Upkeep;
        g.state.priority = p;
        g.state.passes = 0;
        first_draw(&mut g);
        reveal_offer(&mut g);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        free_card(&mut g, CastFace::Normal);
        assert_eq!(g.state.announcing.as_ref().unwrap().item.x, Some(0));
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        pay(&mut g);
        big_helpers::checked(&mut g, Action::FinishResolutionCasts);
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        assert_eq!(stats(&g, target), (1, 1));
    }
}

#[test]
fn sos_dragon_miracle_ability_loss_first_draw_only_and_observed_policy() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let dragon = lorehold(&mut g, p);
        let aura = put(&mut g, p, "Witness Protection", Zone::Hand);
        big_helpers::cast(&mut g, aura, None, &[Target::Card(dragon)]);
        big_helpers::settle(&mut g);
        put(&mut g, p, "Group Project", Zone::Library);
        let draw = put(&mut g, p, "Divination", Zone::Hand);
        big_helpers::cast(&mut g, draw, None, &[]);
        big_helpers::settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
        assert!(g.observe(p).miracle.is_none());

        let mut g = scenario(p);
        lorehold(&mut g, p);
        put(&mut g, p, "Group Project", Zone::Library);
        draw_spell(&mut g, p, "Divination");
        let actions = g.legal_actions();
        let selected = crate::policy::smoke_index(&g, &actions);
        assert_eq!(actions[selected], Action::RevealMiracle);
        big_helpers::checked(&mut g, Action::DeclineMiracle);
        big_helpers::settle(&mut g);
        let second = put(&mut g, p, "Group Project", Zone::Library);
        let draw = put(&mut g, p, "Divination", Zone::Hand);
        big_helpers::cast(&mut g, draw, None, &[]);
        big_helpers::settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 4);
        assert_eq!(g.object_view(second).zone, Zone::Hand);
        assert!(g.observe(p).miracle.is_none());
    }
}

#[test]
fn bug_hunt_multiple_lorehold_miracle_instances_can_be_revealed_independently() {
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Lorehold, the Historian", Zone::Battlefield);
    let copy_spell = put(&mut g, p, "Multiversal Incursion", Zone::Hand);
    big_helpers::cast(&mut g, copy_spell, None, &[]);
    big_helpers::settle(&mut g);
    let dragons = g
        .state
        .objects
        .iter()
        .filter(|o| {
            o.zone == Zone::Battlefield
                && o.controller == p
                && o.card.name() == "Lorehold, the Historian"
        })
        .count();
    assert_eq!(dragons, 2);
    let drawn = put(&mut g, p, "Shock", Zone::Library);
    draw_spell(&mut g, p, "Divination");
    assert_eq!(g.observe(p).miracle.unwrap().object.card, drawn);
    big_helpers::checked(&mut g, Action::RevealMiracle);
    assert!(
        matches!(g.state.pending, Pending::MiracleReveal { .. }),
        "second miracle instance should offer its own reveal: {:?}",
        g.state.pending
    );
}

fn two_lorehold_miracle_draw(p: Player) -> (Game, CardId, CardId) {
    let mut g = scenario(p);
    let original = put(&mut g, p, "Lorehold, the Historian", Zone::Battlefield);
    let copy_spell = put(&mut g, p, "Multiversal Incursion", Zone::Hand);
    big_helpers::cast(&mut g, copy_spell, None, &[]);
    big_helpers::settle(&mut g);
    let second_draw = put(&mut g, p, "Forest", Zone::Library);
    let first = put(&mut g, p, "Group Project", Zone::Library);
    draw_spell(&mut g, p, "Divination");
    assert_eq!(g.observe(p).miracle.unwrap().object.card, first);
    (g, original, second_draw)
}

#[test]
fn multiple_miracle_choices_are_independent_and_defer_draws() {
    for p in [Player::P0, Player::P1] {
        for first_reveal in [false, true] {
            for second_reveal in [false, true] {
                let (mut g, _, second_draw) = two_lorehold_miracle_draw(p);
                let first = g.observe(p).miracle.unwrap().object;
                contracts::branch_contract(g.clone());
                big_helpers::checked(
                    &mut g,
                    if first_reveal {
                        Action::RevealMiracle
                    } else {
                        Action::DeclineMiracle
                    },
                );
                assert!(matches!(g.state.pending, Pending::MiracleReveal { .. }));
                assert_eq!(g.observe(p).miracle.unwrap().object, first);
                assert!(g.observe(p.other()).miracle.is_none());
                assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
                assert_eq!(g.state.objects[second_draw as usize].zone, Zone::Library);
                assert_eq!(g.state.pending_triggers.len(), usize::from(first_reveal));
                contracts::branch_contract(g.clone());
                big_helpers::checked(
                    &mut g,
                    if second_reveal {
                        Action::RevealMiracle
                    } else {
                        Action::DeclineMiracle
                    },
                );
                assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
                assert_eq!(g.state.objects[second_draw as usize].zone, Zone::Hand);
                let triggers: Vec<_> = g
                    .state
                    .stack
                    .iter()
                    .chain(&g.state.pending_triggers)
                    .chain(&g.state.ordering_triggers)
                    .filter(|item| {
                        item.attachment_trigger.as_ref().is_some_and(|grant| {
                            grant.recipient == first
                                && grant.granting_card.name() == "Lorehold, the Historian"
                        })
                    })
                    .collect();
                assert_eq!(
                    triggers.len(),
                    usize::from(first_reveal) + usize::from(second_reveal)
                );
                if triggers.len() == 2 {
                    assert_ne!(triggers[0].id, triggers[1].id);
                    assert_ne!(
                        triggers[0].attachment_trigger.as_ref().unwrap().grantor,
                        triggers[1].attachment_trigger.as_ref().unwrap().grantor
                    );
                }
            }
        }
    }
}

#[test]
fn multiple_miracle_hidden_ineligible_draw_has_equal_private_decisions() {
    for p in [Player::P0, Player::P1] {
        let mut left = scenario(p);
        put(&mut left, p, "Lorehold, the Historian", Zone::Battlefield);
        let copy_spell = put(&mut left, p, "Multiversal Incursion", Zone::Hand);
        big_helpers::cast(&mut left, copy_spell, None, &[]);
        big_helpers::settle(&mut left);
        put(&mut left, p, "Forest", Zone::Library);
        put(&mut left, p, "Shock", Zone::Library);
        let mut right = left.clone();
        let n = right.state.players[p.index()].library.len();
        right.state.players[p.index()].library.swap(n - 1, n - 2);
        draw_spell(&mut left, p, "Divination");
        draw_spell(&mut right, p, "Divination");
        for choice in 0..2 {
            assert_eq!(
                serde_json::to_value(left.observe(p.other())).unwrap(),
                serde_json::to_value(right.observe(p.other())).unwrap()
            );
            assert!(left.observe(p).miracle.unwrap().can_reveal);
            assert!(!right.observe(p).miracle.unwrap().can_reveal);
            reject(&mut right, Action::RevealMiracle);
            big_helpers::checked(&mut left, Action::DeclineMiracle);
            big_helpers::checked(&mut right, Action::DeclineMiracle);
            if choice == 0 {
                assert_eq!(left.state.players[p.index()].cards_drawn_this_turn, 1);
                assert_eq!(right.state.players[p.index()].cards_drawn_this_turn, 1);
                assert!(matches!(left.state.pending, Pending::MiracleReveal { .. }));
                assert!(matches!(right.state.pending, Pending::MiracleReveal { .. }));
            }
        }
        assert_eq!(
            serde_json::to_value(left.observe(p.other())).unwrap(),
            serde_json::to_value(right.observe(p.other())).unwrap()
        );
        assert_eq!(left.state.players[p.index()].cards_drawn_this_turn, 2);
        assert_eq!(right.state.players[p.index()].cards_drawn_this_turn, 2);
    }
}

#[test]
fn multiple_miracle_triggers_survive_providers_and_offer_two_casts() {
    for p in [Player::P0, Player::P1] {
        let (mut g, original, _) = two_lorehold_miracle_draw(p);
        let drawn = g.observe(p).miracle.unwrap().object;
        big_helpers::checked(&mut g, Action::RevealMiracle);
        big_helpers::checked(&mut g, Action::RevealMiracle);
        g.prepare_triggers();
        while matches!(g.state.pending, Pending::OrderTriggers { .. }) {
            choose(&mut g, |action| matches!(action, Action::PutTrigger { .. }));
        }
        assert_eq!(
            g.state
                .stack
                .iter()
                .filter(|item| item
                    .attachment_trigger
                    .as_ref()
                    .is_some_and(|grant| grant.recipient == drawn))
                .count(),
            2
        );
        let providers: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, object)| {
                (object.zone == Zone::Battlefield
                    && object.card.name() == "Lorehold, the Historian")
                    .then_some(id as CardId)
            })
            .collect();
        assert_eq!(providers.len(), 2);
        assert!(providers.contains(&original));
        for provider in providers {
            g.state.players[p.index()].mana = [8; 6];
            let bounce = put(&mut g, p, "Unsummon", Zone::Hand);
            big_helpers::cast(&mut g, bounce, None, &[Target::Card(provider)]);
            for _ in 0..8 {
                if !g.on_battlefield(provider) {
                    break;
                }
                big_helpers::checked(&mut g, Action::Pass);
            }
            assert!(!g.on_battlefield(provider));
        }
        offer(&mut g);
        assert_eq!(
            g.state.resolution_free_casts.as_ref().unwrap().remaining[0],
            drawn
        );
        big_helpers::checked(&mut g, Action::FinishResolutionCasts);
        offer(&mut g);
        assert_eq!(
            g.state.resolution_free_casts.as_ref().unwrap().remaining[0],
            drawn
        );
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        assert_eq!(free_card(&mut g, CastFace::Normal), drawn);
        pay(&mut g);
        assert_eq!(g.object_view(drawn.card).zone, Zone::Stack);
        big_helpers::checked(&mut g, Action::FinishResolutionCasts);
        big_helpers::settle(&mut g);
    }
}
