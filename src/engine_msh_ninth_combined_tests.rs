//! Seven whole candidates combine exact return, secondary LKI, layers and Saga costs.
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::*;
#[track_caller]
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..220 {
        if stop(g) {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                )
            })
            .unwrap_or_else(|| panic!("unexpected {:?} at {:?}", g.state.pending, g.state.step));
        checked(g, a);
    }
    panic!("combined stalled {:?}", g.state.pending);
}
fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn finish(g: &mut Game) {
    while g.state.announcing.is_some() {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .unwrap();
        checked(g, a);
    }
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    priority(g, p);
    let id = named(g, p, name, Zone::Hand);
    let (colored, generic) = g.object_view(id).card.cost();
    g.state.players[p.index()].mana = colored;
    g.state.players[p.index()].mana[5] += generic;
    cast(g, id, None, targets);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    settle(g);
    id
}
fn activate(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}
fn bots(g: &Game, p: Player) -> Vec<ObjectRef> {
    (0..g.state.objects.len() as CardId)
        .filter(|&id| {
            let o = g.object_view(id);
            o.zone == Zone::Battlefield && o.controller == p && o.card == Card::MshDoombotToken
        })
        .map(|id| g.object_ref(id))
        .collect()
}
fn treasures(g: &Game, p: Player) -> Vec<CardId> {
    (0..g.state.objects.len() as CardId)
        .filter(|&id| {
            let o = g.object_view(id);
            o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && g.has_subtype(id, "Treasure")
        })
        .collect()
}
fn orphan(g: Game, a: Action) -> Game {
    let actor = g.actor().unwrap();
    let before = g.snapshot();
    let revision = g.revision();
    let index = g.legal_actions().iter().position(|v| *v == a).unwrap();
    let (mut raw, mut indexed) = (g.clone(), g.clone());
    drop(g);
    raw.apply(actor, a.clone()).unwrap();
    indexed.apply_index(actor, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    let after = raw.snapshot();
    raw.restore(&before).unwrap();
    raw.apply(actor, a).unwrap();
    assert_eq!(raw.snapshot(), after);
    indexed
}
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
#[test]
fn msh_ninth_combined_paid_saga_villains_artifact_costs_animation_return_and_event_lki() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let doom = paid(&mut g, p, "Doctor Doom", &[]);
        assert_eq!(bots(&g, p).len(), 2);
        assert!(g.keywords(doom).contains(Keywords::INDESTRUCTIBLE));
        let hulk = paid(&mut g, p, "Hulkling, Burgeoning Bruiser", &[]);
        let ares = paid(&mut g, p, "Ares, God of War", &[]);
        assert_eq!(g.counter_count(hulk, CounterKind::PlusOne), 1);
        let castle = named(&mut g, p, "Castle Doom", Zone::Hand);
        checked(&mut g, Action::PlayLand { card: castle });
        g.state.players[p.index()].mana = [0; 6];
        checked(
            &mut g,
            Action::ActivateMana {
                card: castle,
                ability: 1,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 1, 0, 0],
            },
        );
        let bow = named(&mut g, p, "Hawkeye's Bow", Zone::Hand);
        cast(&mut g, bow, None, &[]);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        checked(&mut g, activate(bow, 0));
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(hulk),
            },
        );
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.stats(hulk).power, 4);
        let saga = paid(&mut g, p, "Avengers: Under Siege", &[]);
        let car = paid(&mut g, p, "S.H.I.E.L.D. Flying Car", &[]);
        assert_eq!(g.counter_count(saga, CounterKind::Lore), 1);
        assert_eq!(g.counter_count(hulk, CounterKind::PlusOne), 1);
        assert_eq!(treasures(&g, p).len(), 0);
        next_turn(&mut g);
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.step == Step::Main1
                && g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
        });
        assert_eq!(g.state.active, p);
        assert_eq!(g.counter_count(saga, CounterKind::Lore), 2);
        assert_eq!(g.object_view(hulk).damage, 2);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert_eq!(g.state.players[p.index()].life, 19);
        for object in bots(&g, p) {
            assert_eq!(g.object_view(object.card).damage, 0);
        }
        assert_eq!(g.object_view(ares).damage, 0);
        assert_eq!(g.object_view(doom).damage, 0);
        let old_bot = bots(&g, p)[0];
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        checked(&mut g, activate(castle, 2));
        g = orphan(g, Action::ChooseAdditionalCost { object: old_bot });
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(bots(&g, p).len(), 2);
        assert!(!bots(&g, p).contains(&old_bot));
        assert!(g.keywords(doom).contains(Keywords::INDESTRUCTIBLE));
        paid(&mut g, p, "I Am Iron Man", &[Target::Card(hulk)]);
        assert_eq!((g.stats(hulk).power, g.stats(hulk).toughness), (6, 5));
        assert!(
            g.keywords(hulk).contains(
                Keywords::FLYING
                    .union(Keywords::VIGILANCE)
                    .union(Keywords::REACH)
            )
        );
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        for card in [ares, hulk] {
            checked(&mut g, Action::Attack { card });
        }
        checked(&mut g, Action::FinishAttackers);
        assert!(!g.object_view(hulk).tapped);
        priority(&mut g, p);
        checked(&mut g, activate(car, 0));
        let object = g.object_ref(hulk);
        checked(&mut g, Action::ChooseCrewer { object });
        checked(&mut g, Action::FinishCrew);
        until(&mut g, |g| {
            g.state.pending == Pending::Priority && g.state.stack.iter().any(|a| a.card == bow)
        });
        assert!(g.object_view(hulk).tapped);
        priority(&mut g, p);
        let murder = named(&mut g, p, "Murder", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 2, 0, 0, 1];
        cast(&mut g, murder, None, &[Target::Card(hulk)]);
        until(&mut g, |g| g.object_view(hulk).zone == Zone::Graveyard);
        g.prepare_triggers();
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| !matches!(a, Action::Concede))
            .unwrap();
        g = orphan(g, action);
        until(&mut g, |g| g.object_view(hulk).zone == Zone::Hand);
        assert!(g.state.stack.iter().any(|a| a.card == bow));
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 17);
        assert!(g.keywords(doom).contains(Keywords::INDESTRUCTIBLE));
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 2];
        reject(&mut g, Action::AnnounceCast { card: hulk });
        next_turn(&mut g);
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.step == Step::Main1
                && g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
        });
        assert_eq!(g.object_view(saga).zone, Zone::Graveyard);
        assert_eq!(treasures(&g, p).len(), 6);
        assert_eq!(g.state.players[p.index()].life, 18);
        assert_eq!(g.state.players[p.other().index()].life, 13);
        let treasure = treasures(&g, p)[0];
        g.state.players[p.index()].mana = [0; 6];
        checked(
            &mut g,
            Action::ActivateMana {
                card: treasure,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 1, 0, 0],
            },
        );
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, None, &[Target::Player(p.other())]);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
        assert_eq!(treasures(&g, p).len(), 5);
        assert_eq!(g.state.players[p.other().index()].life, 11);
    }
}
