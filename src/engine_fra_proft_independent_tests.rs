use super::*;

#[test]
fn proft_free_cast_still_requires_seven_cards_in_its_casters_graveyard() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Omniscience", Zone::Battlefield);
        let proft = put(&mut g, p, "Proft, Sinister Mastermind", Zone::Hand);
        for _ in 0..6 {
            put(&mut g, p, "Forest", Zone::Graveyard);
        }
        put(&mut g, p.other(), "Forest", Zone::Graveyard);
        let action = Action::AnnounceFree {
            card: proft,
            face: CastFace::Normal,
            flash_surcharge: false,
        };
        reject(&mut g, action.clone());
        put(&mut g, p, "Forest", Zone::Graveyard);
        super::contracts::branch_contract(g.clone());
        act(&mut g, action);
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(proft).zone, Zone::Battlefield);
    }
}

#[test]
fn proft_graveyard_count_excludes_transient_spell_and_card_copies() {
    for p in [Player::P0, Player::P1] {
        for copy_kind in [CopyKind::Spell, CopyKind::Card] {
            let mut g = scenario(p);
            let proft = put(&mut g, p, "Proft, Sinister Mastermind", Zone::Hand);
            for _ in 0..6 {
                put(&mut g, p, "Forest", Zone::Graveyard);
            }
            // Model the transient interval before state-based actions remove the copy.
            let copy = put(&mut g, p, "Forest", Zone::Graveyard);
            g.state.objects[copy as usize].copy_kind = copy_kind;
            let definition = g.state.objects[proft as usize].card;
            assert!(!g.spell_graveyard_requirement_holds(p, proft, definition));
            g.state.objects[copy as usize].copy_kind = CopyKind::Physical;
            assert!(g.spell_graveyard_requirement_holds(p, proft, definition));
        }
    }
}
