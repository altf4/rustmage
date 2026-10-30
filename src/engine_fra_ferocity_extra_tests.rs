use super::*;

#[test]
fn ferocity_death_exile_replacement_and_aura_departure() {
    for p in [Player::P0, Player::P1] {
        for aura_leaves in [false, true] {
            let mut g = scenario(p);
            let victim = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let aura = put(&mut g, p, "Ferocity of the Hunt", Zone::Hand);
            cast(&mut g, aura, Some(Target::Card(victim)));
            settle(&mut g);
            assert_eq!(stats(&g, victim), (3, 2));
            assert!(g.keywords(victim).contains(Keywords::DEATHTOUCH));
            if aura_leaves {
                g.move_object(aura, Zone::Exile);
                assert_eq!(stats(&g, victim), (2, 2));
                assert!(!g.keywords(victim).contains(Keywords::DEATHTOUCH));
            } else {
                put(&mut g, p, "Rest in Peace", Zone::Battlefield);
            }
            g.move_object(victim, Zone::Graveyard);
            g.prepare_triggers();
            let saved = g.snapshot();
            super::super::contracts::branch_contract(g.clone());
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&saved).unwrap();
                }
                settle(&mut g);
                assert_eq!(
                    g.object_view(victim).zone,
                    if aura_leaves {
                        Zone::Graveyard
                    } else {
                        Zone::Exile
                    }
                );
            }
        }
    }
}

#[test]
fn ferocity_spell_rejects_noncreatures_and_fizzles_for_stale_target() {
    for p in [Player::P0, Player::P1] {
        for kind in 0..2 {
            let mut g = scenario(p);
            let victim = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let land = put(&mut g, p.other(), "Forest", Zone::Battlefield);
            let aura = put(&mut g, p, "Ferocity of the Hunt", Zone::Hand);
            act(&mut g, Action::AnnounceCast { card: aura });
            if g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseSpellCost { .. }))
            {
                choose(&mut g, |a| matches!(a, Action::ChooseSpellCost { .. }));
            }
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(land),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(victim),
                },
            );
            pay(&mut g);
            if kind == 0 {
                g.move_object(victim, Zone::Exile);
                g.move_object(victim, Zone::Battlefield);
            } else {
                let reference = g.object_ref(victim);
                g.apply_temporary_copy(
                    reference,
                    CopyRecipe::new(Card::from_name("Forest").unwrap()),
                    CopyDeadline::EndOfTurn,
                );
            }
            let saved = g.snapshot();
            super::super::contracts::branch_contract(g.clone());
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&saved).unwrap();
                }
                settle(&mut g);
                assert_eq!(g.object_view(aura).zone, Zone::Graveyard);
                assert!(g.object_view(victim).attached_to.is_none());
                assert!(!g.keywords(victim).contains(Keywords::DEATHTOUCH));
            }
        }
    }
}

#[test]
fn ferocity_returns_animated_noncreature_card_under_its_owner() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let artifact = put(&mut g, p.other(), "Mox Jasper", Zone::Battlefield);
        let reference = g.object_ref(artifact);
        g.change_controller(reference, p);
        g.change_object_form(
            reference,
            AnimationSpec {
                artifact: false,
                creature_types: AnimationCreatureTypes::Unchanged,
                base: Some((2, 2)),
                keywords: Keywords::NONE,
                duration: AnimationDuration::EndOfTurn,
            },
            characteristic_layers::AnimationTypeMode::AddCreature,
        );
        assert!(g.is_creature(artifact));
        let aura = put(&mut g, p, "Ferocity of the Hunt", Zone::Hand);
        cast(&mut g, aura, Some(Target::Card(artifact)));
        settle(&mut g);
        assert_eq!(stats(&g, artifact), (3, 2));
        g.move_object(artifact, Zone::Graveyard);
        g.prepare_triggers();
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            settle(&mut g);
            assert_eq!(g.object_view(artifact).zone, Zone::Battlefield);
            assert_eq!(g.object_view(artifact).controller, p.other());
            assert!(g.object_view(artifact).tapped);
            assert!(!g.is_creature(artifact));
        }
    }
}
