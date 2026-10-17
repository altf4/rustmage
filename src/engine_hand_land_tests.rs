//! Paid private hand-land choices, real entries and exact public explore predicates.
use super::tests::{scenario, spawn};
use super::*;
fn named(g: &mut Game, p: Player, n: &str, z: Zone) -> CardId {
    let c = Card::from_name(n).unwrap();
    assert!(c.is_supported(), "{n}");
    spawn(g, p, c, z)
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for q in [p, p.other()] {
        g.state.players[q.index()].mana = [40; 6];
    }
    g
}
fn checked(g: &mut Game, action: Action) {
    let actor = g.actor().unwrap();
    let revision = g.revision();
    let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
    let before = g.snapshot();
    assert_eq!(
        g.apply(actor.other(), action.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(g.snapshot(), before);
    let mut branch = {
        let parent = g.clone();
        parent.clone()
    };
    branch.apply_index(actor, revision, index).unwrap();
    g.apply(actor, action.clone()).unwrap();
    assert_eq!(g.snapshot(), branch.snapshot());
    for player in [Player::P0, Player::P1] {
        assert_eq!(
            serde_json::to_value(g.observe(player)).unwrap(),
            serde_json::to_value(branch.observe(player)).unwrap()
        );
    }
    let after = g.snapshot();
    g.restore(&before).unwrap();
    assert_eq!(
        g.apply_index(actor, revision, index),
        Err(Error::StaleRevision)
    );
    g.apply(actor, action).unwrap();
    assert_eq!(g.snapshot(), after);
}

fn next(g: &mut Game, land: Option<CardId>, mode: u16, target: Option<Target>) {
    let actions = g.legal_actions();
    let action = if let Some(id) = land.filter(|id| {
        actions.contains(&Action::PutHandLand {
            object: g.object_ref(*id),
        })
    }) {
        Action::PutHandLand {
            object: g.object_ref(id),
        }
    } else if let Some(target) =
        target.filter(|t| actions.contains(&Action::ChooseTarget { target: *t }))
    {
        Action::ChooseTarget { target }
    } else if actions.contains(&Action::ChooseMode { mode }) {
        Action::ChooseMode { mode }
    } else if actions.contains(&Action::DeclineHandEntry) {
        Action::DeclineHandEntry
    } else if g.state.pending == Pending::Priority {
        Action::Pass
    } else {
        actions
            .iter()
            .find(|a| {
                matches!(
                    a,
                    Action::PayCosts { .. }
                        | Action::FinishTargets
                        | Action::ChooseExplore {
                            graveyard: false,
                            ..
                        }
                )
            })
            .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
            .unwrap()
            .clone()
    };
    checked(g, action);
}
fn settled(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.resolving.is_none()
        && g.state.announcing.is_none()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
}
fn drive(g: &mut Game, land: Option<CardId>, mode: u16, target: Option<Target>) {
    g.prepare_triggers();
    for _ in 0..300 {
        if settled(g) {
            return;
        }
        next(g, land, mode, target)
    }
    panic!("pending {:?}", g.state.pending)
}
fn paid(g: &mut Game, p: Player, n: &str, land: Option<CardId>) -> CardId {
    for _ in 0..10 {
        if g.actor() == Some(p) && g.state.pending == Pending::Priority {
            break;
        }
        next(g, None, 0, None)
    }
    g.state.players[p.index()].mana = [40; 6];
    let id = named(g, p, n, Zone::Hand);
    checked(g, Action::AnnounceCast { card: id });
    drive(g, land, 0, None);
    id
}
fn reach_land_choice(g: &mut Game) {
    for _ in 0..300 {
        if g.state
            .resolving
            .as_ref()
            .and_then(|r| r.choice.as_ref())
            .is_some_and(|c| matches!(c.kind, EffectChoiceKind::HandLand { .. }))
        {
            return;
        }
        next(g, None, 0, None)
    }
    panic!("no land choice")
}
#[test]
fn hand_land_paid_complete_draw_spells_and_etb_apply_real_tapped_entry_without_playing() {
    for p in [Player::P0, Player::P1] {
        for name in [
            "Embrace the Paradox",
            "Lessons from Life",
            "H.E.R.B.I.E. Scout Unit",
        ] {
            for decline in [false, true] {
                let mut g = setup(p);
                for _ in 0..4 {
                    named(&mut g, p, "Shock", Zone::Library);
                }
                let land = named(&mut g, p, "Forest", Zone::Hand);
                g.state.land_played = true;
                let casts = g.state.players[p.index()].spells_cast_this_turn;
                let before = g.state.players[p.index()].cards_drawn_this_turn;
                paid(&mut g, p, name, if decline { None } else { Some(land) });
                assert_eq!(
                    g.state.players[p.index()].cards_drawn_this_turn - before,
                    if name == "H.E.R.B.I.E. Scout Unit" {
                        1
                    } else {
                        3
                    }
                );
                assert_eq!(g.state.players[p.index()].spells_cast_this_turn, casts + 1);
                assert!(g.state.land_played);
                assert_eq!(
                    g.object_view(land).zone,
                    if decline {
                        Zone::Hand
                    } else {
                        Zone::Battlefield
                    }
                );
                if !decline {
                    assert!(g.object_view(land).tapped);
                    assert_eq!(g.object_view(land).controller, p);
                }
            }
        }
    }
}
#[test]
fn hand_land_private_empty_eligibility_and_decline_have_identical_peer_views() {
    for p in [Player::P0, Player::P1] {
        let mut games = Vec::new();
        for eligible in [false, true] {
            let mut g = setup(p);
            for _ in 0..3 {
                named(&mut g, p, "Shock", Zone::Library);
            }
            named(
                &mut g,
                p,
                if eligible { "Forest" } else { "Giant Growth" },
                Zone::Hand,
            );
            let spell = named(&mut g, p, "Embrace the Paradox", Zone::Hand);
            checked(&mut g, Action::AnnounceCast { card: spell });
            reach_land_choice(&mut g);
            assert_eq!(g.actor(), Some(p));
            assert!(g.legal_actions().contains(&Action::DeclineHandEntry));
            assert!(g.observe(p.other()).effect_choice.is_none());
            assert_eq!(
                g.observe(p)
                    .effect_choice
                    .as_ref()
                    .unwrap()
                    .candidates
                    .len(),
                usize::from(eligible)
            );
            games.push(g);
        }
        assert_eq!(
            serde_json::to_value(games[0].observe(p.other())).unwrap(),
            serde_json::to_value(games[1].observe(p.other())).unwrap()
        );
        for mut g in games {
            checked(&mut g, Action::DeclineHandEntry);
            drive(&mut g, None, 0, None);
            assert!(settled(&g));
        }
    }
}
#[test]
fn hand_land_insidious_fungus_is_one_paid_modal_activation_and_all_three_modes_complete() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = setup(p);
            named(&mut g, p, "Forest", Zone::Library);
            let land = named(&mut g, p, "Island", Zone::Hand);
            let victim = named(
                &mut g,
                p.other(),
                if mode == 0 {
                    "Goldvein Pick"
                } else {
                    "Doubling Season"
                },
                Zone::Battlefield,
            );
            let source = paid(&mut g, p, "Insidious Fungus", None);
            assert_eq!(g.state.objects[source as usize].card.activations().len(), 1);
            checked(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            assert_eq!(g.object_view(source).zone, Zone::Battlefield);
            assert!(g.legal_actions().contains(&Action::ChooseMode { mode }));
            checked(&mut g, Action::ChooseMode { mode });
            drive(
                &mut g,
                Some(land),
                mode,
                if mode < 2 {
                    Some(Target::Card(victim))
                } else {
                    None
                },
            );
            assert_eq!(g.object_view(source).zone, Zone::Graveyard);
            if mode < 2 {
                assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
                assert_eq!(g.object_view(land).zone, Zone::Hand)
            } else {
                assert_eq!(g.object_view(land).zone, Zone::Battlefield);
                assert!(g.object_view(land).tapped);
                assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1)
            }
        }
    }
}
#[test]
fn hand_land_nicanzil_binds_actual_revealed_kind_empty_is_neither_and_can_put_another_land() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let conductor = paid(&mut g, p, "Nicanzil, Current Conductor", None);
        let other = named(&mut g, p, "Island", Zone::Hand);
        let revealed = named(&mut g, p, "Forest", Zone::Library);
        paid(&mut g, p, "Cenote Scout", Some(other));
        assert_eq!(g.object_view(other).zone, Zone::Battlefield);
        assert!(g.object_view(other).tapped);
        assert_eq!(g.object_view(revealed).zone, Zone::Hand);
        assert_eq!(g.object_view(conductor).counters.plus_one, 0);
        named(&mut g, p, "Shock", Zone::Library);
        paid(&mut g, p, "Cenote Scout", None);
        assert_eq!(g.object_view(conductor).counters.plus_one, 1);
        let library = g.state.players[p.index()].library.clone();
        for id in library {
            g.move_object(id, Zone::Exile);
        }
        paid(&mut g, p, "Cenote Scout", None);
        assert_eq!(g.object_view(conductor).counters.plus_one, 1);
    }
}
