//! CountSubtype observes each projected object's current zone/control/subtypes.
use super::*;
use crate::engine::assurance::big_helpers::{named, setup};
fn halfling(g: &mut Game, id: CardId) {
    let recipient = g.object_ref(id);
    let timestamp = g.next_role_timestamp();
    g.state
        .roles
        .get_or_insert_with(Default::default)
        .creature_types
        .push(subtypes::CreatureTypesEffectView {
            recipient,
            source: recipient,
            types: &["Halfling"],
            additional: false,
            timestamp,
        });
}
fn production(world: &FundingState<'_>, id: CardId) -> ManaProduction {
    ManaProduction::CountSubtype {
        color: 5,
        subtype: "Halfling",
    }
    .in_world(world, id)
}
#[test]
fn hob_notary_funding_projected_zone_control_subtypes_keys_and_owned_moves() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        for state in &mut g.state.players {
            state.mana = [0; 6];
        }
        let source = named(&mut g, p, "The Notary Hobbits", Zone::Battlefield);
        let buddy = named(&mut g, p, "Fanatical Firebrand", Zone::Battlefield);
        halfling(&mut g, buddy);
        named(&mut g, p.other(), "The Notary Hobbits", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        halfling(&mut g, enemy);
        let saved = g.snapshot();
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 2,
            reserve: None,
            source: None,
        };
        let world = FundingState::from_game(&g, &goal);
        assert_eq!(
            production(&world, source),
            ManaProduction::Fixed([0, 0, 0, 0, 0, 2])
        );
        let mut departed = world.clone();
        departed
            .resources
            .iter_mut()
            .find(|r| r.id == buddy)
            .unwrap()
            .object
            .zone = Zone::Graveyard;
        assert_eq!(
            production(&departed, source),
            ManaProduction::Fixed([0, 0, 0, 0, 0, 1])
        );
        assert!(world.key(&goal) != departed.key(&goal));
        let mut stolen = world.clone();
        stolen
            .resources
            .iter_mut()
            .find(|r| r.id == source)
            .unwrap()
            .object
            .controller = p.other();
        assert_eq!(
            production(&stolen, source),
            ManaProduction::Fixed([0, 0, 0, 0, 0, 3])
        );
        assert!(world.key(&goal) != stolen.key(&goal));
        let step = world
            .moves(&goal)
            .into_iter()
            .find(|m| {
                world.resources[m.resource].id == source && m.production == [0, 0, 0, 0, 0, 2]
            })
            .unwrap();
        let mut applied = None;
        world.visit_applied(&step, &mut |next| {
            applied = Some(next.clone());
            true
        });
        let child = applied.unwrap();
        drop(world);
        assert!(
            child
                .resources
                .iter()
                .find(|r| r.id == source)
                .unwrap()
                .object
                .tapped
        );
        assert_eq!(child.mana, [0, 0, 0, 0, 0, 2]);
        assert_eq!(g.snapshot(), saved);
    }
}
#[test]
fn hob_notary_funding_copied_nonlegendary_mana_source_and_zero_count_match_actual() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        for state in &mut g.state.players {
            state.mana = [0; 6];
        }
        let id = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let mut recipe = CopyRecipe::new(Card::from_name("The Notary Hobbits").unwrap());
        recipe.overrides = Some(CopyOverrides {
            legendary: Some(false),
            ..CopyOverrides::NONE
        });
        g.apply_temporary_copy(g.object_ref(id), recipe, CopyDeadline::EndOfTurn);
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 1,
            reserve: None,
            source: None,
        };
        let world = FundingState::from_game(&g, &goal);
        assert_eq!(
            production(&world, id),
            ManaProduction::Fixed([0, 0, 0, 0, 0, 1])
        );
        assert!(world.moves(&goal).iter().any(|m|world.resources[m.resource].id==id&&m.production==[0,0,0,0,0,1]));
        drop(world);
        let recipient = g.object_ref(id);
        let timestamp = g.next_role_timestamp();
        g.state
            .roles
            .get_or_insert_with(Default::default)
            .creature_types
            .push(subtypes::CreatureTypesEffectView {
                recipient,
                source: recipient,
                types: &["Bear"],
                additional: false,
                timestamp,
            });
        let world = FundingState::from_game(&g, &goal);
        assert_eq!(production(&world, id), ManaProduction::Fixed([0; 6]));
        assert!(!g.can_fund(goal));
    }
}
