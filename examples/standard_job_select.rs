//! Legal Standard acceptance for the complete first Job Select checkpoint.
//! Later checkpoints extend this matrix to the recipient-granted trigger bodies.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Keywords, ObjectRef, Observation, Player,
    StackKind, Target, Zone,
};
const NAMES: [&str; 7] = [
    "Bard's Bow",
    "Dark Knight's Greatsword",
    "Dragoon's Lance",
    "Machinist's Arsenal",
    "Monk's Fist",
    "Samurai's Katana",
    "Warrior's Sword",
];
const TYPES: [&str; 7] = [
    "Bard",
    "Knight",
    "Knight",
    "Artificer",
    "Monk",
    "Samurai",
    "Warrior",
];
const COLORS: [&str; 7] = [
    "Forest", "Swamp", "Plains", "Plains", "Plains", "Mountain", "Mountain",
];
fn deck(i: usize, focal: bool) -> Vec<Card> {
    let mut d = Vec::new();
    for (n, k) in [
        ("Forest", 8),
        ("Plains", 8),
        ("Mountain", 8),
        (COLORS[i], 4),
    ] {
        d.extend(vec![Card::from_name(n).unwrap(); k]);
    }
    for n in [
        if focal { NAMES[i] } else { "Chrome Companion" },
        "Bear Cub",
        "Disenchant",
        "Llanowar Elves",
        "Cartographer's Companion",
        "Ambush Wolf",
        "Shock",
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
    hero: Option<ObjectRef>,
    bear: Option<ObjectRef>,
    paid: bool,
    phase: u8,
    other_turn: bool,
    life_before: i128,
    equipped: bool,
}
fn ready(o: &Observation, p: Player, i: usize) -> bool {
    o.objects
        .iter()
        .any(|v| v.owner == p && v.zone == Zone::Battlefield && v.card.name() == "Bear Cub")
        && (i != 3
            || o.objects.iter().any(|v| {
                v.owner == p
                    && v.zone == Zone::Battlefield
                    && v.card.name() == "Cartographer's Companion"
            }))
}
fn score(o: &Observation, a: &Action, p: Player, i: usize, s: &Pilot) -> i32 {
    let actor = o.actor.unwrap();
    let obj = |id| o.objects.iter().find(|v| v.id == id).unwrap();
    match a {
        Action::Keep => 3000,
        Action::Mulligan | Action::Concede => -10000,
        Action::AnnounceCast { card } => {
            let v = obj(*card);
            let n = v.card.name();
            if s.phase == 3 {
                return if v.card.is_creature() {
                    1200
                } else if n == "Shock" {
                    1100
                } else {
                    -1000
                };
            }
            if actor != p {
                return -1000;
            }
            if n == NAMES[i] {
                return if !s.paid && ready(o, p, i) {
                    1900
                } else {
                    -1000
                };
            }
            if n == "Bear Cub"
                && !o
                    .objects
                    .iter()
                    .any(|v| v.owner == p && v.zone == Zone::Battlefield && v.card.name() == n)
            {
                return 1750;
            }
            if i == 3
                && n == "Cartographer's Companion"
                && !o
                    .objects
                    .iter()
                    .any(|v| v.owner == p && v.zone == Zone::Battlefield && v.card.name() == n)
            {
                return 1750;
            }
            if n == "Disenchant" && s.phase == 2 && (i != 2 || s.other_turn) {
                return 2000;
            }
            -1000
        }
        Action::AnnounceActivate { card, .. } => {
            if actor == p && Some(*card) == s.source && s.phase == 1 && !s.equipped {
                2000
            } else {
                -1000
            }
        }
        Action::Cast { .. }
        | Action::BeginCast { .. }
        | Action::Activate { .. }
        | Action::AnnouncePlot { .. }
        | Action::AnnouncePlotted { .. }
        | Action::AnnounceExile { .. }
        | Action::AnnounceFlashback { .. } => -1000,
        Action::ChooseTarget {
            target: Target::Card(id),
        } => {
            let v = obj(*id);
            if actor == p && s.phase < 3 {
                if s.phase == 1
                    && v.owner == p
                    && v.zone == Zone::Battlefield
                    && v.card.name() == "Bear Cub"
                {
                    return 2300;
                }
                if s.phase == 2 && Some(*id) == s.source {
                    return 2300;
                }
                return 800;
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
        Action::Attack { .. } => {
            if s.phase == 3 {
                1700
            } else {
                -1000
            }
        }
        Action::Block { .. } => {
            if s.phase == 3 {
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
            let keep = actor == p
                && s.phase < 3
                && ((n == NAMES[i] && !s.paid)
                    || n == "Disenchant"
                    || n == "Bear Cub"
                    || (i == 3 && n == "Cartographer's Companion"));
            if keep && count == 1 { -100 } else { 2500 }
        }
        Action::Pass => 0,
        Action::FinishAttackers | Action::FinishBlockers => 1000,
        Action::PutTrigger { .. } | Action::FinishTargets | Action::ArrangeLibrary { .. } => 2200,
        _ => 100,
    }
}
fn boosted(o: &Observation, p: Player, i: usize, v: &rustmage::ObjectView, base: i128) {
    let (power, toughness) = match i {
        0 => (base + 2, base + 2),
        1 => (base + 3, base),
        2 | 4 => (base + 1, base),
        3 => {
            let n = o
                .objects
                .iter()
                .filter(|v| v.owner == p && v.zone == Zone::Battlefield && v.card.is_artifact())
                .count() as i128;
            assert!(n >= 2);
            (base + 2 * n, base + 2 * n)
        }
        5 => (base + 2, base + 2),
        6 => (base + 3, base + 2),
        _ => unreachable!(),
    };
    assert_eq!((v.power, v.toughness), (power, toughness));
    assert!(v.has_subtype(TYPES[i]));
    assert!(!v.card.has_subtype(TYPES[i]));
    let expected = match i {
        0 => Keywords::REACH,
        2 if o.active == p => Keywords::FLYING,
        5 => Keywords::TRAMPLE.union(Keywords::HASTE),
        _ => Keywords::NONE,
    };
    assert!(v.keywords.contains(expected));
    if i == 2 && o.active != p {
        assert!(!v.keywords.contains(Keywords::FLYING));
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse::<usize>()?;
    let (mut paid, mut entered, mut equipped, mut removed) = (
        [[0u64; 2]; 7],
        [[0u64; 2]; 7],
        [[0u64; 2]; 7],
        [[0u64; 2]; 7],
    );
    let (mut decisions, mut replayed, mut closed, mut rejects, mut initial, mut maximum) =
        (0u64, 0u64, 0u64, 0u64, 0usize, 0usize);
    for seed in 0..games {
        let i = seed % 7;
        let p = if (seed / 7) % 2 == 0 {
            Player::P0
        } else {
            Player::P1
        };
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
            if quiet && s.paid {
                let source = o.objects.iter().find(|v| Some(v.id) == s.source).unwrap();
                if s.phase == 0 {
                    let reference = source.attached_to.expect("Job Select attached actual Hero");
                    let hero = o.objects.iter().find(|v| v.reference == reference).unwrap();
                    assert_eq!(hero.zone, Zone::Battlefield);
                    assert!(hero.card.is_token() && hero.card.has_subtype("Hero"));
                    assert_eq!(hero.token.unwrap().colors, 0);
                    boosted(&o, p, i, hero, 1);
                    s.hero = Some(reference);
                    s.phase = 1;
                    entered[i][p.index()] += 1;
                } else if s.phase == 1 && s.equipped {
                    assert_eq!(source.attached_to, s.bear);
                    let bear = o
                        .objects
                        .iter()
                        .find(|v| Some(v.reference) == s.bear)
                        .unwrap();
                    boosted(&o, p, i, bear, 2);
                    let hero = o
                        .objects
                        .iter()
                        .find(|v| Some(v.reference) == s.hero)
                        .unwrap();
                    assert!(!hero.has_subtype(TYPES[i]));
                    assert_eq!((hero.power, hero.toughness), (1, 1));
                    s.phase = 2;
                    equipped[i][p.index()] += 1;
                } else if s.phase == 2 {
                    if o.active != p {
                        s.other_turn = true;
                    }
                    let bear = o
                        .objects
                        .iter()
                        .find(|v| Some(v.reference) == s.bear)
                        .unwrap();
                    if source.zone == Zone::Battlefield {
                        boosted(&o, p, i, bear, 2);
                    } else {
                        assert_eq!(source.zone, Zone::Graveyard);
                        assert!(!bear.has_subtype(TYPES[i]));
                        assert_eq!((bear.power, bear.toughness), (2, 2));
                        s.phase = 3;
                        removed[i][p.index()] += 1;
                    }
                }
            }
            let actions = g.legal_actions();
            let (index, a) = actions
                .iter()
                .enumerate()
                .max_by_key(|(_, a)| score(&o, a, p, i, &s))
                .unwrap();
            assert!(
                score(&o, a, p, i, &s) >= 0,
                "unhandled seed{seed} step{step} {} {actions:?}",
                o.decision
            );
            let a = a.clone();
            if let Action::AnnounceCast { card } = a
                && actor == p
                && !s.paid
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
                && s.phase == 1
                && o.objects
                    .iter()
                    .any(|v| v.id == id && v.owner == p && v.card.name() == "Bear Cub")
            {
                s.bear = Some(o.objects.iter().find(|v| v.id == id).unwrap().reference);
            }
            if matches!(a, Action::PayCosts { .. })
                && let Some(packet) = &o.announcing
                && actor == p
            {
                if packet.kind == StackKind::Spell
                    && packet.source_card.name() == NAMES[i]
                    && !s.paid
                {
                    s.paid = true;
                    paid[i][p.index()] += 1;
                }
                if matches!(packet.kind, StackKind::ActivatedAbility { .. })
                    && Some(packet.card) == s.source
                    && s.phase == 1
                {
                    s.equipped = true;
                    s.life_before = o.players[p.index()].life;
                }
            }
            let saved = g.snapshot();
            let revision = g.revision();
            let mut fork = g.clone();
            fork.apply(actor, a.clone())?;
            let expected = fork.snapshot();
            let child = if s.phase == 1 || s.phase == 2 {
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
            rejects += 1;
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
            if s.equipped && s.phase == 1 && matches!(a, Action::PayCosts { .. }) {
                assert_eq!(
                    g.observe(p).players[p.index()].life,
                    s.life_before - if i == 1 { 3 } else { 0 }
                );
            }
            g.restore(&saved)?;
            assert_eq!(
                g.apply_index(actor, revision, index),
                Err(Error::StaleRevision)
            );
            rejects += 1;
            g.apply(actor, a)?;
            assert_eq!(g.snapshot(), expected);
            decisions += 1;
            replayed += 1;
            maximum = maximum.max(g.state_bytes());
        }
        assert!(
            s.phase == 3 && g.outcome().is_some(),
            "incomplete {} seed{seed} phase{} turn{}",
            NAMES[i],
            s.phase,
            g.observe(p).turn
        );
    }
    if games >= 100 {
        for ledger in [&paid, &entered, &equipped, &removed] {
            assert!(ledger.iter().flatten().all(|n| *n > 0));
        }
        assert!(closed > 0);
    }
    println!(
        "games={games} decisions={decisions} replays={replayed} parent_closed={closed} rejects={rejects} initial_bytes={initial} max_bytes={maximum} paid={paid:?} entered={entered:?} equipped={equipped:?} removed={removed:?}"
    );
    Ok(())
}
