use super::*;

fn loot_resolve_forge_target(g: &mut Game, target: CardId) {
    let action = Action::ChooseTarget {
        target: Target::Card(target),
    };
    let mut selected = false;
    for _ in 0..100 {
        g.prepare_triggers();
        let actions = g.legal_actions();
        if !selected && actions.contains(&action) {
            act(g, action.clone());
            selected = true;
        } else if actions.contains(&Action::FinishTargets) {
            act(g, Action::FinishTargets);
        } else if selected
            && g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
        {
            return;
        } else if let Some(a) = actions
            .into_iter()
            .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
        {
            act(g, a);
        } else {
            panic!("unexpected Forge choice {:?}", g.state.pending);
        }
    }
    panic!("Forge did not resolve");
}

#[test]
fn loot_borrowed_by_forge_uses_bearers_controller_and_funds_real_cast() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        // A temporary type effect makes Loot a legal artifact target; the real
        // Forge trigger exiles Loot and establishes the borrowed ability.
        let loot = put(&mut g, p.other(), "Loot, the Nexus", Zone::Battlefield);
        let reference = g.object_ref(loot);
        let timestamp = g.next_role_timestamp();
        g.state
            .roles
            .get_or_insert_with(Default::default)
            .animations
            .push(AnimationView {
                object: reference,
                timestamp,
                artifact: true,
                land: false,
                preserve_card_types: true,
                food_only: false,
                artifact_noncreature: false,
                colors: None,
                creature_types: None,
                all_creature_types: false,
                base: None,
                keywords: Keywords::NONE,
                expires_turn: None,
            });
        assert_ne!(g.current_kinds(loot) & 16, 0);
        let forge = put(&mut g, p, "Territory Forge", Zone::Hand);
        cast(&mut g, forge, None);
        loot_resolve_forge_target(&mut g, loot);
        assert_eq!(g.object_view(loot).zone, Zone::Exile);
        let donor = g.object_ref(loot);
        assert!(
            g.borrowed_ability_views()
                .iter()
                .any(|a| a.source.card == forge && a.exiled == donor && a.ability == 0)
        );
        put(&mut g, p, "Bear Cub", Zone::Battlefield);
        put(&mut g, p, "Campus Crier", Zone::Battlefield);
        put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let spell = put(&mut g, p, "Bear Cub", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        let saved = g.snapshot();
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: spell })
        );
        for _ in 0..2 {
            act(&mut g, Action::AnnounceCast { card: spell });
            let borrowed = Action::ActivateBorrowedMana {
                card: forge,
                exiled: donor,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 2, 0],
            };
            act(&mut g, borrowed);
            pay(&mut g);
            settle(&mut g);
            assert_eq!(g.object_view(spell).zone, Zone::Battlefield);
            assert!(g.object_view(forge).tapped);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn loot_projected_sacrifice_recounts_creatures_before_real_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let loot = put(&mut g, p, "Loot, the Nexus", Zone::Battlefield);
        let fodder = put(&mut g, p, "Campus Crier", Zone::Battlefield);
        let ring = put(&mut g, p, "Lotus Ring", Zone::Battlefield);
        let grantor = g.object_ref(ring);
        assert!(g.attach_equipment(grantor, g.object_ref(fodder)));
        g.state.objects[loot as usize].summoning_sick = false;
        g.state.objects[fodder as usize].summoning_sick = false;
        let spell = put(&mut g, p, "Hexhaven Invigorator", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        let before = g.snapshot();
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: spell })
        );
        for _ in 0..2 {
            act(&mut g, Action::AnnounceCast { card: spell });
            act(
                &mut g,
                Action::TapGrantedMana {
                    card: fodder,
                    grantor,
                    production: [0, 0, 0, 0, 3, 0],
                },
            );
            assert_eq!(g.object_view(fodder).zone, Zone::Graveyard);
            assert!(g.legal_actions().contains(&Action::ActivateMana {
                card: loot,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 1, 0]
            }));
            act(
                &mut g,
                Action::ActivateMana {
                    card: loot,
                    ability: 0,
                    payment: [0; 6],
                    counter_payment: Counters::default(),
                    production: [0, 0, 0, 0, 1, 0],
                },
            );
            pay(&mut g);
            settle(&mut g);
            assert_eq!(g.object_view(spell).zone, Zone::Battlefield);
            g.restore(&before).unwrap();
        }
    }
}
