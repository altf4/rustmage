//! Independent public Standard acceptance for target-qualified, paid, locked costs.
use rustmage::{
    Action, Card, CastMethod, Config, DeckRules, Error, Game, Keywords, Mana, ObjectRef,
    Observation, Player, SpellView, StackKind, Target, Zone,
};
use std::collections::{BTreeMap, BTreeSet};
const NAMES: [&str; 19] = [
    "Ajani's Response",
    "Brush Off",
    "Dark Endurance",
    "Depower",
    "Dire Downdraft",
    "Fate of the Sun-Cryst",
    "Grounded for Life",
    "Grow Extra Arms",
    "Luminous Rebuke",
    "Magnificent End",
    "Out of Air",
    "Quicksand Whirlpool",
    "Ride's End",
    "Roadside Blowout",
    "Run Behind",
    "Run Over",
    "Seized from Slumber",
    "Swampsnare Trap",
    "Uneasy Partings",
];
const DISCOUNT: [u32; 19] = [3, 1, 1, 2, 1, 2, 3, 1, 3, 3, 2, 3, 3, 2, 1, 1, 3, 1, 1];
fn idx(c: Card) -> Option<usize> {
    NAMES.iter().position(|n| *n == c.name())
}
fn printed(c: Card) -> (Mana, u32) {
    let mut colors = [0; 6];
    let mut generic = 0;
    for symbol in c
        .info()
        .mana_cost
        .split(['{', '}'])
        .filter(|s| !s.is_empty())
    {
        if let Ok(n) = symbol.parse::<u32>() {
            generic += n;
        } else {
            let half = symbol.split('/').next().unwrap();
            if let Ok(n) = half.parse::<u32>() {
                generic += n;
            } else {
                colors["WUBRGC".find(half).unwrap()] += 1;
            }
        }
    }
    (colors, generic)
}
fn subtype(c: Card, s: &str) -> bool {
    c.info()
        .type_line
        .split('—')
        .nth(1)
        .is_some_and(|line| line.split_whitespace().any(|t| t == s))
        || matches!(s, "Mount" | "Spider") && c.info().keywords.contains(Keywords::CHANGELING)
}
fn qualifies(i: usize, p: Player, t: Target, b: &Observation) -> bool {
    match t {
        Target::Spell(id) => b
            .stack
            .iter()
            .find(|s| s.kind == StackKind::Spell && s.card == id)
            .is_some_and(|s| match i {
                1 => {
                    s.source().expect("example expects an object-backed packet").card.is_instant()
                        || s.source().expect("example expects an object-backed packet").card.info().type_line.starts_with("Sorcery")
                }
                10 => s.source().is_some_and(|source| source.card.is_creature()),
                _ => false,
            }),
        Target::Card(id) => b
            .objects
            .iter()
            .find(|v| v.id == id && v.zone == Zone::Battlefield)
            .is_some_and(|v| match i {
                0 | 5 | 6 | 8 | 9 | 11 | 16 => v.card.is_creature() && v.tapped,
                2 => v.card.is_creature() && b.blocking.contains(&id),
                3 | 14 => v.card.is_creature() && b.attackers.contains(&id),
                4 => v.card.is_creature() && (v.tapped || b.attackers.contains(&id)),
                7 => subtype(v.card, "Spider"),
                12 => v.tapped,
                13 => {
                    let (colors, generic) = printed(v.card);
                    colors.iter().sum::<u32>() + generic == 1
                }
                15 => v.owner == p && (subtype(v.card, "Mount") || subtype(v.card, "Vehicle")),
                17 => v.card.is_creature() && v.keywords.contains(Keywords::FLYING),
                18 => v.card.is_creature() && b.attackers.contains(&id) && v.token.is_none(),
                _ => false,
            }),
        _ => false,
    }
}
fn object(o: &Observation, r: ObjectRef) -> Option<&rustmage::ObjectView> {
    o.objects.iter().find(|v| v.reference == r)
}
fn expected(s: &SpellView, b: &Observation) -> (Mana, u32, bool) {
    let i = idx(s.source().expect("example expects an object-backed packet").card).unwrap();
    let (mut colors, generic) = printed(s.source().expect("example expects an object-backed packet").card);
    let discounted = s.targets.iter().any(|t| {
        t.object.is_some_and(|r| object(b, r).is_some()) && qualifies(i, s.controller, t.target, b)
    });
    let mentors =
        if s.source().expect("example expects an object-backed packet").card.is_instant() || s.source().expect("example expects an object-backed packet").card.info().type_line.starts_with("Sorcery") {
            b.objects
                .iter()
                .filter(|v| {
                    v.owner == s.controller
                        && v.zone == Zone::Battlefield
                        && v.card.name() == "Stormcatch Mentor"
                })
                .count() as u32
        } else {
            0
        };
    if discounted && i == 1 {
        colors[1] = colors[1].saturating_sub(1);
    }
    (
        colors,
        generic
            .saturating_sub(mentors)
            .saturating_sub(if discounted { DISCOUNT[i] } else { 0 }),
        discounted,
    )
}
fn deck(f: usize) -> Vec<Card> {
    let mut d = Vec::new();
    for n in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        d.extend([Card::from_name(n).unwrap(); 6]);
    }
    for n in [
        NAMES[f],
        "Llanowar Elves",
        "Three Tree Mascot",
        "Mischievous Sneakling",
        "Malevolent Chandelier",
        "Bear Cub",
        "Stormcatch Mentor",
    ] {
        d.extend([Card::from_name(n).unwrap(); 4]);
    }
    d.extend([Card::from_name("Opt").unwrap(); 2]);
    assert_eq!(d.len(), 60);
    d
}
fn witness(s: &SpellView, b: &Observation, a: &Observation) {
    let i = idx(s.source().expect("example expects an object-backed packet").card).unwrap();
    let targets: Vec<_> = s
        .targets
        .iter()
        .filter_map(|t| t.object)
        .filter_map(|r| object(b, r))
        .collect();
    if targets.is_empty() {
        return;
    }
    match i {
        0 | 5 | 6 | 8 | 16 => {
            let v = targets[0];
            if !v.keywords.contains(Keywords::INDESTRUCTIBLE) {
                assert_eq!(
                    a.objects.iter().find(|x| x.id == v.id).unwrap().zone,
                    Zone::Graveyard
                );
            }
        }
        1 | 10 => {
            assert_eq!(
                a.objects
                    .iter()
                    .find(|x| x.id == targets[0].id)
                    .unwrap()
                    .zone,
                Zone::Graveyard
            );
        }
        2 | 3 | 7 => {
            let v = targets[0];
            if let Some(now) = object(a, v.reference) {
                let power = match i {
                    2 => 2,
                    3 => -4,
                    _ => 4,
                };
                assert_eq!(
                    (now.power, now.toughness),
                    (v.power + power, v.toughness + if i == 7 { 4 } else { 0 })
                );
                if i == 2 {
                    assert!(now.keywords.contains(Keywords::INDESTRUCTIBLE));
                }
            }
            if i == 3 {
                assert_eq!(
                    a.players[s.controller.index()].cards_drawn_this_turn,
                    b.players[s.controller.index()].cards_drawn_this_turn + 1
                );
            }
        }
        4 | 14 | 18 => {
            let event = a.last_target_library_move.expect("public exact tuck event");
            assert_eq!(
                (event.source, event.target_before, event.owner),
                (s.object, targets[0].reference, targets[0].owner)
            );
            assert_eq!(
                event.target_after.generation,
                targets[0].reference.generation + 1
            );
            assert!(!a.objects.iter().any(|v| v.reference == event.target_after));
            assert_eq!(
                a.players[event.owner.index()].library_count,
                b.players[event.owner.index()].library_count + 1
            );
        }
        9 => {
            let v = targets[0];
            let now = a.objects.iter().find(|x| x.id == v.id).unwrap();
            if v.damage + 5 >= v.toughness && !v.keywords.contains(Keywords::INDESTRUCTIBLE) {
                assert_eq!(now.zone, Zone::Graveyard);
            } else {
                assert_eq!(now.damage, v.damage + 5);
            }
        }
        11 | 12 => {
            assert_eq!(
                a.objects
                    .iter()
                    .find(|x| x.id == targets[0].id)
                    .unwrap()
                    .zone,
                Zone::Exile
            );
        }
        13 => {
            assert_eq!(
                a.objects
                    .iter()
                    .find(|x| x.id == targets[0].id)
                    .unwrap()
                    .zone,
                Zone::Hand
            );
            assert_eq!(
                a.players[s.controller.index()].cards_drawn_this_turn,
                b.players[s.controller.index()].cards_drawn_this_turn + 1
            );
        }
        15 => {
            let source = s
                .targets
                .iter()
                .find(|t| t.group == 0)
                .and_then(|t| t.object)
                .and_then(|r| object(b, r))
                .unwrap();
            let target = s
                .targets
                .iter()
                .find(|t| t.group == 1)
                .and_then(|t| t.object)
                .and_then(|r| object(b, r))
                .unwrap();
            let now = a.objects.iter().find(|v| v.id == target.id).unwrap();
            if source.power > 0
                && target.damage + source.power >= target.toughness
                && !target.keywords.contains(Keywords::INDESTRUCTIBLE)
            {
                assert_eq!(now.zone, Zone::Graveyard);
            } else {
                assert_eq!(now.damage, target.damage + source.power.max(0));
            }
        }
        17 => {
            let v = targets[0];
            let aura = a.objects.iter().find(|v| v.id == s.card).unwrap();
            if v.toughness <= 3 {
                assert_eq!(aura.zone, Zone::Graveyard);
            } else {
                let now = object(a, v.reference).unwrap();
                assert_eq!((now.power, now.toughness), (v.power - 5, v.toughness - 3));
                assert_eq!(aura.attached_to, Some(v.reference));
            }
        }
        _ => unreachable!(),
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games = std::env::args()
        .nth(1)
        .map_or(Ok(100), |s| s.parse::<u64>())?;
    let mut paid = [[0u64; 2]; 19];
    let mut effects = [0u64; 19];
    let mut owners = [0; 2];
    let mut decisions = 0;
    let mut replays = 0;
    let mut private = 0;
    let mut locked = 0;
    let mut max_bytes = 0;
    let mut initial_bytes = 0;
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [deck(seed as usize % 19), deck((seed as usize + 9) % 19)],
            seed: seed + 21100,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        initial_bytes = g.state_bytes();
        let mut costs = BTreeMap::new();
        let mut pending = BTreeMap::new();
        let mut seen = BTreeSet::new();
        let mut local = 0;
        while g.outcome().is_none() {
            local += 1;
            assert!(local < 50000, "fixture seed{seed} failed to terminate");
            let p = g.actor().unwrap();
            let b = g.observe(p);
            let actions = g.legal_actions();
            let fallback = rustmage::policy::smoke_index(&g, &actions);
            if b.decision == "pay_costs"
                && let Some(s) = &b.announcing
                && s.source().and_then(|source| idx(source.card)).is_some()
            {
                let want = *costs.entry(s.id).or_insert_with(|| expected(s, &b));
                let request = b.payment_request.as_ref().unwrap();
                assert_eq!((request.required, request.generic), (want.0, want.1));
                locked += 1;
            }
            if b.library_choice.is_some() || b.target_library.is_some() {
                private += 1;
                assert!(g.observe(p.other()).library_choice.is_none());
                assert!(g.observe(p.other()).target_library.is_none());
            }
            let selected = actions
                .iter()
                .enumerate()
                .max_by_key(|(j, a)| {
                    let ordinary = if *j == fallback { 100 } else { 0 };
                    match a {
                        Action::AnnounceCast { card } | Action::Cast { card, .. } => {
                            let v = b.objects.iter().find(|v| v.id == *card).unwrap();
                            if let Some(i) = idx(v.card) {
                                if matches!(a, Action::Cast { .. }) {
                                    -100
                                } else {
                                    let possible = b
                                        .objects
                                        .iter()
                                        .any(|v| qualifies(i, p, Target::Card(v.id), &b))
                                        || b.stack
                                            .iter()
                                            .any(|s| qualifies(i, p, Target::Spell(s.card), &b));
                                    if possible {
                                        400
                                    } else if seed % 4 == 0 {
                                        200
                                    } else {
                                        -100
                                    }
                                }
                            } else {
                                ordinary
                            }
                        }
                        Action::ChooseTarget { target } => {
                            if let Some(s) = &b.announcing
                                && let Some(i) = s.source().and_then(|source| idx(source.card))
                            {
                                if qualifies(i, p, *target, &b) {
                                    700
                                } else {
                                    let beneficial = matches!(i, 2 | 7)
                                        || i == 15 && b.targeting_group == Some(0);
                                    let own = match target {
                                        Target::Card(id) => {
                                            b.objects.iter().any(|v| v.id == *id && v.owner == p)
                                        }
                                        _ => false,
                                    };
                                    if own == beneficial { 500 } else { 300 }
                                }
                            } else {
                                ordinary
                            }
                        }
                        Action::PayCosts { .. } => 900,
                        Action::TapMana { .. } => {
                            if b.decision == "pay_costs" {
                                600
                            } else {
                                -100
                            }
                        }
                        Action::AnnounceActivate { .. }
                        | Action::Activate { .. }
                        | Action::BeginActivate { .. }
                        | Action::AnnounceMana { .. } => -100,
                        Action::ChooseLibraryDestination { destination, .. } => {
                            if *destination == rustmage::LibraryDestination::Top {
                                600
                            } else {
                                500
                            }
                        }
                        Action::Concede => -1000,
                        _ => ordinary,
                    }
                })
                .unwrap()
                .0;
            let action = actions[selected].clone();
            let old = g.snapshot();
            let rev = g.revision();
            let mut fork = g.clone();
            fork.apply(p, action.clone())?;
            g.apply(p, action.clone())?;
            let expected_state = g.snapshot();
            assert_eq!(fork.snapshot(), expected_state);
            g.restore(&old)?;
            assert_eq!(g.apply_index(p, rev, selected), Err(Error::StaleRevision));
            g.apply(p, action.clone())?;
            assert_eq!(g.snapshot(), expected_state);
            replays += 1;
            let a = g.observe(p);
            if let Action::PayCosts { payment } = action
                && let Some(s) = &b.announcing
                && let Some(i) = s.source().and_then(|source| idx(source.card))
            {
                let (colored, generic, discount) = costs[&s.id];
                assert!(
                    payment
                        .iter()
                        .zip(colored)
                        .all(|(paid, need)| *paid >= need)
                );
                assert_eq!(
                    payment
                        .iter()
                        .zip(colored)
                        .map(|(paid, need)| u64::from(paid - need))
                        .sum::<u64>(),
                    u64::from(generic)
                );
                assert_eq!(s.cast.unwrap().origin, Zone::Hand);
                assert_eq!(s.cast.unwrap().method, CastMethod::Normal);
                paid[i][usize::from(discount)] += 1;
                owners[p.index()] += 1;
            }
            for s in &a.stack {
                if s.kind == StackKind::Spell && s.source().and_then(|source| idx(source.card)).is_some() {
                    seen.insert(s.id);
                }
            }
            if matches!(action, Action::Pass)
                && let Some(s) = b.stack.last()
                && s.kind == StackKind::Spell
                && s.source().and_then(|source| idx(source.card)).is_some()
                && !a.stack.iter().any(|t| t.id == s.id)
            {
                pending.insert(s.id, (s.clone(), b.clone()));
            }
            let done: Vec<_> = pending
                .keys()
                .copied()
                .filter(|id| {
                    !a.stack.iter().any(|s| s.id == *id)
                        && a.resolving.as_ref().is_none_or(|s| s.id != *id)
                })
                .collect();
            for id in done {
                let (s, before) = pending.remove(&id).unwrap();
                let valid = s
                    .targets
                    .iter()
                    .any(|t| t.object.is_none_or(|r| object(&before, r).is_some()));
                if valid && a.outcome.is_none() {
                    witness(&s, &before, &a);
                    effects[idx(s.source().expect("example expects an object-backed packet").card).unwrap()] += 1;
                }
            }
            max_bytes = max_bytes.max(g.state_bytes());
            decisions += 1;
        }
    }
    if games >= 100 {
        assert!(paid.iter().all(|n| n[1] > 0), "missing discounts: {paid:?}");
        assert!(effects.iter().all(|n| *n > 0), "{effects:?}");
        assert!(owners.iter().all(|n| *n > 0));
        assert!(private > 0 && locked > 0);
    }
    println!(
        "catalogue={} games={games} decisions={decisions} replays={replays} paid_normal_discount={paid:?} effects={effects:?} owners={owners:?} private={private} locked={locked} initial_bytes={initial_bytes} max_bytes={max_bytes}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
