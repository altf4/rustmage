//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

fn token(g: &mut Game, p: Player, card: Card) -> CardId {
    g.create_tokens(p, card, 1, false);
    g.state
        .objects
        .iter()
        .enumerate()
        .find_map(|(i, o)| {
            (o.zone == Zone::Battlefield && o.is_token() && o.card == card).then_some(i as CardId)
        })
        .unwrap()
}

#[test]
fn forest_tentacle_is_a_summoning_sick_green_land_creature() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let id = token(
            &mut g,
            p,
            Card::from_name("@token/fra-forest-tentacle").unwrap(),
        );
        assert_eq!(stats(&g, id), (3, 3));
        assert_eq!(g.object_view(id).colors, 16);
        assert!(g.has_subtype(id, "Forest") && g.has_subtype(id, "Tentacle"));
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::TapMana { card, .. } if *card == id))
        );
        g.state.objects[id as usize].summoning_sick = false;
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(a,
            Action::TapMana { card, .. } if *card == id)
            })
            .expect("Forest mana");
        let before = g.state.players[p.index()].mana[4];
        act(&mut g, action);
        assert_eq!(g.state.players[p.index()].mana[4], before + 1);
    }
}

#[test]
fn leviathan_has_hexproof_and_eight_eight_blue_characteristics() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let id = token(&mut g, p, Card::from_name("@token/fra-leviathan").unwrap());
        assert_eq!(stats(&g, id), (8, 8));
        assert_eq!(g.object_view(id).colors, 2);
        assert!(g.has_subtype(id, "Leviathan"));
        assert!(g.keywords(id).contains(Keywords::HEXPROOF));
        let aura = put(&mut g, p.other(), "Witness Protection", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(id));
        assert!(!g.keywords(id).contains(Keywords::HEXPROOF));
        g.move_object(aura, Zone::Graveyard);
        assert!(g.keywords(id).contains(Keywords::HEXPROOF));
    }
}
