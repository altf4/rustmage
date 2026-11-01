//! Preserved paid rules evidence for the five inherited complete BIG definitions.
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::*;

fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, &[]);
    settle(g);
    id
}
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn finish(g: &mut Game) {
    for _ in 0..12 {
        if g.state.announcing.is_none() {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
            .unwrap();
        checked(g, action);
    }
    panic!("inherited BIG activation did not finish");
}
fn activate(g: &mut Game, card: CardId, target: Option<Target>) {
    checked(
        g,
        Action::AnnounceActivate {
            card,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    if let Some(target) = target {
        checked(g, Action::ChooseTarget { target });
    }
    finish(g);
}
fn resolve_target(g: &mut Game, target: Target) {
    for _ in 0..80 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.announcing.is_none()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        let actions = g.legal_actions();
        let choose = Action::ChooseTarget { target };
        let action = if actions.contains(&choose) {
            choose
        } else if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            actions
                .iter()
                .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
                .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
                .unwrap()
                .clone()
        };
        checked(g, action);
    }
    panic!("inherited BIG trigger did not resolve");
}
fn clues(g: &Game, p: Player) -> usize {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(id, o)| {
            o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && g.has_subtype(*id as CardId, "Clue")
        })
        .count()
}

#[test]
fn big_inherited_harvester_paid_sweep_discard_duration_and_menace_block_legality() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let small = named(&mut g, p.other(), "Savannah Lions", Zone::Battlefield);
        let source = paid(&mut g, p, "Harvester of Misery");
        assert_eq!(
            g.stats(source),
            Stats {
                power: 5,
                toughness: 4
            }
        );
        assert_eq!(
            g.stats(own),
            Stats {
                power: 1,
                toughness: 1
            }
        );
        assert_eq!(
            g.stats(enemy),
            Stats {
                power: 1,
                toughness: 1
            }
        );
        assert_eq!(g.object_view(small).zone, Zone::Graveyard);
        assert!(g.keywords(source).contains(Keywords::MENACE));
        next_turn(&mut g);
        next_turn(&mut g);
        assert_eq!(
            g.stats(own),
            Stats {
                power: 3,
                toughness: 3
            }
        );
        assert_eq!(
            g.stats(enemy),
            Stats {
                power: 3,
                toughness: 3
            }
        );
        g.state.players[p.index()].mana = [30; 6];
        let discard = named(&mut g, p, "Harvester of Misery", Zone::Hand);
        activate(&mut g, discard, Some(Target::Card(enemy)));
        assert_eq!(
            g.object_view(discard).zone,
            Zone::Graveyard,
            "discard is paid before resolution"
        );
        settle(&mut g);
        assert_eq!(
            g.stats(enemy),
            Stats {
                power: 1,
                toughness: 1
            }
        );
        assert_eq!(
            g.stats(own),
            Stats {
                power: 3,
                toughness: 3
            }
        );
        let second = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: source });
        checked(&mut g, Action::FinishAttackers);
        for _ in 0..8 {
            if g.state.pending == Pending::Blockers {
                break;
            }
            checked(&mut g, Action::Pass);
        }
        checked(
            &mut g,
            Action::Block {
                blocker: enemy,
                attacker: source,
            },
        );
        reject(&mut g, Action::FinishBlockers);
        checked(
            &mut g,
            Action::Block {
                blocker: second,
                attacker: source,
            },
        );
        checked(&mut g, Action::FinishBlockers);
    }
}

#[test]
fn big_inherited_investigator_opponent_discard_clue_cost_and_once_each_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let secret = named(&mut g, p.other(), "Hill Giant", Zone::Hand);
        named(&mut g, p.other(), "Savannah Lions", Zone::Hand);
        let before = g.hand_count(p.other());
        let investigator = paid(&mut g, p, "Hostile Investigator");
        assert_eq!(
            g.stats(investigator),
            Stats {
                power: 4,
                toughness: 3
            }
        );
        assert_eq!(g.hand_count(p.other()), before - 1);
        assert_eq!(clues(&g, p), 1);
        if g.object_view(secret).zone == Zone::Hand {
            assert!(!g.observe(p).objects.iter().any(|o| o.id == secret));
        }
        let cycle = named(&mut g, p, "Pest Control", Zone::Hand);
        activate(&mut g, cycle, None);
        settle(&mut g);
        assert_eq!(
            clues(&g, p),
            1,
            "a second discard cannot retrigger this turn"
        );
        let clue = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(id, o)| {
                o.zone == Zone::Battlefield
                    && o.controller == p
                    && g.has_subtype(*id as CardId, "Clue")
            })
            .unwrap()
            .0 as CardId;
        let cards = g.hand_count(p);
        activate(&mut g, clue, None);
        settle(&mut g);
        assert_eq!(clues(&g, p), 0);
        assert_eq!(g.hand_count(p), cards + 1);
        next_turn(&mut g);
        g.state.players[p.index()].mana = [30; 6];
        checked(&mut g, Action::Pass);
        let next_cycle = named(&mut g, p, "Pest Control", Zone::Hand);
        activate(&mut g, next_cycle, None);
        settle(&mut g);
        assert_eq!(
            clues(&g, p),
            1,
            "the opponent's turn has a fresh trigger allowance"
        );
    }
}

