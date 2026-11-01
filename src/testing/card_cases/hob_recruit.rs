//! Focused printed outcomes; shared engine invariants live in core tests.
//! Recruit: draw first, private mandatory discard, and actual nonland discard token.
use super::*;

fn pause(g: &mut Game) {
    for _ in 0..24 {
        if matches!(g.state.pending, Pending::EffectDiscard { .. }) {
            return;
        }
        assert_eq!(g.state.pending, Pending::Priority);
        act(g, Action::Pass);
    }
    panic!("recruit discard did not arrive");
}
fn soldiers(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| o.zone == Zone::Battlefield && o.controller == p && o.is_token())
        .map(|(id, _)| id as CardId)
        .collect()
}

#[test]
fn hob_recruit_paid_definitions_draw_before_discard_and_complete_token_recipe() {
    {
        let p = Player::P0;
        for (name, body, keyword, cost) in [
            (
                "Long Lake Nuisance",
                (3, 1),
                Keywords::FLYING,
                [0, 1, 0, 0, 0, 3],
            ),
            (
                "Patient Instructor",
                (2, 2),
                Keywords::VIGILANCE,
                [1, 0, 0, 0, 0, 2],
            ),
            (
                "Patient Instructor",
                (2, 2),
                Keywords::VIGILANCE,
                [0, 1, 0, 0, 0, 2],
            ),
            (
                "Esgaroth Garrison",
                (1, 5),
                Keywords::NONE,
                [1, 0, 0, 0, 0, 4],
            ),
        ] {
            for nonland in [false, true] {
                let mut g = scenario(p);
                let discard = put(
                    &mut g,
                    p,
                    if nonland { "Shock" } else { "Forest" },
                    Zone::Hand,
                );
                let drawn = put(&mut g, p, "Island", Zone::Library);
                let source = put(&mut g, p, name, Zone::Hand);
                g.state.players[p.index()].mana = cost;
                cast(&mut g, source, None);
                pause(&mut g);
                assert_eq!(stats(&g, source), body, "{name}");
                assert!(g.object_view(source).keywords.contains(keyword));
                assert_eq!(g.object_view(drawn).zone, Zone::Hand);
                assert_eq!(g.hand_count(p), 2);
                assert_eq!(g.state.players[p.index()].mana, [0; 6]);
                assert!(
                    g.observe(p.other())
                        .objects
                        .iter()
                        .all(|o| o.id != discard && o.id != drawn)
                );
                act(&mut g, Action::Discard { card: discard });
                settle(&mut g);
                let tokens = soldiers(&g, p);
                assert_eq!(tokens.len(), usize::from(nonland), "{name}");
                assert_eq!(g.hand_count(p), 1);
                assert_eq!(g.object_view(discard).zone, Zone::Graveyard);
                if let Some(&token) = tokens.first() {
                    let view = g.object_view(token);
                    assert_eq!(
                        (view.power, view.toughness, view.colors, view.kinds),
                        (1, 1, 1, 1)
                    );
                    assert_eq!(view.card.info().type_line, "Creature — Human Soldier");
                    assert!(!view.tapped);
                    assert!(view.summoning_sick);
                }
                if name == "Esgaroth Garrison" {
                    assert_eq!(stats(&g, source), (if nonland { 2 } else { 1 }, 5));
                    let enemy = put(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
                    assert_eq!(stats(&g, source).0, if nonland { 2 } else { 1 });
                    g.move_object(enemy, Zone::Graveyard);
                    if let Some(&token) = tokens.first() {
                        g.move_object(token, Zone::Exile);
                    }
                    assert_eq!(stats(&g, source), (1, 5));
                    g.move_object(source, Zone::Hand);
                    assert_eq!(stats(&g, source), (0, 5), "CDA works in hand");
                }
            }
        }
    }
}

#[test]
fn hob_recruit_entry_packet_survives_source_leaving_before_resolution() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Long Lake Nuisance", Zone::Hand);
    let drawn = put(&mut g, p, "Shock", Zone::Library);
    cast(&mut g, source, None);
    for _ in 0..12 {
        if g.object_view(source).zone == Zone::Battlefield {
            break;
        }
        act(&mut g, Action::Pass);
    }
    assert_eq!(g.object_view(source).zone, Zone::Battlefield);
    g.move_object(source, Zone::Exile);
    pause(&mut g);
    act(&mut g, Action::Discard { card: drawn });
    settle(&mut g);
    assert_eq!(soldiers(&g, p).len(), 1);
}
