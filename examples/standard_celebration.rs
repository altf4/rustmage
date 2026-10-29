//! Legal Standard celebration games with public effect/history witnesses and exact owned replay.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Keywords, ObjectRef, Observation, Player,
    SpellView, StackKind, Target, Zone,
};
use std::collections::{BTreeMap, BTreeSet};
const NAMES: [&str; 10] = [
    "Armory Mice",
    "Ash, Party Crasher",
    "Belligerent of the Ball",
    "Bespoke Battlegarb",
    "Gallant Pie-Wielder",
    "Grand Ball Guest",
    "Lady of Laughter",
    "Pests of Honor",
    "Raging Battle Mouse",
    "Tuinvale Guide",
];
fn index(c: Card) -> Option<usize> {
    NAMES.iter().position(|n| *n == c.name())
}
fn deck(i: usize) -> Vec<Card> {
    let mut d = vec![Card::Plains; 16];
    d.extend([Card::Mountain; 12]);
    for n in [
        NAMES[i],
        "Novice Inspector",
        "Resolute Reinforcements",
        "Cartographer's Companion",
        "Shock",
        "Thrill of Possibility",
        "Miner's Guidewing",
        "Savannah Lions",
    ] {
        d.extend([Card::from_name(n).unwrap(); 4])
    }
    assert_eq!(d.len(), 60);
    d
}
fn packets(o: &Observation) -> impl Iterator<Item = &SpellView> {
    o.stack
        .iter()
        .chain(&o.waiting_triggers)
        .chain(o.resolving.iter())
        .chain(o.announcing.iter())
}
fn score(o: &Observation, a: &Action, primary: usize, seed: usize, fallback: bool) -> i32 {
    let p = o.actor.unwrap();
    let obj = |id| o.objects.iter().find(|v| v.id == id).unwrap();
    let count = o.players[p.index()].nonland_permanents_entered_this_turn;
    let have = o
        .objects
        .iter()
        .any(|v| v.owner == p && v.zone == Zone::Battlefield && v.card.name() == NAMES[primary]);
    match a {
        Action::Keep => 6000,
        Action::Mulligan | Action::Concede => -10000,
        Action::Cast { card, .. } | Action::AnnounceCast { card } => {
            let n = obj(*card).card.name();
            if o.active != p || !matches!(o.step, rustmage::Step::Main1 | rustmage::Step::Main2) {
                return -1000;
            }
            if n == NAMES[primary] {
                return if have { -500 } else { 3000 };
            }
            if matches!(
                n,
                "Novice Inspector" | "Resolute Reinforcements" | "Cartographer's Companion"
            ) {
                return if count < 2 && have { 2900 } else { 1000 };
            }
            if n == "Shock" {
                return if have { 50 } else { -500 };
            }
            if n == "Thrill of Possibility" {
                return if !have { 900 } else { 200 };
            }
            800
        }
        Action::PlayLand { .. } => 3500,
        Action::PayCosts { .. } => 5000,
        Action::TapMana { .. } => {
            if o.decision == "pay_costs" {
                4500
            } else {
                -1000
            }
        }
        Action::ChooseTarget {
            target: Target::Card(id),
        } => {
            let t = obj(*id);
            if o.announcing
                .as_ref()
                .is_some_and(|s| s.source().is_some_and(|source| source.card.name() == "Shock"))
            {
                return if t.owner != p { 400 } else { -500 };
            }
            if t.owner == p {
                if t.card.name() == "Novice Inspector" {
                    4500
                } else {
                    4000
                }
            } else {
                -1000
            }
        }
        Action::ChooseTarget {
            target: Target::Player(q),
        } => {
            if *q != p {
                600
            } else {
                -1000
            }
        }
        Action::FinishTargets => {
            if primary == 3 && (seed / 10).is_multiple_of(2) {
                5000
            } else {
                500
            }
        }
        Action::ChooseAdditionalCost { .. } | Action::Discard { .. } => {
            let id = match a {
                Action::ChooseAdditionalCost { object } => object.card,
                Action::Discard { card } => *card,
                _ => unreachable!(),
            };
            let v = obj(id);
            if v.card.is_land() {
                2500
            } else if v.card.name() == NAMES[primary] && !have {
                -500
            } else {
                1500
            }
        }
        Action::AnnounceActivate { card, .. } | Action::Activate { card, .. } => {
            let v = obj(*card);
            if v.card.name() == "Bespoke Battlegarb" {
                if (seed / 10).is_multiple_of(2) && v.attached_to.is_none() {
                    2000
                } else {
                    -1000
                }
            } else if v.card.token_info().is_some_and(|t| t.subtypes == "Map") {
                300
            } else {
                -1000
            }
        }
        Action::ChooseExplore { graveyard, .. } => {
            if *graveyard {
                1200
            } else {
                1000
            }
        }
        Action::Attack { card } => {
            let v = obj(*card);
            if !have {
                return -500;
            }
            if v.card.name() == NAMES[primary] && count >= 2 {
                2000
            } else {
                -500
            }
        }
        Action::Block { .. } => -500,
        Action::FinishAttackers | Action::FinishBlockers => 1000,
        Action::PutTrigger { .. } => 4000,
        _ => {
            if fallback {
                100
            } else {
                0
            }
        }
    }
}
fn main() -> Result<(), Error> {
    let games = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse::<usize>()
        .unwrap();
    let mut casts = [[0usize; 2]; 10];
    let mut effects = [[0usize; 2]; 10];
    let mut decisions = 0;
    let mut entries = [0u64; 2];
    let mut token_entries = [0u64; 2];
    let mut land_entries = [0usize; 2];
    let mut departed = [0usize; 2];
    let mut resets = 0;
    let mut discounts = [0usize; 2];
    let mut equips = [0usize; 2];
    let mut optional_zero = [0usize; 2];
    let mut attacks = [0usize; 2];
    let mut peak = 0;
    for seed in 0..games {
        let primary = seed % 10;
        let d = deck(primary);
        let mut g = Game::new(Config {
            decks: [d.clone(), d],
            seed: seed as u64 + 23,
            starting_player: if seed % 2 == 0 {
                Player::P0
            } else {
                Player::P1
            },
            deck_rules: DeckRules::Standard,
        })?;
        let mut active: BTreeMap<u64, (SpellView, Observation)> = BTreeMap::new();
        let mut seen = BTreeSet::<ObjectRef>::new();
        for _ in 0..16000 {
            if g.outcome().is_some() {
                break;
            }
            let p = g.actor().unwrap();
            let before = g.observe(p);
            assert_eq!(before.schema, rustmage::OBSERVATION_SCHEMA);
            let other = g.observe(p.other());
            for q in 0..2 {
                assert_eq!(
                    before.players[q].nonland_permanents_entered_this_turn,
                    other.players[q].nonland_permanents_entered_this_turn
                )
            }
            for item in packets(&before) {
                if item.source().and_then(|source| index(source.card)).is_some()
                    && matches!(
                        item.kind,
                        StackKind::TriggeredAbility { .. } | StackKind::ActivatedAbility { .. }
                    )
                {
                    active
                        .entry(item.id)
                        .and_modify(|(old, _)| *old = item.clone())
                        .or_insert_with(|| (item.clone(), g.observe(item.controller)));
                }
            }
            for v in &before.objects {
                if v.zone != Zone::Battlefield
                    || before.players[v.owner.index()].nonland_permanents_entered_this_turn < 2
                    || seen.contains(&v.reference)
                {
                    continue;
                }
                if let Some(i) = index(v.card) {
                    let good = match i {
                        0 => v.toughness >= 3,
                        4 => v.keywords.contains(Keywords::DOUBLE_STRIKE),
                        5 => {
                            v.power >= 3
                                && v.toughness >= 3
                                && v.keywords.contains(Keywords::TRAMPLE)
                        }
                        9 => {
                            v.power >= 3
                                && v.keywords
                                    .contains(Keywords::FLYING.union(Keywords::LIFELINK))
                        }
                        _ => false,
                    };
                    if good {
                        effects[i][v.owner.index()] += 1;
                        seen.insert(v.reference);
                    }
                }
            }
            let fallback = rustmage::policy::smoke_index(&g, &before.actions);
            let j = before
                .actions
                .iter()
                .enumerate()
                .max_by_key(|(j, a)| score(&before, a, primary, seed, *j == fallback))
                .unwrap()
                .0;
            let a = before.actions[j].clone();
            if matches!(a, Action::PayCosts { .. })
                && let Some(item) = &before.announcing
                && item.kind == StackKind::Spell
            {
                if let Some(i) = item.source().and_then(|source| index(source.card)) {
                    casts[i][p.index()] += 1
                }
                let printed: u32 = item
                    .source().expect("example expects an object-backed packet").card
                    .info()
                    .mana_cost
                    .split('}')
                    .filter_map(|v| v.trim_start_matches('{').parse::<u32>().ok())
                    .sum();
                let mice = before
                    .objects
                    .iter()
                    .filter(|v| {
                        v.owner == p
                            && v.zone == Zone::Battlefield
                            && v.card.name() == "Raging Battle Mouse"
                    })
                    .count() as u32;
                if before.players[p.index()].spells_cast_this_turn == 1 && mice > 0 {
                    let request = before.payment_request.as_ref().unwrap();
                    assert_eq!(request.generic, printed.saturating_sub(mice));
                    discounts[p.index()] += 1;
                }
            }
            if matches!(a, Action::FinishTargets)
                && before.announcing.as_ref().is_some_and(|s| {
                    s.source().is_some_and(|source| source.card.name() == "Bespoke Battlegarb") && s.targets.is_empty()
                })
            {
                optional_zero[p.index()] += 1
            }
            if matches!(a, Action::Attack { .. }) {
                attacks[p.index()] += 1
            }
            let saved = g.snapshot();
            let revision = g.revision();
            let mut fork = g.clone();
            fork.apply(p, a.clone())?;
            g.apply_index(p, revision, j)?;
            let expected = g.snapshot();
            assert_eq!(fork.snapshot(), expected);
            g.restore(&saved)?;
            assert_eq!(g.apply_index(p, revision, j), Err(Error::StaleRevision));
            g.apply(p, a)?;
            assert_eq!(g.snapshot(), expected);
            let after = g.observe(p);
            decisions += 1;
            peak = peak.max(g.state_bytes());
            for q in [Player::P0, Player::P1] {
                let k = q.index();
                let fresh = after
                    .objects
                    .iter()
                    .filter(|v| {
                        v.owner == q
                            && v.zone == Zone::Battlefield
                            && !before
                                .objects
                                .iter()
                                .any(|b| b.reference == v.reference && b.zone == Zone::Battlefield)
                    })
                    .collect::<Vec<_>>();
                let n = fresh.iter().filter(|v| !v.card.is_land()).count() as u64;
                entries[k] += n;
                token_entries[k] += fresh
                    .iter()
                    .filter(|v| v.card.is_token() && !v.card.is_land())
                    .count() as u64;
                land_entries[k] += fresh.iter().filter(|v| v.card.is_land()).count();
                if before.turn == after.turn {
                    assert_eq!(
                        after.players[k].nonland_permanents_entered_this_turn,
                        before.players[k].nonland_permanents_entered_this_turn + n
                    );
                    if after.objects.iter().any(|v| {
                        v.owner == q
                            && v.zone != Zone::Battlefield
                            && before
                                .objects
                                .iter()
                                .any(|b| b.id == v.id && b.zone == Zone::Battlefield)
                    }) {
                        departed[k] += 1;
                    }
                } else {
                    assert_eq!(after.players[k].nonland_permanents_entered_this_turn, 0);
                    resets += 1
                }
            }
            let finished = active
                .keys()
                .copied()
                .filter(|id| !packets(&after).any(|s| s.id == *id))
                .collect::<Vec<_>>();
            for id in finished {
                let (item, base) = active.remove(&id).unwrap();
                let i = item.source().and_then(|source| index(source.card)).unwrap();
                let owner = item.controller;
                let view = g.observe(owner);
                let source = view
                    .objects
                    .iter()
                    .find(|v| v.reference == item.source().expect("example expects an object-backed packet").object && v.zone == Zone::Battlefield);
                let target = item.targets.first().and_then(|t| t.object).and_then(|r| {
                    view.objects
                        .iter()
                        .find(|v| v.reference == r && v.zone == Zone::Battlefield)
                });
                let old_target = item
                    .targets
                    .first()
                    .and_then(|t| t.object)
                    .and_then(|r| base.objects.iter().find(|v| v.reference == r));
                let ok = match i {
                    1 | 7 => source
                        .zip(base.objects.iter().find(|v| v.reference == item.source().expect("example expects an object-backed packet").object))
                        .is_some_and(|(n, b)| n.counters.plus_one == b.counters.plus_one + 1),
                    2 => target.zip(old_target).is_some_and(|(n, b)| {
                        n.power == b.power + 1 && n.keywords.contains(Keywords::MENACE)
                    }),
                    3 => source.zip(target).is_some_and(|(s, t)| {
                        s.attached_to == Some(t.reference)
                            && t.power >= old_target.unwrap().power + 2
                    }),
                    6 => {
                        view.players[owner.index()].cards_drawn_this_turn
                            == base.players[owner.index()].cards_drawn_this_turn + 1
                    }
                    8 => target.zip(old_target).is_some_and(|(n, b)| {
                        n.power == b.power + 1 && n.toughness == b.toughness + 1
                    }),
                    _ => false,
                };
                if ok {
                    effects[i][owner.index()] += 1;
                    if i == 3 && matches!(item.kind, StackKind::ActivatedAbility { .. }) {
                        equips[owner.index()] += 1
                    }
                }
            }
        }
        assert!(g.outcome().is_some(), "unfinished seed{seed}");
    }
    if games >= 100 {
        for (i, n) in NAMES.iter().enumerate() {
            for p in 0..2 {
                assert!(casts[i][p] > 0, "missing paid{n}/{p}");
                assert!(effects[i][p] > 0, "missing effect{n}/{p}: {effects:?}")
            }
        }
        assert!(discounts.iter().all(|n| *n > 0));
        assert!(equips.iter().all(|n| *n > 0));
        assert!(optional_zero.iter().all(|n| *n > 0));
        assert!(token_entries.iter().all(|n| *n > 0));
    }
    println!(
        "Completed {games} celebration Standard games: {decisions} exact decisions/replays; paid {casts:?}; effects {effects:?}; nonland entries {entries:?}/tokens {token_entries:?}/lands {land_entries:?}; historical departure checks {departed:?}/resets {resets}; second-spell discounts {discounts:?}/paid equip {equips:?}/optional zero {optional_zero:?}/real attacks {attacks:?}; max {peak} logical bytes"
    );
    Ok(())
}
