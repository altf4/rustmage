//! Actual paid whole-pair programs, exact mill incarnations and private owned choices.
use super::tests::{scenario, spawn};
use super::*;
const NAMES: [&str; 3] = [
    "Aquatic Alchemist // Bubble Up",
    "Scalding Viper // Steam Clean",
    "Intrepid Trufflesnout // Go Hog Wild",
];
fn act(g: &mut Game, a: Action) {
    let actor = g.actor().unwrap();
    let saved = g.snapshot();
    let rev = g.revision();
    let index = g
        .legal_actions()
        .iter()
        .position(|x| *x == a)
        .unwrap_or_else(|| panic!("illegal {a:?}, {:?}", g.observe(actor)));
    assert_eq!(g.apply(actor.other(), a.clone()), Err(Error::WrongActor));
    assert_eq!(saved, g.snapshot());
    let mut child = {
        let parent = g.clone();
        let c = parent.clone();
        drop(parent);
        c
    };
    child.apply_index(actor, rev, index).unwrap();
    g.apply(actor, a.clone()).unwrap();
    assert_eq!(g.snapshot(), child.snapshot());
    for viewer in [Player::P0, Player::P1] {
        assert_eq!(
            serde_json::to_value(g.observe(viewer)).unwrap(),
            serde_json::to_value(child.observe(viewer)).unwrap()
        );
    }
    let done = g.snapshot();
    g.restore(&saved).unwrap();
    assert_eq!(g.apply_index(actor, rev, index), Err(Error::StaleRevision));
    g.apply(actor, a).unwrap();
    assert_eq!(g.snapshot(), done);
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for player in &mut g.state.players {
        player.mana = [30; 6];
    }
    g
}
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let c = Card::from_name(name).unwrap();
    assert!(c.is_supported(), "{name}");
    spawn(g, p, c, zone)
}
fn choose(g: &mut Game) {
    let actions = g.legal_actions();
    let a = if g.state.pending == Pending::Priority {
        Action::Pass
    } else {
        actions
            .iter()
            .find(|a| matches!(a, Action::FinishTargets))
            .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
            .unwrap()
            .clone()
    };
    act(g, a);
}
fn priority(g: &mut Game, p: Player) {
    for _ in 0..30 {
        if g.actor() == Some(p) && g.state.pending == Pending::Priority {
            return;
        }
        choose(g);
    }
    panic!("priority");
}

