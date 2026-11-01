//! Lunar Convocation's intervening-if triggers and paid draw.
use super::*;

#[test]
fn lunar_end_step_requires_gain_and_loss_at_trigger_time() {
    for p in [Player::P0, Player::P1] {
        for (gain, loss) in [(false, false), (false, true), (true, false), (true, true)] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Lunar Convocation", Zone::Hand);
            big_helpers::cast(&mut g, source, None, &[]);
            big_helpers::settle(&mut g);
            if gain {
                g.gain_life(p, 1);
            }
            if loss {
                g.lose_life(p, 1);
            }
            let enemy_life = g.state.players[p.other().index()].life;
            g.begin_step(Step::End);
            g.give_priority();
            g.prepare_triggers();
            let before = g.snapshot();
            let mut branch = g.clone();
            big_helpers::settle(&mut g);
            assert_eq!(
                g.state.players[p.other().index()].life,
                enemy_life - i128::from(gain)
            );
            let bats = g
                .state
                .objects
                .iter()
                .filter(|o| {
                    o.zone == Zone::Battlefield
                        && o.controller == p
                        && o.card == Card::from_name("@token/6570bb4cb1a0220e62f056e8").unwrap()
                })
                .count();
            assert_eq!(bats, usize::from(gain && loss));
            if gain && loss {
                let bat = g
                    .state
                    .objects
                    .iter()
                    .enumerate()
                    .find(|(_, o)| {
                        o.zone == Zone::Battlefield
                            && o.card == Card::from_name("@token/6570bb4cb1a0220e62f056e8").unwrap()
                    })
                    .unwrap()
                    .0 as CardId;
                assert_eq!((g.stats(bat).power, g.stats(bat).toughness), (1, 1));
                assert!(g.keywords(bat).contains(Keywords::FLYING));
                assert_eq!(g.object_view(bat).card.colors(), 4);
                assert!(g.has_subtype(bat, "Bat"));
            }
            branch.restore(&before).unwrap();
            big_helpers::settle(&mut branch);
            assert_eq!(branch.snapshot(), g.snapshot());
        }
    }
}

#[test]
fn lunar_life_paid_draw_and_late_gain_do_not_retroactively_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Lunar Convocation", Zone::Battlefield);
        let life = g.state.players[p.index()].life;
        let hand = g.hand_count(p);
        big_helpers::checked(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].life, life - 2);
        big_helpers::settle(&mut g);
        assert_eq!(g.hand_count(p), hand + 1);
        let enemy_life = g.state.players[p.other().index()].life;
        g.begin_step(Step::End);
        g.give_priority();
        g.prepare_triggers();
        g.gain_life(p, 1);
        big_helpers::settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, enemy_life);
        assert!(!g.state.objects.iter().any(|o| o.zone == Zone::Battlefield
            && o.controller == p
            && o.card == Card::from_name("@token/6570bb4cb1a0220e62f056e8").unwrap()));
    }
}

#[test]
fn lunar_triggers_only_on_own_end_step_and_survive_source_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Lunar Convocation", Zone::Battlefield);
        g.gain_life(p, 1);
        g.lose_life(p, 1);
        g.state.active = p.other();
        let enemy = g.state.players[p.other().index()].life;
        g.begin_step(Step::End);
        g.give_priority();
        g.prepare_triggers();
        big_helpers::settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, enemy);
        g.state.active = p;
        g.begin_step(Step::End);
        g.give_priority();
        g.prepare_triggers();
        g.move_object(source, Zone::Graveyard);
        big_helpers::settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, enemy - 1);
        assert!(g.state.objects.iter().any(|o| o.zone == Zone::Battlefield
            && o.controller == p
            && o.card == Card::from_name("@token/6570bb4cb1a0220e62f056e8").unwrap()));
    }
}

#[test]
fn lunar_prevented_gain_does_not_satisfy_its_conditions() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Lunar Convocation", Zone::Battlefield);
        put(&mut g, p.other(), "Giant Cindermaw", Zone::Battlefield);
        let life = g.state.players[p.index()].life;
        let enemy = g.state.players[p.other().index()].life;
        g.gain_life(p, 3);
        g.lose_life(p, 1);
        assert_eq!(g.state.players[p.index()].life, life - 1);
        assert_eq!(g.state.players[p.index()].life_gained_this_turn, 0);
        g.begin_step(Step::End);
        g.give_priority();
        g.prepare_triggers();
        big_helpers::settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, enemy);
        assert!(!g.state.objects.iter().any(|o| o.zone == Zone::Battlefield
            && o.controller == p
            && o.card == Card::from_name("@token/6570bb4cb1a0220e62f056e8").unwrap()));
    }
}
