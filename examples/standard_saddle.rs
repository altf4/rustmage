//! Reproducible legal Standard self-play with actual paid Saddle and owned replay.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Keywords, ObjectRef, Observation, Player,
    SpellView, StackKind, Target, Zone,
};
use std::collections::{BTreeMap, BTreeSet};
const NAMES: [&str; 19] = [
    "Alacrian Jaguar",
    "Autarch Mammoth",
    "Bounding Felidar",
    "Bridled Bighorn",
    "Brightfield Glider",
    "Brightfield Mustang",
    "Congregation Gryff",
    "District Mascot",
    "Dracosaur Auxiliary",
    "Drover Grizzly",
    "Gila Courser",
    "Gilded Ghoda",
    "Gloryheath Lynx",
    "Quilled Charger",
    "Seraphic Steed",
    "Stubborn Burrowfiend",
    "Trained Arynx",
    "Unswerving Sloth",
    "Venomsac Lagac",
];
fn deck(i: usize) -> Vec<Card> {
    let mut d = Vec::new();
    for (n, count) in [
        ("Plains", 9),
        ("Island", 4),
        ("Swamp", 2),
        ("Mountain", 7),
        ("Forest", 10),
    ] {
        d.extend(vec![Card::from_name(n).unwrap(); count]);
    }
    for n in [
        NAMES[i],
        "Bear Cub",
        "Three Tree Mascot",
        "Chrome Companion",
        "Hop to It",
        "Llanowar Elves",
        "Candy Trail",
    ] {
        d.extend([Card::from_name(n).unwrap(); 4]);
    }
    assert_eq!(d.len(), 60);
    d
}
fn score(
    o: &Observation,
    a: &Action,
    i: usize,
    paid: &BTreeSet<ObjectRef>,
    attacked: &BTreeSet<ObjectRef>,
    done: &[bool; 2],
) -> i32 {
    let p = o.actor.unwrap();
    let obj = |id| o.objects.iter().find(|v| v.id == id).unwrap();
    let friends = o
        .objects
        .iter()
        .filter(|v| {
            v.owner == p
                && v.zone == Zone::Battlefield
                && v.card.is_creature()
                && v.card.name() != NAMES[i]
        })
        .count();
    let source = o
        .objects
        .iter()
        .find(|v| v.owner == p && v.zone == Zone::Battlefield && v.card.name() == NAMES[i]);
    match a {
        Action::AnnounceCast { card } | Action::Cast { card, .. } => {
            let v = obj(*card);
            if done[p.index()] {
                return -1000;
            }
            if v.card.name() == NAMES[i] {
                if source.is_none() && friends >= 3 && matches!(a, Action::AnnounceCast { .. }) {
                    1000
                } else {
                    -1000
                }
            } else if matches!(
                v.card.name(),
                "Bear Cub" | "Three Tree Mascot" | "Chrome Companion" | "Hop to It"
            ) {
                if friends < 4 { 900 } else { -1000 }
            } else if v.card.name() == "Candy Trail" {
                if !o.objects.iter().any(|v| {
                    v.owner == p && v.zone == Zone::Battlefield && v.card.name() == "Candy Trail"
                }) {
                    850
                } else {
                    -1000
                }
            } else {
                -1000
            }
        }
        Action::AnnounceActivate { card, ability, .. } => {
            let v = obj(*card);
            if v.card.name() != NAMES[i] {
                -1000
            } else if i == 7 && *ability == 0 && v.counters.plus_one >= 2 {
                1200
            } else if !done[p.index()] && !v.saddled && *ability == u16::from(i == 7) {
                1100
            } else {
                -1000
            }
        }
        Action::ChooseSaddler { object } => 1600 + obj(object.card).power.clamp(-100, 100) as i32,
        Action::FinishSaddle => 2000,
        Action::Attack { card } => {
            let v = obj(*card);
            if !done[p.index()]
                && v.card.name() == NAMES[i]
                && v.saddled
                && paid.contains(&v.reference)
                && !attacked.contains(&v.reference)
                && o.objects.iter().any(|v| {
                    v.owner == p.other() && v.zone == Zone::Battlefield && v.card.name() == NAMES[i]
                })
            {
                1400
            } else {
                -1000
            }
        }
        Action::ChooseTarget {
            target: Target::Player(q),
        } => {
            if *q == p.other() {
                1800
            } else {
                0
            }
        }
        Action::ChooseTarget {
            target: Target::Card(id),
        } => {
            let v = obj(*id);
            if i == 7
                && o.announcing
                    .as_ref()
                    .is_some_and(|a| matches!(a.kind, StackKind::ActivatedAbility { ability: 0 }))
            {
                if v.owner != p && v.card.name() == "Candy Trail" {
                    1900
                } else {
                    100
                }
            } else if v.owner == p {
                1500
            } else {
                100
            }
        }
        Action::PlayLand { card } => {
            let n = obj(*card).card.name();
            1300 - 20
                * o.objects
                    .iter()
                    .filter(|v| v.owner == p && v.zone == Zone::Battlefield && v.card.name() == n)
                    .count() as i32
        }
        Action::TapMana { .. } => {
            if o.decision == "pay_costs" {
                1700
            } else {
                -1000
            }
        }
        Action::PayCosts { .. } | Action::Keep => 1900,
        Action::PutTrigger { .. } => 1800,
        Action::FinishTargets => 1400,
        Action::ChooseSearch { .. } => 1600,
        Action::FinishSearch => 1500,
        Action::ArrangeLibrary { .. } => 1500,
        Action::Discard { card } => {
            if obj(*card).card.is_land() {
                1200
            } else {
                1000
            }
        }
        Action::FinishAttackers | Action::FinishBlockers => 700,
        Action::Pass => 0,
        _ => -10000,
    }
}
fn count(o: &Observation, p: Player, subtype: &str) -> usize {
    o.objects
        .iter()
        .filter(|v| {
            v.owner == p
                && v.zone == Zone::Battlefield
                && v.card.token_info().is_some_and(|t| t.subtypes == subtype)
        })
        .count()
}
fn assert_effect(i: usize, s: &SpellView, b: &Observation, a: &Observation) {
    let p = s.controller;
    let old = b
        .objects
        .iter()
        .find(|v| {
            v.reference
                == s.source()
                    .expect("example expects an object-backed packet")
                    .object
        })
        .unwrap();
    let v = a
        .objects
        .iter()
        .find(|v| {
            v.reference
                == s.source()
                    .expect("example expects an object-backed packet")
                    .object
        })
        .unwrap();
    match i {
        0 => assert_eq!((v.power, v.toughness), (old.power + 2, old.toughness + 2)),
        1 => assert_eq!(count(a, p, "Elephant"), count(b, p, "Elephant") + 1),
        2 => {
            let n = b
                .objects
                .iter()
                .filter(|x| {
                    x.owner == p
                        && x.zone == Zone::Battlefield
                        && x.card.is_creature()
                        && x.reference
                            != s.source()
                                .expect("example expects an object-backed packet")
                                .object
                })
                .count();
            assert_eq!(
                a.players[p.index()].life,
                b.players[p.index()].life + n as i128
            );
            for x in b.objects.iter().filter(|x| {
                x.owner == p
                    && x.zone == Zone::Battlefield
                    && x.card.is_creature()
                    && x.reference
                        != s.source()
                            .expect("example expects an object-backed packet")
                            .object
            }) {
                assert_eq!(
                    a.objects
                        .iter()
                        .find(|v| v.reference == x.reference)
                        .unwrap()
                        .counters
                        .plus_one,
                    x.counters.plus_one + 1
                );
            }
        }
        3 => assert_eq!(count(a, p, "Sheep"), count(b, p, "Sheep") + 1),
        4 => {
            assert_eq!((v.power, v.toughness), (old.power + 1, old.toughness + 2));
            assert!(v.keywords.contains(Keywords::FLYING));
        }
        5 => {
            assert!(!v.tapped);
            assert_eq!(v.counters.plus_one, old.counters.plus_one + 1);
        }
        6 => {
            let n = b
                .objects
                .iter()
                .filter(|v| {
                    v.owner == p
                        && v.zone == Zone::Battlefield
                        && (v.card.info().oracle_keywords.contains(&"Changeling")
                            || v.card
                                .info()
                                .type_line
                                .split('—')
                                .nth(1)
                                .is_some_and(|s| s.split_whitespace().any(|s| s == "Mount")))
                })
                .count() as i128;
            assert_eq!((v.power, v.toughness), (old.power + n, old.toughness + n));
        }
        7 => assert_eq!(v.counters.plus_one, old.counters.plus_one + 1),
        8 => assert_eq!(
            a.players[p.other().index()].life,
            b.players[p.other().index()].life - 2
        ),
        9 => assert!(
            a.objects
                .iter()
                .filter(|v| v.owner == p && v.zone == Zone::Battlefield && v.card.is_creature())
                .all(|v| v.keywords.contains(Keywords::TRAMPLE))
        ),
        10 => assert_eq!(
            a.objects
                .iter()
                .filter(|v| v.owner == p && v.zone == Zone::Exile)
                .count(),
            b.objects
                .iter()
                .filter(|v| v.owner == p && v.zone == Zone::Exile)
                .count()
                + 1
        ),
        11 => assert_eq!(count(a, p, "Treasure"), count(b, p, "Treasure") + 1),
        12 => assert!(a.known_hand.iter().any(|k| {
            a.objects
                .iter()
                .any(|v| v.reference == *k && v.owner == p && v.card.name() == "Plains")
        })),
        13 => {
            assert_eq!((v.power, v.toughness), (old.power + 1, old.toughness + 2));
            assert!(v.keywords.contains(Keywords::MENACE));
        }
        14 => assert_eq!(count(a, p, "Angel"), count(b, p, "Angel") + 1),
        15 => {
            let n = a
                .objects
                .iter()
                .filter(|v| v.owner == p && v.zone == Zone::Graveyard && v.card.is_creature())
                .count() as i128;
            assert_eq!((v.power, v.toughness), (old.power + n, old.toughness + n));
        }
        16 => assert!(v.keywords.contains(Keywords::FIRST_STRIKE)),
        17 => {
            assert!(v.keywords.contains(Keywords::INDESTRUCTIBLE));
            assert!(
                a.objects
                    .iter()
                    .filter(|v| v.owner == p && v.zone == Zone::Battlefield && v.card.is_creature())
                    .all(|v| !v.tapped)
            );
        }
        18 => assert_eq!((v.power, v.toughness), (old.power, old.toughness + 3)),
        _ => unreachable!(),
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse::<usize>()?;
    let (mut decisions, mut frames, mut invalid, mut initial, mut maximum) =
        (0u64, 0u64, 0u64, 0usize, 0usize);
    let (mut casts, mut costs, mut effects) = ([[0u64; 2]; 19], [[0u64; 2]; 19], [[0u64; 2]; 19]);
    for seed in 0..games {
        let i = seed % 19;
        let mut g = Game::new(Config {
            decks: [deck(i), deck(i)],
            seed: seed as u64,
            starting_player: if seed % 2 == 0 {
                Player::P0
            } else {
                Player::P1
            },
            deck_rules: DeckRules::Standard,
        })?;
        initial = initial.max(g.state_bytes());
        let mut paid = BTreeSet::new();
        let mut attacked = BTreeSet::new();
        let mut packets: BTreeMap<u64, (SpellView, Observation)> = BTreeMap::new();
        let mut done = [false; 2];
        for _ in 0..30000 {
            if g.outcome().is_some() {
                break;
            }
            let p = g.actor().unwrap();
            let b = g.observe(p);
            let actions = g.legal_actions();
            let j = (0..actions.len())
                .max_by_key(|j| {
                    (
                        score(&b, &actions[*j], i, &paid, &attacked, &done),
                        -(*j as i64),
                    )
                })
                .unwrap();
            let action = actions[j].clone();
            if let Some(f) = &b.saddle_choice {
                frames += 1;
                assert!(g.observe(p.other()).saddle_choice.is_none());
                assert!(b.payment_request.is_none());
                let snap = g.snapshot();
                let rev = g.revision();
                assert_eq!(
                    g.apply(p.other(), Action::FinishSaddle),
                    Err(Error::WrongActor)
                );
                invalid += 1;
                if !f.can_finish {
                    assert_eq!(g.apply(p, Action::FinishSaddle), Err(Error::IllegalAction));
                    invalid += 1;
                }
                assert_eq!(g.snapshot(), snap);
                assert_eq!(g.revision(), rev);
                for r in &f.selected {
                    let v = b.objects.iter().find(|v| v.reference == *r).unwrap();
                    assert!(!v.tapped);
                    assert_eq!(v.owner, p);
                    assert_ne!(*r, f.source);
                }
            }
            if let (Some(s), Action::PayCosts { .. }) = (&b.announcing, &action)
                && s.source()
                    .is_some_and(|source| source.card.name() == NAMES[i])
                && s.kind == StackKind::Spell
            {
                casts[i][p.index()] += 1;
            }
            if action == Action::FinishSaddle {
                let s = b.announcing.as_ref().unwrap();
                paid.insert(
                    s.source()
                        .expect("example expects an object-backed packet")
                        .object,
                );
                costs[i][p.index()] += 1;
            }
            if let Action::Attack { card } = action {
                attacked.insert(b.objects.iter().find(|v| v.id == card).unwrap().reference);
            }
            if action == Action::Pass
                && let Some(s) = b.stack.last()
                && s.source()
                    .is_some_and(|source| source.card.name() == NAMES[i])
                && matches!(s.kind, StackKind::TriggeredAbility { .. })
                && (i == 15
                    || attacked.contains(
                        &s.source()
                            .expect("example expects an object-backed packet")
                            .object,
                    ))
                && !done[s.controller.index()]
            {
                packets.entry(s.id).or_insert((s.clone(), b.clone()));
            }
            let saved = g.snapshot();
            let revision = g.revision();
            let mut fork = g.clone();
            fork.apply(p, action.clone())?;
            g.apply_index(p, revision, j)?;
            let expected = g.snapshot();
            assert_eq!(fork.snapshot(), expected);
            g.restore(&saved)?;
            assert_eq!(g.apply_index(p, revision, j), Err(Error::StaleRevision));
            g.apply(p, action)?;
            assert_eq!(g.snapshot(), expected);
            decisions += 1;
            maximum = maximum.max(g.state_bytes());
            let a = g.observe(p);
            let finished = packets
                .iter()
                .filter(|(id, _)| {
                    !a.stack.iter().any(|s| s.id == **id)
                        && a.resolving.as_ref().is_none_or(|s| s.id != **id)
                        && a.announcing.as_ref().is_none_or(|s| s.id != **id)
                })
                .map(|(id, _)| *id)
                .collect::<Vec<_>>();
            for id in finished {
                let (s, before) = packets.remove(&id).unwrap();
                assert_effect(i, &s, &before, &a);
                effects[i][s.controller.index()] += 1;
                done[s.controller.index()] = true;
            }
        }
        assert!(g.outcome().is_some(), "unfinished game {seed}");
    }
    if games >= 100 {
        assert!(casts.iter().flatten().all(|v| *v > 0), "casts {casts:?}");
        assert!(costs.iter().flatten().all(|v| *v > 0), "costs {costs:?}");
        assert!(
            effects.iter().flatten().all(|v| *v > 0),
            "effects {effects:?}"
        );
    }
    println!(
        "Completed {games} Saddle Standard games: {decisions} decisions/replays; casts {casts:?}; actual tap costs {costs:?}; full effects {effects:?}; {frames} private cost frames/{invalid} invalid denials; initial {initial}/max {maximum} logical bytes"
    );
    Ok(())
}
