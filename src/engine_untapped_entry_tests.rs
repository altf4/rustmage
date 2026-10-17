//! Paid whole-body and event-local replacement-order controls.
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
    let action = if let Some(effect) = g
        .entry_tap_view(g.actor().unwrap())
        .and_then(|v| v.effects.first().copied())
    {
        Action::ChooseEntryReplacement {
            source: effect.source,
            intrinsic: effect.intrinsic,
        }
    } else if actions.contains(&Action::ChooseEntryLife { pay: false }) {
        Action::ChooseEntryLife { pay: false }
    } else if let Some(id) = land.filter(|id| {
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

fn stage(g: &mut Game, p: Player, id: CardId, tapped: bool) {
    let mut movement = g.entry_move(id, Zone::Battlefield, tapped);
    movement.controller = p;
    g.stage_entries(vec![movement], None);
    g.advance_entries();
}
fn replace(g: &mut Game, source: CardId, intrinsic: bool) {
    checked(
        g,
        Action::ChooseEntryReplacement {
            source: g.object_ref(source),
            intrinsic,
        },
    );
}
#[test]
fn paid_spelunking_draw_optional_actual_cave_bonus_and_decline() {
    for p in [Player::P0, Player::P1] {
        for name in [Some("Captivating Cave"), Some("Forest"), None] {
            let mut g = setup(p);
            let land = name.map(|n| named(&mut g, p, n, Zone::Hand));
            let hand = g.hand_count(p);
            let life = g.state.players[p.index()].life;
            let source = paid(&mut g, p, "Spelunking", land);
            assert_eq!(g.object_view(source).zone, Zone::Battlefield);
            assert_eq!(g.hand_count(p), hand + 1 - usize::from(land.is_some()));
            assert_eq!(
                g.state.players[p.index()].life,
                life + if name == Some("Captivating Cave") {
                    4
                } else {
                    0
                }
            );
            if let Some(id) = land {
                assert_eq!(g.object_view(id).zone, Zone::Battlefield);
                assert!(!g.object_view(id).tapped);
            }
        }
    }
}
#[test]
fn paid_spelunking_intrinsic_order_is_not_an_unconditional_boolean_override() {
    for p in [Player::P0, Player::P1] {
        for untapped_last in [false, true] {
            let mut g = setup(p);
            let source = paid(&mut g, p, "Spelunking", None);
            let land = named(&mut g, p, "Baron, Airship Kingdom", Zone::Hand);
            checked(&mut g, Action::PlayLand { card: land });
            assert!(matches!(g.state.pending,Pending::EntryReplacement {player} if player==p));
            assert_eq!(g.object_view(land).zone, Zone::Hand);
            let first = if untapped_last { land } else { source };
            replace(&mut g, first, untapped_last);
            drive(&mut g, None, 0, None);
            assert_eq!(g.object_view(land).zone, Zone::Battlefield);
            assert_eq!(g.object_view(land).tapped, !untapped_last);
            assert!(g.state.land_played);
            // Both used source references are no longer legal choices.
            let before = g.snapshot();
            assert!(
                g.apply(
                    g.actor().unwrap(),
                    Action::ChooseEntryReplacement {
                        source: g.object_ref(source),
                        intrinsic: false
                    }
                )
                .is_err()
            );
            assert_eq!(g.snapshot(), before);
        }
    }
}
#[test]
fn paid_spelunking_shockland_payment_can_precede_or_follow_untapped_replacement() {
    for p in [Player::P0, Player::P1] {
        for untapped_last in [false, true] {
            for pay in [false, true] {
                let mut g = setup(p);
                let source = paid(&mut g, p, "Spelunking", None);
                let land = named(&mut g, p, "Breeding Pool", Zone::Hand);
                let life = g.state.players[p.index()].life;
                checked(&mut g, Action::PlayLand { card: land });
                replace(
                    &mut g,
                    if untapped_last { land } else { source },
                    untapped_last,
                );
                assert!(matches!(g.state.pending, Pending::EntryLife { .. }));
                assert_eq!(g.object_view(land).zone, Zone::Hand);
                checked(&mut g, Action::ChooseEntryLife { pay });
                drive(&mut g, None, 0, None);
                assert_eq!(
                    g.state.players[p.index()].life,
                    life - if pay { 2 } else { 0 }
                );
                assert_eq!(g.object_view(land).tapped, !untapped_last && !pay);
            }
        }
    }
}
#[test]
fn untapped_replacement_changes_instruction_event_but_not_a_simultaneously_entering_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, "Spelunking", None);
        let land = named(&mut g, p, "Forest", Zone::Hand);
        stage(&mut g, p, land, true);
        drive(&mut g, None, 0, None);
        assert!(!g.object_view(land).tapped);
        let same = named(&mut g, p, "Spelunking", Zone::Hand);
        let tapped = named(&mut g, p, "Baron, Airship Kingdom", Zone::Hand);
        // Remove the old source before staging a simultaneous source+land event.
        g.move_object(source, Zone::Graveyard);
        g.stage_entries(
            vec![
                g.entry_move(same, Zone::Battlefield, false),
                g.entry_move(tapped, Zone::Battlefield, false),
            ],
            None,
        );
        g.advance_entries();
        drive(&mut g, None, 0, None);
        assert!(g.object_view(tapped).tapped);
        assert_eq!(g.object_view(same).zone, Zone::Battlefield);
    }
}
#[test]
fn paid_minstrel_full_threshold_all_colors_and_other_creature_town_snapshot() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, "The Wandering Minstrel", None);
        let ally = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        for _ in 0..4 {
            named(&mut g, p, "Capital City", Zone::Battlefield);
        }
        g.begin_step(Step::BeginCombat);
        g.give_priority();
        drive(&mut g, None, 0, None);
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Battlefield
                    && o.card.is_token()
                    && o.card.has_subtype("Elemental"))
                .count(),
            0
        );
        named(&mut g, p, "Capital City", Zone::Battlefield);
        g.begin_step(Step::BeginCombat);
        g.give_priority();
        drive(&mut g, None, 0, None);
        let elementals: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| {
                o.zone == Zone::Battlefield && o.card.is_token() && o.card.has_subtype("Elemental")
            })
            .map(|(i, _)| i as CardId)
            .collect();
        assert_eq!(elementals.len(), 1);
        assert_eq!(g.state.objects[elementals[0] as usize].card.colors(), 31);
        // A real, fully paid activated program snapshots five controlled Towns.
        g.state.players[p.index()].mana = [40; 6];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        drive(&mut g, None, 0, None);
        assert_eq!(g.object_view(source).power, 1);
        assert_eq!(g.object_view(ally).power, 7);
        assert_eq!(g.object_view(enemy).power, 2);
        assert_eq!(g.object_view(elementals[0]).power, 7);
        g.begin_step(Step::End);
        g.give_priority();
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert_eq!(g.object_view(ally).power, 2);
    }
}

