//! Legal Standard explore/Map self-play. Every action has an owned clone/restore replay.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Keywords, ObjectRef, Observation, Player,
    SpellView, StackKind, Target, Zone,
};
use std::collections::{BTreeMap, BTreeSet};
const NAMES: [&str; 15] = [
    "Cartographer's Companion",
    "Cenote Scout",
    "Fanatical Offering",
    "Get Lost",
    "Kinjalli's Dawnrunner",
    "Miner's Guidewing",
    "Over the Edge",
    "Pathfinding Axejaw",
    "River Herald Guide",
    "River Herald Scout",
    "Seeker of Sunlight",
    "Sentinel of the Nameless City",
    "Spyglass Siren",
    "Waterwind Scout",
    "Merfolk Cave-Diver",
];
fn index(c: Card) -> Option<usize> {
    NAMES.iter().position(|n| *n == c.name())
}
fn is_map(c: Card) -> bool {
    c.token_info().is_some_and(|t| t.subtypes == "Map")
}
fn deck(i: usize) -> Vec<Card> {
    let mut d = Vec::new();
    for (name, n) in [("Plains", 8), ("Island", 8), ("Swamp", 4), ("Forest", 12)] {
        d.extend(vec![Card::from_name(name).unwrap(); n]);
    }
    let mut used = BTreeSet::new();
    for name in [
        NAMES[i],
        "Bear Cub",
        "Cenote Scout",
        "Spyglass Siren",
        "Vampiric Rites",
        "Fell",
        "Opt",
    ] {
        let name = if used.contains(name) {
            if !used.contains("River Herald Scout") {
                "River Herald Scout"
            } else {
                "Three Tree Mascot"
            }
        } else {
            name
        };
        assert!(used.insert(name));
        d.extend([Card::from_name(name).unwrap(); 4]);
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
fn score(o: &Observation, a: &Action, i: usize, seed: usize, fallback: bool) -> i32 {
    let p = o.actor.unwrap();
    let obj = |id| o.objects.iter().find(|v| v.id == id).unwrap();
    let primary = o
        .objects
        .iter()
        .any(|v| v.owner == p && v.zone == Zone::Battlefield && v.card.name() == NAMES[i]);
    let wing = o.objects.iter().any(|v| {
        v.owner == p && v.zone == Zone::Battlefield && v.card.name() == "Miner's Guidewing"
    });
    match a {
        Action::Keep => 2000,
        Action::Mulligan | Action::Concede => -10000,
        Action::Cast { card, .. } | Action::AnnounceCast { card } => {
            let n = obj(*card).card.name();
            if n == NAMES[i] {
                if !primary || matches!(i, 1 | 4 | 7 | 8 | 9) {
                    1000
                } else {
                    -100
                }
            } else if n == "Fell" {
                if i == 5 && wing { 1200 } else { -100 }
            } else if n == "Cenote Scout" || n == "River Herald Scout" {
                if i == 14 && !primary { -100 } else { 650 }
            } else if n == "Opt" {
                400
            } else {
                600
            }
        }
        Action::Activate { card, .. }
        | Action::BeginActivate { card, .. }
        | Action::AnnounceActivate { card, .. } => {
            let v = obj(*card);
            if is_map(v.card) {
                1250
            } else if v.card.name() == "Seeker of Sunlight" {
                900
            } else if v.card.name() == "Vampiric Rites" && i == 5 && wing {
                1300
            } else {
                -100
            }
        }
        Action::ChooseAdditionalCost { object } => {
            if obj(object.card).card.name() == "Miner's Guidewing" && i == 5 {
                2000
            } else if is_map(obj(object.card).card) {
                1600
            } else {
                1000
            }
        }
        Action::ChooseTarget {
            target: Target::Card(id),
        } => {
            let v = obj(*id);
            let source = o.announcing.as_ref().unwrap();
            if source.source().is_some_and(|source| source.card.name() == "Fell") {
                if v.owner == p && v.card.name() == "Miner's Guidewing" {
                    2000
                } else {
                    50
                }
            } else if source.source().is_some_and(|source| source.card.name() == "Get Lost") {
                if v.owner != p && v.card.is_creature() {
                    1800
                } else {
                    500
                }
            } else if source.source().is_some_and(|source| source.card.name() == "Over the Edge") && source.mode == Some(0) {
                if is_map(v.card) {
                    2000
                } else if v.owner != p {
                    1500
                } else {
                    700
                }
            } else if v.owner == p {
                if v.card.name() == "Bear Cub" || i == 14 && v.card.name() == NAMES[i] {
                    1900
                } else {
                    1200
                }
            } else {
                100
            }
        }
        Action::ChooseTarget { .. } => 100,
        Action::ChooseExplore { graveyard, .. } => {
            if *graveyard == (seed + o.turn as usize + p.index()).is_multiple_of(2) {
                2000
            } else {
                1500
            }
        }
        Action::ChooseMode { mode } => {
            if *mode == ((seed / 15 + p.index()) % 2) as u16 {
                2000
            } else {
                1000
            }
        }
        Action::PlayLand { card } => {
            let v = obj(*card);
            let have = o
                .objects
                .iter()
                .filter(|x| x.owner == p && x.zone == Zone::Battlefield && x.card == v.card)
                .count();
            1500 - have as i32 * 20
        }
        Action::PayCosts { .. } | Action::PutTrigger { .. } | Action::FinishTargets => 1800,
        Action::TapMana { .. } | Action::ActivateMana { .. } | Action::AnnounceMana { .. } => {
            if o.decision == "pay_costs" {
                1200
            } else {
                -100
            }
        }
        Action::Attack { card } => {
            if matches!(
                obj(*card).card.name(),
                "Sentinel of the Nameless City" | "Merfolk Cave-Diver"
            ) {
                1000
            } else {
                -100
            }
        }
        Action::Block { .. } => {
            if fallback {
                500
            } else {
                200
            }
        }
        Action::ArrangeLibrary { destination, .. } => {
            if *destination == rustmage::LibraryDestination::Bottom {
                1000
            } else {
                800
            }
        }
        Action::Pass | Action::FinishAttackers | Action::FinishBlockers => 0,
        _ => {
            if fallback {
                900
            } else {
                100
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
        mut rejected,
        mut reveals,
        mut events,
        mut lands,
        mut bins,
        mut keeps,
        mut empties,
        mut max_bytes,
    ) = (0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0usize);
    let (mut casts, mut effects, mut activations) =
        ([[0u64; 2]; 15], [[0u64; 2]; 15], [[0u64; 2]; 15]);
    let (mut modes, mut attacks, mut buffs, mut unblockable, mut cleanup) =
        ([[0u64; 2]; 2], [0u64; 2], [0u64; 2], [0u64; 2], [0u64; 2]);
    for seed in 0..games {
        let i = seed % 15;
        let mut g = Game::new(Config {
            decks: [deck(i), deck(i)],
            seed: seed as u64,
            starting_player: if (seed / 15).is_multiple_of(2) {
                Player::P0
            } else {
                Player::P1
            },
            deck_rules: DeckRules::Standard,
        })?;
        let mut active = BTreeMap::<u64, (SpellView, Observation)>::new();
        let mut provenance = BTreeMap::<ObjectRef, usize>::new();
        let mut buffed = BTreeMap::<ObjectRef, (Player, u32)>::new();
        let mut declared = BTreeSet::new();
        for _ in 0..100000 {
            if g.outcome().is_some() {
                break;
            }
            let p = g.actor().unwrap();
            let before = g.observe(p);
            let other = g.observe(p.other());
            assert_eq!(before.explore_choice, other.explore_choice);
            assert_eq!(before.last_explore, other.last_explore);
            assert_eq!(before.known_library_top, other.known_library_top);
            if let Some(c) = &before.explore_choice {
                reveals += 1;
                assert!(other.actions.is_empty());
                assert!(!c.revealed.card.is_land());
                assert_eq!(
                    before.known_library_top[c.explorer.controller.index()],
                    Some(c.revealed)
                );
                assert!(before.library_choice.is_none());
                let saved = g.snapshot();
                assert_eq!(
                    g.apply(
                        p.other(),
                        Action::ChooseExplore {
                            object: c.revealed.object,
                            graveyard: false
                        }
                    ),
                    Err(Error::WrongActor)
                );
                assert_eq!(g.snapshot(), saved);
                let wrong = ObjectRef {
                    card: c.revealed.object.card,
                    generation: c.revealed.object.generation + 1,
                };
                assert_eq!(
                    g.apply(
                        p,
                        Action::ChooseExplore {
                            object: wrong,
                            graveyard: false
                        }
                    ),
                    Err(Error::IllegalAction)
                );
                assert_eq!(g.snapshot(), saved);
                rejected += 2;
            }
            for r in before.known_library_top.iter().flatten() {
                assert!(!before.objects.iter().any(|v| v.reference == r.object));
            }
            for v in &before.objects {
                if v.zone == Zone::Battlefield
                    && v.card.name() == "Merfolk Cave-Diver"
                    && v.keywords.contains(Keywords::CANT_BE_BLOCKED)
                {
                    buffed.entry(v.reference).or_insert((v.owner, before.turn));
                }
                if v.zone == Zone::Battlefield
                    && let Some(&(owner, turn)) = buffed.get(&v.reference)
                    && turn != before.turn
                    && !v.keywords.contains(Keywords::CANT_BE_BLOCKED)
                {
                    cleanup[owner.index()] += 1;
                    buffed.remove(&v.reference);
                }
            }
            if before.decision == "blockers" {
                for v in before.objects.iter().filter(|v| {
                    v.attacking
                        && v.card.name() == "Merfolk Cave-Diver"
                        && v.keywords.contains(Keywords::CANT_BE_BLOCKED)
                }) {
                    assert!(
                        !before
                            .actions
                            .iter()
                            .any(|a| matches!(a,Action::Block{attacker,..}if *attacker==v.id))
                    );
                    if before.objects.iter().any(|o| {
                        o.owner == p
                            && o.zone == Zone::Battlefield
                            && o.card.is_creature()
                            && !o.tapped
                    }) {
                        unblockable[v.owner.index()] += 1;
                    }
                }
            }
            let fallback = rustmage::policy::smoke_index(&g, &before.actions);
            let j = before
                .actions
                .iter()
                .enumerate()
                .max_by_key(|(j, a)| score(&before, a, i, seed, *j == fallback))
                .unwrap()
                .0;
            let action = before.actions[j].clone();
            if let Action::PayCosts { .. } = action
                && let Some(item) = &before.announcing
            {
                match item.kind {
                    StackKind::CasualtyCopy
                    | StackKind::CopyEndStepSacrifice
                    | StackKind::DelayedReturn
                    | StackKind::DelayedTokenCreation { .. }
                    | StackKind::DelayedSacrifice
                    | StackKind::DelayedWarpExile
                    | StackKind::DelayedManifestDread
                    | StackKind::SiegeDefeat => {
                        panic!("no delayed producer in this fixture")
                    }
                    StackKind::Spell => {
                        if let Some(k) = item.source().and_then(|source| index(source.card)) {
                            casts[k][p.index()] += 1;
                        }
                    }
                    StackKind::ActivatedAbility { .. } if item.source().is_some_and(|source| is_map(source.card)) => {
                        if let Some(k) = provenance.get(&item.source().expect("example expects an object-backed packet").object) {
                            activations[*k][p.index()] += 1;
                        }
                    }
                    _ => {}
                }
            }
            if matches!(action, Action::Pass)
                && let Some(item) = before.stack.last()
                && (item.source().and_then(|source| index(source.card)).is_some() || item.source().is_some_and(|source| is_map(source.card)))
            {
                active.insert(item.id, (item.clone(), g.observe(item.controller)));
            }
            if let Action::Attack { card } = action
                && let Some(v) = before.objects.iter().find(|v| v.id == card)
                && v.card.name() == "Sentinel of the Nameless City"
            {
                declared.insert(v.reference);
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
            g.apply(p, action.clone())?;
            assert_eq!(g.snapshot(), expected);
            let after = g.observe(p);
            assert_eq!(after.schema, rustmage::OBSERVATION_SCHEMA);
            decisions += 1;
            max_bytes = max_bytes.max(g.state_bytes());
            if matches!(action, Action::FinishAttackers) {
                for r in &declared {
                    if let Some(v) = after.objects.iter().find(|v| v.reference == *r) {
                        assert!(!v.tapped);
                        attacks[v.owner.index()] += 1;
                    }
                }
                declared.clear();
            }
            let old_seq = before.last_explore.as_ref().map_or(0, |e| e.sequence);
            let new_seq = after.last_explore.as_ref().map_or(0, |e| e.sequence);
            assert!(new_seq >= old_seq);
            if new_seq > old_seq {
                let e = after.last_explore.as_ref().unwrap();
                events += new_seq - old_seq;
                let selected_nonland = if let Action::ChooseExplore { graveyard, .. } = action {
                    if graveyard {
                        bins += 1;
                    } else {
                        keeps += 1;
                    }
                    1
                } else {
                    0
                };
                let public_lands = after
                    .known_hand
                    .iter()
                    .filter(|r| !before.known_hand.contains(r))
                    .count() as u64;
                for r in after
                    .known_hand
                    .iter()
                    .filter(|r| !before.known_hand.contains(r))
                {
                    assert!(
                        after
                            .objects
                            .iter()
                            .find(|v| v.reference == *r)
                            .unwrap()
                            .card
                            .is_land()
                    );
                }
                lands += public_lands;
                assert!(new_seq - old_seq >= selected_nonland + public_lands);
                empties += new_seq - old_seq - selected_nonland - public_lands;
                let owner = e.explorer.controller;
                assert_eq!(
                    after.players[owner.index()].cards_drawn_this_turn,
                    before.players[owner.index()].cards_drawn_this_turn
                );
                match e.destination {
                    Some(Zone::Hand) => {
                        let card = e.revealed.unwrap();
                        assert!(card.card.is_land());
                        let result = e.result.unwrap();
                        assert_eq!(result.card, card.object.card);
                        assert_eq!(result.generation, card.object.generation + 1);
                        assert!(after.known_hand.contains(&result));
                        assert!(!e.counter_added);
                    }
                    Some(Zone::Graveyard) => {
                        assert!(!e.revealed.unwrap().card.is_land());
                        assert_eq!(
                            e.result.unwrap().generation,
                            e.revealed.unwrap().object.generation + 1
                        );
                    }
                    Some(Zone::Library) => {
                        assert_eq!(e.result, Some(e.revealed.unwrap().object));
                        assert_eq!(after.known_library_top[owner.index()], e.revealed);
                    }
                    None => {
                        assert!(e.revealed.is_none());
                        assert!(e.result.is_none());
                    }
                    other => panic!("unexpected explore destination {other:?}"),
                }
            }
            if let Some(c) = &after.explore_choice {
                let changed = before.explore_choice.is_none() || new_seq > old_seq;
                if changed && c.counter_added {
                    let old = before
                        .objects
                        .iter()
                        .find(|v| v.reference == c.explorer.object)
                        .unwrap();
                    let now = after
                        .objects
                        .iter()
                        .find(|v| v.reference == c.explorer.object)
                        .unwrap();
                    assert_eq!(now.counters.plus_one, old.counters.plus_one + 1);
                }
            }
            let finished = active
                .keys()
                .copied()
                .filter(|id| !packets(&after).any(|s| s.id == *id))
                .collect::<Vec<_>>();
            for id in finished {
                let (item, base) = active.remove(&id).unwrap();
                let view = g.observe(item.controller);
                let owner = item.controller;
                let k = item.source().and_then(|source| index(source.card));
                let invalid = !item.targets.is_empty()
                    && item.targets.iter().all(|t| {
                        t.object
                            .is_some_and(|r| !base.objects.iter().any(|v| v.reference == r))
                    });
                if invalid {
                    continue;
                }
                if item.kind == StackKind::Spell && item.source().is_some_and(|source| source.card.is_creature()) {
                    let v = view.objects.iter().find(|v| v.id == item.source().expect("example expects an object-backed packet").id).unwrap();
                    assert_eq!(v.zone, Zone::Battlefield);
                    let expected = [
                        (2, 1),
                        (1, 1),
                        (0, 0),
                        (0, 0),
                        (1, 1),
                        (1, 1),
                        (0, 0),
                        (4, 3),
                        (3, 1),
                        (1, 2),
                        (1, 1),
                        (3, 4),
                        (1, 1),
                        (2, 2),
                        (2, 4),
                    ][k.unwrap()];
                    assert_eq!((v.power, v.toughness), expected);
                    continue;
                }
                let before_maps = BTreeSet::from_iter(
                    base.objects
                        .iter()
                        .filter(|v| v.zone == Zone::Battlefield && is_map(v.card))
                        .map(|v| v.reference),
                );
                let new_maps = view
                    .objects
                    .iter()
                    .filter(|v| {
                        v.zone == Zone::Battlefield
                            && is_map(v.card)
                            && !before_maps.contains(&v.reference)
                    })
                    .collect::<Vec<_>>();
                if let Some(k) = k {
                    for v in &new_maps {
                        provenance.insert(v.reference, k);
                    }
                }
                if let Some(k) = k {
                    match k {
                        0 | 11 | 12 | 13 => {
                            assert_eq!(new_maps.len(), 1);
                            assert_eq!(new_maps[0].owner, owner);
                            effects[k][owner.index()] += 1;
                        }
                        2 => {
                            assert_eq!(
                                view.players[owner.index()].cards_drawn_this_turn,
                                base.players[owner.index()].cards_drawn_this_turn
                                    + base.players[owner.index()].library_count.min(2) as u64
                            );
                            assert_eq!(new_maps.len(), 1);
                            if base.players[owner.index()].library_count >= 2 {
                                effects[k][owner.index()] += 1;
                            }
                        }
                        3 => {
                            assert_eq!(new_maps.len(), 2);
                            let old = item.targets[0].object.unwrap();
                            let target = base.objects.iter().find(|v| v.reference == old).unwrap();
                            for v in &new_maps {
                                assert_eq!(v.owner, target.owner);
                            }
                            if !target.keywords.contains(Keywords::INDESTRUCTIBLE) {
                                assert!(
                                    !view
                                        .objects
                                        .iter()
                                        .any(|v| v.reference == old && v.zone == Zone::Battlefield)
                                );
                            }
                            effects[k][owner.index()] += 1;
                        }
                        6 => {
                            let mode = item.mode.unwrap();
                            modes[mode as usize][owner.index()] += 1;
                            if mode == 1 {
                                assert_eq!(
                                    view.last_explore.as_ref().unwrap().sequence,
                                    base.last_explore.as_ref().map_or(0, |e| e.sequence) + 2
                                );
                            } else {
                                for target in &item.targets {
                                    let r = target.object.unwrap();
                                    assert!(
                                        !view.objects.iter().any(
                                            |v| v.reference == r && v.zone == Zone::Battlefield
                                        )
                                    );
                                }
                            }
                            effects[k][owner.index()] += 1;
                        }
                        1 | 4 | 5 | 7 | 8 | 9 | 10 => {
                            assert_eq!(
                                view.last_explore.as_ref().unwrap().sequence,
                                base.last_explore.as_ref().map_or(0, |e| e.sequence) + 1
                            );
                            effects[k][owner.index()] += 1;
                        }
                        14 => {
                            if let Some(v) =
                                view.objects.iter().find(|v| v.reference == item.source().expect("example expects an object-backed packet").object)
                            {
                                let old = base
                                    .objects
                                    .iter()
                                    .find(|v| v.reference == item.source().expect("example expects an object-backed packet").object)
                                    .unwrap();
                                assert_eq!(v.power, old.power + 1);
                                assert!(v.keywords.contains(Keywords::CANT_BE_BLOCKED));
                                buffs[owner.index()] += 1;
                                effects[k][owner.index()] += 1;
                            }
                        }
                        _ => unreachable!(),
                    }
                }
            }
        }
        assert!(g.outcome().is_some(), "unfinished seed {seed}");
    }
    if games >= 100 {
        for (k, n) in NAMES.iter().enumerate() {
            for p in 0..2 {
                assert!(casts[k][p] > 0, "missing cast {n}/{p}");
                assert!(effects[k][p] > 0, "missing full effect {n}/{p}");
            }
        }
        for k in [0, 2, 3, 11, 12, 13] {
            assert!(
                activations[k].iter().all(|n| *n > 0),
                "missing Map use from {}",
                NAMES[k]
            );
        }
        for p in 0..2 {
            assert!(modes[0][p] > 0 && modes[1][p] > 0, "both OverEdge modes");
            assert!(
                attacks[p] > 0 && buffs[p] > 0 && unblockable[p] > 0 && cleanup[p] > 0,
                "attack/buff/unblockable/cleanup/{p}: {attacks:?}/{buffs:?}/{unblockable:?}/{cleanup:?}"
            );
        }
        assert!(lands > 0 && bins > 0 && keeps > 0);
        assert_eq!(events, lands + bins + keeps + empties);
    }
    println!(
        "Completed {games} Standard explore games: {decisions} exact decisions/replays, cast[card][seat] {casts:?}, full effects {effects:?}, provider Map activations {activations:?}, mode[mode][seat] {modes:?}, Sentinel attacks {attacks:?}, Cave buffs {buffs:?}/unblockable {unblockable:?}/cleanup {cleanup:?}, {events} completed explores/{reveals} public choices/{lands} land puts/{bins} grave/{keeps} top/{empties} empty/{rejected} invalid-action denials, max {max_bytes} logical bytes"
    );
    Ok(())
}
