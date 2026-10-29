use super::*;

#[test]
fn machinations_allows_real_way_loyalty_grant_over_opponents_spell() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let machinations = put(&mut g, player, "Opt", Zone::Hand);
        g.state.objects[machinations as usize].card = Card::from_name("Jace's Machinations").unwrap();
        act(&mut g, Action::AnnounceCast { card: machinations });
        pay(&mut g);
        settle(&mut g);
        let jace = g.state.objects.iter().enumerate().find_map(|(id, o)|
            (o.zone == Zone::Battlefield && o.controller == player && o.card == Card::JacePlaneswalkerToken)
                .then_some(id as CardId)).unwrap();
        let way = put(&mut g, player, "Way of the Pyromancer", Zone::Battlefield);
        let grant = Action::AnnouncePlaneswalkerGrant {
            source: g.object_ref(jace), grantor: g.object_ref(way), ability: 0,
        };
        g.state.active = player.other();
        g.state.priority = player.other();
        g.state.step = Step::BeginCombat;
        let shock = put(&mut g, player.other(), "Shock", Zone::Hand);
        cast(&mut g, shock, Some(Target::Player(player)));
        act(&mut g, Action::Pass);
        assert_eq!(g.actor(), Some(player));
        let saved = g.snapshot();
        for _ in 0..2 {
            g.state.players[player.index()].jace_loyalty_anytime = false;
            reject(&mut g, grant.clone());
            g.restore(&saved).unwrap();
            let before_loyalty = g.counter_count(jace, CounterKind::Loyalty);
            let before_red = g.state.players[player.index()].mana[3];
            act(&mut g, grant.clone());
            pay(&mut g);
            assert_eq!(g.counter_count(jace, CounterKind::Loyalty), before_loyalty + 1);
            assert_eq!(g.state.players[player.index()].mana[3], before_red);
            assert!(g.state.stack.iter().any(|item| item.card == shock));
            reject(&mut g, grant.clone());
            settle(&mut g);
            assert_eq!(g.state.players[player.index()].mana[3], before_red + 1);
            g.restore(&saved).unwrap();
        }
    }
}
