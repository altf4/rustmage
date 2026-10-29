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

#[test]
fn machinations_allows_jace_loyalty_in_response_to_an_opponents_spell() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let spell = put(&mut g, player, "Opt", Zone::Hand);
        g.state.objects[spell as usize].card = Card::from_name("Jace's Machinations").unwrap();
        act(&mut g, Action::AnnounceCast { card: spell });
        pay(&mut g);
        settle(&mut g);
        let jace = g.state.objects.iter().enumerate().find(|(_, o)| o.zone == Zone::Battlefield
            && o.controller == player && o.card == Card::JacePlaneswalkerToken).unwrap().0 as CardId;
        g.state.active = player.other();
        g.state.priority = player.other();
        let shock = put(&mut g, player.other(), "Shock", Zone::Hand);
        cast(&mut g, shock, Some(Target::Player(player)));
        act(&mut g, Action::Pass);
        let saved = g.snapshot();
        for _ in 0..2 {
            assert!(g.state.stack.iter().any(|item| item.card == shock));
            let action = Action::AnnounceActivate { card: jace, ability: 1, counter_payment: Counters::default() };
            act(&mut g, action.clone());
            pay(&mut g);
            assert!(g.state.stack.iter().any(|item| item.card == shock));
            reject(&mut g, action);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn machinations_copied_spell_empowers_twice_and_grants_timing() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let spell = put(&mut g, player, "Opt", Zone::Hand);
        g.state.objects[spell as usize].card = Card::from_name("Jace's Machinations").unwrap();
        act(&mut g, Action::AnnounceCast { card: spell });
        pay(&mut g);
        let sparks = put(&mut g, player, "Choreographed Sparks", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: sparks });
        act(&mut g, Action::ChooseMode { mode: 0 });
        act(&mut g, Action::ChooseTarget { target: Target::Spell(spell) });
        pay(&mut g);
        let saved = g.snapshot();
        for _ in 0..2 {
            settle(&mut g);
            let jace = g.state.objects.iter().enumerate().find(|(_, o)| o.zone == Zone::Battlefield
                && o.controller == player && o.card == Card::JacePlaneswalkerToken).unwrap().0 as CardId;
            assert_eq!(g.counter_count(jace, CounterKind::Loyalty), 16);
            assert_eq!(g.observe(player.other()).players[player.index()].jace_loyalty_anytime, Some(true));
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn machinations_timing_follows_copied_jace_and_controller() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let spell = put(&mut g, player, "Opt", Zone::Hand);
        g.state.objects[spell as usize].card = Card::from_name("Jace's Machinations").unwrap();
        act(&mut g, Action::AnnounceCast { card: spell });
        pay(&mut g);
        settle(&mut g);
        let source = put(&mut g, player, "Ajani, Caller of the Pride", Zone::Battlefield);
        g.state.objects[source as usize].counters.loyalty = 4;
        let reference = g.object_ref(source);
        g.apply_temporary_copy(reference, CopyRecipe::new(Card::JacePlaneswalkerToken), CopyDeadline::EndOfTurn);
        assert!(g.has_subtype(source, "Jace"));
        g.state.active = player.other();
        g.state.priority = player;
        g.state.step = Step::BeginCombat;
        let action = Action::AnnounceActivate { card: source, ability: 1, counter_payment: Counters::default() };
        let saved = g.snapshot();
        for _ in 0..2 {
            act(&mut g, action.clone());
            pay(&mut g);
            g.restore(&saved).unwrap();
        }
        g.change_controller(reference, player.other());
        reject(&mut g, action);
    }
}
