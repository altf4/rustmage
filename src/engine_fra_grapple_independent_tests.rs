use super::*;

fn independent_grapple(g: &mut Game, spell: CardId, victim: CardId, dealer: CardId) {
    act(g, Action::AnnounceCast { card: spell });
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(victim),
        },
    );
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(dealer),
        },
    );
    pay(g);
}

#[test]
fn grapple_blanks_mana_funding_until_cleanup_even_after_control_change() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let victim = put(&mut g, p.other(), "Ruby, Daring Tracker", Zone::Battlefield);
        let dealer = put(&mut g, p, "Aegis Turtle", Zone::Battlefield);
        let spell = put(&mut g, p, "Flourishing Grapple", Zone::Hand);
        independent_grapple(&mut g, spell, victim, dealer);
        settle(&mut g);
        assert!(!g.keywords(victim).contains(Keywords::HASTE));
        g.change_controller(g.object_ref(victim), p);
        g.state.objects[victim as usize].summoning_sick = false;
        g.state.players[p.index()].mana = [0; 6];
        let goal = funding::FundingGoal {
            actor: p,
            required: [0, 0, 0, 0, 1, 0],
            generic: 0,
            reserve: None,
            source: None,
        };
        assert!(!g.can_fund(goal.clone()));
        let snapshot = g.snapshot();
        g.finish_cleanup();
        assert!(g.keywords(victim).contains(Keywords::HASTE));
        assert!(g.can_fund(goal.clone()));
        g.restore(&snapshot).unwrap();
        assert!(!g.can_fund(goal));
        assert!(!g.keywords(victim).contains(Keywords::HASTE));
    }
}

#[test]
fn grapple_both_targets_illegal_does_not_create_removal() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let victim = put(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        let dealer = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Flourishing Grapple", Zone::Hand);
        independent_grapple(&mut g, spell, victim, dealer);
        g.change_controller(g.object_ref(victim), p);
        g.move_object(dealer, Zone::Exile);
        settle(&mut g);
        assert!(g.keywords(victim).contains(Keywords::FLYING));
        assert_eq!(g.state.objects[victim as usize].damage, 0);
    }
}
