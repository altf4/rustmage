use super::*;

fn cast_anguish_choose(g: &mut Game, source: CardId, target: Target, x: u32) {
    act(g, Action::AnnounceCast { card: source });
    act(g, Action::ChooseX { value: x });
    pay(g);
    for _ in 0..48 {
        g.prepare_triggers();
        if g.legal_actions().contains(&Action::ChooseTarget { target }) {
            act(g, Action::ChooseTarget { target });
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(g, Action::FinishTargets);
            }
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Anguish entry target not offered");
}

#[test]
fn anguish_queued_entry_damage_keeps_paid_x_after_source_departure_and_target_reentry() {
    for p in [Player::P0, Player::P1] {
        for reentry in [false, true] {
            let mut g = scenario(p);
            let victim = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let source = put(&mut g, p, "Ajani's Anguish", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 2];
            cast_anguish_choose(&mut g, source, Target::Card(victim), 2);
            g.move_object(source, Zone::Graveyard);
            if reentry {
                g.move_object(victim, Zone::Exile);
                g.move_object(victim, Zone::Battlefield);
            }
            let saved = g.snapshot();
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&saved).unwrap();
                }
                settle(&mut g);
                assert_eq!(
                    g.state.objects[victim as usize].damage,
                    if reentry { 0 } else { 2 }
                );
            }
        }
    }
}

#[test]
fn anguish_trample_grant_tracks_source_and_recipient_control_and_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let anguish = put(&mut g, p, "Ajani's Anguish", Zone::Battlefield);
        assert!(g.keywords(own).contains(Keywords::TRAMPLE));
        assert!(!g.keywords(enemy).contains(Keywords::TRAMPLE));
        g.change_controller(g.object_ref(anguish), p.other());
        assert!(!g.keywords(own).contains(Keywords::TRAMPLE));
        assert!(g.keywords(enemy).contains(Keywords::TRAMPLE));
        g.change_controller(g.object_ref(own), p.other());
        assert!(g.keywords(own).contains(Keywords::TRAMPLE));
        g.move_object(anguish, Zone::Exile);
        assert!(!g.keywords(own).contains(Keywords::TRAMPLE));
        assert!(!g.keywords(enemy).contains(Keywords::TRAMPLE));
    }
}