fn drive(g: &mut Game, target: Option<Target>) {
    for _ in 0..160 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.announcing.is_none()
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
        {
            return;
        }
        let actions = g.legal_actions();
        let a = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            target
                .and_then(|t| {
                    actions
                        .iter()
                        .find(|a| **a == Action::ChooseTarget { target: t })
                })
                .or_else(|| {
                    actions.iter().find(|a| {
                        matches!(
                            a,
                            Action::FinishTargets
                                | Action::PayCosts { .. }
                                | Action::PutTrigger { .. }
                        )
                    })
                })
                .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
                .unwrap()
                .clone()
        };
        act(g, a);
    }
    panic!("unsettled event game");
}
fn paid(g: &mut Game, p: Player, n: &str, target: Option<Target>) -> CardId {
    priority(g, p);
    let id = named(g, p, n, Zone::Hand);
    act(g, Action::AnnounceCast { card: id });
    drive(g, target);
    id
}
fn food(g: &Game, p: Player) -> usize {
    g.state
        .objects
        .iter()
        .filter(|o| {
            o.zone == Zone::Battlefield
                && o.controller == p
                && o.card.token_info().is_some_and(|t| t.name == "Food Token")
        })
        .count()
}
#[test]
fn qualifying_first_cast_ignores_permanents_counts_preentry_and_resets_both_players() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Bear Cub", None);
        let source = paid(&mut g, p, NAMES[0], None);
        paid(&mut g, p, "Quick Study", None);
        assert_eq!(g.object_view(source).power, 3);
        paid(&mut g, p, "Quick Study", None);
        assert_eq!(g.object_view(source).power, 3);
        assert_eq!(g.state.players[p.index()].instant_sorcery_cast_this_turn, 2);
        // Exercise the actual turn-start reset for BOTH players, including an
        // off-turn caster's history; no source-local ability-use substitute.
        g.state.active = p.other();
        g.begin_turn();
        assert!(
            g.state
                .players
                .iter()
                .all(|s| s.instant_sorcery_cast_this_turn == 0)
        );
        paid(&mut g, p, "Quick Study", None);
        assert_eq!(g.state.players[p.index()].instant_sorcery_cast_this_turn, 1);
        let mut before = setup(p);
        paid(&mut before, p, "Quick Study", None);
        let entered = paid(&mut before, p, NAMES[0], None);
        paid(&mut before, p, "Quick Study", None);
        assert_eq!(before.object_view(entered).power, 1);
    }
}
#[test]
fn chosen_adventure_face_consumes_qualifying_ordinal_while_normal_parent_does_not() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let alchemist = paid(&mut g, p, NAMES[0], None);
        let boar = named(&mut g, p, NAMES[2], Zone::Hand);
        priority(&mut g, p);
        act(&mut g, Action::AnnounceAdventure { card: boar });
        drive(&mut g, Some(Target::Card(alchemist)));
        assert_eq!(g.object_view(alchemist).power, 5);
        assert_eq!(g.state.players[p.index()].instant_sorcery_cast_this_turn, 1);
        priority(&mut g, p);
        act(&mut g, Action::AnnounceExile { card: boar });
        drive(&mut g, None);
        assert_eq!(g.state.players[p.index()].instant_sorcery_cast_this_turn, 1);
        paid(&mut g, p, "Quick Study", None);
        assert_eq!(g.object_view(alchemist).power, 5);
    }
}
#[test]
fn viper_binds_actual_opposing_caster_nontargeted_event_and_source_lki() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, NAMES[1], None);
        let before = g.state.players[p.other().index()].life;
        paid(&mut g, p, "Quick Study", None);
        assert_eq!(g.state.players[p.other().index()].life, before);
        // The opposing instant is legal off turn; the <=3 cast trigger is put
        // above it. Remove the source before that frozen packet resolves.
        priority(&mut g, p.other());
        let shock = named(&mut g, p.other(), "Shock", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: shock });
        while g.state.announcing.is_some() {
            let actions = g.legal_actions();
            let a = actions
                .iter()
                .find(|a| {
                    **a == Action::ChooseTarget {
                        target: Target::Player(p),
                    }
                })
                .or_else(|| {
                    actions
                        .iter()
                        .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
                })
                .unwrap()
                .clone();
            act(&mut g, a);
        }
        assert!(g.state.stack.iter().any(|i| {
            i.card == source
                && i.cast_event
                    .as_ref()
                    .is_some_and(|e| e.controller == p.other())
        }));
        // Exact source incarnation disappears; the already-triggered damage
        // remains attributed to its frozen source/controller and caster.
        g.move_object(source, Zone::Graveyard);
        drive(&mut g, None);
        assert_eq!(g.state.players[p.other().index()].life, before - 1);
        assert_eq!(g.state.objects[source as usize].owner, p);
    }
}
#[test]
fn attack_alone_is_declared_batch_not_live_attackers_and_creates_real_food() {
    for p in [Player::P0, Player::P1] {
        for count in [1, 2] {
            let mut g = setup(p);
            let boar = paid(&mut g, p, NAMES[2], None);
            let ally = paid(&mut g, p, "Bear Cub", None);
            for id in [boar, ally] {
                g.state.objects[id as usize].summoning_sick = false;
            }
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            act(&mut g, Action::Attack { card: boar });
            if count == 2 {
                act(&mut g, Action::Attack { card: ally });
            }
            act(&mut g, Action::FinishAttackers);
            // An event qualification is not an intervening-if: later removing
            // the companion cannot create a trigger; adding an attacker cannot
            // invalidate the already-created lone trigger.
            if count == 2 {
                g.move_object(ally, Zone::Graveyard);
            } else {
                g.state.attackers.push(Attacker {
                    card: ally,
                    defender: Some(CombatDefender::Player(p.other())),
                    blocked: false,
                });
            }
            drive(&mut g, None);
            assert_eq!(food(&g, p), usize::from(count == 1));
            if count == 1 {
                let token = g
                    .state
                    .objects
                    .iter()
                    .position(|o| {
                        o.zone == Zone::Battlefield
                            && o.card.token_info().is_some_and(|t| t.name == "Food Token")
                    })
                    .unwrap() as CardId;
                priority(&mut g, p);
                let life = g.state.players[p.index()].life;
                let a = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a,Action::AnnounceActivate{card,..}if *card==token))
                    .unwrap();
                act(&mut g, a);
                drive(&mut g, None);
                assert_eq!(g.state.players[p.index()].life, life + 3);
                assert_ne!(g.state.objects[token as usize].zone, Zone::Battlefield);
            }
        }
    }
}

