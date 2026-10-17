use super::tests::{scenario, spawn};
use super::*;

fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    assert!(card.is_supported(), "{name}");
    spawn(g, p, card, zone)
}
fn checked(g: &mut Game, action: Action) {
    let actor = g.actor().unwrap();
    let revision = g.revision();
    let index = g
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap_or_else(|| panic!("illegal independent {action:?} {:?}", g.state.pending));
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
fn settle_on(g: &mut Game, chosen: Option<Target>) {
    g.prepare_triggers();
    for _ in 0..100 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.announcing.is_none()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        let actions = g.legal_actions();
        let action = if let Some(target) =
            chosen.filter(|t| actions.contains(&Action::ChooseTarget { target: *t }))
        {
            Action::ChooseTarget { target }
        } else if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            actions
                .iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .or_else(|| actions.iter().find(|a| matches!(a, Action::FinishTargets)))
                .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
                .unwrap()
                .clone()
        };
        checked(g, action);
    }
    panic!("unsettled {:?}", g.state.pending);
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for state in &mut g.state.players {
        state.mana = [30; 6];
    }
    g
}

fn settle(g: &mut Game) {
    settle_on(g, None);
}
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let rev = g.revision();
    assert!(g.apply(g.actor().unwrap(), a).is_err());
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), rev);
}
fn cast(g: &mut Game, id: CardId, mode: Option<u16>, targets: &[Target]) {
    checked(g, Action::AnnounceCast { card: id });
    if let Some(mode) = mode {
        reject(g, Action::ChooseMode { mode: 3 });
        checked(g, Action::ChooseMode { mode });
    }
    for &target in targets {
        for _ in 0..3 {
            if g.legal_actions().contains(&Action::ChooseTarget { target }) {
                break;
            }
            checked(g, Action::FinishTargets);
        }
        checked(g, Action::ChooseTarget { target });
    }
    for _ in 0..8 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets))
            .or_else(|| {
                g.legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::PayCosts { .. }))
            })
            .unwrap();
        checked(g, a);
    }
    panic!("unpaid");
}

#[test]
fn live_static_rules_paid_maha_foreign_controller_ward_survives_bounce_with_private_original_payer()
{
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            let q = p.other();
            let mut g = setup(p);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let maha = named(&mut g, p, "Maha, Its Feathers Night", Zone::Hand);
            cast(&mut g, maha, None, &[]);
            settle(&mut g);
            g.state.objects[maha as usize].controller = q;
            assert_eq!(g.stats(bear).toughness, 1);
            let resource = named(&mut g, p, "Forest", Zone::Hand);
            let wrong = named(&mut g, q, "Forest", Zone::Hand);
            let murder = named(&mut g, p, "Murder", Zone::Hand);
            cast(&mut g, murder, None, &[Target::Card(maha)]);
            assert!(g.state.stack.iter().any(|v| v.controller == q
                && v.card == maha
                && matches!(v.kind, StackKind::TriggeredAbility { .. })));
            checked(&mut g, Action::Pass);
            assert_eq!(g.actor(), Some(q));
            let bounce = named(&mut g, q, "Unsummon", Zone::Hand);
            cast(&mut g, bounce, None, &[Target::Card(maha)]);
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            assert_eq!(g.state.objects[maha as usize].zone, Zone::Hand);
            assert_eq!(g.state.objects[maha as usize].owner, p);
            assert_eq!(g.stats(bear).toughness, 2);
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            assert_eq!(g.actor(), Some(p));
            assert!(g.observe(q).effect_choice.is_none());
            assert!(g.observe(q).actions.is_empty());
            reject(&mut g, Action::Discard { card: wrong });
            reject(&mut g, Action::Pass);
            checked(
                &mut g,
                if pay {
                    Action::Discard { card: resource }
                } else {
                    Action::DeclineEffectPayment
                },
            );
            assert_eq!(
                g.state.objects[resource as usize].zone,
                if pay { Zone::Graveyard } else { Zone::Hand }
            );
            assert_eq!(g.state.objects[wrong as usize].zone, Zone::Hand);
            settle(&mut g);
            assert_eq!(g.state.objects[murder as usize].zone, Zone::Graveyard);
            assert!(
                g.observe(p)
                    .live_static_sources
                    .iter()
                    .all(|s| s.source.card != maha)
            );
        }
    }
}
#[test]
fn live_static_rules_paid_burden_current_controller_branch_changes_legal_detective_blocks_without_new_timestamp()
 {
    for p in [Player::P0, Player::P1] {
        let q = p.other();
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let detective = named(&mut g, p, "April, Reporter of the Weird", Zone::Battlefield);
        let attacker = named(&mut g, q, "April, Reporter of the Weird", Zone::Battlefield);
        let ordinary = named(&mut g, q, "Bear Cub", Zone::Battlefield);
        let first = named(&mut g, p, "Burden of Proof", Zone::Hand);
        cast(&mut g, first, None, &[Target::Card(bear)]);
        settle(&mut g);
        let second = named(&mut g, p, "Burden of Proof", Zone::Hand);
        cast(&mut g, second, None, &[Target::Card(detective)]);
        settle(&mut g);
        assert_eq!(
            g.stats(bear),
            Stats {
                power: 1,
                toughness: 1
            }
        );
        assert_eq!(
            g.stats(detective),
            Stats {
                power: 4,
                toughness: 4
            }
        );
        let timestamp = g
            .observe(p)
            .live_static_sources
            .iter()
            .find(|s| s.source.card == second)
            .unwrap()
            .timestamp;
        for _ in 0..200 {
            if g.state.active == q && g.state.pending == Pending::Attackers {
                break;
            }
            let actions = g.legal_actions();
            let action = actions
                .iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::Pass | Action::FinishAttackers | Action::FinishBlockers
                    )
                })
                .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
                .unwrap()
                .clone();
            checked(&mut g, action);
        }
        assert_eq!(g.state.active, q);
        assert_eq!(g.state.pending, Pending::Attackers);
        checked(&mut g, Action::Attack { card: attacker });
        checked(&mut g, Action::Attack { card: ordinary });
        checked(&mut g, Action::FinishAttackers);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert_eq!(g.state.pending, Pending::Blockers);
        reject(
            &mut g,
            Action::Block {
                blocker: bear,
                attacker,
            },
        );
        assert!(g.legal_actions().contains(&Action::Block {
            blocker: bear,
            attacker: ordinary
        }));
        assert!(g.legal_actions().contains(&Action::Block {
            blocker: detective,
            attacker
        }));
        g.state.objects[second as usize].controller = q;
        assert_eq!(g.state.objects[second as usize].owner, p);
        assert_eq!(
            g.stats(detective),
            Stats {
                power: 1,
                toughness: 1
            }
        );
        reject(
            &mut g,
            Action::Block {
                blocker: detective,
                attacker,
            },
        );
        assert_eq!(
            g.observe(q)
                .live_static_sources
                .iter()
                .find(|s| s.source.card == second)
                .unwrap()
                .timestamp,
            timestamp
        );
        g.state.objects[second as usize].controller = p;
        assert_eq!(
            g.stats(detective),
            Stats {
                power: 4,
                toughness: 4
            }
        );
        checked(
            &mut g,
            Action::Block {
                blocker: detective,
                attacker,
            },
        );
        checked(
            &mut g,
            Action::Block {
                blocker: bear,
                attacker: ordinary,
            },
        );
        checked(&mut g, Action::FinishBlockers);
    }
}

