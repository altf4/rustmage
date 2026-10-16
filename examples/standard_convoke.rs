//! Legal Standard Convoke games, full effects, both seats, and owned branch replay.
use rustmage::{
    Action, Card, Config, ConvokeContribution, DeckRules, Error, Game, Keywords, Observation,
    Player, StackKind, Target, Zone,
};
use std::collections::BTreeMap;
const NAMES: [&str; 10] = [
    "Appeal to Eirdu",
    "Disruptor of Currents",
    "Lofty Dreams",
    "Protective Response",
    "Sun-Dappled Celebrant",
    "Unexpected Assistance",
    "Vote Out",
    "Winter, Team Player",
    "Merrow Skyswimmer",
    "Bespoke Bō",
];
fn deck(i: usize, focal: bool) -> Vec<Card> {
    let mut d = Vec::new();
    for (n, count) in if focal {
        [
            ("Plains", 9),
            ("Island", 9),
            ("Swamp", 4),
            ("Mountain", 4),
            ("Forest", 6),
        ]
    } else {
        [
            ("Plains", 14),
            ("Island", 0),
            ("Swamp", 0),
            ("Mountain", 0),
            ("Forest", 18),
        ]
    } {
        d.extend(vec![Card::from_name(n).unwrap(); count]);
    }
    for n in if focal {
        [
            NAMES[i],
            "Savannah Lions",
            "Aegis Turtle",
            "Chrome Companion",
            "Bear Cub",
            "Candy Trail",
            "Shock",
        ]
    } else {
        [
            "Llanowar Elves",
            "Savannah Lions",
            "Aegis Turtle",
            "Chrome Companion",
            "Bear Cub",
            "Candy Trail",
            "Shock",
        ]
    } {
        d.extend([Card::from_name(n).unwrap(); 4]);
    }
    assert_eq!(d.len(), 60);
    d
}
#[derive(Default)]
struct Pilot {
    paid: bool,
    finished: bool,
    done: bool,
    armed: bool,
    extra: bool,
    residual_paid: bool,
    source: Option<u32>,
    targets: Vec<u32>,
    baseline: BTreeMap<u32, (i128, i128)>,
    library: usize,
    discarded: bool,
    equipped: Option<u32>,
    contributions: usize,
}
fn score(o: &Observation, a: &Action, i: usize, p: Player, s: &Pilot, round: usize) -> i32 {
    let actor = o.actor.unwrap();
    let other = p.other();
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
        .collect::<Vec<_>>();
    let enemy = o
        .objects
        .iter()
        .filter(|v| v.owner == other && v.zone == Zone::Battlefield && v.card.name() == "Bear Cub")
        .count();
    let zero = (round / 2) % 5 == 4;
    let target_zero = (round / 2) % 4 == 3;
    match a {
        Action::AnnounceCast { card }
        | Action::Cast { card, .. }
        | Action::BeginCast { card, .. } => {
            let v = obj(*card);
            let n = v.card.name();
            if s.done {
                return if n == "Bear Cub" || n == "Savannah Lions" {
                    1200
                } else {
                    -1000
                };
            }
            if actor == other {
                return if n == "Bear Cub" && enemy < 2 {
                    1000
                } else if n == "Candy Trail"
                    && !o.objects.iter().any(|v| {
                        v.owner == other && v.zone == Zone::Battlefield && v.card.name() == n
                    })
                {
                    900
                } else {
                    -1000
                };
            }
            if s.finished && i == 7 && !s.extra {
                return if n == "Candy Trail" { 1800 } else { -1000 };
            }
            if n == NAMES[i] {
                if zero {
                    let lands = o
                        .objects
                        .iter()
                        .filter(|v| {
                            v.owner == p
                                && v.zone == Zone::Battlefield
                                && v.card.is_land()
                                && !v.tapped
                        })
                        .collect::<Vec<_>>();
                    let colored = match i {
                        0 | 3 => [("Plains", 1), ("Island", 0)],
                        1 | 2 | 5 => [("Island", 2), ("Plains", 0)],
                        4 => [("Plains", 2), ("Island", 0)],
                        6 => [("Swamp", 1), ("Island", 0)],
                        7 => [("Mountain", 1), ("Island", 0)],
                        8 => [("Plains", 2), ("Island", 2)],
                        _ => [("Island", 1), ("Plains", 0)],
                    };
                    if lands.len() < 6
                        || colored.iter().any(|(name, n)| {
                            lands.iter().filter(|v| v.card.name() == *name).count() < *n
                        })
                    {
                        return -1000;
                    }
                }
                return if !s.paid
                    && friends.len() >= 3
                    && (i != 8
                        || ["Plains", "Island"].iter().all(|b| {
                            o.objects
                                .iter()
                                .filter(|v| {
                                    v.owner == p
                                        && v.zone == Zone::Battlefield
                                        && v.card.name() == *b
                                        && !v.tapped
                                })
                                .count()
                                >= 2
                        }) && o
                            .objects
                            .iter()
                            .filter(|v| {
                                v.owner == p && v.zone == Zone::Battlefield && v.card.is_land()
                            })
                            .count()
                            >= 6)
                    && matches!(a, Action::AnnounceCast { .. })
                {
                    1500
                } else {
                    -1000
                };
            }
            if matches!(
                n,
                "Savannah Lions" | "Aegis Turtle" | "Chrome Companion" | "Bear Cub"
            ) && friends.len() < 4
            {
                1100
            } else {
                -1000
            }
        }
        Action::AnnounceActivate { card, .. }
        | Action::Activate { card, .. }
        | Action::BeginActivate { card, .. } => {
            if actor == p
                && s.finished
                && i == 9
                && !s.residual_paid
                && Some(*card) == s.source
                && matches!(a, Action::AnnounceActivate { .. })
            {
                1900
            } else {
                -1000
            }
        }
        Action::ChooseTarget {
            target: Target::Card(id),
        } => {
            let v = obj(*id);
            if s.finished && i == 9 {
                return if friends.iter().any(|f| f.id == *id) {
                    2000
                } else {
                    -1000
                };
            }
            if o.announcing
                .as_ref()
                .is_none_or(|a| a.source_card.name() != NAMES[i])
            {
                return 1000;
            }
            if i == 1 || i == 9 {
                return if target_zero {
                    -1000
                } else if (round / 2) % 2 == 1 {
                    if v.owner == p && Some(v.id) != s.source {
                        2100
                    } else {
                        1000
                    }
                } else if v.owner == other && v.card.name() == "Candy Trail" {
                    2100
                } else if v.owner == other {
                    2000
                } else {
                    1000
                };
            }
            if i == 0 || i == 2 {
                if friends.iter().any(|f| f.id == *id) {
                    2000
                } else {
                    1000
                }
            } else if v.owner == other {
                2100
            } else {
                1000
            }
        }
        Action::ChooseTarget { .. } => -1000,
        Action::FinishTargets => {
            if target_zero && (i == 1 || i == 9)
                || i == 0 && s.targets.len() >= if (round / 2).is_multiple_of(2) { 2 } else { 1 }
            {
                2200
            } else {
                1000
            }
        }
        Action::ChooseSpellCost { cost } => {
            if usize::from(*cost) == (round / 2) % 3 {
                2100
            } else {
                -1000
            }
        }
        Action::BeginConvoke => {
            if zero {
                -1000
            } else {
                2100
            }
        }
        Action::ChooseConvoker { .. } => {
            if o.convoke_choice.as_ref().unwrap().selected.is_empty() {
                2100
            } else {
                1900
            }
        }
        Action::PayCosts { .. } => 2000,
        Action::TapMana { .. } | Action::ActivateMana { .. } => {
            if o.decision == "pay_costs" {
                1900
            } else {
                -1000
            }
        }
        Action::Attack { card } => {
            if s.done && obj(*card).power > 0
                || actor == other && i == 3 && !s.paid && s.armed && friends.len() >= 3
            {
                1500
            } else {
                -1000
            }
        }
        Action::PlayLand { card } => {
            1300 - 20
                * o.objects
                    .iter()
                    .filter(|v| {
                        v.owner == actor && v.zone == Zone::Battlefield && v.card == obj(*card).card
                    })
                    .count() as i32
        }
        Action::Keep => 2500,
        Action::Mulligan | Action::Concede => -10000,
        Action::Pass => 0,
        Action::FinishAttackers | Action::FinishBlockers => 1000,
        Action::PutTrigger { .. } | Action::Discard { .. } | Action::ArrangeLibrary { .. } => 2100,
        _ => -1000,
    }
}
fn assert_effect(i: usize, p: Player, s: &Pilot, o: &Observation, target_zero: bool) {
    let object = |id| o.objects.iter().find(|v| v.id == id).unwrap();
    let source = object(s.source.unwrap());
    match i {
        0 => {
            assert!(!s.targets.is_empty() && s.targets.len() <= 2);
            for t in &s.targets {
                let v = object(*t);
                let before = s.baseline[t];
                assert_eq!((v.power, v.toughness), (before.0 + 2, before.1 + 1));
            }
        }
        1 | 9 => {
            if !target_zero {
                assert_eq!(object(s.targets[0]).zone, Zone::Hand);
            }
        }
        2 => {
            let v = object(s.targets[0]);
            let before = s.baseline[&v.id];
            assert_eq!((v.power, v.toughness), (before.0 + 2, before.1 + 2));
            assert!(v.keywords.contains(Keywords::FLYING));
            assert_eq!(o.players[p.index()].library_count, s.library - 1);
        }
        3 | 6 => assert_eq!(object(s.targets[0]).zone, Zone::Graveyard),
        4 => {
            assert_eq!((source.power, source.toughness), (5, 6));
            assert!(source.keywords.contains(Keywords::VIGILANCE));
        }
        5 => {
            assert!(s.discarded);
            assert_eq!(o.players[p.index()].library_count, s.library - 3);
        }
        7 => {}
        8 => {
            assert!(source.keywords.contains(Keywords::FLYING));
            assert!(source.keywords.contains(Keywords::VIGILANCE));
            let tokens = o
                .objects
                .iter()
                .filter(|v| {
                    v.owner == p
                        && v.zone == Zone::Battlefield
                        && v.token.is_some_and(|t| t.subtypes == "Merfolk")
                })
                .collect::<Vec<_>>();
            assert_eq!(tokens.len(), 1);
            let token = tokens[0];
            assert_eq!(
                (token.power, token.toughness, token.keywords),
                (1, 1, Keywords::NONE)
            );
            assert_eq!(token.token.unwrap().colors, 3);
        }
        _ => unreachable!(),
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse::<usize>()?;
    let (mut casts, mut effects, mut convoked, mut zeros, mut modes) = (
        [[0u64; 2]; 10],
        [[0u64; 2]; 10],
        [[0u64; 2]; 9],
        [[0u64; 2]; 9],
        [[0u64; 2]; 3],
    );
    let (
        mut decisions,
        mut replays,
        mut choices,
        mut frames,
        mut rejected,
        mut taps,
        mut equip,
        mut winter,
    ) = (0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64);
    let (mut initial, mut maximum) = (0, 0);
    for seed in 0..games {
        let i = seed % 10;
        let round = seed / 10;
        let p = if round % 2 == 0 {
            Player::P0
        } else {
            Player::P1
        };
        let target_zero = (round / 2) % 4 == 3;
        let a = deck(i, true);
        let b = deck(i, false);
        let mut g = Game::new(Config {
            decks: if p == Player::P0 { [a, b] } else { [b, a] },
            seed: seed as u64,
            starting_player: p,
            deck_rules: DeckRules::Constructed,
        })?;
        if seed == 0 {
            initial = g.state_bytes();
        }
        let mut s = Pilot::default();
        for _ in 0..24000 {
            if g.outcome().is_some() {
                break;
            }
            let actor = g.actor().unwrap();
            let o = g.observe(actor);
            let objects = &o.objects;
            if actor == p
                && objects
                    .iter()
                    .any(|v| v.owner == p && v.zone == Zone::Hand && v.card.name() == NAMES[i])
            {
                s.armed = true;
            }
            let quiet = o.stack.is_empty()
                && o.announcing.is_none()
                && o.resolving.is_none()
                && o.decision == "priority";
            if s.paid && quiet && !s.finished {
                assert_effect(i, p, &s, &o, target_zero);
                s.finished = true;
                if i != 7 && i != 9 {
                    s.done = true;
                    effects[i][p.index()] += 1;
                }
            }
            if s.finished && quiet && !s.done {
                if i == 7 && s.extra {
                    for (id, stats) in &s.baseline {
                        if let Some(v) = objects
                            .iter()
                            .find(|v| v.id == *id && v.zone == Zone::Battlefield)
                        {
                            assert_eq!(v.power, stats.0 + 1);
                        }
                    }
                    assert_eq!(
                        objects
                            .iter()
                            .find(|v| Some(v.id) == s.source)
                            .unwrap()
                            .power,
                        4
                    );
                    s.done = true;
                    effects[i][p.index()] += 1;
                    winter += 1;
                }
                if i == 9 && s.residual_paid {
                    let v = objects.iter().find(|v| Some(v.id) == s.equipped).unwrap();
                    let before = s.baseline[&v.id];
                    assert_eq!((v.power, v.toughness), (before.0 + 2, before.1 + 1));
                    assert!(v.keywords.contains(Keywords::VIGILANCE));
                    s.done = true;
                    effects[i][p.index()] += 1;
                    equip += 1;
                }
            }
            let actions = &o.actions;
            let index = actions
                .iter()
                .enumerate()
                .max_by_key(|(_, a)| score(&o, a, i, p, &s, round))
                .unwrap()
                .0;
            let action = actions[index].clone();
            assert!(
                score(&o, &action, i, p, &s, round) >= 0,
                "{} {:?} {:?}",
                NAMES[i],
                o.decision,
                actions
            );
            if let Some(f) = &o.convoke_choice {
                frames += 1;
                assert!(g.observe(p.other()).convoke_choice.is_none());
                assert!(g.observe(p.other()).payment_request.is_none());
                assert!(!actions.iter().any(|a| matches!(
                    a,
                    Action::TapMana { .. } | Action::ActivateMana { .. } | Action::BeginConvoke
                )));
                for c in &f.selected {
                    let v = objects.iter().find(|v| v.id == c.object.card).unwrap();
                    assert_eq!(v.reference, c.object);
                    assert!(!v.tapped);
                }
                let bad = Action::ChooseConvoker {
                    object: f.spell,
                    contribution: ConvokeContribution::Generic,
                };
                let before = g.snapshot();
                assert_eq!(g.apply(actor, bad), Err(Error::IllegalAction));
                assert_eq!(g.snapshot(), before);
                rejected += 1;
            }
            if let Action::ChooseTarget {
                target: Target::Card(id),
            } = action
                && o.announcing
                    .as_ref()
                    .is_some_and(|a| a.source_card.name() == NAMES[i])
                && !(s.finished && i == 9)
            {
                s.targets.push(id);
            }
            if matches!(action, Action::Discard { .. }) && i == 5 {
                s.discarded = true;
            }
            if let Action::ChooseSpellCost { cost } = action {
                modes[cost as usize][p.index()] += 1;
            }
            if let Action::ChooseConvoker { .. } = action {
                choices += 1;
            }
            let mut committed = Vec::new();
            if matches!(action, Action::PayCosts { .. })
                && let Some(a) = &o.announcing
            {
                if a.source_card.name() == NAMES[i] && a.kind == StackKind::Spell {
                    s.paid = true;
                    s.source = Some(a.card);
                    s.library = o.players[p.index()].library_count;
                    s.baseline = objects
                        .iter()
                        .map(|v| (v.id, (v.power, v.toughness)))
                        .collect();
                    casts[i][p.index()] += 1;
                    if i < 9 {
                        let selected = o
                            .convoke_choice
                            .as_ref()
                            .map(|f| f.selected.clone())
                            .unwrap_or_default();
                        s.contributions = selected.len();
                        committed = selected.iter().map(|c| c.object).collect();
                        if selected.is_empty() {
                            zeros[i][p.index()] += 1;
                        } else {
                            convoked[i][p.index()] += 1;
                            taps += selected.len() as u64;
                        }
                    }
                }
                if s.finished
                    && i == 7
                    && a.controller == p
                    && a.kind == StackKind::Spell
                    && a.source_card.name() == "Candy Trail"
                {
                    s.extra = true;
                    s.baseline = objects
                        .iter()
                        .filter(|v| {
                            v.owner == p && v.zone == Zone::Battlefield && v.card.is_creature()
                        })
                        .map(|v| (v.id, (v.power, v.toughness)))
                        .collect();
                }
                if s.finished
                    && i == 9
                    && a.source_card.name() == NAMES[i]
                    && matches!(a.kind, StackKind::ActivatedAbility { .. })
                {
                    s.residual_paid = true;
                    s.equipped = a.targets.iter().find_map(|t| {
                        if let Target::Card(id) = t.target {
                            Some(id)
                        } else {
                            None
                        }
                    });
                }
            }
            let saved = g.snapshot();
            let revision = g.revision();
            let mut fork = g.clone();
            fork.apply(actor, action.clone())?;
            g.apply_index(actor, revision, index)?;
            let end = g.snapshot();
            assert_eq!(fork.snapshot(), end);
            g.restore(&saved)?;
            assert_eq!(
                g.apply_index(actor, revision, index),
                Err(Error::StaleRevision)
            );
            g.apply(actor, action)?;
            assert_eq!(g.snapshot(), end);
            for r in committed {
                assert!(
                    g.observe(p)
                        .objects
                        .iter()
                        .find(|v| v.reference == r)
                        .unwrap()
                        .tapped
                );
            }
            decisions += 1;
            replays += 1;
            maximum = maximum.max(g.state_bytes());
        }
        assert!(s.done, "incomplete effect {} seed{seed}", NAMES[i]);
        assert!(g.outcome().is_some(), "unfinished game {seed}");
    }
    if games >= 100 {
        assert!(casts.iter().flatten().all(|n| *n > 0));
        assert!(effects.iter().flatten().all(|n| *n > 0));
        assert!(convoked.iter().flatten().all(|n| *n > 0));
        assert!(zeros.iter().flatten().all(|n| *n > 0));
        assert!(modes.iter().flatten().all(|n| *n > 0));
        assert!(equip > 0 && winter > 0);
    }
    println!(
        "games={games} decisions={decisions} replays={replays} frames={frames} choices={choices} rejected={rejected} taps={taps} equip={equip} winter={winter} initial_bytes={initial} max_bytes={maximum} casts={casts:?} effects={effects:?} convoked={convoked:?} zero_contributors={zeros:?} hybrid={modes:?}"
    );
    Ok(())
}
