//! Paid whole-pair fixtures; target membership, owned branches and replay at every decision.
use super::tests::{scenario, spawn};
use super::*;
const NAME: &str = "Feral Deathgorger // Dusk Sight";
fn act(g: &mut Game, a: Action) {
    let p = g.actor().unwrap();
    let index = g
        .legal_actions()
        .iter()
        .position(|v| *v == a)
        .expect("legal action");
    let before = g.snapshot();
    let revision = g.revision();
    assert!(g.apply(p.other(), a.clone()).is_err());
    assert_eq!(g.snapshot(), before);
    let mut child = {
        let parent = g.clone();
        let child = parent.clone();
        drop(parent);
        child
    };
    child.apply(p, a.clone()).unwrap();
    g.apply_index(p, revision, index).unwrap();
    assert_eq!(g.snapshot(), child.snapshot());
    let done = g.snapshot();
    g.restore(&before).unwrap();
    assert_eq!(g.apply_index(p, revision, index), Err(Error::StaleRevision));
    g.apply(p, a).unwrap();
    assert_eq!(g.snapshot(), done);
}
fn next(g: &mut Game) {
    let actions = g.legal_actions();
    let a = actions
        .iter()
        .find(|a| {
            matches!(
                a,
                Action::PayCosts { .. } | Action::PutTrigger { .. } | Action::FinishTargets
            )
        })
        .or_else(|| actions.iter().find(|a| matches!(a, Action::Pass)))
        .unwrap_or_else(|| panic!("{:?} {actions:?}", g.state.pending))
        .clone();
    act(g, a);
}
fn settle(g: &mut Game) {
    for _ in 0..100 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
        {
            return;
        }
        next(g);
    }
    panic!("unsettled");
}
fn setup(p: Player) -> (Game, CardId) {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    // Fixtures supply resources; AnnounceCast/PayCosts still perform real costs.
    g.state.players[p.index()].mana = [20; 6];
    let card = spawn(&mut g, p, Card::from_name(NAME).unwrap(), Zone::Hand);
    (g, card)
}
fn normal_targets(g: &mut Game, id: CardId) {
    act(g, Action::AnnounceCast { card: id });
    for _ in 0..60 {
        if matches!(g.state.pending, Pending::Targets { .. }) {
            return;
        }
        next(g);
    }
    panic!("no ETB target selection");
}
fn choose(g: &mut Game, id: CardId) {
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(id),
        },
    );
}
fn deny(g: &mut Game, id: CardId) {
    let a = Action::ChooseTarget {
        target: Target::Card(id),
    };
    assert!(!g.legal_actions().contains(&a));
    let s = g.snapshot();
    let r = g.revision();
    assert!(g.apply(g.actor().unwrap(), a).is_err());
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), r);
}
#[test]
fn feral_paid_normal_zero_one_two_either_graveyard_and_mixed_denial_both_seats() {
    for p in [Player::P0, Player::P1] {
        for owner in [p, p.other()] {
            for count in 0..=2 {
                let (mut g, id) = setup(p);
                let a = spawn(&mut g, owner, Card::Forest, Zone::Graveyard);
                let b = spawn(&mut g, owner, Card::Swamp, Zone::Graveyard);
                let foreign = spawn(&mut g, owner.other(), Card::Shock, Zone::Graveyard);
                let mana = g.state.players[p.index()].mana;
                normal_targets(&mut g, id);
                assert_ne!(g.state.players[p.index()].mana, mana);
                assert_eq!(g.state.objects[id as usize].zone, Zone::Battlefield);
                assert!(g.keywords(id).contains(Keywords::FLYING));
                assert!(g.keywords(id).contains(Keywords::DEATHTOUCH));
                if count > 0 {
                    choose(&mut g, a);
                    deny(&mut g, a);
                    deny(&mut g, foreign);
                }
                if count > 1 {
                    choose(&mut g, b);
                }
                settle(&mut g);
                for (target, exiled) in [(a, count > 0), (b, count > 1), (foreign, false)] {
                    assert_eq!(
                        g.state.objects[target as usize].zone,
                        if exiled { Zone::Exile } else { Zone::Graveyard }
                    );
                }
                assert!(g.state.adventures.is_none());
            }
        }
    }
}
#[test]
fn feral_empty_graveyards_and_required_group_counts_do_not_combine_players() {
    for p in [Player::P0, Player::P1] {
        let (mut g, id) = setup(p);
        act(&mut g, Action::AnnounceCast { card: id });
        settle(&mut g);
        assert_eq!(g.state.objects[id as usize].zone, Zone::Battlefield);
        let mut spec = Card::from_name(NAME).unwrap().triggers()[0]
            .targets
            .single()
            .unwrap();
        spec.min = 2;
        let source = g.object_ref(id);
        spawn(&mut g, p, Card::Forest, Zone::Graveyard);
        spawn(&mut g, p.other(), Card::Forest, Zone::Graveyard);
        assert!(!g.targets_available(TargetPlan::Single(spec), p, source));
        assert!(!g.cast_targets_available(TargetPlan::Single(spec), p, source));
        spawn(&mut g, p.other(), Card::Swamp, Zone::Graveyard);
        assert!(g.targets_available(TargetPlan::Single(spec), p, source));
        assert!(g.cast_targets_available(TargetPlan::Single(spec), p, source));
    }
}
#[test]
fn feral_partial_all_illegal_and_returned_incarnations_never_retarget() {
    for p in [Player::P0, Player::P1] {
        for removed in 1..=2 {
            let (mut g, id) = setup(p);
            let a = spawn(&mut g, p.other(), Card::Forest, Zone::Graveyard);
            let b = spawn(&mut g, p.other(), Card::Swamp, Zone::Graveyard);
            normal_targets(&mut g, id);
            choose(&mut g, a);
            choose(&mut g, b);
            while g.state.pending != Pending::Priority {
                next(&mut g);
            }
            // Adverse fixture simulates responses leaving and returning exact card IDs.
            g.move_object(a, Zone::Hand);
            g.move_object(a, Zone::Graveyard);
            if removed == 2 {
                g.move_object(b, Zone::Hand);
                g.move_object(b, Zone::Graveyard);
            }
            settle(&mut g);
            assert_eq!(g.state.objects[a as usize].zone, Zone::Graveyard);
            assert_eq!(
                g.state.objects[b as usize].zone,
                if removed == 1 {
                    Zone::Exile
                } else {
                    Zone::Graveyard
                }
            );
        }
    }
}
#[test]
fn feral_complete_dusk_sight_optional_counter_draw_and_owned_omen_shuffle() {
    for p in [Player::P0, Player::P1] {
        for selected in [false, true] {
            let (mut g, id) = setup(p);
            let ally = spawn(
                &mut g,
                p.other(),
                Card::from_name("Bear Cub").unwrap(),
                Zone::Battlefield,
            );
            let library = g.state.players[p.index()].library.len();
            act(&mut g, Action::AnnounceOmen { card: id });
            if selected {
                choose(&mut g, ally);
            }
            settle(&mut g);
            assert_eq!(
                g.state.objects[ally as usize].counters.plus_one,
                u64::from(selected)
            );
            assert_eq!(g.state.players[p.index()].library.len(), library); // draw one, shuffle source back
            assert_eq!(g.state.objects[id as usize].zone, Zone::Library);
            assert_eq!(
                g.state
                    .objects
                    .iter()
                    .filter(|o| o.owner == p && o.zone == Zone::Hand)
                    .count(),
                1
            );
            assert!(g.state.adventures.is_none());
        }
    }
}
#[test]
fn feral_group_template_unlocks_complete_sunshield_and_tracker_bodies() {
    for p in [Player::P0, Player::P1] {
        for name in ["Arashin Sunshield", "Griffnaut Tracker"] {
            let (mut g, unused) = setup(p);
            g.state.objects[unused as usize].zone = Zone::Exile;
            let id = spawn(&mut g, p, Card::from_name(name).unwrap(), Zone::Hand);
            let a = spawn(&mut g, p.other(), Card::Forest, Zone::Graveyard);
            let b = spawn(&mut g, p.other(), Card::Swamp, Zone::Graveyard);
            let enemy = spawn(
                &mut g,
                p.other(),
                Card::from_name("Bear Cub").unwrap(),
                Zone::Battlefield,
            );
            let land = spawn(&mut g, p, Card::Plains, Zone::Battlefield);
            normal_targets(&mut g, id);
            choose(&mut g, a);
            choose(&mut g, b);
            settle(&mut g);
            assert_eq!(g.state.objects[a as usize].zone, Zone::Exile);
            assert_eq!(g.state.objects[b as usize].zone, Zone::Exile);
            assert_eq!(g.stats(id).power, 3);
            if name == "Griffnaut Tracker" {
                assert_eq!(g.stats(id).toughness, 2);
                assert!(g.keywords(id).contains(Keywords::FLYING));
                continue;
            }
            assert_eq!(g.stats(id).toughness, 4);
            assert!(
                !g.legal_actions()
                    .iter()
                    .any(|a| matches!(a,Action::AnnounceActivate{card,..}if *card==id))
            );
            for _ in 0..100 {
                if !g.state.objects[id as usize].summoning_sick
                    && g.actor() == Some(p)
                    && g.state.pending == Pending::Priority
                {
                    break;
                }
                let a = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| {
                        matches!(
                            a,
                            Action::Pass | Action::FinishAttackers | Action::FinishBlockers
                        )
                    })
                    .unwrap();
                act(&mut g, a);
            }
            assert!(!g.state.objects[id as usize].summoning_sick);
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: id,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            choose(&mut g, enemy);
            act(&mut g, Action::TapMana { card: land });
            let pay = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap();
            act(&mut g, pay);
            assert!(g.state.objects[id as usize].tapped);
            settle(&mut g);
            assert!(g.state.objects[enemy as usize].tapped);
        }
    }
}
