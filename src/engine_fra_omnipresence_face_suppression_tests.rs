use super::*;

#[test]
fn omnipresence_uses_the_adventure_faces_mana_value() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Omnipresence", Zone::Battlefield);
        let first = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let second = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let card = put(
            &mut g,
            p,
            "Great Ugly-Looking Goblin // Clap! Snap!",
            Zone::Hand,
        );
        g.state.players[p.index()].mana = [0; 6];
        let adventure = Action::AnnounceFree {
            card,
            face: CastFace::Adventure,
            flash_surcharge: false,
        };
        let front = Action::AnnounceFree {
            card,
            face: CastFace::Normal,
            flash_surcharge: false,
        };
        assert!(g.legal_actions().contains(&adventure));
        assert!(!g.legal_actions().contains(&front));
        let before = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        g.move_object(first, Zone::Graveyard);
        assert!(!g.legal_actions().contains(&adventure));
        g.restore(&before).unwrap();
        act(&mut g, adventure);
        assert_eq!(g.effective_spell_card(card).fixed_mana_value(), 2);
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(card).zone, Zone::Exile);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.object_view(second).zone, Zone::Battlefield);
    }
}

#[test]
fn omnipresence_permission_is_lost_while_blanked_and_restored_afterward() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Omnipresence", Zone::Battlefield);
        put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Shock", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        let free = Action::AnnounceFree {
            card: spell,
            face: CastFace::Normal,
            flash_surcharge: false,
        };
        assert!(g.legal_actions().contains(&free));
        // Animate the enchantment so Fresh Start can legally enchant it.
        let source_ref = g.object_ref(source);
        let timestamp = g.next_role_timestamp();
        g.state
            .roles
            .get_or_insert_with(Default::default)
            .animations
            .push(AnimationView {
                object: source_ref,
                timestamp,
                artifact: false,
                land: false,
                preserve_card_types: false,
                food_only: false,
                artifact_noncreature: false,
                enchantment_noncreature: false,
                colors: None,
                creature_types: None,
                all_creature_types: false,
                base: Some((3, 3)),
                keywords: Keywords::NONE,
                expires_turn: None,
            });
        assert!(g.is_creature(source));
        let aura = put(&mut g, p.other(), "Fresh Start", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(source_ref);
        assert!(!g.legal_actions().contains(&free));
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        g.move_object(aura, Zone::Graveyard);
        assert!(g.legal_actions().contains(&free));
        g.restore(&saved).unwrap();
        assert!(!g.legal_actions().contains(&free));
    }
}

#[test]
fn omnipresence_and_omniscience_offer_one_free_cast_and_survive_either_source_leaving() {
    for p in [Player::P0, Player::P1] {
        for remove_first in [false, true] {
            let mut g = scenario(p);
            let omnipresence = put(&mut g, p, "Omnipresence", Zone::Battlefield);
            let omniscience = put(&mut g, p, "Omniscience", Zone::Battlefield);
            put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let spell = put(&mut g, p, "Shock", Zone::Hand);
            g.state.players[p.index()].mana = [0; 6];
            let free = Action::AnnounceFree {
                card: spell,
                face: CastFace::Normal,
                flash_surcharge: false,
            };
            assert_eq!(g.legal_actions().iter().filter(|a| **a == free).count(), 1);
            let saved = g.snapshot();
            super::super::contracts::branch_contract(g.clone());
            let (first, second) = if remove_first {
                (omnipresence, omniscience)
            } else {
                (omniscience, omnipresence)
            };
            g.move_object(first, Zone::Graveyard);
            assert_eq!(g.legal_actions().iter().filter(|a| **a == free).count(), 1);
            g.move_object(second, Zone::Graveyard);
            assert!(!g.legal_actions().contains(&free));
            g.restore(&saved).unwrap();
            assert_eq!(g.legal_actions().iter().filter(|a| **a == free).count(), 1);
        }
    }
}
