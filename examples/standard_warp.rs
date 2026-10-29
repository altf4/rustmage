//! Legal Standard Warp lifecycles and complete shared-grammar collateral bodies.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Keywords, LibraryDestination, ObjectRef,
    Observation, Player, StackKind, Target, Zone,
};
const NAMES: [&str; 14] = [
    "Bygone Colossus",
    "Eusocial Engineering",
    "Germinating Wurm",
    "Knight Luminary",
    "Red Tiger Mechan",
    "Sinister Cryologist",
    "Starbreach Whale",
    "Weftblade Enhancer",
    "Drix Fatemaker",
    "Mechanozoa",
    "Memorial Team Leader",
    "Nova Hellkite",
    "Runaway Boulder",
    "Waylaying Pirates",
];
const COLORS: [&str; 14] = [
    "Plains", "Forest", "Forest", "Plains", "Mountain", "Island", "Island", "Plains", "Forest",
    "Island", "Mountain", "Mountain", "Plains", "Island",
];
const STATS: [(i128, i128); 14] = [
    (9, 9),
    (0, 0),
    (5, 5),
    (3, 2),
    (3, 3),
    (2, 3),
    (3, 5),
    (3, 4),
    (3, 2),
    (5, 5),
    (4, 3),
    (4, 5),
    (0, 0),
    (3, 3),
];
fn deck(i: usize, focal: bool) -> Vec<Card> {
    let mut d = Vec::new();
    for (n, k) in [
        ("Forest", 8),
        ("Island", 8),
        ("Mountain", 8),
        (COLORS[i], 4),
    ] {
        d.extend(vec![Card::from_name(n).unwrap(); k]);
    }
    for n in [
        if focal { NAMES[i] } else { "Chrome Companion" },
        "Bear Cub",
        "Llanowar Elves",
        "Cartographer's Companion",
        "Ambush Wolf",
        "Shock",
        "Scorching Shot",
        "Thrill of Possibility",
    ] {
        d.extend([Card::from_name(n).unwrap(); 4]);
    }
    assert_eq!(d.len(), 60);
    d
}
#[derive(Default)]
struct Pilot {
    source: Option<u32>,
    stage: u8,
    paid: u8,
    land_turn: Option<u32>,
    reference: Option<ObjectRef>,
    exiled_turn: Option<u32>,
    target: Option<ObjectRef>,
    target_stun: u64,
    target_damage: i128,
    library: usize,
    life: i128,
    tokens: usize,
    bear_counters: u64,
    own_turn: bool,
    other_turn: bool,
}
fn field(o: &Observation, p: Player, n: &str) -> bool {
    o.objects
        .iter()
        .any(|v| v.owner == p && v.zone == Zone::Battlefield && v.card.name() == n)
}
fn token_count(o: &Observation, p: Player, i: usize) -> usize {
    o.objects
        .iter()
        .filter(|v| {
            v.owner == p
                && v.zone == Zone::Battlefield
                && v.card.is_token()
                && v.card.has_subtype(if i == 1 { "Robot" } else { "Human" })
        })
        .count()
}
fn ready(o: &Observation, p: Player, i: usize, s: &Pilot) -> bool {
    (![7, 8, 10].contains(&i) || field(o, p, "Bear Cub"))
        && (i != 13 || field(o, p, "Cartographer's Companion"))
        && (![5, 11, 12].contains(&i) || field(o, p.other(), "Bear Cub"))
        && (![9, 13].contains(&i) || field(o, p.other(), "Cartographer's Companion"))
        && (i != 1
            || (s.land_turn != Some(o.turn)
                && o.objects
                    .iter()
                    .any(|v| v.owner == p && v.zone == Zone::Hand && v.card.is_land())))
}
fn score(
    o: &Observation,
    a: &Action,
    p: Player,
    i: usize,
    s: &Pilot,
    warp: bool,
    cycling: bool,
) -> i32 {
    let actor = o.actor.unwrap();
    let obj = |id| o.objects.iter().find(|v| v.id == id).unwrap();
    match a {
        Action::Keep => 3000,
        Action::Mulligan | Action::Concede => -10000,
        Action::AnnounceWarp { card } => {
            if actor == p
                && warp
                && i < 12
                && s.stage == 0
                && obj(*card).card.name() == NAMES[i]
                && ready(o, p, i, s)
            {
                1900
            } else {
                -1000
            }
        }
        Action::AnnounceExile { card } => {
            if actor == p && s.stage == 3 && Some(*card) == s.source && ready(o, p, i, s) {
                1900
            } else {
                -1000
            }
        }
        Action::AnnounceActivate { card, .. } => {
            if actor == p
                && cycling
                && s.stage == 0
                && obj(*card).zone == Zone::Hand
                && obj(*card).card.name() == NAMES[i]
            {
                1900
            } else {
                -1000
            }
        }
        Action::Cast { .. }
        | Action::BeginCast { .. }
        | Action::Activate { .. }
        | Action::AnnouncePlotted { .. }
        | Action::AnnouncePlot { .. }
        | Action::AnnounceFlashback { .. } => -1000,
        Action::AnnounceCast { card } => {
            let v = obj(*card);
            let n = v.card.name();
            if s.stage == 6 {
                return if v.card.is_creature() {
                    1200
                } else if n == "Shock" || n == "Scorching Shot" {
                    1100
                } else {
                    -1000
                };
            }
            if actor != p {
                return if (([5, 11, 12].contains(&i) && n == "Bear Cub")
                    || ([9, 13].contains(&i) && n == "Cartographer's Companion"))
                    && !field(o, actor, n)
                {
                    1750
                } else {
                    -1000
                };
            }
            if n == NAMES[i] {
                return if !warp
                    && !cycling
                    && s.stage == 0
                    && ready(o, p, i, s)
                    && (i != 12 || o.active != p)
                {
                    1900
                } else {
                    -1000
                };
            }
            if (([7, 8, 10].contains(&i) && n == "Bear Cub")
                || (i == 13 && n == "Cartographer's Companion"))
                && !field(o, p, n)
            {
                return 1750;
            }
            -1000
        }
        Action::ChooseTarget {
            target: Target::Card(id),
        } => {
            let v = obj(*id);
            if actor == p && s.stage < 6 {
                if i == 7 && Some(*id) == s.source && v.zone == Zone::Battlefield {
                    return 2600;
                }
                if [7, 8].contains(&i)
                    && v.owner == p
                    && v.zone == Zone::Battlefield
                    && v.card.name() == "Bear Cub"
                {
                    return 2500;
                }
                if [5, 11, 12].contains(&i)
                    && v.owner != p
                    && v.zone == Zone::Battlefield
                    && v.card.name() == "Bear Cub"
                {
                    return 2500;
                }
                if [9, 13].contains(&i) && v.owner != p && v.zone == Zone::Battlefield {
                    return if v.card.has_subtype("Map") {
                        2600
                    } else {
                        2400
                    };
                }
                return 700;
            }
            if v.owner != actor { 1800 } else { 700 }
        }
        Action::ChooseTarget {
            target: Target::Player(who),
        } => {
            if *who != actor {
                1800
            } else {
                700
            }
        }
        Action::PayCosts { .. } | Action::ChooseSpellCost { .. } => 2400,
        Action::TapMana { .. } | Action::ActivateMana { .. } | Action::AnnounceMana { .. } => {
            if o.decision == "pay_costs" {
                2100
            } else {
                -1000
            }
        }
        Action::PlayLand { card } => {
            let n = obj(*card).card.name();
            let count = o
                .objects
                .iter()
                .filter(|v| v.owner == actor && v.zone == Zone::Battlefield && v.card.name() == n)
                .count();
            if n == COLORS[i] && count < 2 {
                1650
            } else if n == "Forest" && count < 2 {
                1620
            } else {
                1550 - 20 * count as i32
            }
        }
        Action::ArrangeLibrary { destination, .. } => {
            if *destination == LibraryDestination::Graveyard {
                2500
            } else {
                2300
            }
        }
        Action::Attack { .. } => {
            if s.stage == 6 {
                1700
            } else {
                -1000
            }
        }
        Action::Block { .. } => {
            if s.stage == 6 {
                1600
            } else {
                -1000
            }
        }
        Action::AssignDamage { amount, .. } => 1800 + (*amount).min(100) as i32,
        Action::Discard { card } | Action::Bottom { card } => {
            let v = obj(*card);
            let n = v.card.name();
            let count = o
                .objects
                .iter()
                .filter(|v| v.owner == actor && v.zone == Zone::Hand && v.card.name() == n)
                .count();
            let keep = s.stage < 6
                && ((actor == p
                    && ((n == NAMES[i] && s.stage == 0)
                        || ([7, 8, 10].contains(&i) && n == "Bear Cub")
                        || (i == 13 && n == "Cartographer's Companion")))
                    || (actor != p
                        && (([5, 11, 12].contains(&i) && n == "Bear Cub")
                            || ([9, 13].contains(&i) && n == "Cartographer's Companion"))));
            if keep && count == 1 { -100 } else { 2500 }
        }
        Action::Pass => 0,
        Action::FinishAttackers | Action::FinishBlockers => 1000,
        Action::PutTrigger { .. } | Action::FinishTargets => 2200,
        _ => 100,
    }
}
fn witness(o: &Observation, p: Player, i: usize, s: &mut Pilot, warp: bool) -> bool {
    let source = o.objects.iter().find(|v| Some(v.id) == s.source).unwrap();
    assert_eq!(source.zone, Zone::Battlefield);
    s.reference = Some(source.reference);
    if i != 1 && i != 12 {
        let mut expected = STATS[i];
        if i == 7 {
            expected.0 += 1;
            expected.1 += 1;
        }
        assert_eq!((source.power, source.toughness), expected);
    }
    if [6, 11].contains(&i) {
        assert!(source.keywords.contains(Keywords::FLYING));
    }
    if [4, 11].contains(&i) {
        assert!(source.keywords.contains(Keywords::HASTE));
    }
    match i {
        1 => {
            if token_count(o, p, i) == s.tokens {
                return false;
            }
            assert_eq!(token_count(o, p, i), s.tokens + 1);
            assert!(o.objects.iter().any(|v| v.owner == p
                && v.zone == Zone::Battlefield
                && v.card.is_token()
                && v.card.has_subtype("Robot")
                && v.card.is_artifact()
                && v.power == 2
                && v.toughness == 2
                && v.token.unwrap().colors == 0));
        }
        2 => assert_eq!(o.players[p.index()].life, s.life + 2),
        3 => {
            assert_eq!(token_count(o, p, i), s.tokens + 1);
            assert!(o.objects.iter().any(|v| v.owner == p
                && v.zone == Zone::Battlefield
                && v.card.is_token()
                && v.card.has_subtype("Human")
                && v.card.has_subtype("Soldier")
                && v.power == 1
                && v.toughness == 1
                && v.token.unwrap().colors == 1));
        }
        5 => {
            let t = o
                .objects
                .iter()
                .find(|v| Some(v.reference) == s.target)
                .unwrap();
            assert_eq!((t.power, t.toughness), (-1, 2));
        }
        6 => assert_eq!(o.players[p.index()].library_count, s.library - 2),
        7 | 8 => {
            let bear = o
                .objects
                .iter()
                .find(|v| {
                    v.owner == p && v.zone == Zone::Battlefield && v.card.name() == "Bear Cub"
                })
                .unwrap();
            assert_eq!(bear.counters.plus_one, s.bear_counters + 1);
            if i == 8 {
                assert!(bear.keywords.contains(Keywords::TRAMPLE));
                assert!(!source.keywords.contains(Keywords::TRAMPLE));
            }
        }
        9 | 13 => {
            let t = o
                .objects
                .iter()
                .find(|v| Some(v.reference) == s.target)
                .unwrap();
            assert!(t.card.is_artifact() && !t.card.is_creature());
            assert!(t.tapped);
            assert_eq!(t.counters.stun, s.target_stun + 1);
        }
        10 => {
            let bear = o
                .objects
                .iter()
                .find(|v| {
                    v.owner == p && v.zone == Zone::Battlefield && v.card.name() == "Bear Cub"
                })
                .unwrap();
            assert_eq!(
                (bear.power, bear.toughness),
                (if o.active == p { 3 } else { 2 }, 2)
            );
            if o.active == p {
                s.own_turn = true;
            } else {
                s.other_turn = true;
            }
            if !(s.own_turn && ((warp && s.paid == 1) || s.other_turn)) {
                return false;
            }
        }
        11 => {
            let t = o
                .objects
                .iter()
                .find(|v| Some(v.reference) == s.target)
                .unwrap();
            assert_eq!(t.damage, s.target_damage + 1);
        }
        12 => {
            assert!(o.objects.iter().any(|v| {
                s.target
                    .is_some_and(|r| v.id == r.card && v.reference.generation > r.generation)
                    && v.zone == Zone::Graveyard
            }));
            assert!(source.keywords.contains(Keywords::FLASH));
        }
        _ => {}
    }
    true
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse::<usize>()?;
    let (
        mut hard,
        mut warped,
        mut recast,
        mut hard_effects,
        mut warp_effects,
        mut recast_effects,
        mut exiled,
    ) = (
        [[0u64; 2]; 14],
        [[0u64; 2]; 12],
        [[0u64; 2]; 12],
        [[0u64; 2]; 14],
        [[0u64; 2]; 12],
        [[0u64; 2]; 12],
        [[0u64; 2]; 12],
    );
    let mut cycled = [0u64; 2];
    let (
        mut decisions,
        mut replays,
        mut closed,
        mut rejected,
        mut frames,
        mut initial,
        mut maximum,
    ) = (0u64, 0u64, 0u64, 0u64, 0u64, 0usize, 0usize);
    for seed in 0..games {
        let i = seed % 14;
        let p = if (seed / 14) % 2 == 0 {
            Player::P0
        } else {
            Player::P1
        };
        let alternate = (seed / 28) % 2 == 1;
        let warp = alternate && i < 12;
        let cycling = alternate && i == 12;
        let (a, b) = (deck(i, true), deck(i, false));
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
        for step in 0..20000 {
            if g.outcome().is_some() {
                break;
            }
            let actor = g.actor().unwrap();
            let o = g.observe(actor);
            assert_eq!(o.schema, rustmage::OBSERVATION_SCHEMA);
            let quiet = o.decision == "priority"
                && o.stack.is_empty()
                && o.announcing.is_none()
                && o.resolving.is_none();
            if quiet && [1, 4].contains(&s.stage) {
                if warp && s.paid == 1 {
                    let source = o.objects.iter().find(|v| Some(v.id) == s.source).unwrap();
                    assert!(
                        o.delayed_warp_exiles
                            .iter()
                            .any(|d| d.source == source.reference
                                && d.controller == p
                                && d.source_card == source.card)
                    );
                }
                if witness(&o, p, i, &mut s, warp) {
                    if warp && s.paid == 1 {
                        warp_effects[i][p.index()] += 1;
                        s.stage = 2;
                    } else {
                        if warp {
                            recast_effects[i][p.index()] += 1;
                        } else {
                            hard_effects[i][p.index()] += 1;
                        }
                        s.stage = 6;
                    }
                }
            }
            if s.stage == 2
                && let Some(right) = o
                    .warped_cards
                    .iter()
                    .find(|v| Some(v.object.card) == s.source)
            {
                let old = s.reference.unwrap();
                assert_eq!(right.object.generation, old.generation + 1);
                assert_eq!(right.owner, p);
                assert_eq!(right.exiled_turn, o.turn);
                assert!(
                    o.objects
                        .iter()
                        .any(|v| v.reference == right.object && v.zone == Zone::Exile)
                );
                s.exiled_turn = Some(right.exiled_turn);
                s.stage = 3;
                exiled[i][p.index()] += 1;
            }
            if quiet && s.stage == 5 {
                assert_eq!(o.players[p.index()].library_count, s.library - 1);
                assert!(
                    o.objects
                        .iter()
                        .any(|v| Some(v.id) == s.source && v.zone == Zone::Graveyard)
                );
                cycled[p.index()] += 1;
                s.stage = 6;
            }
            if s.stage == 3 && actor == p && s.exiled_turn == Some(o.turn) {
                let saved = g.snapshot();
                let rev = g.revision();
                for a in [
                    Action::AnnounceExile {
                        card: s.source.unwrap(),
                    },
                    Action::AnnounceWarp {
                        card: s.source.unwrap(),
                    },
                    Action::AnnouncePlotted {
                        card: s.source.unwrap(),
                    },
                ] {
                    assert_eq!(g.apply(p, a), Err(Error::IllegalAction));
                    assert_eq!(g.snapshot(), saved);
                    assert_eq!(g.revision(), rev);
                    rejected += 1;
                }
            }
            let actions = g.legal_actions();
            let (index, a) = actions
                .iter()
                .enumerate()
                .max_by_key(|(_, a)| score(&o, a, p, i, &s, warp, cycling))
                .unwrap();
            assert!(
                score(&o, a, p, i, &s, warp, cycling) >= 0,
                "unhandled {} seed{seed} step{step} {} {actions:?}",
                NAMES[i],
                o.decision
            );
            let a = a.clone();
            if let Action::AnnounceCast { card }
            | Action::AnnounceWarp { card }
            | Action::AnnounceActivate { card, .. } = a
                && actor == p
                && s.stage == 0
                && o.objects
                    .iter()
                    .any(|v| v.id == card && v.card.name() == NAMES[i])
            {
                s.source = Some(card);
            }
            if let Action::ChooseTarget {
                target: Target::Card(id),
            } = a
                && actor == p
                && o.objects.iter().any(|v| v.id == id && v.owner != p)
            {
                let t = o.objects.iter().find(|v| v.id == id).unwrap();
                s.target = Some(t.reference);
                s.target_stun = t.counters.stun;
                s.target_damage = t.damage;
            }
            let mut paid_spell = false;
            if matches!(a, Action::PayCosts { .. })
                && let Some(packet) = &o.announcing
                && actor == p
                && Some(
                    packet
                        .source()
                        .expect("example expects an object-backed packet")
                        .id,
                ) == s.source
                && s.stage < 6
            {
                s.library = o.players[p.index()].library_count;
                s.life = o.players[p.index()].life;
                s.tokens = token_count(&o, p, i);
                s.bear_counters = o
                    .objects
                    .iter()
                    .find(|v| {
                        v.owner == p && v.zone == Zone::Battlefield && v.card.name() == "Bear Cub"
                    })
                    .map_or(0, |v| v.counters.plus_one);
                s.own_turn = false;
                s.other_turn = false;
                if packet.kind == StackKind::Spell {
                    let cast = serde_json::to_value(packet.cast)?;
                    let later = s.paid > 0;
                    assert_eq!(
                        cast["method"],
                        if warp && !later { "Warp" } else { "Normal" }
                    );
                    assert_eq!(cast["origin"], if later { "Exile" } else { "Hand" });
                    if warp && later {
                        assert!(o.turn > s.exiled_turn.unwrap());
                        recast[i][p.index()] += 1;
                        s.stage = 4;
                    } else {
                        if warp {
                            warped[i][p.index()] += 1;
                        } else {
                            hard[i][p.index()] += 1;
                        }
                        if i == 12 {
                            assert_ne!(o.active, p);
                        }
                        s.stage = 1;
                    }
                    s.paid += 1;
                    paid_spell = true;
                } else {
                    assert!(cycling);
                    assert!(matches!(packet.kind, StackKind::ActivatedAbility { .. }));
                    s.stage = 5;
                }
            }
            let frame = o.library_choice.is_some();
            if frame {
                frames += 1;
                assert_eq!(actor, p);
                assert!(g.observe(p.other()).library_choice.is_none());
                let saved = g.snapshot();
                let rev = g.revision();
                assert_eq!(g.apply(actor, Action::Pass), Err(Error::IllegalAction));
                assert_eq!(g.snapshot(), saved);
                assert_eq!(g.revision(), rev);
                rejected += 1;
            }
            let saved = g.snapshot();
            let revision = g.revision();
            let mut fork = g.clone();
            fork.apply(actor, a.clone())?;
            let expected = fork.snapshot();
            let child = if frame || !o.delayed_warp_exiles.is_empty() || !o.warped_cards.is_empty()
            {
                let parent = g.clone();
                let child = parent.clone();
                drop(parent);
                Some(child)
            } else {
                None
            };
            assert!(g.apply(actor.other(), a.clone()).is_err());
            assert_eq!(g.snapshot(), saved);
            assert_eq!(g.revision(), revision);
            rejected += 1;
            g.apply_index(actor, revision, index)?;
            assert_eq!(g.snapshot(), expected);
            if let Some(mut child) = child {
                child.apply(actor, a.clone())?;
                assert_eq!(child.snapshot(), expected);
                closed += 1;
            }
            for viewer in [Player::P0, Player::P1] {
                assert_eq!(
                    serde_json::to_value(g.observe(viewer))?,
                    serde_json::to_value(fork.observe(viewer))?
                );
            }
            if paid_spell {
                assert_eq!(
                    g.observe(p).players[p.index()].spells_cast_this_turn,
                    o.players[p.index()].spells_cast_this_turn + 1
                );
            }
            g.restore(&saved)?;
            assert_eq!(
                g.apply_index(actor, revision, index),
                Err(Error::StaleRevision)
            );
            rejected += 1;
            g.apply(actor, a.clone())?;
            assert_eq!(g.snapshot(), expected);
            if matches!(a, Action::PlayLand { .. }) && actor == p {
                s.land_turn = Some(o.turn);
            }
            decisions += 1;
            replays += 1;
            maximum = maximum.max(g.state_bytes());
        }
        assert!(
            s.stage == 6 && g.outcome().is_some(),
            "incomplete {} seed{seed} stage{} paid{} turn{}",
            NAMES[i],
            s.stage,
            s.paid,
            g.observe(p).turn
        );
    }
    if games >= 100 {
        for a in [&hard, &hard_effects] {
            assert!(a.iter().flatten().all(|n| *n > 0));
        }
        for a in [&warped, &recast, &warp_effects, &recast_effects, &exiled] {
            assert!(a.iter().flatten().all(|n| *n > 0));
        }
        assert!(cycled.iter().all(|n| *n > 0) && frames > 0 && closed > 0);
    }
    println!(
        "games={games} decisions={decisions} replays={replays} parent_closed={closed} rejected={rejected} private_frames={frames} initial_bytes={initial} max_bytes={maximum} hard={hard:?} warp={warped:?} recast={recast:?} hard_effects={hard_effects:?} warp_effects={warp_effects:?} recast_effects={recast_effects:?} exiled={exiled:?} cycled={cycled:?}"
    );
    Ok(())
}