fn foreign_grants(actor: Player, names: [&str; 3]) -> (Game, [CardId; 3]) {
    let mut g = setup(actor);
    let cards = names.map(|name| {
        spawn(
            &mut g,
            actor.other(),
            Card::from_name(name).unwrap(),
            Zone::Library,
        )
    });
    // The opponent naturally draws once on each intervening turn before III.
    for _ in 0..2 {
        spawn(&mut g, actor.other(), Card::Forest, Zone::Library);
    }
    let saga = spawn(
        &mut g,
        actor,
        Card::from_name("Korvold and the Noble Thief").unwrap(),
        Zone::Hand,
    );
    act(&mut g, Action::AnnounceCast { card: saga });
    drive(&mut g, None);
    assert_eq!(g.state.objects[saga as usize].counters.lore, 1);
    let mut second = false;
    for _ in 0..1500 {
        if g.state.objects[saga as usize].counters.lore == 2 {
            second = true;
        }
        if cards.iter().all(|&id| g.can_play_exiled(actor, id)) {
            break;
        }
        let actions = g.legal_actions();
        let action = actions
            .iter()
            .find(|a| matches!(a, Action::Pass | Action::FinishTargets))
            .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
            .unwrap()
            .clone();
        act(&mut g, action);
    }
    assert!(second);
    assert_eq!(g.state.objects[saga as usize].zone, Zone::Graveyard);
    assert!(cards.iter().all(|&id| g.can_play_exiled(actor, id)));
    assert!(
        cards
            .iter()
            .all(|&id| !g.can_play_exiled(actor.other(), id))
    );
    assert_eq!(
        g.state
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Battlefield
                && o.controller == actor
                && o.card
                    .token_info()
                    .is_some_and(|t| t.name == "Treasure Token"))
            .count(),
        2
    );
    g.state.players[actor.index()].mana = [30; 6];
    (g, cards)
}

#[test]
fn actual_foreign_permission_casts_both_faces_and_controller_owned_event_effects() {
    for p in [Player::P0, Player::P1] {
        for (i, name) in NAMES.iter().enumerate() {
            let (mut g, [source, _, _]) = foreign_grants(p, [name, "Bear Cub", "Forest"]);
            let target = match i {
                0 => named(&mut g, p, "Shock", Zone::Graveyard),
                _ => named(&mut g, p, "Bear Cub", Zone::Battlefield),
            };
            let target_ref = g.object_ref(target);
            act(&mut g, Action::AnnounceAdventure { card: source });
            drive(&mut g, Some(Target::Card(target)));
            assert_eq!(g.state.objects[source as usize].zone, Zone::Exile);
            assert!(g.can_play_adventure(p, source));
            assert!(!g.can_play_adventure(p.other(), source));
            if i == 0 {
                assert_eq!(g.state.players[p.index()].library.last(), Some(&target));
            }
            if i == 1 {
                assert_eq!(g.state.objects[target as usize].zone, Zone::Hand);
            }
            if i == 2 {
                assert_eq!(g.object_view(target).power, 4);
                assert_eq!(g.object_ref(target), target_ref);
            }
            priority(&mut g, p);
            act(&mut g, Action::AnnounceExile { card: source });
            drive(&mut g, None);
            assert_eq!(g.object_view(source).controller, p);
            assert_eq!(g.object_view(source).owner, p.other());
            if i == 0 {
                // Bubble Up was already the first qualifying cast before this normal entered.
                paid(&mut g, p, "Quick Study", None);
                assert_eq!(g.object_view(source).power, 1);
            } else if i == 1 {
                let before = g.state.players[p.other().index()].life;
                paid(&mut g, p, "Quick Study", None);
                assert_eq!(g.state.players[p.other().index()].life, before);
                paid(&mut g, p.other(), "Quick Study", None);
                assert_eq!(g.state.players[p.other().index()].life, before - 1);
            } else {
                g.state.objects[source as usize].summoning_sick = false;
                g.state.step = Step::Attackers;
                g.state.pending = Pending::Attackers;
                act(&mut g, Action::Attack { card: source });
                act(&mut g, Action::FinishAttackers);
                drive(&mut g, None);
                assert_eq!(food(&g, p), 1);
                assert_eq!(food(&g, p.other()), 0);
                assert!(g.state.objects.iter().any(|o| o.zone == Zone::Battlefield
                    && o.card.token_info().is_some_and(|t| t.name == "Food Token")
                    && o.owner == p
                    && o.controller == p));
            }
        }
    }
}
