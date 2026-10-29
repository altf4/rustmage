use super::*;

fn grapple_cast(g: &mut Game, spell: CardId, victim: CardId, dealer: CardId) {
    act(g, Action::AnnounceCast { card: spell });
    act(g, Action::ChooseTarget { target: Target::Card(victim) });
    act(g, Action::ChooseTarget { target: Target::Card(dealer) });
    pay(g);
}

#[test]
fn grapple_removes_abilities_before_power_damage_and_until_cleanup() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let victim = put(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        let dealer = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Flourishing Grapple", Zone::Hand);
        assert!(g.keywords(victim).contains(Keywords::FLYING));
        grapple_cast(&mut g, spell, victim, dealer);
        super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        assert_eq!(g.state.objects[victim as usize].damage, 2);
        assert!(!g.keywords(victim).contains(Keywords::FLYING));
        assert!(!g.keywords(victim).contains(Keywords::VIGILANCE));
        g.finish_cleanup();
        assert!(g.keywords(victim).contains(Keywords::FLYING));
        assert!(g.keywords(victim).contains(Keywords::VIGILANCE));
    }
}

#[test]
fn grapple_resolves_each_target_independently() {
    for p in [Player::P0, Player::P1] {
        for lose_dealer in [false, true] {
            let mut g = scenario(p);
            let victim = put(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
            let dealer = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let spell = put(&mut g, p, "Flourishing Grapple", Zone::Hand);
            grapple_cast(&mut g, spell, victim, dealer);
            if lose_dealer { g.move_object(dealer, Zone::Exile); }
            else { g.change_controller(g.object_ref(victim), p); }
            settle(&mut g);
            assert_eq!(g.state.objects[victim as usize].damage, 0);
            assert_eq!(g.keywords(victim).contains(Keywords::FLYING), !lose_dealer);
        }
    }
}

#[test]
fn grapple_requires_opponents_red_or_white_creature_or_planeswalker() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let victim = put(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        let green = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let own = put(&mut g, p, "Serra Angel", Zone::Battlefield);
        let spell = put(&mut g, p, "Flourishing Grapple", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: spell });
        reject(&mut g, Action::ChooseTarget { target: Target::Card(green) });
        reject(&mut g, Action::ChooseTarget { target: Target::Card(own) });
        act(&mut g, Action::ChooseTarget { target: Target::Card(victim) });
        reject(&mut g, Action::ChooseTarget { target: Target::Card(green) });
        act(&mut g, Action::ChooseTarget { target: Target::Card(own) });
        pay(&mut g);
    }
}

#[test]
fn grapple_removal_obeys_keyword_timestamps_and_exact_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let victim = put(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        let dealer = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let reference = g.object_ref(victim);
        g.grant_keyword_effects(&[victim], reference, Keywords::TRAMPLE);
        let spell = put(&mut g, p, "Flourishing Grapple", Zone::Hand);
        grapple_cast(&mut g, spell, victim, dealer);
        settle(&mut g);
        assert!(!g.keywords(victim).contains(Keywords::TRAMPLE));
        g.grant_keyword_effects(&[victim], reference, Keywords::LIFELINK);
        assert!(g.keywords(victim).contains(Keywords::LIFELINK));
        assert!(!g.keywords(victim).contains(Keywords::FLYING));
        g.move_object(victim, Zone::Exile);
        g.move_object(victim, Zone::Battlefield);
        assert!(g.keywords(victim).contains(Keywords::FLYING));
        assert!(!g.keywords(victim).contains(Keywords::LIFELINK));
    }
}
