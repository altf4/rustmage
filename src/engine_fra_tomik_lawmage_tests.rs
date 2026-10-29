use super::*;

#[test]
fn tomik_restricts_each_planeswalker_independently_and_tracks_control() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let tomik = put(&mut g, p.other(), "Tomik, Orzhov Lawmage", Zone::Battlefield);
        let one = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let two = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let a = put(&mut g, p.other(), "Ajani, Caller of the Pride", Zone::Battlefield);
        let b = put(&mut g, p.other(), "Kiora of Salt and Sand", Zone::Battlefield);
        let ar = g.object_ref(a); let br = g.object_ref(b);
        g.state.step = Step::Attackers; g.state.pending = Pending::Attackers;
        act(&mut g, Action::AttackPlaneswalker { card: one, defender: ar });
        let forbidden = Action::AttackPlaneswalker { card: two, defender: ar };
        reject(&mut g, forbidden.clone());
        assert!(g.legal_actions().contains(&Action::AttackPlaneswalker { card: two, defender: br }));
        assert!(g.legal_actions().contains(&Action::Attack { card: two }));
        let checkpoint = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for _ in 0..2 {
            g.change_controller(g.object_ref(tomik), p);
            assert!(g.legal_actions().contains(&forbidden));
            g.restore(&checkpoint).unwrap();
            assert!(!g.legal_actions().contains(&forbidden));
        }
    }
}

#[test]
fn tomik_counter_target_rechecked_and_tap_cost_committed() {
    for p in [Player::P0, Player::P1] {
        for remove in [false, true] {
            let mut g = scenario(p);
            let tomik = put(&mut g, p, "Tomik, Orzhov Lawmage", Zone::Battlefield);
            let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let without = put(&mut g, p, "Hill Giant", Zone::Battlefield);
            g.add_counter(target, CounterKind::PlusOne, 1);
            act(&mut g, Action::AnnounceActivate { card: tomik, ability: 0, counter_payment: Counters::default() });
            reject(&mut g, Action::ChooseTarget { target: Target::Card(without) });
            act(&mut g, Action::ChooseTarget { target: Target::Card(target) });
            pay(&mut g);
            assert!(g.object_view(tomik).tapped);
            if remove { g.state.objects[target as usize].counters.plus_one = 0; }
            let saved = g.snapshot();
            for _ in 0..2 {
                settle(&mut g);
                assert_eq!(g.keywords(target).contains(Keywords::FLYING), !remove);
                g.restore(&saved).unwrap();
            }
        }
    }
}

#[test]
fn tomik_competing_mandatory_attackers_maximize_requirements() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p.other(), "Tomik, Orzhov Lawmage", Zone::Battlefield);
        put(&mut g, p.other(), "Storm, Windrider", Zone::Battlefield);
        let walker = put(&mut g, p.other(), "Ajani, Caller of the Pride", Zone::Battlefield);
        let a = put(&mut g, p, "Flamewake Phoenix", Zone::Battlefield);
        let b = put(&mut g, p, "Flamewake Phoenix", Zone::Battlefield);
        let defender = g.object_ref(walker);
        g.state.step = Step::Attackers; g.state.pending = Pending::Attackers;
        assert!(!g.legal_actions().contains(&Action::FinishAttackers));
        act(&mut g, Action::AttackPlaneswalker { card: a, defender });
        assert!(!g.legal_actions().contains(&Action::AttackPlaneswalker { card: b, defender }));
        assert!(g.legal_actions().contains(&Action::FinishAttackers));
        let saved = g.snapshot();
        for _ in 0..2 {
            act(&mut g, Action::FinishAttackers);
            assert_eq!(g.state.attackers.len(), 1);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn tomik_recipient_and_provider_ability_removal_follow_layer_timestamps() {
    for p in [Player::P0, Player::P1] {
        for remove_recipient in [false, true] {
            let mut g = scenario(p);
            let tomik = put(&mut g, p.other(), "Tomik, Orzhov Lawmage", Zone::Battlefield);
            let walker = put(&mut g, p.other(), "Ajani, Caller of the Pride", Zone::Battlefield);
            let recipient = if remove_recipient { walker } else { tomik };
            let aura = put(&mut g, p, "Fresh Start", Zone::Battlefield);
            // Low-level layer fixture: the recipient can be a planeswalker that
            // also has creature type in an ordinary game.
            g.state.objects[aura as usize].attached_to = Some(g.object_ref(recipient));
            assert!(!g.planeswalker_attack_limited(g.object_ref(walker)));
            g.move_object(aura, Zone::Graveyard);
            assert!(g.planeswalker_attack_limited(g.object_ref(walker)));
        }
    }
}

#[test]
fn copied_tomik_grants_to_current_controller_and_disappears_with_copy() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let own = put(&mut g, p, "Ajani, Caller of the Pride", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Kiora of Salt and Sand", Zone::Battlefield);
        g.apply_temporary_copy(g.object_ref(source), CopyRecipe::new(Card::from_name("Tomik, Orzhov Lawmage").unwrap()), CopyDeadline::EndOfTurn);
        assert!(g.planeswalker_attack_limited(g.object_ref(own)));
        assert!(!g.planeswalker_attack_limited(g.object_ref(enemy)));
        let saved = g.snapshot();
        for _ in 0..2 {
            g.change_controller(g.object_ref(source), p.other());
            assert!(!g.planeswalker_attack_limited(g.object_ref(own)));
            assert!(g.planeswalker_attack_limited(g.object_ref(enemy)));
            g.finish_cleanup();
            assert!(!g.planeswalker_attack_limited(g.object_ref(enemy)));
            g.restore(&saved).unwrap();
        }
    }
}
