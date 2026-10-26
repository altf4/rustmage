//! The actual funding world must read its own sparse rows and virtual objects.
use super::*;
use crate::cards::{AnimationCreatureTypes, AnimationDuration, AnimationSpec};
use crate::engine::assurance::big_helpers::{named, setup};

fn fixture(p: Player) -> (Game, CardId, CardId) {
    let mut g = setup(p);
    let bearer = named(
        &mut g,
        p,
        "Doc Samson, Super Psychiatrist",
        Zone::Battlefield,
    );
    let blade = named(&mut g, p, "Goldvein Pick", Zone::Battlefield);
    assert!(g.attach_equipment(g.object_ref(blade), g.object_ref(bearer)));
    g.add_counter(blade, CounterKind::Hone, 2);
    g.state.players[p.index()].mana = [0; 6];
    (g, bearer, blade)
}
fn goal(p: Player, amount: u32) -> FundingGoal {
    FundingGoal {
        actor: p,
        required: [0, 0, 0, 0, amount, 0],
        generic: 0,
        reserve: None,
        source: None,
    }
}

#[test]
fn hob_hone_real_source_power_mana_and_funding_agree_with_counter_bonus() {
    for p in [Player::P0, Player::P1] {
        let (mut g, bearer, _blade) = fixture(p);
        assert_eq!(g.stats(bearer).power, 6);
        let saved = g.snapshot();
        assert!(g.can_fund(goal(p, 6)));
        assert!(!g.can_fund(goal(p, 7)));
        assert_eq!(g.snapshot(), saved);
        let action=g.legal_actions().into_iter().find(|a|matches!(a,Action::ActivateMana {card,production:[0,0,0,0,6,0],..}if *card==bearer)).expect("hone-aware actual mana action");
        g.apply(p, action).unwrap();
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 6, 0]);
    }
}

#[test]
fn hob_hone_funding_virtual_counter_payments_and_departures_do_not_read_real_rows() {
    for p in [Player::P0, Player::P1] {
        let (mut g, bearer, blade) = fixture(p);
        // This deliberately observes the continuous layer before the attachment SBA.
        // Creature Equipment still has a counter effect while it remains attached.
        g.change_object_form(
            g.object_ref(blade),
            AnimationSpec {
                artifact: true,
                creature_types: AnimationCreatureTypes::Unchanged,
                base: Some((1, 1)),
                keywords: Keywords::NONE,
                duration: AnimationDuration::EndOfTurn,
            },
            characteristic_layers::AnimationTypeMode::AddCreature,
        );
        assert_eq!(g.stats(bearer).power, 5);
        let saved = g.snapshot();
        let goal = goal(p, 5);
        let mut world = FundingState::from_game(&g, &goal);
        assert_eq!(world.power(bearer), 5);
        let key_before = world.key(&goal);
        assert!(world.remove_creature_counter(g.object_ref(blade), CounterKind::Hone));
        assert_eq!(world.counter_count(blade, CounterKind::Hone), 1);
        assert_eq!(world.power(bearer), 4);
        assert!(world.key(&goal) != key_before);
        // Production uses SourceTappedWorld, which must forward the paid row too.
        let production = ManaProduction::TapSourcePowerChoice { mask: 16 }.in_world(&world, bearer);
        let mut seen = Vec::new();
        production.visit(|mana| {
            seen.push(mana);
            false
        });
        assert_eq!(seen, vec![[0, 0, 0, 0, 4, 0]]);
        assert_eq!(g.counter_count(blade, CounterKind::Hone), 2);
        assert_eq!(g.snapshot(), saved);
        let resource = world.resources.iter_mut().find(|r| r.id == blade).unwrap();
        resource.object.zone = Zone::Graveyard;
        // Retain stale counter rows deliberately: the virtual object's departure
        // itself must make the boost cease, even before normal row cleanup.
        assert_eq!(world.power(bearer), 3);
        assert_eq!(g.stats(bearer).power, 5);
        assert_eq!(g.snapshot(), saved);
    }
}
