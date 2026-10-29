//! Paid fixed kicker in legal Standard self-play, with independent branch and seat ledgers.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Keywords, ObjectRef, Observation, Player,
    SpellView, StackKind, Target, Zone,
};
use std::collections::{BTreeMap, BTreeSet};
const NAMES: [&str; 10] = [
    "Burst Lightning",
    "Firebending Lesson",
    "Gatekeeper of Malakir",
    "Grow from the Ashes",
    "Into the Roil",
    "Whoosh!",
    "Nullpriest of Oblivion",
    "Sun-Blessed Healer",
    "Zuko's Conviction",
    "Gnarlid Colony",
];
fn index(c: Card) -> Option<usize> {
    NAMES.iter().position(|n| *n == c.name())
}
fn cost(c: Card, pay: bool) -> ([u32; 6], u32) {
    let (mut colored, mut generic) = ([0; 6], 0);
    for x in c
        .info()
        .mana_cost
        .split('{')
        .skip(1)
        .map(|s| s.trim_end_matches('}'))
    {
        if let Ok(n) = x.parse::<u32>() {
            generic += n
        } else {
            colored["WUBRGC".find(x).unwrap()] += 1;
        }
    }
    if pay {
        let k = c.kicker_cost().unwrap();
        generic += k.generic;
        for (a, b) in colored.iter_mut().zip(k.colored) {
            *a += b;
        }
    }
    (colored, generic)
}
fn deck(i: usize) -> Vec<Card> {
    let mut d = Vec::new();
    for (n, k) in [
        ("Plains", 6),
        ("Island", 6),
        ("Swamp", 8),
        ("Mountain", 6),
        ("Forest", 6),
    ] {
        d.extend(std::iter::repeat_n(Card::from_name(n).unwrap(), k));
    }
    for n in [
        NAMES[i],
        "Bear Cub",
        "Three Tree Mascot",
        "Thrill of Possibility",
        "Savannah Lions",
        "Hop to It",
        "Candlestick",
    ] {
        d.extend([Card::from_name(n).unwrap(); 4]);
    }
    assert_eq!(d.len(), 60);
    d
}
fn packets(o: &Observation) -> impl Iterator<Item = &SpellView> {
    o.stack
        .iter()
        .chain(&o.waiting_triggers)
        .chain(o.announcing.iter())
        .chain(o.resolving.iter())
}
fn score(
    o: &Observation,
    a: &Action,
    i: usize,
    want: bool,
    done: [bool; 2],
    fallback: bool,
) -> i32 {
    let p = o.actor.unwrap();
    let object = |id| o.objects.iter().find(|v| v.id == id).unwrap();
    let friends = o
        .objects
        .iter()
        .filter(|v| v.owner == p && v.zone == Zone::Battlefield && v.card.is_creature())
        .count();
    let grave = o
        .objects
        .iter()
        .any(|v| v.owner == p && v.zone == Zone::Graveyard && v.card.name() == "Bear Cub");
    let handbear = o
        .objects
        .iter()
        .any(|v| v.owner == p && v.zone == Zone::Hand && v.card.name() == "Bear Cub");
    let needgrave = matches!(i, 6..=8);
    let required = cost(Card::from_name(NAMES[i]).unwrap(), want);
    let mut potential = o.players[p.index()].mana;
    for v in &o.objects {
        if v.zone == Zone::Battlefield
            && v.owner == p
            && !v.tapped
            && let Some(k) = ["Plains", "Island", "Swamp", "Mountain", "Forest"]
                .iter()
                .position(|n| *n == v.card.name())
        {
            potential[k] += 1;
        }
    }
    let fundable = potential.iter().zip(required.0).all(|(a, b)| *a >= b)
        && potential.iter().sum::<u32>() >= required.0.iter().sum::<u32>() + required.1;
    match a {
        Action::AnnounceCast { card } | Action::Cast { card, .. } => {
            let n = object(*card).card.name();
            if n == NAMES[i] {
                if fundable && (!needgrave || grave) && matches!(a, Action::AnnounceCast { .. }) {
                    1100
                } else {
                    -1000
                }
            } else if n == "Thrill of Possibility" {
                if needgrave && !grave && handbear && matches!(a, Action::AnnounceCast { .. }) {
                    1000
                } else {
                    -1000
                }
            } else if matches!(n, "Bear Cub" | "Three Tree Mascot" | "Savannah Lions") {
                if needgrave && !grave && n == "Bear Cub" {
                    -1000
                } else if friends < 3 || done.iter().all(|b| *b) {
                    700
                } else {
                    -1000
                }
            } else if n == "Hop to It" && done.iter().all(|b| *b) {
                650
            } else {
                -1000
            }
        }
        Action::ChooseKicker { pay } => {
            if *pay == want {
                1500
            } else {
                -1000
            }
        }
        Action::ChooseAdditionalCost { object: r } => {
            if object(r.card).card.name() == "Bear Cub" {
                1500
            } else {
                -1000
            }
        }
        Action::ChooseTarget {
            target: Target::Player(t),
        } => {
            if *t == p.other() {
                1500
            } else {
                -1000
            }
        }
        Action::ChooseTarget {
            target: Target::Card(t),
        } => {
            let v = object(*t);
            if needgrave && v.owner == p && v.zone == Zone::Graveyard && v.card.name() == "Bear Cub"
            {
                1400
            } else if v.owner == p.other() && v.zone == Zone::Battlefield && v.card.is_creature() {
                1300
            } else {
                100
            }
        }
        Action::PlayLand { card } => {
            let c = object(*card).card;
            let have = o
                .objects
                .iter()
                .filter(|v| v.owner == p && v.zone == Zone::Battlefield && v.card == c)
                .count();
            let k = ["Plains", "Island", "Swamp", "Mountain", "Forest"]
                .iter()
                .position(|n| *n == c.name())
                .unwrap();
            1200 - have as i32 * 10 + if required.0[k] > have as u32 { 100 } else { 0 }
        }
        Action::PayCosts { .. } => 1500,
        Action::TapMana { .. } => {
            if o.decision == "pay_costs" {
                800
            } else {
                -1000
            }
        }
        Action::ChooseSearch { .. } | Action::Sacrifice { .. } | Action::PutTrigger { .. } => 1400,
        Action::FinishSearch | Action::FinishTargets => 1100,
        Action::Discard { card } => {
            if needgrave && !grave && object(*card).card.name() == "Bear Cub" {
                1500
            } else if object(*card).card.is_land() {
                1200
            } else {
                1000
            }
        }
        Action::Attack { .. } => {
            if done.iter().all(|b| *b) {
                800
            } else {
                -1000
            }
        }
        Action::FinishAttackers | Action::FinishBlockers => 500,
        Action::Block { .. }
        | Action::Concede
        | Action::AnnounceActivate { .. }
        | Action::Activate { .. }
        | Action::BeginActivate { .. } => -1000,
        Action::Keep => 1600,
        Action::Mulligan => -1000,
        _ => {
            if fallback {
                100
            } else {
                0
            }
        }
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse::<usize>()?;
    let (
        mut decisions,
        mut replayed,
        mut invalid,
        mut private,
        mut searches,
        mut lands,
        mut grants,
        mut max_bytes,
    ) = (0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0usize);
    let (mut casts, mut effects) = ([[[0u64; 2]; 2]; 10], [[[0u64; 2]; 2]; 10]);
    let mut damage = [0u64; 2];
    let mut fizzles = 0;
    let mut sacrifice = 0;
    let mut reanimate = 0;
    let mut bounced = 0;
    let mut drawn = 0;
    for seed in 0..games {
        let i = seed % 10;
        let round = seed / 10;
        let mut g = Game::new(Config {
            decks: [deck(i), deck(i)],
            seed: seed as u64,
            starting_player: if round.is_multiple_of(2) {
                Player::P0
            } else {
                Player::P1
            },
            deck_rules: DeckRules::Standard,
        })?;
        let mut done = [false; 2];
        let mut entries = BTreeMap::<ObjectRef, bool>::new();
        let mut active = BTreeMap::<u64, (SpellView, Observation)>::new();
        let mut seen = BTreeSet::new();
        let mut selected = BTreeMap::<u64, Vec<u32>>::new();
        for _ in 0..100000 {
            if g.outcome().is_some() {
                break;
            }
            let actor = g.actor().unwrap();
            let before = g.observe(actor);
            let want = (round + actor.index()).is_multiple_of(2);
            for packet in packets(&before) {
                if let Some(k) = packet.source().and_then(|source| index(source.card)) {
                    match packet.kind {
                        StackKind::CasualtyCopy
                        | StackKind::CopyEndStepSacrifice
                        | StackKind::CopyEndStepConditionalSacrifice
                        | StackKind::DelayedReturn
                        | StackKind::DelayedTokenCreation { .. }
                        | StackKind::DelayedSacrifice
                        | StackKind::DelayedWarpExile
                        | StackKind::DelayedManifestDread
                        | StackKind::SiegeDefeat | StackKind::InherentRule { .. } => {
                            panic!("no delayed producer in this fixture")
                        }
                        StackKind::Spell => {
                            let c = packet.cast.unwrap();
                            assert_eq!(c.origin, Zone::Hand);
                            assert_eq!(c.method, rustmage::CastMethod::Normal);
                            assert_eq!(c.offspring, None);
                            if c.kicker.is_some() && seen.insert(packet.id) {
                                assert!(
                                    packet.targets.is_empty()
                                        || packet.cast.unwrap().kicker.is_some()
                                );
                            }
                        }
                        StackKind::TriggeredAbility { .. } => {
                            assert!(matches!(k, 2 | 6 | 7));
                            assert_eq!(packet.cast, None);
                            assert_eq!(entries.get(&packet.source().expect("example expects an object-backed packet").object), Some(&true));
                        }
                        _ => panic!("unexpected kicker activation"),
                    }
                }
            }
            if before.decision == "kicker" {
                assert!(before.payment_request.is_none());
                let item = before.announcing.as_ref().unwrap();
                assert!(item.targets.is_empty());
                assert_eq!(item.cast.unwrap().kicker, None);
                assert!(before.actions.contains(&Action::ChooseKicker { pay: want }));
                let saved = g.snapshot();
                assert_eq!(
                    g.apply(actor.other(), Action::ChooseKicker { pay: want }),
                    Err(Error::WrongActor)
                );
                assert_eq!(g.snapshot(), saved);
                invalid += 1;
            }
            if let Some(frame) = &before.search_choice {
                assert_eq!(i, 3);
                let root = before.resolving.as_ref().unwrap();
                let pay = root.cast.unwrap().kicker.unwrap();
                assert_eq!(frame.maximum, if pay { 2 } else { 1 });
                assert!(!frame.tapped);
                assert_eq!(frame.destination, rustmage::SearchDestination::Battlefield);
                let other = g.observe(actor.other());
                assert!(other.search_choice.is_none() && other.actions.is_empty());
                assert!(!other.objects.iter().any(|v| v.zone == Zone::Library));
                selected.entry(root.id).or_default();
                private += 1;
            }
            let fallback = rustmage::policy::smoke_index(&g, &before.actions);
            let j = before
                .actions
                .iter()
                .enumerate()
                .max_by_key(|(j, a)| score(&before, a, i, want, done, *j == fallback))
                .unwrap()
                .0;
            let a = before.actions[j].clone();
            if before.decision == "targets" {
                let s = g.snapshot();
                assert_eq!(
                    g.apply(
                        actor,
                        Action::ChooseTarget {
                            target: Target::Card(u32::MAX)
                        }
                    ),
                    Err(Error::IllegalAction)
                );
                assert_eq!(g.snapshot(), s);
                invalid += 1;
            }
            if let Action::PayCosts { payment } = a
                && let Some(ann) = &before.announcing
                && let Some(k) = ann.source().and_then(|source| index(source.card))
            {
                let pay = ann.cast.unwrap().kicker.unwrap();
                let expected = cost(ann.source().expect("example expects an object-backed packet").card, pay);
                let request = before.payment_request.as_ref().unwrap();
                assert_eq!((request.required, request.generic), expected);
                assert_eq!(
                    payment.iter().sum::<u32>(),
                    expected.0.iter().sum::<u32>() + expected.1
                );
                casts[k][usize::from(pay)][actor.index()] += 1;
                done[actor.index()] = true;
            }
            if let Action::ChooseSearch { card } = a {
                selected
                    .get_mut(&before.resolving.as_ref().unwrap().id)
                    .unwrap()
                    .push(card);
            }
            if matches!(a, Action::Pass)
                && let Some(packet) = before.stack.last()
                && packet.source().and_then(|source| index(source.card)).is_some()
            {
                active.insert(packet.id, (packet.clone(), g.observe(packet.controller)));
            }
            let checkpoint = g.snapshot();
            let revision = g.revision();
            let mut fork = g.clone();
            fork.apply(actor, a.clone())?;
            g.apply_index(actor, revision, j)?;
            let expected = g.snapshot();
            assert_eq!(fork.snapshot(), expected);
            g.restore(&checkpoint)?;
            assert_eq!(g.apply_index(actor, revision, j), Err(Error::StaleRevision));
            g.apply(actor, a.clone())?;
            assert_eq!(g.snapshot(), expected);
            let after = g.observe(actor);
            assert_eq!(after.schema, rustmage::OBSERVATION_SCHEMA);
            decisions += 1;
            replayed += 1;
            max_bytes = max_bytes.max(g.state_bytes());
            if before.search_choice.is_some() && after.search_choice.is_some() {
                assert_eq!(
                    after.players[actor.index()].library_count,
                    before.players[actor.index()].library_count
                );
                for id in selected
                    .get(&before.resolving.as_ref().unwrap().id)
                    .unwrap()
                {
                    assert!(!after.objects.iter().any(|v| v.id == *id));
                }
            }
            let complete = active
                .keys()
                .copied()
                .filter(|id| !packets(&after).any(|s| s.id == *id))
                .collect::<Vec<_>>();
            for id in complete {
                let (packet, base) = active.remove(&id).unwrap();
                let view = g.observe(packet.controller);
                let p = packet.controller.index();
                let k = packet.source().and_then(|source| index(source.card)).unwrap();
                if g.outcome() == Some(rustmage::Outcome::Winner(packet.controller.other()))
                    && base.players[p].library_count == 0
                {
                    continue;
                }
                // Responses may move the same graveyard target before an older spell.
                // An all-invalid packet performs no operation and cannot draw.
                if !packet.targets.is_empty()
                    && packet.targets.iter().all(|t| {
                        t.object
                            .is_some_and(|r| !base.objects.iter().any(|v| v.reference == r))
                    })
                {
                    assert_eq!(
                        view.players[p].cards_drawn_this_turn,
                        base.players[p].cards_drawn_this_turn
                    );
                    fizzles += 1;
                    continue;
                }
                let pay = if packet.kind == StackKind::Spell {
                    packet.cast.unwrap().kicker.unwrap()
                } else {
                    *entries.get(&packet.source().expect("example expects an object-backed packet").object).unwrap()
                };
                if packet.kind == StackKind::Spell && packet.source().is_some_and(|source| source.card.is_creature()) {
                    let creature = view.objects.iter().find(|v| v.id == packet.source().expect("example expects an object-backed packet").id).unwrap();
                    assert_eq!(creature.zone, Zone::Battlefield);
                    assert_eq!(creature.reference.generation, packet.source().expect("example expects an object-backed packet").object.generation + 1);
                    entries.insert(creature.reference, pay);
                    if matches!(k, 2 | 6 | 7) && pay {
                        continue;
                    }
                    if k == 9 {
                        assert_eq!(creature.counters.plus_one, if pay { 2 } else { 0 });
                        assert_eq!(
                            (creature.power, creature.toughness),
                            if pay { (4, 4) } else { (2, 2) }
                        );
                        let colony = view.objects.iter().any(|v| {
                            v.owner == packet.controller
                                && v.zone == Zone::Battlefield
                                && v.card.name() == "Gnarlid Colony"
                        });
                        for v in view.objects.iter().filter(|v| {
                            v.owner == packet.controller
                                && v.zone == Zone::Battlefield
                                && v.card.is_creature()
                        }) {
                            assert_eq!(
                                v.keywords.contains(Keywords::TRAMPLE),
                                colony && v.counters.plus_one > 0
                            );
                            grants += 1;
                        }
                    }
                    effects[k][usize::from(pay)][p] += 1;
                    continue;
                }
                match k {
                    0 => {
                        assert_eq!(
                            view.players[packet.controller.other().index()].life,
                            base.players[packet.controller.other().index()].life
                                - if pay { 4 } else { 2 }
                        );
                        damage[usize::from(pay)] += if pay { 4 } else { 2 };
                    }
                    1 => {
                        for t in &packet.targets {
                            let v = view.objects.iter().find(|v| v.id == t.object.unwrap().card);
                            assert!(v.is_none_or(|v| v.zone == Zone::Graveyard));
                        }
                        damage[usize::from(pay)] += if pay { 5 } else { 2 };
                    }
                    2 => {
                        let victim = packet.targets[0].target;
                        let Target::Player(victim) = victim else {
                            unreachable!()
                        };
                        let old = base
                            .objects
                            .iter()
                            .filter(|v| {
                                v.owner == victim
                                    && v.zone == Zone::Battlefield
                                    && v.card.is_creature()
                            })
                            .count();
                        let new = view
                            .objects
                            .iter()
                            .filter(|v| {
                                v.owner == victim
                                    && v.zone == Zone::Battlefield
                                    && v.card.is_creature()
                            })
                            .count();
                        assert_eq!(new, old.saturating_sub(1));
                        sacrifice += u64::from(old > 0);
                    }
                    3 => {
                        let chosen = selected.remove(&id).unwrap();
                        assert_eq!(chosen.len(), if pay { 2 } else { 1 });
                        for target in chosen {
                            let v = view.objects.iter().find(|v| v.id == target).unwrap();
                            assert_eq!(v.zone, Zone::Battlefield);
                            assert!(!v.tapped);
                            lands += 1;
                        }
                        searches += 1;
                    }
                    4 | 5 => {
                        for t in &packet.targets {
                            let r = t.object.unwrap();
                            if let Some(v) = view.objects.iter().find(|v| v.id == r.card) {
                                assert_eq!(v.zone, Zone::Hand);
                                assert_eq!(v.reference.generation, r.generation + 1);
                            }
                            bounced += 1;
                        }
                        assert_eq!(
                            view.players[p].cards_drawn_this_turn,
                            base.players[p].cards_drawn_this_turn + u64::from(pay)
                        );
                        drawn += u64::from(pay);
                    }
                    6..=8 => {
                        for t in &packet.targets {
                            let r = t.object.unwrap();
                            let v = view.objects.iter().find(|v| v.id == r.card).unwrap();
                            assert_eq!(v.zone, if pay { Zone::Battlefield } else { Zone::Hand });
                            assert_eq!(v.reference.generation, r.generation + 1);
                            if k == 8 {
                                assert_eq!(v.tapped, pay);
                            }
                            reanimate += u64::from(pay);
                        }
                    }
                    _ => unreachable!(),
                }
                effects[k][usize::from(pay)][p] += 1;
            }
        }
        assert!(g.outcome().is_some(), "unfinished seed {seed}");
    }
    if games >= 100 {
        assert!(
            casts.iter().flatten().flatten().all(|n| *n > 0),
            "casts {casts:?}"
        );
        assert!(
            effects.iter().flatten().flatten().all(|n| *n > 0),
            "effects {effects:?}"
        );
        assert!(
            searches > 0
                && lands > searches
                && grants > 0
                && sacrifice > 0
                && reanimate > 0
                && bounced > 0
                && drawn > 0
        );
    }
    println!(
        "Completed {games} fixed-kicker Standard games, {decisions} decisions/{replayed} replay boundaries; cast[card][kicked][controller] {casts:?}; effect[card][kicked][controller] {effects:?}; damage2/boosted {damage:?}, {searches} searches/{lands} normal-entry lands/{private} private frames, {grants} live counter grants, {sacrifice} sacrifices, {reanimate} reanimations, {bounced} bounces/{drawn} extra draws, {fizzles} all-invalid fizzles, {invalid} rejected actions; max {max_bytes} logical bytes"
    );
    Ok(())
}