#[test]
fn live_static_rules_late_crewed_vehicle_uses_current_type_and_source_membership() {
    for p in [Player::P0, Player::P1] {
        let q = p.other();
        let mut g = setup(q);
        let maha = named(&mut g, p, "Maha, Its Feathers Night", Zone::Battlefield);
        let crewer = named(&mut g, q, "Hill Giant", Zone::Battlefield);
        let vehicle = named(&mut g, q, "Cultivator's Caravan", Zone::Hand);
        cast(&mut g, vehicle, None, &[]);
        settle(&mut g);
        assert!(!g.is_creature(vehicle));
        assert_eq!(g.stats(vehicle), Stats::default());
        let ability = g.state.objects[vehicle as usize]
            .card
            .activations()
            .iter()
            .position(|a| a.crew_minimum().is_some())
            .unwrap() as u16;
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: vehicle,
                ability,
                counter_payment: Counters::default(),
            },
        );
        let object = g.object_ref(crewer);
        checked(&mut g, Action::ChooseCrewer { object });
        checked(&mut g, Action::FinishCrew);
        settle(&mut g);
        assert!(g.is_creature(vehicle));
        assert_eq!(
            g.stats(vehicle),
            Stats {
                power: 5,
                toughness: 1
            }
        );
        if g.actor() != Some(p) {
            checked(&mut g, Action::Pass);
        }
        let bounce = named(&mut g, p, "Unsummon", Zone::Hand);
        cast(&mut g, bounce, None, &[Target::Card(maha)]);
        settle(&mut g);
        assert_eq!(
            g.stats(vehicle),
            Stats {
                power: 5,
                toughness: 5
            }
        );
        assert!(
            g.observe(p)
                .live_static_sources
                .iter()
                .all(|row| row.source.card != maha)
        );
    }
}
#[test]
fn live_static_rules_simultaneous_opposing_sources_follow_apnap_for_conflicting_axes() {
    for active in [Player::P0, Player::P1] {
        let p = Player::P0;
        let q = Player::P1;
        let mut g = setup(q);
        let bearer = named(&mut g, q, "Bear Cub", Zone::Battlefield);
        let aura = named(&mut g, q, "Pacifism", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(bearer)]);
        settle(&mut g);
        let maha = named(&mut g, p, "Maha, Its Feathers Night", Zone::Graveyard);
        let archon = named(&mut g, q, "Archon of the Wild Rose", Zone::Graveyard);
        g.state.active = active;
        g.state.priority = active;
        let before = g.snapshot();
        let mut child = {
            let parent = g.clone();
            parent.clone()
        };
        for state in [&mut g, &mut child] {
            let moves = vec![
                state.entry_move(maha, Zone::Battlefield, false),
                state.entry_move(archon, Zone::Battlefield, false),
            ];
            state.stage_entries(moves, None);
            assert!(state.state.entry_batch.is_none());
        }
        assert_eq!(g.snapshot(), child.snapshot());
        let expected = Stats {
            power: 4,
            toughness: if active == p { 4 } else { 1 },
        };
        assert_eq!(g.stats(bearer), expected);
        assert!(g.keywords(bearer).contains(Keywords::FLYING));
        let after = g.snapshot();
        g.restore(&before).unwrap();
        let moves = vec![
            g.entry_move(archon, Zone::Battlefield, false),
            g.entry_move(maha, Zone::Battlefield, false),
        ];
        g.stage_entries(moves, None);
        assert_eq!(g.stats(bearer), expected);
        // Independent source-vector permutations yield the same selected timestamp outcome.
        for viewer in [p, q] {
            let timestamps = |game: &Game| {
                let mut rows: Vec<_> = game
                    .observe(viewer)
                    .live_static_sources
                    .iter()
                    .map(|row| (row.source, row.timestamp))
                    .collect();
                rows.sort_unstable();
                rows
            };
            assert_eq!(timestamps(&g), timestamps(&child));
        }
        g.restore(&after).unwrap();
        assert_eq!(g.stats(bearer), expected);
    }
}
