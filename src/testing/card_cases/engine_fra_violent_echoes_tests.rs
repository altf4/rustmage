//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

fn controlled_jaces(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && o.card == Card::JacePlaneswalkerToken)
                .then_some(id as CardId)
        })
        .collect()
}

#[test]
fn violent_echoes_uses_existing_jace_and_does_not_create_another() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        g.create_tokens(p, Card::JacePlaneswalkerToken, 1, false);
        let jace = controlled_jaces(&g, p)[0];
        g.add_counter(jace, CounterKind::Loyalty, 1);
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Violent Echoes", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(target)));
        settle(&mut g);
        assert_eq!(controlled_jaces(&g, p), vec![jace]);
        assert_eq!(g.counter_count(jace, CounterKind::Loyalty), 5);
    }
}

#[test]
fn violent_echoes_uses_greatest_excess_for_creature_planeswalker() {
    {
        let p = Player::P0;
        for (loyalty, toughness) in [(2, 5), (5, 2)] {
            let mut g = scenario(p);
            let target = put(
                &mut g,
                p.other(),
                "Ajani, Caller of the Pride",
                Zone::Battlefield,
            );
            g.state.objects[target as usize].counters.loyalty = loyalty;
            g.change_object_form(
                g.object_ref(target),
                AnimationSpec {
                    artifact: false,
                    creature_types: AnimationCreatureTypes::Unchanged,
                    base: Some((2, toughness)),
                    keywords: Keywords::NONE,
                    duration: AnimationDuration::EndOfTurn,
                },
                characteristic_layers::AnimationTypeMode::AddCreature,
            );
            let spell = put(&mut g, p, "Violent Echoes", Zone::Hand);
            cast(&mut g, spell, Some(Target::Card(target)));
            settle(&mut g);
            let jaces = controlled_jaces(&g, p);
            assert_eq!(jaces.len(), 1);
            assert_eq!(g.counter_count(jaces[0], CounterKind::Loyalty), 4);
        }
    }
}
