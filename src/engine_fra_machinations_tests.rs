use super::*;

#[test]
fn machinations_enables_only_jace_loyalty_until_cleanup() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let spell = put(&mut g, player, "Opt", Zone::Hand);
        g.state.objects[spell as usize].card = Card::from_name("Jace's Machinations").unwrap();
        let ajani = put(&mut g, player, "Ajani, Caller of the Pride", Zone::Battlefield);
        act(&mut g, Action::AnnounceCast { card: spell });
        pay(&mut g);
        settle(&mut g);
        let jace = g.state.objects.iter().enumerate().find(|(_, o)|
            o.zone == Zone::Battlefield && o.controller == player && o.card == Card::JacePlaneswalkerToken
        ).map(|(id, _)| id as CardId).unwrap();
        assert_eq!(g.counter_count(jace, CounterKind::Loyalty), 8);
        assert_eq!(g.observe(player.other()).players[player.index()].jace_loyalty_anytime, Some(true));
        g.state.active = player.other();
        g.state.priority = player;
        g.state.step = Step::BeginCombat;
        let jace_action = Action::AnnounceActivate { card: jace, ability: 0, counter_payment: Counters::default() };
        let ajani_action = Action::AnnounceActivate { card: ajani, ability: 0, counter_payment: Counters::default() };
        let saved = g.snapshot();
        for _ in 0..2 {
            reject(&mut g, ajani_action.clone());
            act(&mut g, jace_action.clone());
            pay(&mut g);
            reject(&mut g, jace_action.clone());
            g.restore(&saved).unwrap();
        }
        g.finish_cleanup();
        assert_eq!(g.observe(player).players[player.index()].jace_loyalty_anytime, None);
        reject(&mut g, jace_action);
    }
}
