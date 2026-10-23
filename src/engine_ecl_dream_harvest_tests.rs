//! Actual paid harvests, exact foreign exile permissions and ordinary timing.
use super::ability_timeline_tests::{checked, named, settle, setup};
use super::*;
fn reject(g: &mut Game, a: Action) {
    let b = g.snapshot();
    let r = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), b);
    assert_eq!(g.revision(), r);
}
fn free(card: CardId) -> Action {
    Action::AnnounceFree {
        card,
        face: CastFace::Normal,
        flash_surcharge: false,
    }
}
fn finish(g: &mut Game) {
    for _ in 0..20 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .or_else(|| {
                g.legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::FinishTargets | Action::ChooseSpellCost { .. }))
            })
            .unwrap_or_else(|| panic!("{:?}", g.legal_actions()));
        checked(g, a);
    }
    panic!("announcement");
}
fn paid(g: &mut Game, p: Player, black: u32) -> CardId {
    let id = named(g, p, "Dream Harvest", Zone::Hand);
    let mana = [0, 2 - black, black, 0, 0, 5];
    g.state.players[p.index()].mana = mana;
    checked(g, Action::AnnounceCast { card: id });
    checked(
        g,
        Action::ChooseSpellCost {
            cost: (2 - black) as u16,
        },
    );
    checked(g, Action::PayCosts { payment: mana });
    settle(g);
    g.state.priority = p;
    id
}
#[test]
fn ecl_dream_paid_hybrids_exile_only_opponent_until_total_five_including_zero_value_cards() {
    for p in [Player::P0, Player::P1] {
        for black in 0..=2 {
            let mut g = setup(p);
            for state in &mut g.state.players {
                state.library.clear();
            }
            let own = named(&mut g, p, "Forest", Zone::Library);
            let bottom = named(&mut g, p.other(), "Hill Giant", Zone::Library);
            let a = named(&mut g, p.other(), "Bear Cub", Zone::Library);
            let b = named(&mut g, p.other(), "Bear Cub", Zone::Library);
            let shock = named(&mut g, p.other(), "Shock", Zone::Library);
            let land = named(&mut g, p.other(), "Forest", Zone::Library);
            g.state.players[p.other().index()].library = vec![bottom, a, b, shock, land].into();
            let source = paid(&mut g, p, black);
            assert_eq!(g.state.objects[source as usize].zone, Zone::Graveyard);
            assert_eq!(g.state.players[p.index()].library.as_slice(), &[own]);
            assert_eq!(
                g.state.players[p.other().index()].library.as_slice(),
                &[bottom]
            );
            for id in [a, b, shock, land] {
                assert_eq!(g.state.objects[id as usize].zone, Zone::Exile);
                assert!(g.can_cast_exiled_free(p, id));
                assert!(!g.can_cast_exiled_free(p.other(), id));
            }
            assert!(!g.legal_actions().contains(&Action::PlayLand { card: land }));
            g.state.players[p.index()].mana = [0; 6];
            checked(&mut g, free(a));
            finish(&mut g);
            let item = g.state.stack.last().unwrap();
            assert_eq!(item.cast.unwrap().method, CastMethod::Free);
            assert_eq!(item.cast.unwrap().payment, [0; 6]);
            settle(&mut g);
            assert_eq!(g.state.objects[a as usize].controller, p);
            assert_eq!(g.state.objects[a as usize].owner, p.other());
        }
    }
}
#[test]
fn ecl_dream_empty_and_short_library_exhaust_without_draw_or_invented_cards() {
    for p in [Player::P0, Player::P1] {
        for amount in 0..=3 {
            let mut g = setup(p);
            for state in &mut g.state.players {
                state.library.clear();
            }
            let cards: Vec<_> = (0..amount)
                .map(|_| named(&mut g, p.other(), "Forest", Zone::Library))
                .collect();
            paid(&mut g, p, 2);
            assert!(g.state.players[p.other().index()].library.is_empty());
            assert!(g.outcome().is_none());
            assert_eq!(g.state.exile_permissions.len(), amount);
            for id in cards {
                assert_eq!(g.state.objects[id as usize].zone, Zone::Exile);
            }
        }
    }
}
#[test]
fn ecl_dream_permissions_are_exact_expire_at_cleanup_and_preserve_independent_paid_routes() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        for state in &mut g.state.players {
            state.library.clear();
        }
        let creature = named(&mut g, p.other(), "Hill Giant", Zone::Library);
        let spell = named(&mut g, p.other(), "Shock", Zone::Library);
        paid(&mut g, p, 1);
        g.state.players[p.index()].mana = [30; 6];
        reject(&mut g, Action::AnnounceExile { card: spell });
        g.grant_exile_cast_permission(p, g.object_ref(spell));
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceExile { card: spell })
        );
        assert!(g.legal_actions().contains(&free(spell)));
        g.move_object(spell, Zone::Hand);
        g.move_object(spell, Zone::Exile);
        assert!(!g.can_cast_exiled_free(p, spell));
        assert!(!g.can_cast_exiled(p, spell));
        assert!(g.can_cast_exiled_free(p, creature));
        g.finish_cleanup();
        assert!(!g.can_cast_exiled_free(p, creature));
        assert!(!g.legal_actions().contains(&free(creature)));
    }
}
#[test]
fn ecl_dream_normal_timing_prohibitions_and_instant_opponent_turn_cast() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        for state in &mut g.state.players {
            state.library.clear();
        }
        let creature = named(&mut g, p.other(), "Hill Giant", Zone::Library);
        let shock = named(&mut g, p.other(), "Shock", Zone::Library);
        paid(&mut g, p, 2);
        g.state.active = p.other();
        g.state.priority = p;
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, free(creature));
        g.state.casting_prohibited = true;
        reject(&mut g, free(shock));
        g.state.casting_prohibited = false;
        checked(&mut g, free(shock));
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}
#[test]
fn ecl_dream_mandatory_discard_is_paid_by_caster_and_cannot_be_bypassed_for_free() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        for state in &mut g.state.players {
            state.library.clear();
        }
        named(&mut g, p.other(), "Hill Giant", Zone::Library);
        let thrill = named(&mut g, p.other(), "Thrill of Possibility", Zone::Library);
        paid(&mut g, p, 2);
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, free(thrill));
        let own = named(&mut g, p, "Forest", Zone::Hand);
        let foreign = named(&mut g, p.other(), "Forest", Zone::Hand);
        checked(&mut g, free(thrill));
        assert!(
            !g.legal_actions()
                .contains(&Action::PayCosts { payment: [0; 6] })
        );
        let object = g.object_ref(foreign);
        reject(&mut g, Action::ChooseAdditionalCost { object });
        let object = g.object_ref(own);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        finish(&mut g);
        assert_eq!(g.state.objects[own as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.stack.last().unwrap().cast.unwrap().payment, [0; 6]);
        // Avoid unrelated empty-library draws while verifying the actual resolving spell.
        named(&mut g, p, "Forest", Zone::Library);
        named(&mut g, p, "Forest", Zone::Library);
        settle(&mut g);
        assert_eq!(g.state.objects[thrill as usize].zone, Zone::Graveyard);
    }
}
#[test]
fn ecl_dream_alternate_spell_faces_and_free_x_zero_use_printed_spell_rules() {
    for p in [Player::P0, Player::P1] {
        for (name, face) in [
            (
                "Bilbo Baggins, Burglar // Take a Glance",
                CastFace::Adventure,
            ),
            ("Whirlwing Stormbrood // Dynamic Soar", CastFace::Omen),
        ] {
            let mut g = setup(p);
            for state in &mut g.state.players {
                state.library.clear();
            }
            let other = named(&mut g, p.other(), "Hill Giant", Zone::Library);
            let id = named(&mut g, p.other(), name, Zone::Library);
            paid(&mut g, p, 2);
            g.state.players[p.index()].mana = [0; 6];
            let target = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let action = Action::AnnounceFree {
                card: id,
                face,
                flash_surcharge: false,
            };
            checked(&mut g, action);
            if face == CastFace::Omen {
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(target),
                    },
                );
            }
            finish(&mut g);
            assert_eq!(g.state.stack.last().unwrap().cast.unwrap().face, face);
            settle(&mut g);
            if face == CastFace::Adventure {
                assert_eq!(g.state.objects[id as usize].zone, Zone::Exile);
                assert!(g.can_play_adventure(p, id));
            } else {
                assert_eq!(g.state.objects[id as usize].zone, Zone::Library);
            }
            let _ = other;
        }
        let mut g = setup(p);
        for state in &mut g.state.players {
            state.library.clear();
        }
        named(&mut g, p.other(), "Hill Giant", Zone::Library);
        let x = named(&mut g, p.other(), "Exsanguinate", Zone::Library);
        paid(&mut g, p, 2);
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, free(x));
        finish(&mut g);
        assert_eq!(g.state.stack.last().unwrap().x, Some(0));
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 20);
    }
}
