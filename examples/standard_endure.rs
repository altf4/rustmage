//! Legal Standard self-play with paid full-body Endure and exact branch ledgers.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Keywords, Observation, Player, StackKind, Target,
    Zone,
};
const NAMES: [&str; 8] = [
    "Anafenza, Unyielding Lineage",
    "Descendant of Storms",
    "Dusyut Earthcarver",
    "Fortress Kin-Guard",
    "Inspirited Vanguard",
    "Kin-Tree Nurturer",
    "Sandskitter Outrider",
    "Sinkhole Surveyor",
];
const AMOUNTS: [u16; 8] = [2, 1, 3, 1, 2, 1, 2, 1];
fn deck(i: usize, focal: bool) -> Vec<Card> {
    let mut d = Vec::new();
    for (n, k) in [("Plains", 12), ("Forest", 8), ("Swamp", 8)] {
        d.extend(vec![Card::from_name(n).unwrap(); k]);
    }
    for n in [
        if focal { NAMES[i] } else { "Chrome Companion" },
        "Savannah Lions",
        "Bear Cub",
        "Stab",
        "Cartographer's Companion",
        "Resolute Reinforcements",
        "Novice Inspector",
        "Llanowar Elves",
    ] {
        d.extend([Card::from_name(n).unwrap(); 4]);
    }
    assert_eq!(d.len(), 60);
    d
}
#[derive(Default)]
struct Pilot {
    paid: bool,
    source: Option<u32>,
    attacked: bool,
    killed: bool,
    kill_target: Option<u32>,
    choices: usize,
    declined: bool,
    optional_paid: bool,
    done: bool,
}
fn score(
    o: &Observation,
    a: &Action,
    i: usize,
    p: Player,
    s: &Pilot,
    counters: bool,
    decline: bool,
) -> i32 {
    let actor = o.actor.unwrap();
    let object = |id| o.objects.iter().find(|v| v.id == id).unwrap();
    let lions = o
        .objects
        .iter()
        .find(|v| v.owner == p && v.zone == Zone::Battlefield && v.card.name() == "Savannah Lions");
    match a {
        Action::Cast { .. } | Action::BeginCast { .. } => -1000,
        Action::AnnounceCast { card } => {
            let v = object(*card);
            let n = v.card.name();
            if actor == p && n == NAMES[i] {
                return if !s.paid
                    && (i != 0 || lions.is_some())
                    && matches!(a, Action::AnnounceCast { .. })
                {
                    2000
                } else {
                    -1000
                };
            }
            if n == "Stab" {
                return if actor == p
                    && i == 0
                    && s.paid
                    && !s.killed
                    && lions.is_some()
                    && o.objects
                        .iter()
                        .any(|v| Some(v.id) == s.source && v.zone == Zone::Battlefield)
                {
                    2300
                } else if s.done
                    && o.objects.iter().any(|v| {
                        v.owner != actor && v.zone == Zone::Battlefield && v.card.is_creature()
                    })
                {
                    1100
                } else {
                    -1000
                };
            }
            if !v.card.is_creature() {
                return -1000;
            }
            if s.done {
                return 1000;
            }
            if actor == p {
                return if i == 0 && n == "Savannah Lions" && !s.paid && lions.is_none() {
                    1800
                } else {
                    -1000
                };
            }
            if o.objects
                .iter()
                .filter(|v| v.owner == actor && v.zone == Zone::Battlefield && v.card.is_creature())
                .count()
                < 3
            {
                900
            } else {
                -1000
            }
        }
        Action::ChooseTarget {
            target: Target::Card(id),
        } => {
            let v = object(*id);
            if actor == p && i == 0 && s.paid && !s.killed {
                if lions.is_some_and(|l| l.id == *id) {
                    2400
                } else {
                    -1000
                }
            } else if v.owner != actor {
                2000
            } else {
                -1000
            }
        }
        Action::ChooseEndure { put_counters } => {
            if *put_counters == counters {
                2500
            } else {
                -1000
            }
        }
        Action::PayResolutionMana { .. } => {
            if decline {
                -1000
            } else {
                2400
            }
        }
        Action::DeclineResolutionMana => {
            if decline {
                2500
            } else if s.done {
                1000
            } else {
                -1000
            }
        }
        Action::PayCosts { .. } | Action::ChooseSpellCost { .. } => 2200,
        Action::TapMana { .. } | Action::ActivateMana { .. } | Action::AnnounceMana { .. } => {
            if o.decision == "pay_costs" || o.decision == "resolution_mana" {
                2000
            } else {
                -1000
            }
        }
        Action::Attack { card } => {
            if s.done {
                1500
            } else if actor == p
                && Some(*card) == s.source
                && !s.attacked
                && [1, 4, 7].contains(&i)
                && o.objects
                    .iter()
                    .filter(|v| v.owner == p && v.zone == Zone::Battlefield && v.card.is_land())
                    .count()
                    >= 6
            {
                1800
            } else {
                -1000
            }
        }
        Action::PlayLand { card } => {
            1400 - 20
                * o.objects
                    .iter()
                    .filter(|v| {
                        v.owner == actor
                            && v.zone == Zone::Battlefield
                            && v.card == object(*card).card
                    })
                    .count() as i32
        }
        Action::Discard { card } => {
            let v = object(*card);
            let n = v.card.name();
            let required = actor == p
                && !s.done
                && (n == NAMES[i] && !s.paid
                    || i == 0 && n == "Stab" && !s.killed
                    || i == 0 && n == "Savannah Lions" && !s.paid && lions.is_none());
            let copies = o
                .objects
                .iter()
                .filter(|o| o.owner == actor && o.zone == Zone::Hand && o.card == v.card)
                .count();
            if required && copies == 1 { -100 } else { 2500 }
        }
        Action::AssignDamage { amount, .. } => 1000 + (*amount).min(100) as i32,
        Action::Keep => 3000,
        Action::Concede | Action::Mulligan => -10000,
        Action::Pass => 0,
        Action::FinishAttackers | Action::FinishBlockers => 1000,
        Action::PutTrigger { .. } | Action::FinishTargets | Action::ArrangeLibrary { .. } => 2100,
        _ => -1000,
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse::<usize>()?;
    let (mut casts, mut effects, mut counter_choices, mut token_choices, mut attacks) = (
        [[0u64; 2]; 8],
        [[0u64; 2]; 8],
        [[0u64; 2]; 8],
        [[0u64; 2]; 8],
        [[0u64; 2]; 8],
    );
    let (mut optional_paid, mut declined, mut anafen_deaths, mut sinkhole_life) =
        ([0u64; 2], [0u64; 2], [0u64; 2], [0u64; 2]);
    let mut spirit_sizes = [[0u64; 2]; 3];
    let mut decline_effects = [0u64; 2];
    let (mut decisions, mut replays, mut frames, mut rejected, mut parent_closed) =
        (0u64, 0u64, 0u64, 0u64, 0u64);
    let (mut initial, mut maximum) = (0, 0);
    for seed in 0..games {
        let i = seed % 8;
        let p = if (seed / 8) % 2 == 0 {
            Player::P0
        } else {
            Player::P1
        };
        let mode = (seed / 16) % 3;
        let counters = mode == 1;
        let decline = i == 1 && mode == 2;
        let a = deck(i, true);
        let b = deck(i, false);
        let mut g = Game::new(Config {
            decks: if p == Player::P0 { [a, b] } else { [b, a] },
            seed: seed as u64,
            starting_player: p,
            deck_rules: DeckRules::Standard,
        })?;
        if seed == 0 {
            initial = g.state_bytes();
        }
        let mut s = Pilot::default();
        while g.outcome().is_none() {
            let actor = g.actor().unwrap();
            let o = g.observe(actor);
            assert_eq!(o.schema, rustmage::OBSERVATION_SCHEMA);
            let quiet = o.stack.is_empty()
                && o.announcing.is_none()
                && o.resolving.is_none()
                && o.decision == "priority";
            let ready = s.paid && (i != 0 || s.killed) && (![1, 4, 7].contains(&i) || s.attacked);
            if !s.done
                && ready
                && quiet
                && (if decline {
                    s.declined
                } else {
                    s.choices >= if i == 4 { 2 } else { 1 }
                })
            {
                let source = o
                    .objects
                    .iter()
                    .find(|v| Some(v.id) == s.source && v.zone == Zone::Battlefield)
                    .unwrap();
                match i {
                    0 => {
                        assert!(source.keywords.contains(Keywords::FLASH));
                        assert!(source.keywords.contains(Keywords::FIRST_STRIKE));
                        let victim = o
                            .objects
                            .iter()
                            .find(|v| Some(v.id) == s.kill_target)
                            .unwrap();
                        assert_eq!(victim.zone, Zone::Graveyard);
                        anafen_deaths[p.index()] += 1;
                    }
                    2 => assert!(source.keywords.contains(Keywords::REACH)),
                    5 => assert!(source.keywords.contains(Keywords::LIFELINK)),
                    6 => assert!(source.keywords.contains(Keywords::MENACE)),
                    7 => {
                        assert!(source.keywords.contains(Keywords::FLYING));
                        assert_eq!(o.players[p.index()].life, 19);
                    }
                    _ => {}
                }
                if i == 1 {
                    if decline {
                        assert!(!s.optional_paid);
                        assert_eq!(s.choices, 0);
                        decline_effects[p.index()] += 1;
                    } else {
                        assert!(s.optional_paid);
                    }
                }
                s.done = true;
                effects[i][p.index()] += 1;
            }
            let actions = g.legal_actions();
            let (index, action) = actions
                .iter()
                .enumerate()
                .max_by_key(|(_, a)| score(&o, a, i, p, &s, counters, decline))
                .unwrap();
            assert!(
                score(&o, action, i, p, &s, counters, decline) >= 0,
                "unhandled {} seed{seed} {} {actions:?}",
                NAMES[i],
                o.decision
            );
            let action = action.clone();
            if matches!(action, Action::PayCosts { .. })
                && let Some(a) = &o.announcing
            {
                if a.kind == StackKind::Spell
                    && a.controller == p
                    && a.source().is_some_and(|source| source.card.name() == NAMES[i])
                    && !s.paid
                {
                    s.paid = true;
                    s.source = Some(a.card);
                    casts[i][p.index()] += 1;
                }
                if a.kind == StackKind::Spell
                    && a.controller == p
                    && a.source().is_some_and(|source| source.card.name() == "Stab")
                    && !s.done
                    && i == 0
                {
                    s.killed = true;
                    s.kill_target = a.targets.iter().find_map(|t| {
                        if let Target::Card(id) = t.target {
                            Some(id)
                        } else {
                            None
                        }
                    });
                }
            }
            if let Action::Attack { card } = action
                && Some(card) == s.source
                && !s.attacked
            {
                s.attacked = true;
                attacks[i][p.index()] += 1;
            }
            if matches!(action, Action::PayResolutionMana { .. }) && i == 1 {
                s.optional_paid = true;
                optional_paid[p.index()] += 1;
                let r = o.resolution_payment.as_ref().unwrap();
                assert_eq!((r.cost.colored, r.cost.generic), ([1, 0, 0, 0, 0, 0], 1));
            }
            if matches!(action, Action::DeclineResolutionMana) && i == 1 {
                s.declined = true;
                declined[p.index()] += 1;
            }
            let frame = o.endure_choice.clone();
            if let Some(f) = &frame {
                frames += 1;
                assert_eq!(actor, p);
                assert_eq!(f.amount, AMOUNTS[i]);
                assert!(g.observe(p.other()).endure_choice.is_none());
                assert!(!actions.iter().any(|a| matches!(
                    a,
                    Action::TapMana { .. }
                        | Action::ActivateMana { .. }
                        | Action::PayResolutionMana { .. }
                )));
                let old = g.snapshot();
                let rev = g.revision();
                assert!(g.apply(p.other(), action.clone()).is_err());
                assert_eq!(g.snapshot(), old);
                assert_eq!(g.revision(), rev);
                assert_eq!(g.apply(p, Action::Pass), Err(Error::IllegalAction));
                assert_eq!(g.snapshot(), old);
                assert_eq!(g.revision(), rev);
                rejected += 2;
                if i == 7 && !s.done {
                    assert_eq!(o.players[p.index()].life, 19);
                    sinkhole_life[p.index()] += 1;
                }
            }
            let saved = g.snapshot();
            let rev = g.revision();
            let mut detached = frame.as_ref().map(|_| {
                let parent = g.clone();
                let child = parent.clone();
                drop(parent);
                child
            });
            let mut fork = g.clone();
            fork.apply(actor, action.clone())?;
            g.apply_index(actor, rev, index)?;
            let expected = g.snapshot();
            assert_eq!(fork.snapshot(), expected);
            if frame.is_some() {
                let child = detached.as_mut().unwrap();
                child.apply(actor, action.clone())?;
                assert_eq!(child.snapshot(), expected);
                parent_closed += 1;
                for viewer in [Player::P0, Player::P1] {
                    assert_eq!(
                        serde_json::to_value(fork.observe(viewer))?,
                        serde_json::to_value(g.observe(viewer))?
                    );
                }
            }
            g.restore(&saved)?;
            assert_eq!(g.apply_index(actor, rev, index), Err(Error::StaleRevision));
            rejected += 1;
            g.apply(actor, action.clone())?;
            assert_eq!(g.snapshot(), expected);
            decisions += 1;
            replays += 1;
            maximum = maximum.max(g.state_bytes());
            if let Action::PayResolutionMana { payment } = action {
                assert_eq!(payment.iter().map(|n| u64::from(*n)).sum::<u64>(), 2);
                assert!(payment[0] >= 1);
                let after = g.observe(p);
                for (index, paid) in payment.iter().enumerate() {
                    assert_eq!(
                        after.players[p.index()].mana[index],
                        o.players[p.index()].mana[index] - paid
                    );
                }
            }
            if matches!(action, Action::DeclineResolutionMana) {
                assert_eq!(
                    g.observe(p).players[p.index()].mana,
                    o.players[p.index()].mana
                );
            }
            if let Some(f) = frame {
                let after = g.observe(p);
                let before = o.objects.iter().find(|v| v.reference == f.source).unwrap();
                let current = after
                    .objects
                    .iter()
                    .find(|v| v.reference == f.source)
                    .unwrap();
                let n = u64::from(f.amount);
                let spirits = |x: &Observation| {
                    x.objects
                        .iter()
                        .filter(|v| {
                            v.owner == p
                                && v.zone == Zone::Battlefield
                                && v.token.is_some_and(|t| t.subtypes == "Spirit")
                        })
                        .count()
                };
                if counters {
                    assert_eq!(current.counters.plus_one, before.counters.plus_one + n);
                    assert_eq!(
                        (current.power, current.toughness),
                        (
                            before.power + i128::from(f.amount),
                            before.toughness + i128::from(f.amount)
                        )
                    );
                    assert_eq!(spirits(&after), spirits(&o));
                    counter_choices[i][p.index()] += 1;
                } else {
                    assert_eq!(current.counters.plus_one, before.counters.plus_one);
                    assert_eq!(spirits(&after), spirits(&o) + 1);
                    let created = after
                        .objects
                        .iter()
                        .filter(|v| {
                            v.owner == p
                                && v.zone == Zone::Battlefield
                                && v.token.is_some_and(|t| t.subtypes == "Spirit")
                                && o.objects.iter().all(|b| b.reference != v.reference)
                        })
                        .collect::<Vec<_>>();
                    assert_eq!(created.len(), 1);
                    let token = created[0];
                    assert_eq!(
                        (token.power, token.toughness),
                        (i128::from(f.amount), i128::from(f.amount))
                    );
                    assert_eq!(token.card, f.token);
                    assert_eq!(token.token.unwrap().colors, 1);
                    assert!(!token.keywords.contains(Keywords::FLYING));
                    spirit_sizes[usize::from(f.amount) - 1][p.index()] += 1;
                    token_choices[i][p.index()] += 1;
                }
                s.choices += 1;
            }
        }
        assert!(
            s.done,
            "incomplete {} seed{seed} paid{} attacked{} choices{} killed{}",
            NAMES[i], s.paid, s.attacked, s.choices, s.killed
        );
    }
    if games >= 100 {
        assert!(casts.iter().flatten().all(|n| *n > 0));
        assert!(effects.iter().flatten().all(|n| *n > 0));
        assert!(counter_choices.iter().flatten().all(|n| *n > 0));
        assert!(token_choices.iter().flatten().all(|n| *n > 0));
        assert!(spirit_sizes.iter().flatten().all(|n| *n > 0));
        assert!(optional_paid.iter().all(|n| *n > 0));
        assert!(declined.iter().all(|n| *n > 0));
        assert!(decline_effects.iter().all(|n| *n > 0));
        assert!(anafen_deaths.iter().all(|n| *n > 0));
        assert!(sinkhole_life.iter().all(|n| *n > 0));
        for i in [1, 4, 7] {
            assert!(attacks[i].iter().all(|n| *n > 0));
        }
    }
    println!(
        "games={games} decisions={decisions} replays={replays} frames={frames} rejected={rejected} parent_closed={parent_closed} initial_bytes={initial} max_bytes={maximum} casts={casts:?} effects={effects:?} counters={counter_choices:?} tokens={token_choices:?} spirit_sizes={spirit_sizes:?} attacks={attacks:?} optional_paid={optional_paid:?} declined={declined:?} decline_effects={decline_effects:?} anafen_deaths={anafen_deaths:?} sinkhole_life={sinkhole_life:?}"
    );
    Ok(())
}
