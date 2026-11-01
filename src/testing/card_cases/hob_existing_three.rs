//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..24 {
        if ready(g) {
            return;
        }
        assert_eq!(g.state.pending, Pending::Priority);
        act(g, Action::Pass);
    }
    panic!("expected draw or recruit decision did not arrive");
}
fn paid(g: &mut Game, p: Player, name: &str, mana: [u32; 6]) -> CardId {
    let card = put(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = mana;
    cast(g, card, None);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    card
}
fn councillors(g: &mut Game, p: Player) -> CardId {
    let card = paid(g, p, "Master's Councillors", [0, 1, 0, 0, 0, 1]);
    settle(g);
    card
}

#[test]
fn councillors_paid_live_zero_one_two_graveyard_thresholds_and_vigilance() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = councillors(&mut g, p);
        let other = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        assert_eq!(stats(&g, source), (1, 3));
        for owner in [p, p.other()] {
            for _ in 0..6 {
                put(&mut g, owner, "Forest", Zone::Graveyard);
            }
        }
        assert_eq!(
            stats(&g, source),
            (1, 3),
            "six in both graveyards gives no bonus"
        );
        let own_seventh = put(&mut g, p, "Shock", Zone::Hand);
        g.move_object(own_seventh, Zone::Graveyard);
        assert_eq!(stats(&g, source), (3, 3));
        let enemy_seventh = put(&mut g, p.other(), "Bear Cub", Zone::Hand);
        g.move_object(enemy_seventh, Zone::Graveyard);
        assert_eq!(stats(&g, source), (5, 3));
        assert_eq!(stats(&g, other), (2, 2), "only the source is boosted");
        g.move_object(own_seventh, Zone::Exile);
        assert_eq!(
            stats(&g, source),
            (3, 3),
            "opponent-only graveyard qualifies"
        );
        g.move_object(enemy_seventh, Zone::Exile);
        assert_eq!(stats(&g, source), (1, 3));
        // Advance to the next turn normally to clear entry summoning sickness.
        g.state.turn += 1;
        g.begin_turn();
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: source });
        act(&mut g, Action::FinishAttackers);
        settle(&mut g);
        assert!(g.object_view(source).keywords.contains(Keywords::VIGILANCE));
        assert!(!g.object_view(source).tapped);
    }
}
fn draw_one(g: &mut Game, p: Player) {
    // Resolution returns priority to the active player, including on the opponent's turn.
    if g.actor() != Some(p) {
        act(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
    paid(g, p, "Plunder the Trollshaws", [0, 1, 0, 0, 0, 1]);
}
fn resolve_mill(g: &mut Game, target: Player) {
    until(g, |g| {
        g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Player(target),
        })
    });
    act(
        g,
        Action::ChooseTarget {
            target: Target::Player(target),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}
#[test]
fn councillors_second_draw_only_targets_either_player_each_turn_and_packet_survives_source() {
    {
        let p = Player::P0;
        for target in [p, p.other()] {
            let mut g = scenario(p);
            let source = councillors(&mut g, p);
            for owner in [p, p.other()] {
                for _ in 0..8 {
                    put(&mut g, owner, "Forest", Zone::Library);
                }
            }
            draw_one(&mut g, p);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
            let before = g.state.players[target.index()].library.len();
            draw_one(&mut g, p);
            resolve_mill(&mut g, target);
            settle(&mut g);
            assert_eq!(
                g.state.players[target.index()].library.len(),
                before - 3 - usize::from(target == p)
            );
            draw_one(&mut g, p);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 3);
            // Turn-reset and instant-speed draws during the opponent's turn.
            g.state.turn += 1;
            g.begin_turn();
            g.state.active = p.other();
            g.state.priority = p;
            g.state.step = Step::Main1;
            draw_one(&mut g, p);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
            let before = g.state.players[target.index()].library.len();
            draw_one(&mut g, p);
            resolve_mill(&mut g, target);
            g.move_object(source, Zone::Exile);
            settle(&mut g);
            assert_eq!(
                g.state.players[target.index()].library.len(),
                before - 3 - usize::from(target == p)
            );
        }
    }
}

#[test]
fn boat_paid_crew_two_taps_actual_creatures_and_recruits_once_when_attacking() {
    {
        let p = Player::P0;
        for nonland in [false, true] {
            let mut g = scenario(p);
            let boat = paid(&mut g, p, "Great Gilded Boat", [0, 1, 0, 0, 0, 2]);
            settle(&mut g);
            assert!(!g.is_creature(boat));
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
            g.state.turn += 1;
            g.begin_turn();
            g.state.step = Step::Main1;
            let a = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
            let b = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: boat,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            let object = g.object_ref(enemy);
            reject(&mut g, Action::ChooseCrewer { object });
            let object = g.object_ref(a);
            act(&mut g, Action::ChooseCrewer { object });
            reject(&mut g, Action::FinishCrew);
            let object = g.object_ref(b);
            act(&mut g, Action::ChooseCrewer { object });
            act(&mut g, Action::FinishCrew);
            if g.state.announcing.is_some() {
                pay(&mut g);
            }
            settle(&mut g);
            assert!(g.object_view(a).tapped && g.object_view(b).tapped);
            assert!(g.is_creature(boat));
            assert_eq!(stats(&g, boat), (4, 4));
            let fellow = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let discard = put(
                &mut g,
                p,
                if nonland { "Shock" } else { "Forest" },
                Zone::Library,
            );
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            act(&mut g, Action::Attack { card: boat });
            act(&mut g, Action::Attack { card: fellow });
            act(&mut g, Action::FinishAttackers);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectDiscard { .. })
            });
            assert!(g.object_view(boat).tapped);
            assert_eq!(g.object_view(discard).zone, Zone::Hand);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
            assert!(g.observe(p.other()).objects.iter().all(|o| o.id != discard));
            act(&mut g, Action::Discard { card: discard });
            settle(&mut g);
            let tokens: Vec<_> = g
                .state
                .objects
                .iter()
                .enumerate()
                .filter(|(_, o)| o.controller == p && o.zone == Zone::Battlefield && o.is_token())
                .map(|(id, _)| id as CardId)
                .collect();
            assert_eq!(tokens.len(), usize::from(nonland));
            if nonland {
                let token = g.object_view(tokens[0]);
                assert_eq!((token.power, token.toughness, token.colors), (1, 1, 1));
                assert!(g.has_subtype(tokens[0], "Human") && g.has_subtype(tokens[0], "Soldier"));
            }
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                1,
                "one trigger for two attackers"
            );
            g.finish_cleanup();
            assert!(!g.is_creature(boat));
        }
    }
}
