//! Genuine Standard games: actual paid casts, low/high mana events and owned replay.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Keywords, LibraryDestination, Observation,
    Player, StackKind, Target, Zone,
};
const NAMES: [&str; 20] = [
    "Aberrant Manawurm",
    "Blazing Bomb",
    "Cuboid Colony",
    "Deluge Virtuoso",
    "Exhibition Tidecaller",
    "Expressive Firedancer",
    "Freestrider Commando",
    "Hungry Graffalon",
    "Muse Seeker",
    "Prompto Argentum",
    "Rancorous Archaic",
    "Sahagin",
    "Shantotto, Tactician Magician",
    "Spectacular Skywhale",
    "Tackle Artist",
    "Textbook Tabulator",
    "Thunderdrum Soloist",
    "Ultros, Obnoxious Octopus",
    "Unravel",
    "Arcane Omens",
];
fn deck(i: usize, focal: bool) -> Vec<Card> {
    let lands = if i == 10 || i == 19 {
        [5, 7, 10, 5, 5]
    } else {
        [5, 10, 5, 6, 6]
    };
    let mut d = Vec::new();
    for (n, k) in ["Plains", "Island", "Swamp", "Mountain", "Forest"]
        .into_iter()
        .zip(lands)
    {
        d.extend(vec![Card::from_name(n).unwrap(); k]);
    }
    for n in [
        if focal { NAMES[i] } else { "Bygone Colossus" },
        "Shock",
        "Mind Spring",
        "Bear Cub",
        "Thrill of Possibility",
        "Cartographer's Companion",
        "Three Tree Mascot",
    ] {
        d.extend([Card::from_name(n).unwrap(); 4]);
    }
    assert_eq!(d.len(), 60);
    d
}
#[derive(Default)]
struct Pilot {
    stage: u8,
    source: Option<u32>,
    event: Option<u32>,
    target: Option<u32>,
    plot_turn: u32,
    power: i128,
    toughness: i128,
    counters: u64,
    stun: u64,
    own_library: usize,
    enemy_library: usize,
    enemy_hand: usize,
    drawn: u64,
    discarded: u64,
    life: i128,
    tokens: usize,
    unravel_ready: bool,
}
fn field(o: &Observation, p: Player, n: &str) -> Option<u32> {
    o.objects
        .iter()
        .find(|v| v.owner == p && v.zone == Zone::Battlefield && v.card.name() == n)
        .map(|v| v.id)
}
fn lands(o: &Observation, p: Player, n: Option<&str>) -> usize {
    o.objects
        .iter()
        .filter(|v| {
            v.owner == p
                && v.zone == Zone::Battlefield
                && v.card.is_land()
                && n.is_none_or(|n| v.card.name() == n)
        })
        .count()
}
fn treasures(o: &Observation, p: Player) -> usize {
    o.objects
        .iter()
        .filter(|v| {
            v.owner == p
                && v.zone == Zone::Battlefield
                && v.card.is_token()
                && v.card.has_subtype("Treasure")
        })
        .count()
}
fn score(o: &Observation, a: &Action, p: Player, i: usize, s: &Pilot, high: bool) -> i32 {
    let actor = o.actor.unwrap();
    let obj = |id| o.objects.iter().find(|v| v.id == id).unwrap();
    let colors_ready = if high {
        ["Plains", "Island", "Swamp", "Mountain", "Forest"]
            .iter()
            .all(|n| lands(o, p, Some(n)) > 0)
    } else {
        lands(o, p, Some("Swamp")) >= 5
    };
    match a {
        Action::Keep => 3000,
        Action::Mulligan | Action::Concede => -10000,
        Action::AnnounceCast { card } => {
            let v = obj(*card);
            let n = v.card.name();
            if s.stage == 5 {
                return if v.card.is_creature() {
                    1200
                } else if n == "Shock" {
                    1100
                } else {
                    -1000
                };
            }
            if actor != p {
                if i == 18 && s.unravel_ready && s.stage == 0 && !high && n == "Shock" {
                    return 1850;
                }
                return if n == "Bear Cub" && field(o, actor, n).is_none() {
                    1750
                } else {
                    -1000
                };
            }
            if n == NAMES[i] && s.stage == 0 {
                if i == 18 {
                    return if o.stack.iter().any(|v| {
                        v.kind == StackKind::Spell
                            && v.controller != p
                            && v.source().expect("spell has source").card.name()
                                == if high { "Bygone Colossus" } else { "Shock" }
                    }) {
                        2300
                    } else {
                        -1000
                    };
                }
                if i == 19 {
                    return if colors_ready && o.players[p.other().index()].hand_count >= 5 {
                        1850
                    } else {
                        -1000
                    };
                }
                return if !(i == 6 && high)
                    && field(o, p.other(), "Bear Cub").is_some()
                    && lands(o, p, None) >= 8
                    && (!(i == 10) || colors_ready)
                {
                    1850
                } else {
                    -1000
                };
            }
            if s.stage == 2
                && n == if high { "Mind Spring" } else { "Shock" }
                && lands(o, p, None) >= if high && i == 17 { 8 } else { 5 }
                && o.objects
                    .iter()
                    .filter(|v| {
                        v.owner == p && v.zone == Zone::Battlefield && v.card.is_land() && !v.tapped
                    })
                    .count()
                    >= if high && i == 17 {
                        8
                    } else if high {
                        5
                    } else {
                        1
                    }
                && o.objects
                    .iter()
                    .filter(|v| {
                        v.owner == p
                            && v.zone == Zone::Battlefield
                            && !v.tapped
                            && v.card.name() == if high { "Island" } else { "Mountain" }
                    })
                    .count()
                    >= if high { 2 } else { 1 }
            {
                return 1850;
            }
            -1000
        }
        Action::AnnounceWarp { card } => {
            if actor != p
                && i == 18
                && high
                && s.unravel_ready
                && s.stage == 0
                && obj(*card).card.name() == "Bygone Colossus"
            {
                1850
            } else {
                -1000
            }
        }
        Action::AnnouncePlot { card } => {
            if actor == p
                && i == 6
                && high
                && s.stage == 0
                && obj(*card).card.name() == NAMES[i]
                && field(o, p.other(), "Bear Cub").is_some()
            {
                1850
            } else {
                -1000
            }
        }
        Action::AnnouncePlotted { card } => {
            if actor == p && s.stage == 11 && s.source == Some(*card) && o.turn > s.plot_turn {
                1850
            } else {
                -1000
            }
        }
        Action::AnnounceActivate { card, .. } => {
            if actor == p && i == 1 && s.stage == 6 && s.source == Some(*card) {
                2000
            } else {
                -1000
            }
        }
        Action::ChooseX { value } => {
            if *value == if i == 17 { 6 } else { 3 } {
                2600
            } else {
                -1000
            }
        }
        Action::ChooseTarget {
            target: Target::Card(id),
        } => {
            if obj(*id).owner != actor && Some(*id) == field(o, p.other(), "Bear Cub") {
                2500
            } else {
                700
            }
        }
        Action::ChooseTarget {
            target: Target::Player(q),
        } => {
            if *q == actor.other() {
                2700
            } else {
                700
            }
        }
        Action::ChooseTarget {
            target: Target::Spell(_),
        } => 2400,
        Action::PlayLand { card } => {
            let n = obj(*card).card.name();
            let k = lands(o, actor, Some(n));
            let needed = if i == 10 || i == 19 {
                if high {
                    usize::from(k == 0) * 200
                } else {
                    usize::from(n == "Swamp" && k < 5) * 200
                }
            } else {
                usize::from(n == "Island" && k < 2) * 120
            };
            1550 + needed as i32 - 10 * k as i32
        }
        Action::TapMana { card } => {
            if o.decision == "pay_costs" || o.plot_payment.is_some() {
                if actor == p && (i == 10 || i == 19) && s.event.is_none() && s.stage != 5 {
                    let n = obj(*card).card.name();
                    let c = ["Plains", "Island", "Swamp", "Mountain", "Forest"]
                        .iter()
                        .position(|x| *x == n)
                        .unwrap();
                    if high {
                        if o.players[p.index()].mana[c] == 0 {
                            2300
                        } else {
                            -1000
                        }
                    } else if n == "Swamp" {
                        2300
                    } else {
                        -1000
                    }
                } else {
                    2100
                }
            } else {
                -1000
            }
        }
        Action::PayCosts { payment } => {
            if actor == p
                && (i == 10 || i == 19)
                && s.stage < 5
                && o.announcing
                    .as_ref()
                    .is_some_and(|v| v.source().is_some_and(|source| source.card.name() == NAMES[i]))
            {
                if *payment
                    == if high {
                        [1, 1, 1, 1, 1, 0]
                    } else {
                        [0, 0, 5, 0, 0, 0]
                    }
                {
                    3000
                } else {
                    -1000
                }
            } else {
                2400
            }
        }
        Action::ActivateMana { .. } | Action::AnnounceMana { .. } => {
            if o.decision == "pay_costs" {
                2100
            } else {
                -1000
            }
        }
        Action::PayPlot { .. } | Action::ChooseSpellCost { .. } => 2400,
        Action::PutTrigger { .. } | Action::FinishTargets => 2200,
        Action::ArrangeLibrary { destination, .. } => {
            if *destination == LibraryDestination::Top {
                2300
            } else {
                2200
            }
        }
        Action::Discard { card } | Action::Bottom { card } => {
            let n = obj(*card).card.name();
            let duplicate = o
                .objects
                .iter()
                .filter(|v| v.owner == actor && v.zone == Zone::Hand && v.card.name() == n)
                .count()
                > 1;
            let keep = if actor == p {
                s.stage < 5 && (n == NAMES[i] || n == if high { "Mind Spring" } else { "Shock" })
            } else {
                i == 18 && (n == "Shock" || n == "Bygone Colossus")
            };
            if keep && !duplicate { -100 } else { 2500 }
        }
        Action::Attack { .. } | Action::Block { .. } => {
            if s.stage == 5 {
                1700
            } else {
                -1000
            }
        }
        Action::AssignDamage { amount, .. } => 1800 + (*amount).min(100) as i32,
        Action::FinishAttackers | Action::FinishBlockers => 1000,
        Action::Pass => 0,
        Action::Cast { .. }
        | Action::BeginCast { .. }
        | Action::Activate { .. }
        | Action::AnnounceExile { .. }
        | Action::AnnounceFlashback { .. } => -1000,
        _ => 100,
    }
}
fn baseline(o: &Observation, p: Player, s: &mut Pilot) {
    s.own_library = o.players[p.index()].library_count;
    s.enemy_library = o.players[p.other().index()].library_count;
    s.enemy_hand = o.players[p.other().index()].hand_count;
    s.drawn = o.players[p.index()].cards_drawn_this_turn;
    s.discarded = o.players[p.index()].cards_discarded_this_turn;
    s.life = o.players[p.other().index()].life;
    s.tokens = treasures(o, p);
    if let Some(v) = o.objects.iter().find(|v| Some(v.id) == s.source) {
        s.power = v.power;
        s.toughness = v.toughness;
        s.counters = v.counters.plus_one;
    }
    if let Some(id) = field(o, p.other(), "Bear Cub") {
        s.target = Some(id);
        s.stun = o.objects.iter().find(|v| v.id == id).unwrap().counters.stun;
    }
}
fn witness(o: &Observation, p: Player, i: usize, s: &Pilot, high: bool) {
    if i == 18 {
        assert_eq!(
            o.players[p.index()].library_count,
            s.own_library - usize::from(high)
        );
        return;
    }
    if i == 19 {
        assert_eq!(
            o.players[p.other().index()].hand_count,
            s.enemy_hand - if high { 5 } else { 1 }
        );
        return;
    }
    let v = o.objects.iter().find(|v| Some(v.id) == s.source).unwrap();
    assert_eq!(v.zone, Zone::Battlefield);
    let spent = if high { if i == 17 { 8 } else { 5 } } else { 1 };
    let add = if high { 2 } else { 1 };
    match i {
        0 | 12 => {
            assert_eq!(v.power, s.power + spent);
            assert_eq!(v.toughness, s.toughness);
        }
        1 | 11 => assert_eq!(v.counters.plus_one, s.counters + u64::from(high)),
        2 | 7 | 15 => assert_eq!(
            v.counters.plus_one,
            s.counters + u64::from(spent > s.power || spent > s.toughness)
        ),
        3 => assert_eq!((v.power, v.toughness), (s.power + add, s.toughness + add)),
        4 => assert_eq!(
            o.players[p.other().index()].library_count,
            s.enemy_library - if high { 10 } else { 3 }
        ),
        5 => {
            assert_eq!((v.power, v.toughness), (s.power + 1, s.toughness + 1));
            assert_eq!(v.keywords.contains(Keywords::DOUBLE_STRIKE), high);
        }
        6 => assert_eq!(v.counters.plus_one, if high { 2 } else { 0 }),
        8 => {
            assert_eq!(
                o.players[p.index()].cards_drawn_this_turn,
                s.drawn + if high { 4 } else { 1 }
            );
            assert_eq!(
                o.players[p.index()].cards_discarded_this_turn,
                s.discarded + u64::from(!high)
            );
        }
        9 => assert_eq!(treasures(o, p), s.tokens + usize::from(high)),
        10 => assert_eq!(v.counters.plus_one, if high { 5 } else { 1 }),
        13 => assert_eq!(
            (v.power, v.toughness),
            (s.power + 3, s.toughness + if high { 3 } else { 0 })
        ),
        14 => assert_eq!(v.counters.plus_one, s.counters + if high { 2 } else { 1 }),
        16 => assert_eq!(o.players[p.other().index()].life, s.life - 3),
        17 => {
            assert_eq!(v.counters.plus_one, s.counters + if high { 8 } else { 0 });
            let t = o.objects.iter().find(|v| Some(v.id) == s.target).unwrap();
            assert_eq!(t.counters.stun, s.stun + u64::from(high));
        }
        _ => unreachable!(),
    }
    if i == 11 && high {
        assert!(v.keywords.contains(Keywords::CANT_BE_BLOCKED));
    }
    if i == 12 {
        assert_eq!(
            o.players[p.index()].cards_drawn_this_turn,
            s.drawn + if high { 4 } else { 0 }
        );
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse::<usize>()?;
    let start = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "0".into())
        .parse::<usize>()?;
    let (mut decisions, mut closed, mut rejected, mut frames, mut maximum, mut initial) =
        (0u64, 0u64, 0u64, 0u64, 0usize, 0usize);
    let mut cells = [[[0u64; 2]; 2]; 20];
    for seed in start..start + games {
        let i = seed % 20;
        let p = if (seed / 20) % 2 == 0 {
            Player::P0
        } else {
            Player::P1
        };
        let high = (seed / 40) % 2 == 1;
        let (a, b) = (deck(i, true), deck(i, false));
        let mut g = Game::new(Config {
            decks: if p == Player::P0 { [a, b] } else { [b, a] },
            seed: seed as u64,
            starting_player: p,
            deck_rules: DeckRules::Standard,
        })?;
        if seed == start {
            initial = g.state_bytes();
        }
        let mut s = Pilot::default();
        for step in 0..20000 {
            if g.outcome().is_some() {
                break;
            }
            let actor = g.actor().unwrap();
            let o = g.observe(actor);
            let quiet = o.decision == "priority"
                && o.stack.is_empty()
                && o.announcing.is_none()
                && o.resolving.is_none();
            if actor == p && i == 18 {
                s.unravel_ready = o
                    .objects
                    .iter()
                    .any(|v| v.owner == p && v.zone == Zone::Hand && v.card.name() == NAMES[i])
                    && lands(&o, p, Some("Island")) >= 2;
            }
            if quiet && s.stage == 1 {
                let v = o.objects.iter().find(|v| Some(v.id) == s.source).unwrap();
                assert_eq!(v.zone, Zone::Battlefield);
                if i == 3 {
                    let t = o
                        .objects
                        .iter()
                        .find(|v| {
                            v.owner != p
                                && v.zone == Zone::Battlefield
                                && v.card.name() == "Bear Cub"
                        })
                        .unwrap();
                    assert_eq!(t.counters.stun, 1);
                    assert!(t.tapped);
                }
                if i == 6 {
                    assert_eq!(v.counters.plus_one, if high { 2 } else { 0 });
                }
                if i == 10 {
                    assert_eq!(v.counters.plus_one, if high { 5 } else { 1 });
                }
                s.stage = 2;
            }
            if quiet && s.stage == 4 {
                witness(&o, p, i, &s, high);
                cells[i][p.index()][usize::from(high)] += 1;
                s.stage = if i == 1 { 6 } else { 5 };
            }
            if quiet && s.stage == 7 {
                assert!(
                    o.objects
                        .iter()
                        .any(|v| Some(v.id) == s.source && v.zone == Zone::Graveyard)
                );
                assert!(o.objects.iter().any(|v| Some(v.id) == s.target
                    && if high {
                        v.zone == Zone::Graveyard
                    } else {
                        v.zone == Zone::Battlefield && v.damage == 1
                    }));
                s.stage = 5;
            }
            let actions = g.legal_actions();
            let (index, a) = actions
                .iter()
                .enumerate()
                .max_by_key(|(_, a)| score(&o, a, p, i, &s, high))
                .unwrap();
            assert!(
                score(&o, a, p, i, &s, high) >= 0,
                "unhandled {} seed{seed} step{step} stage{} {} {actions:?}",
                NAMES[i],
                s.stage,
                o.decision
            );
            let mut a = a.clone();
            // A raw valid vector is intentionally accepted through the complete public domain,
            // whether or not the bounded concrete representatives include it.
            if actor == p
                && (i == 10 || i == 19)
                && s.stage < 5
                && o.mana_payment.is_some()
                && o.announcing
                    .as_ref()
                    .is_some_and(|v| v.source().is_some_and(|source| source.card.name() == NAMES[i]))
            {
                let payment = if high {
                    [1, 1, 1, 1, 1, 0]
                } else {
                    [0, 0, 5, 0, 0, 0]
                };
                if o.mana_payment
                    .as_ref()
                    .is_some_and(|domain| domain.contains(payment))
                {
                    a = Action::PayCosts { payment };
                }
            }
            if let Action::AnnouncePlot { card } = a
                && actor == p
            {
                s.source = Some(card);
                s.stage = 10;
            }
            if matches!(a, Action::PayPlot { .. }) && actor == p {
                s.stage = 11;
                s.plot_turn = o.turn;
            }
            if let Action::AnnounceCast { card } | Action::AnnouncePlotted { card } = a
                && actor == p
                && (s.stage < 5 || s.stage == 11)
            {
                let n = o.objects.iter().find(|v| v.id == card).unwrap().card.name();
                if n == NAMES[i] {
                    s.source = Some(card);
                    s.stage = if i >= 18 { 3 } else { 1 };
                    if i >= 18 {
                        s.event = Some(card);
                    }
                } else if s.stage == 2 {
                    s.event = Some(card);
                    s.stage = 3;
                }
            }
            if matches!(a, Action::AnnounceActivate { .. }) && actor == p && s.stage == 6 {
                s.stage = 7;
            }
            let mut check_payment = None;
            if let Action::PayCosts { payment } = a
                && actor == p
                && let Some(packet) = &o.announcing
            {
                if packet.kind == StackKind::Spell && Some(packet.card) == s.source {
                    check_payment = Some((packet.card, payment));
                }
                if Some(packet.card) == s.event && s.stage == 3 {
                    baseline(&o, p, &mut s);
                    s.stage = 4;
                    check_payment = Some((packet.card, payment));
                    if i == 18 {
                        let target = o
                            .stack
                            .iter()
                            .find(|v| v.kind == StackKind::Spell && v.controller != p)
                            .unwrap();
                        assert_eq!(
                            target
                                .cast
                                .unwrap()
                                .payment
                                .into_iter()
                                .map(u64::from)
                                .sum::<u64>(),
                            if high { 3 } else { 1 }
                        );
                        assert_eq!(target.mana_value(), if high { 9 } else { 1 });
                    }
                }
            }
            let saved = g.snapshot();
            let revision = g.revision();
            let mut fork = {
                let parent = g.clone();
                let child = parent.clone();
                drop(parent);
                child
            };
            fork.apply(actor, a.clone())?;
            let expected = fork.snapshot();
            assert!(g.apply(actor.other(), a.clone()).is_err());
            assert_eq!(g.snapshot(), saved);
            assert_eq!(g.revision(), revision);
            rejected += 1;
            if actions[index] == a {
                g.apply_index(actor, revision, index)?;
            } else {
                g.apply(actor, a.clone())?;
            }
            assert_eq!(g.snapshot(), expected);
            closed += 1;
            for viewer in [Player::P0, Player::P1] {
                assert_eq!(
                    serde_json::to_value(g.observe(viewer))?,
                    serde_json::to_value(fork.observe(viewer))?
                );
            }
            if let Some((card, payment)) = check_payment {
                let v = g.observe(p);
                let spell = v
                    .stack
                    .iter()
                    .find(|v| v.kind == StackKind::Spell && v.card == card)
                    .unwrap();
                assert_eq!(spell.cast.unwrap().payment, payment);
            }
            if o.mana_payment.is_some() {
                assert!(g.observe(actor.other()).mana_payment.is_none());
                frames += 1;
            }
            g.restore(&saved)?;
            assert_eq!(
                g.apply_index(actor, revision, index),
                Err(Error::StaleRevision)
            );
            rejected += 1;
            g.apply(actor, a)?;
            assert_eq!(g.snapshot(), expected);
            decisions += 1;
            maximum = maximum.max(g.state_bytes());
        }
        assert!(
            s.stage == 5 && g.outcome().is_some(),
            "incomplete {} seed{seed} high{high} stage{} turn{}",
            NAMES[i],
            s.stage,
            g.observe(p).turn
        );
        eprintln!(
            "completed seed={seed} card={} high={high} decisions={decisions}",
            NAMES[i]
        );
    }
    if games >= 80 && start % 80 == 0 {
        assert!(cells.iter().flatten().flatten().all(|n| *n > 0));
        assert!(frames > 0);
    }
    println!(
        "games={games} decisions={decisions} replays={decisions} parent_closed={closed} rejected={rejected} payment_frames={frames} initial_bytes={initial} max_bytes={maximum} full_effect_cells={cells:?}"
    );
    Ok(())
}