#[test]
fn multiple_untapped_sources_are_once_per_event_and_follow_prospective_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let first = paid(&mut g, p, "Spelunking", None);
        let second = paid(&mut g, p, "The Wandering Minstrel", None);
        let land = named(&mut g, p.other(), "Baron, Airship Kingdom", Zone::Hand);
        stage(&mut g, p, land, false);
        replace(&mut g, first, false);
        replace(&mut g, land, true);
        drive(&mut g, None, 0, None);
        assert!(!g.object_view(land).tapped);
        assert_eq!(g.object_view(land).controller, p);
        assert_eq!(g.object_view(land).owner, p.other());
        // Both static sources now belong to the opponent; owner is immaterial.
        g.state.objects[first as usize].controller = p.other();
        g.state.objects[second as usize].controller = p.other();
        let next = named(&mut g, p, "Baron, Airship Kingdom", Zone::Hand);
        stage(&mut g, p, next, false);
        drive(&mut g, None, 0, None);
        assert!(g.object_view(next).tapped);
        let theirs = named(&mut g, p, "Baron, Airship Kingdom", Zone::Hand);
        stage(&mut g, p.other(), theirs, true);
        // The actual incoming controller chooses its intrinsic replacement first.
        replace(&mut g, theirs, true);
        drive(&mut g, None, 0, None);
        assert!(!g.object_view(theirs).tapped);
    }
}
#[test]
fn minstrel_intervening_town_condition_and_actual_entry_completion_reject_failed_moves() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "The Wandering Minstrel", None);
        let mut towns = Vec::new();
        for _ in 0..5 {
            towns.push(named(&mut g, p, "Capital City", Zone::Battlefield));
        }
        g.begin_step(Step::BeginCombat);
        g.give_priority();
        g.prepare_triggers();
        g.move_object(towns[0], Zone::Graveyard);
        drive(&mut g, None, 0, None);
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Battlefield && o.card.is_token())
                .count(),
            0
        );
        // Explicit transaction-boundary fixture: a replacement redirected the
        // selected Cave away from the battlefield, so no success bonus is earned.
        let cave = named(&mut g, p, "Captivating Cave", Zone::Hand);
        let life = g.state.players[p.index()].life;
        let mut movement = g.entry_move(cave, Zone::Exile, false);
        movement.life_bonus = Some((p, "Cave", 4));
        g.stage_entries(vec![movement], None);
        g.advance_entries();
        drive(&mut g, None, 0, None);
        assert_eq!(g.object_view(cave).zone, Zone::Exile);
        assert_eq!(g.state.players[p.index()].life, life);
    }
}

