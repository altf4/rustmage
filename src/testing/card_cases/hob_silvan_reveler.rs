//! Focused printed outcomes; shared engine invariants live in core tests.
//! Mandatory discarded-land recovery and exact graveyard landfall payments.
use super::*;
const FIXTURES: &[&str] = &[
    "Silvan Reveler",
    "Forest",
    "Island",
    "Bear Cub",
    "Shock",
    "Rest in Peace",
    "Secluded Courtyard",
    "Lorehold, the Historian",
];
fn fixture(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    assert!(FIXTURES.contains(&name), "unlisted {name}");
    put(g, p, name, zone)
}
fn quiet(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
        && g.state.resolving.is_none()
}
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..96 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Silvan budget {:?}", g.state.pending)
}
fn finish(g: &mut Game) {
    until(g, quiet)
}
fn paid(g: &mut Game, p: Player) -> CardId {
    let id = fixture(g, p, "Silvan Reveler", Zone::Hand);
    g.state.players[p.index()].mana = [0, 1, 0, 0, 1, 2];
    cast(g, id, None);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn discard(g: &mut Game) {
    until(g, |g| {
        matches!(g.state.pending, Pending::EffectDiscard { .. })
    });
}
fn first_hand(g: &Game, p: Player) -> CardId {
    g.state
        .objects
        .iter()
        .enumerate()
        .find_map(|(i, o)| (o.zone == Zone::Hand && o.owner == p).then_some(i as CardId))
        .unwrap()
}
fn landfall(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = fixture(g, p, name, Zone::Hand);
    act(g, Action::PlayLand { card: id });
    id
}
fn payment(g: &mut Game) {
    until(g, |g| {
        matches!(g.state.pending, Pending::ResolutionMana { .. })
    });
}

#[test]
fn silvan_nonland_discard_does_not_recover_unrelated_land_and_source_can_leave() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let bear = fixture(&mut g, p, "Bear Cub", Zone::Hand);
        let oldland = fixture(&mut g, p, "Forest", Zone::Graveyard);
        let source = paid(&mut g, p);
        discard(&mut g);
        g.move_object(source, Zone::Exile);
        act(&mut g, Action::Discard { card: bear });
        finish(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
        assert_eq!(g.object_view(oldland).zone, Zone::Graveyard);
        assert_eq!(g.hand_count(p), 1);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
    }
}
#[test]
fn silvan_discard_replaced_by_exile_is_not_put_on_battlefield() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        fixture(&mut g, p.other(), "Rest in Peace", Zone::Battlefield);
        paid(&mut g, p);
        discard(&mut g);
        let land = first_hand(&g, p);
        let old = g.object_ref(land);
        act(&mut g, Action::Discard { card: land });
        finish(&mut g);
        assert_eq!(g.object_view(land).zone, Zone::Exile);
        assert_eq!(g.object_ref(land).generation, old.generation + 1);
        assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 1);
        assert!(g.observe(p).effect_choice.is_none());
    }
}

#[test]
fn silvan_graveyard_landfall_paid_decline_unpayable_and_battlefield_exile_hand_excluded() {
    {
        let p = Player::P0;
        for mode in 0..3 {
            let mut g = scenario(p);
            let source = fixture(&mut g, p, "Silvan Reveler", Zone::Graveyard);
            for zone in [Zone::Battlefield, Zone::Hand, Zone::Exile] {
                fixture(&mut g, p, "Silvan Reveler", zone);
            }
            g.state.players[p.index()].mana = if mode == 2 {
                [0; 6]
            } else {
                [0, 1, 0, 0, 1, 1]
            };
            landfall(&mut g, p, "Forest");
            payment(&mut g);
            assert_eq!(
                g.state
                    .resolving
                    .as_ref()
                    .unwrap()
                    .item
                    .source
                    .unwrap()
                    .object,
                g.object_ref(source)
            );
            assert_eq!(g.state.resolving.as_ref().unwrap().item.controller, p);
            reject(&mut g, Action::PayResolutionMana { payment: [0; 6] });
            if mode == 0 {
                act(
                    &mut g,
                    Action::PayResolutionMana {
                        payment: [0, 1, 0, 0, 1, 1],
                    },
                );
            } else {
                if mode == 2 {
                    assert!(
                        !g.legal_actions()
                            .iter()
                            .any(|a| matches!(a, Action::PayResolutionMana { .. }))
                    );
                }
                act(&mut g, Action::DeclineResolutionMana);
            }
            finish(&mut g);
            assert_eq!(
                g.object_view(source).zone,
                if mode == 0 {
                    Zone::Hand
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(
                g.state.players[p.index()].mana,
                if mode == 1 {
                    [0, 1, 0, 0, 1, 1]
                } else {
                    [0; 6]
                }
            );
            if mode == 0 {
                assert!(g.observe(p.other()).objects.iter().any(|o| o.id == source));
            }
        }
    }
}
