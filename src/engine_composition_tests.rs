//! Actual paid cross-mechanic regressions. Native fixtures seed hands, mana and
//! opposing creatures; every cast, choice and turn advance below uses legal actions.
use super::tests::{scenario, spawn};
use super::*;
#[derive(Default)]
struct Trace {
    casts: usize,
    ward_paid: usize,
    ward_declined: usize,
}
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    spawn(g, p, Card::from_name(name).unwrap(), zone)
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for q in [p, p.other()] {
        g.state.players[q.index()].mana = [50; 6];
    }
    g
}
fn replay(g: &mut Game, a: Action, t: &mut Trace) {
    let p = g.actor().unwrap();
    let saved = g.snapshot();
    let rev = g.revision();
    let index = g
        .legal_actions()
        .iter()
        .position(|v| v == &a)
        .unwrap_or_else(|| {
            panic!(
                "illegal fixture action {a:?} at {:?}: {:?}",
                g.state.pending,
                g.legal_actions()
            )
        });
    if matches!(a, Action::PayCosts { .. })
        && g.state
            .announcing
            .as_ref()
            .is_some_and(|v| v.item.kind == StackKind::Spell)
    {
        t.casts += 1;
    }
    if matches!(a, Action::PayResolutionMana { .. }) {
        t.ward_paid += 1;
    }
    if matches!(a, Action::DeclineResolutionMana) {
        t.ward_declined += 1;
    }
    assert_eq!(g.apply(p.other(), a.clone()), Err(Error::WrongActor));
    assert_eq!(g.snapshot(), saved);
    let parent = g.clone();
    let mut child = parent.clone();
    drop(parent);
    child.apply(p, a.clone()).unwrap();
    g.apply_index(p, rev, index).unwrap();
    assert_eq!(child.snapshot(), g.snapshot());
    let after = g.snapshot();
    g.restore(&saved).unwrap();
    assert_eq!(g.apply_index(p, rev, index), Err(Error::StaleRevision));
    g.apply(p, a).unwrap();
    assert_eq!(g.snapshot(), after);
    for viewer in [p, p.other()] {
        let mut expected = serde_json::to_value(child.observe(viewer)).unwrap();
        let mut actual = serde_json::to_value(g.observe(viewer)).unwrap();
        expected["revision"] = 0.into();
        actual["revision"] = 0.into();
        assert_eq!(actual, expected);
        assert_eq!(g.observe(viewer).schema, OBSERVATION_SCHEMA);
    }
}
fn idle(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
        && g.state.announcing.is_none()
        && g.state.resolving.is_none()
}
fn settle(g: &mut Game, targets: &[CardId], pay_ward: bool, t: &mut Trace) {
    for _ in 0..250 {
        if idle(g) {
            return;
        }
        let actions = g.legal_actions();
        let action = actions
            .iter()
            .max_by_key(|a| match a {
                Action::ChooseTarget {
                    target: Target::Card(id),
                } => {
                    if targets.contains(id) {
                        1000
                    } else {
                        10
                    }
                }
                Action::PayResolutionMana { .. } => {
                    if pay_ward {
                        1500
                    } else {
                        -10
                    }
                }
                Action::DeclineResolutionMana => {
                    if pay_ward {
                        -10
                    } else {
                        1500
                    }
                }
                Action::PayCosts { .. } => 1200,
                Action::PutTrigger { .. } => 1100,
                Action::FinishTargets => 900,
                Action::ChooseReplacement { .. } | Action::ChooseOptional { accept: true } => 1000,
                Action::Pass => 100,
                Action::Concede
                | Action::AnnounceCast { .. }
                | Action::AnnounceAdventure { .. }
                | Action::PlayLand { .. } => -1000,
                _ => 0,
            })
            .unwrap()
            .clone();
        replay(g, action, t);
    }
    panic!("unsettled cross-mechanic frame {:?}", g.state.pending)
}
fn cast(g: &mut Game, id: CardId, adventure: bool, targets: &[CardId], pay: bool, t: &mut Trace) {
    replay(
        g,
        if adventure {
            Action::AnnounceAdventure { card: id }
        } else if g.object_view(id).zone == Zone::Exile {
            Action::AnnounceExile { card: id }
        } else {
            Action::AnnounceCast { card: id }
        },
        t,
    );
    settle(g, targets, pay, t);
}
#[test]
fn paid_foreign_attachment_ward_counters_inset_then_saga_counts_physical_grave_face() {
    for p in [Player::P0, Player::P1] {
        let q = p.other();
        let mut g = setup(p);
        let mut t = Trace::default();
        let enemy = named(&mut g, q, "Hill Giant", Zone::Battlefield);
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, false, &[enemy], false, &mut t);
        assert_eq!(g.object_view(enemy).damage, 2);
        let strength = named(&mut g, p, "Super Strength", Zone::Hand);
        cast(&mut g, strength, false, &[enemy], false, &mut t);
        assert_eq!(
            g.object_view(strength).attached_to,
            Some(g.object_ref(enemy))
        );
        let parent = Card::from_name("Frolicking Familiar // Blow Off Steam").unwrap();
        let familiar = spawn(&mut g, p, parent, Zone::Hand);
        cast(&mut g, familiar, true, &[enemy], false, &mut t);
        assert_eq!(t.ward_declined, 1);
        assert_eq!(g.object_view(familiar).zone, Zone::Graveyard);
        assert_eq!(g.object_view(familiar).card, parent);
        assert!(!g.can_play_adventure(p, familiar));
        assert!(g.state.adventures.is_none());
        assert_eq!(g.object_view(enemy).damage, 2);
        let ramuh = named(&mut g, p, "Summon: Esper Ramuh", Zone::Hand);
        cast(&mut g, ramuh, false, &[enemy], true, &mut t);
        assert_eq!(g.object_view(ramuh).counters.lore, 1);
        assert_eq!(
            g.object_view(enemy).damage,
            3,
            "only the actual Shock counts; the countered inset is a physical creature in the graveyard"
        );
        assert_eq!(t.ward_paid, 1);
        assert_eq!(t.ward_declined, 1);
        assert_eq!(t.casts, 4);
    }
}
fn next_main(g: &mut Game, p: Player, t: &mut Trace) {
    let turn = g.state.turn;
    for _ in 0..500 {
        if g.state.turn > turn && g.state.active == p && g.state.step == Step::Main1 {
            return;
        }
        let actions = g.legal_actions();
        let action = actions
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass | Action::FinishAttackers | Action::FinishBlockers
                )
            })
            .expect("ordinary empty-combat turn advance");
        replay(g, action, t);
    }
    panic!("did not reach next real precombat main")
}
#[test]
fn paid_adventure_front_with_ward_receives_doubled_saga_counters_after_real_turn_boundary() {
    for p in [Player::P0, Player::P1] {
        let q = p.other();
        let mut g = setup(p);
        let mut t = Trace::default();
        let enemy = named(&mut g, q, "Hill Giant", Zone::Battlefield);
        let season = named(&mut g, p, "Doubling Season", Zone::Hand);
        cast(&mut g, season, false, &[], false, &mut t);
        let familiar = named(
            &mut g,
            p,
            "Frolicking Familiar // Blow Off Steam",
            Zone::Hand,
        );
        cast(&mut g, familiar, true, &[enemy], false, &mut t);
        assert!(g.can_play_adventure(p, familiar));
        let grant = g.object_ref(familiar);
        cast(&mut g, familiar, false, &[], false, &mut t);
        assert_ne!(g.object_ref(familiar), grant);
        assert_eq!(g.object_view(familiar).zone, Zone::Battlefield);
        assert!(!g.can_play_adventure(p, familiar));
        let shelter = named(&mut g, p, "Sheltered by Ghosts", Zone::Hand);
        cast(&mut g, shelter, false, &[familiar, enemy], false, &mut t);
        assert_eq!(g.object_view(enemy).zone, Zone::Exile);
        assert_eq!(
            g.object_view(shelter).attached_to,
            Some(g.object_ref(familiar))
        );
        let shock = named(&mut g, q, "Shock", Zone::Hand);
        replay(&mut g, Action::Pass, &mut t);
        assert_eq!(g.actor(), Some(q));
        cast(&mut g, shock, false, &[familiar], false, &mut t);
        assert_eq!(t.ward_declined, 1);
        assert_eq!(g.object_view(shock).zone, Zone::Graveyard);
        let sweet = named(&mut g, p, "Welcome to Sweettooth", Zone::Hand);
        cast(&mut g, sweet, false, &[], false, &mut t);
        assert_eq!(g.object_view(sweet).counters.lore, 2);
        let count = |g: &Game, subtype: &str| {
            g.state
                .objects
                .iter()
                .filter(|v| {
                    v.zone == Zone::Battlefield && v.owner == p && v.card.has_subtype(subtype)
                })
                .count()
        };
        assert_eq!(count(&g, "Food"), 2);
        assert_eq!(count(&g, "Human"), 2);
        next_main(&mut g, p, &mut t);
        assert_eq!(
            g.object_view(sweet).counters.lore,
            3,
            "ordinary turn-based lore is not doubled"
        );
        assert!(!g.saga_sacrifice_required(sweet));
        settle(&mut g, &[familiar], false, &mut t);
        let body = g.object_view(familiar);
        assert_eq!(body.counters.plus_one, 6);
        assert_eq!((body.power, body.toughness), (9, 8));
        assert!(body.keywords.contains(Keywords::FLYING));
        assert!(body.keywords.contains(Keywords::LIFELINK));
        assert_eq!(g.object_view(sweet).zone, Zone::Graveyard);
        assert_eq!(
            t.ward_declined, 1,
            "own Saga chapter must not trigger the recipient's Ward"
        );
        assert_eq!(t.casts, 6);
    }
}