#[test]
fn paid_foreign_spelunking_keeps_draw_land_cave_bonus_with_actual_resolver() {
    for p in [Player::P0, Player::P1] {
        let (mut g, [source, _, _]) =
            super::control_tests::korvold_grants_for(p, ["Spelunking", "Forest", "Forest"]);
        let cave = named(&mut g, p, "Captivating Cave", Zone::Hand);
        let foreign = named(&mut g, p.other(), "Captivating Cave", Zone::Hand);
        g.state.players[p.index()].mana = [80; 6];
        let life = g.state.players[p.index()].life;
        g.state.land_played = true;
        checked(&mut g, Action::AnnounceExile { card: source });
        for _ in 0..200 {
            if g.state
                .resolving
                .as_ref()
                .and_then(|r| r.choice.as_ref())
                .is_some()
            {
                break;
            }
            next(&mut g, None, 0, None);
        }
        let invalid = Action::PutHandLand {
            object: g.object_ref(foreign),
        };
        let saved = g.snapshot();
        assert!(g.apply(p, invalid).is_err());
        assert_eq!(saved, g.snapshot());
        drive(&mut g, Some(cave), 0, None);
        assert_eq!(g.object_view(source).owner, p.other());
        assert_eq!(g.object_view(source).controller, p);
        assert_eq!(g.object_view(cave).zone, Zone::Battlefield);
        assert_eq!(g.object_view(foreign).zone, Zone::Hand);
        assert_eq!(g.state.players[p.index()].life, life + 4);
        assert!(g.state.land_played);
    }
}
#[test]
fn actual_cave_bonus_capacity_failure_rolls_back_entry_and_owned_continuation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let cave = named(&mut g, p, "Captivating Cave", Zone::Hand);
        let source = named(&mut g, p, "Spelunking", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: source });
        for _ in 0..200 {
            if g.state
                .resolving
                .as_ref()
                .and_then(|r| r.choice.as_ref())
                .is_some()
            {
                break;
            }
            next(&mut g, None, 0, None);
        }
        g.state.players[p.index()].life = i128::MAX;
        let before = g.snapshot();
        let rev = g.revision();
        assert!(matches!(
            g.apply(
                p,
                Action::PutHandLand {
                    object: g.object_ref(cave)
                }
            ),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(before, g.snapshot());
        assert_eq!(g.revision(), rev);
        assert_eq!(g.object_view(cave).zone, Zone::Hand);
        assert!(g.observe(p).effect_choice.is_some());
    }
}