#[test]
fn big_inherited_extruder_any_target_and_paid_other_artifact_golem() {
    for p in [Player::P0, Player::P1] {
        for creature_target in [false, true] {
            let mut g = setup(p);
            let victim = named(&mut g, p.other(), "Savannah Lions", Zone::Battlefield);
            let offering = named(&mut g, p, "Hedron Archive", Zone::Battlefield);
            let foreign = named(&mut g, p.other(), "Hedron Archive", Zone::Battlefield);
            let source = named(&mut g, p, "Legion Extruder", Zone::Hand);
            cast(&mut g, source, None, &[]);
            let target = if creature_target {
                Target::Card(victim)
            } else {
                Target::Player(p.other())
            };
            resolve_target(&mut g, target);
            if creature_target {
                assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
            } else {
                assert_eq!(g.state.players[p.other().index()].life, 18);
            }
            let pool = g.state.players[p.index()].mana.iter().sum::<u32>();
            checked(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            let self_ref = g.object_ref(source);
            let foreign_ref = g.object_ref(foreign);
            let offering_ref = g.object_ref(offering);
            reject(&mut g, Action::ChooseAdditionalCost { object: self_ref });
            reject(
                &mut g,
                Action::ChooseAdditionalCost {
                    object: foreign_ref,
                },
            );
            checked(
                &mut g,
                Action::ChooseAdditionalCost {
                    object: offering_ref,
                },
            );
            finish(&mut g);
            assert_eq!(g.object_view(offering).zone, Zone::Graveyard);
            assert!(g.object_view(source).tapped);
            assert_eq!(
                g.state.players[p.index()].mana.iter().sum::<u32>(),
                pool - 2
            );
            settle(&mut g);
            let golems: Vec<_> = g
                .state
                .objects
                .iter()
                .enumerate()
                .filter(|(id, o)| {
                    o.zone == Zone::Battlefield
                        && o.controller == p
                        && o.is_token()
                        && g.has_subtype(*id as CardId, "Golem")
                })
                .map(|(id, _)| id as CardId)
                .collect();
            assert_eq!(golems.len(), 1);
            let token = g.object_view(golems[0]);
            assert_eq!(
                (token.power, token.toughness, token.colors, token.kinds & 17),
                (3, 3, 0, 17)
            );
        }
    }
}

#[test]
fn big_inherited_pest_control_nonland_threshold_both_seats_and_cycling_two() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let mut removed = Vec::new();
        let mut preserved = Vec::new();
        for owner in [p, p.other()] {
            for name in ["Savannah Lions", "Mox Jasper", "Authority of the Consuls"] {
                removed.push(named(&mut g, owner, name, Zone::Battlefield));
            }
            for name in ["Forest", "Hill Giant", "Hedron Archive"] {
                preserved.push(named(&mut g, owner, name, Zone::Battlefield));
            }
        }
        paid(&mut g, p, "Pest Control");
        assert!(
            removed
                .iter()
                .all(|&id| g.object_view(id).zone == Zone::Graveyard)
        );
        assert!(
            preserved
                .iter()
                .all(|&id| g.object_view(id).zone == Zone::Battlefield)
        );
        let survivor = named(&mut g, p, "Savannah Lions", Zone::Battlefield);
        let cycle = named(&mut g, p, "Pest Control", Zone::Hand);
        let hand = g.hand_count(p);
        let pool = g.state.players[p.index()].mana.iter().sum::<u32>();
        activate(&mut g, cycle, None);
        assert_eq!(g.object_view(cycle).zone, Zone::Graveyard);
        assert_eq!(
            g.state.players[p.index()].mana.iter().sum::<u32>(),
            pool - 2
        );
        settle(&mut g);
        assert_eq!(g.hand_count(p), hand);
        assert_eq!(
            g.object_view(survivor).zone,
            Zone::Battlefield,
            "cycling never casts the destruction spell"
        );
    }
}

#[test]
fn big_inherited_torpor_orb_suppresses_creature_entries_and_other_listeners_only() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let orb = paid(&mut g, p, "Torpor Orb");
        let listener = named(&mut g, p, "Hinterland Sanctifier", Zone::Battlefield);
        let creature = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let life = g.state.players[p.index()].life;
        let harvester = paid(&mut g, p, "Harvester of Misery");
        assert_eq!(g.object_view(listener).zone, Zone::Battlefield);
        assert_eq!(
            g.stats(creature),
            Stats {
                power: 3,
                toughness: 3
            }
        );
        assert_eq!(
            g.state.players[p.index()].life,
            life,
            "another creature's entry listener is suppressed"
        );
        let secret = named(&mut g, p.other(), "Savannah Lions", Zone::Hand);
        paid(&mut g, p, "Hostile Investigator");
        assert_eq!(g.object_view(secret).zone, Zone::Hand);
        let extruder = named(&mut g, p, "Legion Extruder", Zone::Hand);
        cast(&mut g, extruder, None, &[]);
        resolve_target(&mut g, Target::Player(p.other()));
        assert_eq!(
            g.state.players[p.other().index()].life,
            18,
            "noncreature artifact entry still triggers"
        );
        let removal = named(&mut g, p, "Disenchant", Zone::Hand);
        cast(&mut g, removal, None, &[Target::Card(orb)]);
        settle(&mut g);
        assert_eq!(g.object_view(orb).zone, Zone::Graveyard);
        paid(&mut g, p, "Savannah Lions");
        assert_eq!(g.state.players[p.index()].life, life + 1);
        assert_eq!(
            g.stats(harvester),
            Stats {
                power: 5,
                toughness: 4
            }
        );
    }
}