// The Saga is actually paid on its controller's turn. Seeded lands fund every
// subsequent opponent cast/tax after a real turn boundary has drained fixture mana.
fn omen_cross_setup(p: Player, blue: bool) -> (Game, Trace, CardId) {
    let q = p.other();
    let mut g = setup(q);
    let mut t = Trace::default();
    named(&mut g, q, "Bear Cub", Zone::Battlefield);
    for _ in 0..8 {
        named(&mut g, p, "Forest", Zone::Battlefield);
    }
    for _ in 0..6 {
        named(
            &mut g,
            p,
            if blue { "Island" } else { "Swamp" },
            Zone::Battlefield,
        );
    }
    let saga = named(&mut g, q, "Summon: Choco/Mog", Zone::Hand);
    cast(&mut g, saga, false, &[], true, &mut t);
    assert_eq!(g.object_view(saga).counters.lore, 1);
    next_main(&mut g, p, &mut t);
    let aura = named(&mut g, p, "Super Strength", Zone::Hand);
    cast(&mut g, aura, false, &[saga], true, &mut t);
    assert_eq!(g.object_view(aura).attached_to, Some(g.object_ref(saga)));
    (g, t, saga)
}
#[test]
fn paid_omen_against_foreign_warded_saga_shuffles_only_after_success_both_seats() {
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            let (mut g, mut t, saga) = omen_cross_setup(p, true);
            let card = named(
                &mut g,
                p,
                "Dirgur Island Dragon // Skimming Strike",
                Zone::Hand,
            );
            let rng = g.state.rng;
            let drawn = g.state.players[p.index()].cards_drawn_this_turn;
            replay(&mut g, Action::AnnounceOmen { card }, &mut t);
            settle(&mut g, &[saga], pay, &mut t);
            assert_eq!(
                g.object_view(card).zone,
                if pay { Zone::Library } else { Zone::Graveyard }
            );
            assert_eq!(g.object_view(saga).tapped, pay);
            assert_eq!(g.object_view(saga).counters.lore, 1);
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                drawn + u64::from(pay)
            );
            assert_eq!(g.state.rng == rng, !pay);
            assert_eq!(t.ward_paid, usize::from(pay));
            assert_eq!(t.ward_declined, usize::from(!pay));
            assert!(g.state.adventures.is_none());
            assert!(g.state.exile_permissions.is_empty());
        }
    }
}
#[test]
fn paid_purging_normal_etb_removes_saga_lore_only_after_recipient_ward_payment() {
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            let (mut g, mut t, saga) = omen_cross_setup(p, false);
            let card = named(
                &mut g,
                p,
                "Purging Stormbrood // Absorb Essence",
                Zone::Hand,
            );
            cast(&mut g, card, false, &[saga], pay, &mut t);
            assert_eq!(g.object_view(card).zone, Zone::Battlefield);
            assert_eq!(g.object_view(saga).counters.lore, u64::from(!pay));
            assert_eq!(t.ward_paid, usize::from(pay));
            assert_eq!(t.ward_declined, usize::from(!pay));
            next_main(&mut g, p.other(), &mut t);
            assert_eq!(g.object_view(saga).counters.lore, if pay { 1 } else { 2 });
            settle(&mut g, &[], true, &mut t);
            assert_eq!(g.object_view(saga).zone, Zone::Battlefield);
            assert!(g.state.adventures.is_none());
        }
    }
}
