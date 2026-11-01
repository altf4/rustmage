//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;
fn armies(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(id, o)| {
            o.zone == Zone::Battlefield && o.controller == p && g.has_subtype(*id as CardId, "Army")
        })
        .map(|(id, _)| id as CardId)
        .collect()
}
#[test]
fn hob_amass_spell_and_entry_outcomes() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let rage = put(&mut g, p, "Rage into the Valley", Zone::Hand);
        cast(&mut g, rage, None);
        settle(&mut g);
        let army = armies(&g, p)[0];
        assert_eq!(stats(&g, army), (2, 2));
        assert!(g.has_subtype(army, "Goblin"));
        assert_eq!(g.object_view(army).colors, 4);
        assert_eq!(g.state.players[p.index()].life, 19);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        let flunkies = put(&mut g, p, "Goblin-town Flunkies", Zone::Hand);
        cast(&mut g, flunkies, None);
        settle(&mut g);
        assert_eq!(armies(&g, p), vec![army]);
        assert_eq!(stats(&g, army), (3, 3));

        let gathering = put(&mut g, p, "Gathering of Darkness", Zone::Hand);
        let dead = put(&mut g, p, "Llanowar Elves", Zone::Graveyard);
        cast(&mut g, gathering, Some(Target::Card(dead)));
        settle(&mut g);
        assert_eq!(g.state.objects[dead as usize].zone, Zone::Hand);
        assert_eq!(stats(&g, army), (6, 6));
    }

    covered("audited/hob_amass::hob_amass_spell_and_entry_outcomes/3338");
    covered("audited/hob_amass::hob_amass_spell_and_entry_outcomes/1813");
    covered("audited/hob_amass::hob_amass_spell_and_entry_outcomes/1700");
}
#[test]
fn hob_amass_trigger_and_death_outcomes() {
    let p = Player::P0;
    let mut g = scenario(p);
    let noise = put(&mut g, p, "Bothersome Noisemaker", Zone::Battlefield);
    let rage = put(&mut g, p, "Rage into the Valley", Zone::Hand);
    cast(&mut g, rage, None);
    settle(&mut g);
    let army = armies(&g, p)[0];
    assert_eq!(stats(&g, army), (3, 3));
    assert_eq!(stats(&g, noise), (2, 2));
    let pair = put(&mut g, p, "Fearsome Goblin Pair", Zone::Hand);
    cast(&mut g, pair, None);
    settle(&mut g);
    assert_eq!(stats(&g, army), (3, 3));
    let murder = put(&mut g, p, "Murder", Zone::Hand);
    cast(&mut g, murder, Some(Target::Card(pair)));
    settle(&mut g);
    assert_eq!(g.state.objects[pair as usize].zone, Zone::Graveyard);
    assert_eq!(stats(&g, army), (8, 8));

    covered("audited/hob_amass::hob_amass_trigger_and_death_outcomes/511");
    covered("audited/hob_amass::hob_amass_trigger_and_death_outcomes/1475");
}
#[test]
fn hob_amass_attack_is_once_per_declaration() {
    let p = Player::P1;
    let mut g = scenario(p);
    let raider = put(&mut g, p, "Misty Mountains Raider", Zone::Battlefield);
    let a = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
    let b = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    act(&mut g, Action::Attack { card: a });
    act(&mut g, Action::Attack { card: b });
    act(&mut g, Action::FinishAttackers);
    settle(&mut g);
    assert_eq!(stats(&g, armies(&g, p)[0]), (2, 2));
    assert_eq!(stats(&g, raider), (4, 4));

    covered("audited/hob_amass::hob_amass_attack_is_once_per_declaration/2799");
}
