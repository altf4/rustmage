//! Paid scalar-X Standard games, exact domains, owned clone/snapshot replay and effects.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Keywords, Observation, Player, SpellView,
    StackKind, Target, Zone,
};
use std::collections::BTreeMap;
const NAMES: [&str; 12] = [
    "Choco-Comet",
    "Mind Spring",
    "Pedal to the Metal",
    "Procrastinate",
    "Slice from the Shadows",
    "Song of Totentanz",
    "Spectral Denial",
    "Syncopate",
    "Traumatic Critique",
    "United Front",
    "Morlun, Devourer of Spiders",
    "Pterafractyl",
];
fn deck(i: usize) -> Vec<Card> {
    let mut d = Vec::new();
    for (n, k) in [
        ("Plains", 6),
        ("Island", 8),
        ("Swamp", 6),
        ("Mountain", 6),
        ("Forest", 6),
    ] {
        d.extend(vec![Card::from_name(n).unwrap(); k]);
    }
    for n in [
        NAMES[i],
        "Bear Cub",
        "Shock",
        "Angry Rabble",
        "The Scarlet Witch",
        "Spectral Sailor",
        "Three Tree Mascot",
    ] {
        d.extend([Card::from_name(n).unwrap(); 4]);
    }
    assert_eq!(d.len(), 60);
    d
}
fn printed_mana_value(c: Card) -> u64 {
    c.info()
        .mana_cost
        .split('{')
        .skip(1)
        .map(|s| {
            let symbol = s.trim_end_matches('}');
            if symbol == "X" {
                0
            } else {
                symbol.parse::<u64>().unwrap_or(1)
            }
        })
        .sum()
}
fn index(c: Card) -> Option<usize> {
    NAMES.iter().position(|n| *n == c.name())
}
fn packets(o: &Observation) -> impl Iterator<Item = &SpellView> {
    o.stack
        .iter()
        .chain(&o.waiting_triggers)
        .chain(o.announcing.iter())
        .chain(o.resolving.iter())
}
fn score(o: &Observation, a: &Action, i: usize, paid: &[[u64; 2]; 12]) -> i32 {
    let p = o.actor.unwrap();
    let object = |id| o.objects.iter().find(|v| v.id == id).unwrap();
    match a {
        Action::Keep => 5000,
        Action::Mulligan | Action::Concede => -10000,
        Action::PlayLand { card } => {
            let c = object(*card).card;
            let n = o
                .objects
                .iter()
                .filter(|v| v.owner == p && v.zone == Zone::Battlefield && v.card == c)
                .count();
            2000 - n as i32 * 100
        }
        Action::Cast { card, .. }
        | Action::AnnounceCast { card }
        | Action::BeginCast { card, .. } => {
            let c = object(*card).card;
            let lands = o
                .objects
                .iter()
                .filter(|v| v.owner == p && v.zone == Zone::Battlefield && v.card.is_land())
                .count();
            if index(c) == Some(i) {
                if lands < 5 { -1000 } else { 1800 }
            } else {
                match c.name() {
                    "Bear Cub" => {
                        if o.objects.iter().any(|v| {
                            v.owner == p
                                && v.zone == Zone::Battlefield
                                && v.card.name() == "Bear Cub"
                        }) {
                            -1000
                        } else {
                            1700
                        }
                    }
                    "Shock" => {
                        if matches!(i, 6 | 7) && o.stack.is_empty() {
                            1600
                        } else if paid[i].iter().all(|n| *n > 0) {
                            1000
                        } else {
                            -1000
                        }
                    }
                    "Angry Rabble" | "The Scarlet Witch" => {
                        if o.objects
                            .iter()
                            .any(|v| v.owner == p && v.zone == Zone::Battlefield && v.card == c)
                        {
                            -1000
                        } else {
                            1400
                        }
                    }
                    _ => -1000,
                }
            }
        }
        Action::ChooseX { value } => {
            3000 + (*value).min(3) as i32 - if *value > 3 { 1000 } else { 0 }
        }
        Action::ChooseTarget { target } => match target {
            Target::Player(q) => {
                if *q == p.other() {
                    2500
                } else {
                    -1000
                }
            }
            Target::Spell(_) => 2500,
            Target::Card(id) => {
                let v = object(*id);
                if v.card.name() == "Bear Cub" && v.owner == p {
                    2500
                } else {
                    1000
                }
            }
        },
        Action::PayCosts { .. } => 4000,
        Action::TapMana { .. } => {
            if o.decision == "pay_costs" {
                3000
            } else {
                -1000
            }
        }
        Action::PutTrigger { .. } | Action::FinishTargets => 4000,
        Action::DeclineResolutionMana => 4000,
        Action::PayResolutionMana { .. } => -1000,
        Action::Discard { card } => {
            if object(*card).card.is_land() {
                2500
            } else {
                2000
            }
        }
        Action::Attack { .. } | Action::Block { .. } => -1000,
        Action::FinishAttackers | Action::FinishBlockers => 1000,
        Action::Activate { .. }
        | Action::AnnounceActivate { .. }
        | Action::ActivateMana { .. }
        | Action::AnnounceMana { .. } => -1000,
        Action::Pass => 0,
        _ => 100,
    }
}
fn token_count(o: &Observation, p: Player, subtype: &str) -> usize {
    o.objects
        .iter()
        .filter(|v| {
            v.owner == p
                && v.zone == Zone::Battlefield
                && v.card.token_info().is_some_and(|t| t.subtypes == subtype)
        })
        .count()
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse::<usize>()?;
    let (mut decisions, mut domains, mut rejects, mut maximum, mut initial) =
        (0u64, 0u64, 0u64, 0usize, 0usize);
    let (mut paid, mut positive, mut effects, mut entry) = (
        [[0u64; 2]; 12],
        [[0u64; 2]; 12],
        [[0u64; 2]; 12],
        [[0u64; 2]; 12],
    );
    for seed in 0..games {
        let i = seed % 12;
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
        let mut active: BTreeMap<u64, (SpellView, Observation)> = BTreeMap::new();
        for _ in 0..100000 {
            if g.outcome().is_some() {
                break;
            }
            let actor = g.actor().unwrap();
            let before = g.observe(actor);
            let aa = g.legal_actions();
            let j = (0..aa.len())
                .max_by_key(|&j| (score(&before, &aa[j], i, &paid), -(j as i64)))
                .unwrap();
            let a = aa[j].clone();
            if let Some(f) = &before.x_choice {
                assert_eq!(Some(f.clone()), g.legal_action_space().x_choice);
                assert!(g.observe(actor.other()).x_choice.is_none());
                assert!(before.payment_request.is_none());
                let snap = g.snapshot();
                let rev = g.revision();
                assert_eq!(
                    g.apply(
                        actor.other(),
                        Action::ChooseX {
                            value: f.intervals[0].minimum
                        }
                    ),
                    Err(Error::WrongActor)
                );
                if let Some(last) = f.intervals.last()
                    && last.maximum < u32::MAX
                {
                    assert_eq!(
                        g.apply(
                            actor,
                            Action::ChooseX {
                                value: last.maximum + 1
                            }
                        ),
                        Err(Error::IllegalAction)
                    );
                    rejects += 1;
                }
                assert_eq!(g.snapshot(), snap);
                assert_eq!(g.revision(), rev);
                domains += 1;
                rejects += 1;
            }
            if let (Some(ann), Action::PayCosts { payment, .. }, Some(req)) =
                (&before.announcing, &a, before.payment_request.as_ref())
            {
                assert_eq!(
                    payment.iter().map(|v| u64::from(*v)).sum::<u64>(),
                    req.required.iter().map(|v| u64::from(*v)).sum::<u64>()
                        + u64::from(req.generic)
                );
                if let Some(k) = index(ann.source_card) {
                    paid[k][actor.index()] += 1;
                    if ann.x.unwrap() > 0 {
                        positive[k][actor.index()] += 1;
                    }
                    assert_eq!(
                        ann.mana_value(),
                        printed_mana_value(ann.source_card) + u64::from(ann.x.unwrap())
                    );
                }
            }
            if a == Action::Pass
                && let Some(s) = before.stack.last()
                && index(s.source_card).is_some()
            {
                active.insert(s.id, (s.clone(), before.clone()));
            }
            let snap = g.snapshot();
            let rev = g.revision();
            let mut fork = g.clone();
            fork.apply(actor, a.clone())?;
            g.apply_index(actor, rev, j)?;
            let expected = g.snapshot();
            assert_eq!(fork.snapshot(), expected);
            g.restore(&snap)?;
            assert_eq!(g.apply_index(actor, rev, j), Err(Error::StaleRevision));
            g.apply(actor, a.clone())?;
            assert_eq!(g.snapshot(), expected);
            decisions += 1;
            maximum = maximum.max(g.state_bytes());
            let after = g.observe(actor);
            let done = active
                .keys()
                .copied()
                .filter(|id| !packets(&after).any(|v| v.id == *id))
                .collect::<Vec<_>>();
            for id in done {
                let (s, b) = active.remove(&id).unwrap();
                let k = index(s.source_card).unwrap();
                let p = s.controller;
                let x = s.x.unwrap();
                if after.outcome.is_some() {
                    continue;
                }
                if s.kind != StackKind::Spell {
                    if k == 10 {
                        assert_eq!(
                            after.players[p.other().index()].life,
                            b.players[p.other().index()].life - i128::from(x)
                        );
                    }
                    if k == 11 {
                        assert_eq!(after.players[p.index()].life, b.players[p.index()].life + 2);
                    }
                    entry[k][p.index()] += 1;
                    continue;
                }
                if x == 0 {
                    continue;
                }
                let target = s.targets.first().and_then(|t| match t.target {
                    Target::Card(id) => Some(id),
                    _ => None,
                });
                match k {
                    0 => {
                        assert_eq!(
                            token_count(&after, p, "Bird"),
                            token_count(&b, p, "Bird") + 1
                        );
                        assert_eq!(
                            after.players[p.other().index()].life,
                            b.players[p.other().index()].life - i128::from(x)
                        );
                    }
                    1 => assert_eq!(
                        after.players[p.index()].cards_drawn_this_turn,
                        b.players[p.index()].cards_drawn_this_turn + u64::from(x)
                    ),
                    2 => {
                        let id = target.unwrap();
                        let old = b.objects.iter().find(|v| v.id == id).unwrap();
                        let new = after.objects.iter().find(|v| v.id == id).unwrap();
                        assert_eq!(new.power, old.power + i128::from(x));
                        assert!(new.keywords.contains(Keywords::FIRST_STRIKE));
                    }
                    3 => {
                        let id = target.unwrap();
                        let old = b.objects.iter().find(|v| v.id == id).unwrap();
                        let new = after.objects.iter().find(|v| v.id == id).unwrap();
                        assert!(new.tapped);
                        assert_eq!(new.counters.stun, old.counters.stun + 2 * u64::from(x));
                    }
                    4 => {
                        let id = target.unwrap();
                        let old = b.objects.iter().find(|v| v.id == id).unwrap();
                        let new = after.objects.iter().find(|v| v.id == id).unwrap();
                        if old.toughness <= i128::from(x) {
                            assert_eq!(new.zone, Zone::Graveyard);
                        } else {
                            assert_eq!(new.toughness, old.toughness - i128::from(x));
                        }
                    }
                    5 => {
                        assert_eq!(
                            token_count(&after, p, "Rat"),
                            token_count(&b, p, "Rat") + x as usize
                        );
                        for v in after.objects.iter().filter(|v| {
                            v.owner == p && v.zone == Zone::Battlefield && v.card.is_creature()
                        }) {
                            assert!(v.keywords.contains(Keywords::HASTE));
                        }
                    }
                    6 | 7 => {
                        let t = match s.targets[0].target {
                            Target::Spell(id) => id,
                            _ => unreachable!(),
                        };
                        assert_eq!(
                            after.objects.iter().find(|v| v.id == t).unwrap().zone,
                            if k == 7 { Zone::Exile } else { Zone::Graveyard }
                        );
                    }
                    8 => {
                        assert_eq!(
                            after.players[p.other().index()].life,
                            b.players[p.other().index()].life - i128::from(x)
                        );
                        assert_eq!(
                            after.players[p.index()].cards_drawn_this_turn,
                            b.players[p.index()].cards_drawn_this_turn + 2
                        );
                        assert_eq!(
                            after.players[p.index()].cards_discarded_this_turn,
                            b.players[p.index()].cards_discarded_this_turn + 1
                        );
                    }
                    9 => {
                        assert_eq!(
                            token_count(&after, p, "Ally"),
                            token_count(&b, p, "Ally") + x as usize
                        );
                        for v in after.objects.iter().filter(|v| {
                            v.owner == p && v.zone == Zone::Battlefield && v.card.is_creature()
                        }) {
                            let old = b
                                .objects
                                .iter()
                                .find(|o| o.id == v.id)
                                .map_or(0, |o| o.counters.plus_one);
                            assert_eq!(v.counters.plus_one, old + 1);
                        }
                    }
                    10 | 11 => {
                        let v = after.objects.iter().find(|v| v.id == s.card).unwrap();
                        assert_eq!(v.zone, Zone::Battlefield);
                        assert_eq!(v.counters.plus_one, u64::from(x));
                        for t in after
                            .waiting_triggers
                            .iter()
                            .filter(|t| t.object.card == s.card)
                        {
                            assert_eq!(t.x, Some(x));
                            assert_eq!(t.mana_value(), printed_mana_value(s.source_card));
                        }
                    }
                    _ => unreachable!(),
                }
                effects[k][p.index()] += 1;
            }
        }
        assert!(g.outcome().is_some(), "unfinished seed {seed}");
    }
    if games >= 100 {
        for i in 0..12 {
            assert!(
                positive[i].iter().all(|n| *n > 0),
                "positive {i}: {:?}",
                positive[i]
            );
            assert!(
                effects[i].iter().all(|n| *n > 0),
                "effects {i}: {:?}",
                effects[i]
            );
        }
        for i in [10, 11] {
            assert!(entry[i].iter().all(|n| *n > 0), "entry {i}: {:?}", entry[i]);
        }
    }
    println!(
        "Completed {games} scalar-X Standard games, {decisions} decisions/replays, paid {paid:?}, positive {positive:?}, effects {effects:?}, entry {entry:?}, {domains} exact domains, {rejects} illegal rejections, initial {initial} / maximum {maximum} logical bytes"
    );
    Ok(())
}
