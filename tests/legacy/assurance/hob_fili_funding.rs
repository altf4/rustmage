//! The grouped conditional bonus must use the projected world, not the live Game.
use super::*;
use crate::engine::assurance::big_helpers::{named, setup};
fn act(g: &mut Game, a: Action) {
    g.apply(g.actor().unwrap(), a).unwrap();
}
fn pay(g: &mut Game) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    act(g, a);
}
fn settle(g: &mut Game) {
    for _ in 0..32 {
        if g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending == Pending::Priority
        {
            return;
        }
        assert_eq!(g.state.pending, Pending::Priority);
        act(g, Action::Pass);
    }
    panic!("funding fixture resolution budget");
}
fn cast(g: &mut Game, id: CardId, target: Option<Target>) {
    g.state.players[g.actor().unwrap().index()].mana = [20; 6];
    act(g, Action::AnnounceCast { card: id });
    if let Some(target) = target {
        act(g, Action::ChooseTarget { target });
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
}
#[test]
fn hob_fili_funding_group_condition_and_borrowed_power_share_projected_designation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let fili = named(&mut g, p, "Fíli the Pathfinder", Zone::Hand);
        cast(&mut g, fili, None);
        settle(&mut g);
        let body = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let wasp = named(&mut g, p, "The Wondrous Wasp", Zone::Hand);
        cast(&mut g, wasp, None);
        let target = Target::Card(fili);
        for _ in 0..24 {
            if g.legal_actions().contains(&Action::ChooseTarget { target }) {
                act(&mut g, Action::ChooseTarget { target });
                if g.legal_actions().contains(&Action::FinishTargets) {
                    act(&mut g, Action::FinishTargets);
                }
                break;
            }
            act(&mut g, Action::Pass);
        }
        settle(&mut g);
        assert!(!g.printed_abilities_available(fili));
        let ring = named(&mut g, p, "Lotus Ring", Zone::Battlefield);
        assert!(g.attach_equipment(g.object_ref(ring), g.object_ref(wasp)));
        named(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        g.refresh_controllers();
        g.state.objects[wasp as usize].summoning_sick = false;
        g.state.players[p.index()].mana = [0; 6];
        assert!(!g.state.players[p.index()].enduring_story);
        assert_eq!(g.stats(body).power, 2);
        let saved = g.snapshot();
        let goal = FundingGoal {
            actor: p,
            required: [0, 0, 0, 3, 0, 0],
            generic: 0,
            reserve: None,
            source: None,
        };
        let world = FundingState::from_game(&g, &goal);
        let step = world
            .moves(&goal)
            .into_iter()
            .find(|step| {
                world.resources[step.resource].id == wasp && step.production == [0, 0, 0, 3, 0, 0]
            })
            .unwrap();
        let mut projected = None;
        world.visit_applied(&step, &mut |next| {
            projected = Some(next.clone());
            true
        });
        let after = projected.unwrap();
        assert!(after.has_enduring_story(p));
        let query = characteristic_query::CharacteristicQuery::in_world(&after);
        assert_eq!(query.stats(fili).power, 3);
        assert_eq!(query.stats(body).power, 3);
        assert_eq!(
            ManaProduction::TapSourcePowerChoice { mask: 8 }.in_world(&after, body),
            ManaProduction::Choice { mask: 8, amount: 3 }
        );
        assert_eq!(
            g.snapshot(),
            saved,
            "projection must not mutate live story or stats"
        );
        drop(after);
        drop(world);
        let grantor = g.object_ref(ring);
        act(
            &mut g,
            Action::TapGrantedMana {
                card: wasp,
                grantor,
                production: [0, 0, 0, 3, 0, 0],
            },
        );
        assert!(g.state.players[p.index()].enduring_story);
        assert_eq!(g.stats(fili).power, 3);
        assert_eq!(g.stats(body).power, 3);
    }
}
