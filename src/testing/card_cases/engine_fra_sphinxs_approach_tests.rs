//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

fn put_approach(g: &mut Game, player: Player, zone: Zone) -> CardId {
    let id = put(g, player, "Shock", zone);
    g.state.objects[id as usize].card = Card::from_name("Sphinx's Approach").unwrap();
    id
}

#[test]
fn approach_decline_or_insufficient_graveyard_leaves_no_partial_payment() {
    {
        let p = Player::P0;
        for count in [3, 4] {
            let mut g = scenario(p);
            let cards: Vec<_> = (0..count)
                .map(|_| put_approach(&mut g, p, Zone::Graveyard))
                .collect();
            let spell = put_approach(&mut g, p, Zone::Hand);
            cast(&mut g, spell, None);
            for _ in 0..96 {
                if matches!(g.state.pending, Pending::EffectChoice { .. })
                    || (g.state.pending == Pending::Priority
                        && g.state.stack.is_empty()
                        && g.state.resolving.is_none())
                {
                    break;
                }
                choose(&mut g, |a| {
                    matches!(a, Action::Pass | Action::PutTrigger { .. })
                });
            }
            if count == 4 {
                let reference = g.object_ref(cards[0]);
                act(&mut g, Action::SelectResolutionExile { object: reference });
                act(&mut g, Action::DeclineEffectPayment);
                settle(&mut g);
            }
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            assert!(
                cards
                    .iter()
                    .all(|&id| g.object_view(id).zone == Zone::Graveyard)
            );
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
        }
    }
}

#[test]
fn transient_copy_graveyard_transitions_do_not_trigger_card_observers() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        put(&mut g, p, "Owlin Historian", Zone::Battlefield);
        let spell = put(&mut g, p, "Shock", Zone::Hand);
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        cast(&mut g, spell, Some(Target::Card(target)));
        let original = g.state.stack.last().unwrap().clone();
        let copy = g.allocate_spell_copy(&original, p);
        let fake = copy.source.unwrap().id;
        g.state.stack.push(copy);
        let entry = g.move_object(fake, Zone::Graveyard);
        assert!(!entry.was_card);
        assert!(!g.graveyard_entry_this_turn(entry.after));
        let pending_before = g.state.pending_triggers.len();
        let departure = g.move_object(fake, Zone::Exile);
        assert!(!departure.was_card);
        assert_eq!(g.state.pending_triggers.len(), pending_before);
        let physical = put(&mut g, p, "Shock", Zone::Graveyard);
        let departure = g.move_object(physical, Zone::Exile);
        assert!(departure.was_card);
        assert!(g.state.pending_triggers.len() > pending_before);
    }
}
