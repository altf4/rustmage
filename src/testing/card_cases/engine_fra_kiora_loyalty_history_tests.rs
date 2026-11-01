//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

fn walker(g: &mut Game, p: Player) -> CardId {
    g.create_tokens(p, Card::JacePlaneswalkerToken, 1, false);
    g.state
        .objects
        .iter()
        .enumerate()
        .find_map(|(id, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && g.has_subtype(id as CardId, "Jace"))
            .then_some(id as CardId)
        })
        .unwrap()
}

#[test]
fn kiora_grants_loyalty_minus_eight_and_creates_hexproof_leviathan() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let kiora = put(&mut g, p, "Kiora of Salt and Sand", Zone::Battlefield);
        let jace = walker(&mut g, p);
        g.state.objects[jace as usize].counters.loyalty = 7;
        let action = Action::AnnouncePlaneswalkerGrant {
            source: g.object_ref(jace),
            grantor: g.object_ref(kiora),
            ability: 0,
        };
        reject(&mut g, action.clone());
        assert!(!g.observe(p.other()).players[p.index()].loyalty_ability_activated_this_turn);
        g.state.objects[jace as usize].counters.loyalty = 9;
        assert!(g.legal_actions().contains(&action));
        act(&mut g, action);
        pay(&mut g);
        assert_eq!(g.counter_count(jace, CounterKind::Loyalty), 1);
        assert!(g.observe(p.other()).players[p.index()].loyalty_ability_activated_this_turn);
        g.move_object(kiora, Zone::Graveyard);
        settle(&mut g);
        let leviathan = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, o)| {
                (o.is_token()
                    && o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.card == Card::from_name("@token/fra-leviathan").unwrap())
                .then_some(id as CardId)
            })
            .unwrap();
        assert_eq!(stats(&g, leviathan), (8, 8));
        assert!(g.keywords(leviathan).contains(Keywords::HEXPROOF));
    }
}
