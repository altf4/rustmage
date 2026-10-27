//! Linked exile type checks use projected object incarnations.
use super::*;
use crate::engine::assurance::big_helpers::{named, setup};

#[test]
fn projected_linked_exile_counts_only_matching_exiled_incarnations() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Keen-Eyed Curator", Zone::Battlefield);
        let exiled = named(&mut g, p, "Solemn Simulacrum", Zone::Exile);
        let link = linked_choices::LinkedExileView {
            source: g.object_ref(source),
            exiled: g.object_ref(exiled),
        };
        g.state
            .linked_choices
            .get_or_insert_default()
            .links
            .push(link);
        let saved = g.snapshot();
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 0,
            reserve: None,
            source: None,
        };
        let world = FundingState::from_game(&g, &goal);
        assert_eq!(world.value(StatValue::LinkedExileCardTypes, source), 2);
        let query = characteristic_query::CharacteristicQuery::in_world(&world);
        assert_eq!(query.value(StatValue::LinkedExileCardTypes, source), 2);
        let mut changed = world.clone();
        if let Some(r) = changed.resources.iter_mut().find(|r| r.id == exiled) {
            r.object.zone = Zone::Graveyard;
        } else {
            let mut object = g.state.objects[exiled as usize].clone();
            object.zone = Zone::Graveyard;
            changed.resources.push(Resource {
                id: exiled,
                attacking: false,
                object,
            });
        }
        assert_eq!(changed.value(StatValue::LinkedExileCardTypes, source), 0);
        assert_eq!(g.snapshot(), saved);
    }
}
