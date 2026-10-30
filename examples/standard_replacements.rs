//! Actual legal Standard games exercising all replacement bodies and paid residuals.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Keywords, LibraryDestination, Observation,
    Player, StackKind, Target, Zone,
};
const NAMES: [&str; 9] = [
    "Doubling Season",
    "Exalted Sunborn",
    "Loading Zone",
    "Michelangelo, Weirdness to 11",
    "Yoshimaru, Beloved Companion",
    "Caradora, Heart of Alacria",
    "Quina, Qu Gourmet",
    "Draconic Visitor",
    "Bartolomé del Presidio",
];
fn deck(i: usize, focal: bool) -> Vec<Card> {
    let mut d = Vec::new();
    for (n, k) in [
        ("Forest", 9),
        ("Plains", 7),
        ("Mountain", 6),
        ("Swamp", 4),
        ("Island", 2),
    ] {
        d.extend(vec![Card::from_name(n).unwrap(); k]);
    }
    for n in [
        if focal { NAMES[i] } else { "Chrome Companion" },
        "Bear Cub",
        "Candy Trail",
        "Cartographer's Companion",
        "Snakeskin Veil",
        "Alacrian Jaguar",
        "Thrill of Possibility",
        "Llanowar Elves",
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
    tokens: usize,
    counters: u64,
    search: bool,
    round: u8,
    exiled_turn: Option<u32>,
    goal: Option<u32>,
}
fn field<'a>(o: &'a Observation, p: Player, n: &str) -> Option<&'a rustmage::ObjectView> {
    o.objects
        .iter()
        .find(|v| v.owner == p && v.zone == Zone::Battlefield && v.card.name() == n)
}
fn token_count(o: &Observation, p: Player) -> usize {
    o.objects
        .iter()
        .filter(|v| v.owner == p && v.zone == Zone::Battlefield && v.card.is_token())
        .count()
}
fn score(o: &Observation, a: &Action, p: Player, i: usize, s: &Pilot, warp: bool) -> i32 {
    let actor = o.actor.unwrap();
    let obj = |id| o.objects.iter().find(|v| v.id == id).unwrap();
    let bear = field(o, p, "Bear Cub");
    let lands = o
        .objects
        .iter()
        .filter(|v| v.owner == p && v.zone == Zone::Battlefield && v.card.is_land())
        .count();
    let ready = lands >= 8
        && bear.is_some()
        && (i != 8 || field(o, p, "Candy Trail").is_some())
        && (!warp
            || o.objects.iter().any(|v| {
                v.owner == p
                    && v.zone == Zone::Hand
                    && v.card.name()
                        == if i == 1 {
                            "Cartographer's Companion"
                        } else {
                            "Snakeskin Veil"
                        }
            }));
    match a {
        Action::Keep => 3000,
        Action::Mulligan | Action::Concede => -10000,
        Action::Cast { .. }
        | Action::BeginCast { .. }
        | Action::Activate { .. }
        | Action::AnnounceFlashback { .. }
        | Action::AnnouncePlot { .. }
        | Action::AnnouncePlotted { .. } => -1000,
        Action::AnnounceWarp { card } => {
            if actor == p && warp && s.stage == 0 && obj(*card).card.name() == NAMES[i] && ready {
                2100
            } else {
                -1000
            }
        }
        Action::AnnounceExile { card } => {
            if actor == p
                && s.stage == 7
                && Some(*card) == s.source
                && o.turn > s.exiled_turn.unwrap()
            {
                2100
            } else {
                -1000
            }
        }
        Action::AnnounceCast { card } => {
            let n = obj(*card).card.name();
            if actor != p {
                return -1000;
            }
            if s.stage == 0 {
                if n == "Bear Cub" && bear.is_none()
                    || n == "Candy Trail" && i == 8 && field(o, p, n).is_none()
                {
                    return 1900;
                }
                if n == NAMES[i] && ready && !warp {
                    return 2100;
                }
            }
            if s.stage == 2
                && ([0, 1, 6, 7].contains(&i) && n == "Cartographer's Companion"
                    || [2, 5].contains(&i) && n == "Snakeskin Veil")
            {
                return 2100;
            }
            if n == "Thrill of Possibility" && s.stage < 6 {
                return 1700;
            }
            -1000
        }
        Action::AnnounceActivate { card, .. } => {
            if actor == p
                && s.stage == 2
                && ([4, 8].contains(&i) && Some(*card) == s.source
                    || i == 3 && obj(*card).card.has_subtype("Mutagen"))
                || actor == p && s.stage == 4 && i == 6 && Some(*card) == s.source
            {
                2100
            } else {
                -1000
            }
        }
        Action::ChooseTarget {
            target: Target::Card(id),
        } => {
            if Some(*id) == (if i == 4 { s.source } else { bear.map(|v| v.id) }) {
                2500
            } else {
                500
            }
        }
        Action::ChooseTarget { .. } => 500,
        Action::ChooseAdditionalCost { object } => {
            let v = obj(object.card);
            if i == 8 && v.card.name() == "Candy Trail" || i == 6 && v.card.has_subtype("Frog") {
                2500
            } else if v.card.name() == NAMES[i] || v.card.name() == "Bear Cub" && bear.is_none() {
                100
            } else {
                1800
            }
        }
        Action::ChooseSearch { .. } => 2500,
        Action::FinishSearch => 1000,
        Action::ChooseOptional { accept } => {
            if *accept {
                2000
            } else {
                1000
            }
        }
        Action::ChooseReplacement { .. } | Action::ChooseCreatedToken { .. } => 2400,
        Action::PlayLand { card } => {
            let n = obj(*card).card;
            let count = o
                .objects
                .iter()
                .filter(|v| v.owner == actor && v.zone == Zone::Battlefield && v.card == n)
                .count();
            1700 - 50 * count as i32
        }
        Action::TapMana { .. } | Action::ActivateMana { .. } | Action::AnnounceMana { .. } => {
            if o.decision == "pay_costs" {
                2000
            } else {
                -1000
            }
        }
        Action::PayCosts { .. } | Action::ChooseSpellCost { .. } | Action::PutTrigger { .. } => {
            2200
        }
        Action::ArrangeLibrary { destination, .. } => {
            if *destination == LibraryDestination::Top {
                1900
            } else {
                1800
            }
        }
        Action::Discard { card } | Action::Bottom { card } => {
            let v = obj(*card);
            let n = v.card.name();
            if actor == p
                && s.stage < 6
                && (n == NAMES[i]
                    || n == "Bear Cub" && bear.is_none()
                    || n == "Snakeskin Veil" && [2, 5].contains(&i)
                    || n == "Cartographer's Companion" && [0, 1, 6, 7].contains(&i)
                    || n == "Candy Trail" && i == 8)
            {
                100
            } else {
                1800
            }
        }
        Action::Attack { .. } => {
            if s.stage == 8 {
                100
            } else {
                -1000
            }
        }
        Action::Block { .. } => -1000,
        Action::FinishTargets | Action::FinishAttackers | Action::FinishBlockers => 1000,
        Action::Pass => 0,
        _ => 500,
    }
}
fn witness(o: &Observation, p: Player, i: usize, s: &Pilot) {
    let v = o.objects.iter().find(|v| Some(v.id) == s.source).unwrap();
    assert_eq!(v.zone, Zone::Battlefield);
    if [0, 1, 6, 7].contains(&i) {
        assert_eq!(
            token_count(o, p),
            s.tokens + if [6, 7].contains(&i) { 1 } else { 2 }
        );
        if i == 6 {
            assert_eq!(v.counters.plus_one, 1);
        }
        if i == 7 {
            let dragon = o
                .objects
                .iter()
                .find(|v| {
                    v.owner == p
                        && v.zone == Zone::Battlefield
                        && v.card.is_token()
                        && v.card.has_subtype("Dragon")
                })
                .unwrap();
            assert_eq!((dragon.power, dragon.toughness), (5, 5));
            assert!(dragon.keywords.contains(Keywords::FLYING));
            assert_eq!(dragon.card.colors(), 8);
        }
    } else if i == 4 {
        assert_eq!(v.counters.plus_one, 2);
    } else if i == 8 {
        assert_eq!(v.counters.plus_one, 1);
        assert!(field(o, p, "Candy Trail").is_none());
    } else {
        assert_eq!(
            field(o, p, "Bear Cub").unwrap().counters.plus_one,
            s.counters + 2
        );
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse::<usize>()?;
    let (mut casts, mut effects, mut warpcasts, mut exiles, mut recasts) = (
        [[0u64; 2]; 9],
        [[0u64; 2]; 9],
        [[0u64; 2]; 2],
        [[0u64; 2]; 2],
        [[0u64; 2]; 2],
    );
    let (mut hard_effects, mut warp_effects, mut recast_effects) =
        ([[0u64; 2]; 9], [[0u64; 2]; 2], [[0u64; 2]; 2]);
    let (mut decisions, mut replays, mut closed, mut denied, mut frames, mut initial, mut maximum) =
        (0u64, 0u64, 0u64, 0u64, 0u64, 0usize, 0usize);
    for seed in 0..games {
        let i = seed % 9;
        let p = if seed / 9 % 2 == 0 {
            Player::P0
        } else {
            Player::P1
        };
        let warp = [1, 2].contains(&i) && seed / 18 % 2 == 1;
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
            if quiet
                && s.stage == 1
                && o.objects
                    .iter()
                    .any(|v| Some(v.id) == s.source && v.zone == Zone::Battlefield)
            {
                let v = o.objects.iter().find(|v| Some(v.id) == s.source).unwrap();
                if i == 1 {
                    assert!(
                        v.keywords
                            .contains(Keywords::FLYING.union(Keywords::LIFELINK))
                    );
                }
                if i == 3 {
                    assert!(o.objects.iter().any(|v| v.owner == p
                        && v.zone == Zone::Battlefield
                        && v.card.has_subtype("Mutagen")));
                }
                if i == 5 {
                    assert!(s.search);
                    assert!(o.objects.iter().any(|v| v.owner == p
                        && v.zone == Zone::Hand
                        && v.card.has_subtype("Mount")));
                }
                s.tokens = token_count(&o, p);
                s.counters = field(&o, p, "Bear Cub").unwrap().counters.plus_one;
                s.stage = 2;
            }
            if quiet && s.stage == 3 {
                if i == 6 {
                    s.stage = 4;
                } else {
                    witness(&o, p, i, &s);
                    effects[i][p.index()] += 1;
                    if warp {
                        if s.round == 1 {
                            warp_effects[i - 1][p.index()] += 1;
                        } else {
                            recast_effects[i - 1][p.index()] += 1;
                        }
                    } else {
                        hard_effects[i][p.index()] += 1;
                    }
                    s.stage = if warp && s.round == 1 { 6 } else { 8 };
                }
            }
            if quiet && s.stage == 5 {
                witness(&o, p, i, &s);
                effects[i][p.index()] += 1;
                hard_effects[i][p.index()] += 1;
                s.stage = 8;
            }
            if s.stage == 6
                && let Some(right) = o
                    .warped_cards
                    .iter()
                    .find(|r| Some(r.object.card) == s.source)
            {
                assert_eq!(right.owner, p);
                assert_eq!(right.exiled_turn, o.turn);
                s.exiled_turn = Some(right.exiled_turn);
                s.stage = 7;
                exiles[i - 1][p.index()] += 1;
            }
            let actions = g.legal_actions();
            let (index, a) = actions
                .iter()
                .enumerate()
                .max_by_key(|(_, a)| score(&o, a, p, i, &s, warp))
                .unwrap();
            let a = a.clone();
            assert!(
                score(&o, &a, p, i, &s, warp) >= 0,
                "seed{seed} step{step} {} {:?}",
                o.decision,
                actions
            );
            if let Action::AnnounceCast { card }
            | Action::AnnounceWarp { card }
            | Action::AnnounceExile { card } = a
                && actor == p
            {
                let name = o.objects.iter().find(|v| v.id == card).unwrap().card.name();
                if name == NAMES[i] && [0, 7].contains(&s.stage) {
                    s.source = Some(card);
                } else if s.stage == 2
                    && ["Snakeskin Veil", "Cartographer's Companion"].contains(&name)
                {
                    s.goal = Some(card);
                    s.stage = 3;
                }
            }
            if let Action::AnnounceActivate { card, .. } = a
                && actor == p
                && (s.stage == 2
                    && ([4, 8].contains(&i) && Some(card) == s.source
                        || i == 3
                            && o.objects
                                .iter()
                                .any(|v| v.id == card && v.card.has_subtype("Mutagen")))
                    || s.stage == 4 && i == 6 && Some(card) == s.source)
            {
                s.stage = 3;
                if i == 6 {
                    s.stage = 5;
                }
            }
            if matches!(a, Action::ChooseSearch { .. }) {
                s.search = true;
                assert!(g.observe(actor.other()).search_choice.is_none());
            }
            if matches!(a, Action::PayCosts { .. })
                && let Some(item) = &o.announcing
                && item.kind == StackKind::Spell
                && Some(
                    item.source()
                        .expect("example expects an object-backed packet")
                        .id,
                ) == s.source
            {
                s.round += 1;
                s.stage = 1;
                if warp && s.round == 1 {
                    warpcasts[i - 1][p.index()] += 1;
                } else if warp {
                    recasts[i - 1][p.index()] += 1;
                } else {
                    casts[i][p.index()] += 1;
                }
            }
            let frame = o.replacement_choice.is_some()
                || o.created_token_choice.is_some()
                || o.search_choice.is_some()
                || o.library_choice.is_some();
            if frame {
                frames += 1;
            }
            let saved = g.snapshot();
            let rev = g.revision();
            let mut fork = g.clone();
            fork.apply(actor, a.clone())?;
            let expected = fork.snapshot();
            let parent = g.clone();
            let mut child = parent.clone();
            drop(parent);
            child.apply(actor, a.clone())?;
            assert_eq!(child.snapshot(), expected);
            closed += 1;
            assert!(g.apply(actor.other(), a.clone()).is_err());
            assert_eq!(g.snapshot(), saved);
            assert_eq!(g.revision(), rev);
            denied += 1;
            g.apply_index(actor, rev, index)?;
            assert_eq!(g.snapshot(), expected);
            for viewer in [Player::P0, Player::P1] {
                assert_eq!(
                    serde_json::to_value(g.observe(viewer))?,
                    serde_json::to_value(fork.observe(viewer))?
                );
            }
            g.restore(&saved)?;
            assert_eq!(g.apply_index(actor, rev, index), Err(Error::StaleRevision));
            denied += 1;
            g.apply(actor, a)?;
            assert_eq!(g.snapshot(), expected);
            replays += 1;
            decisions += 1;
            maximum = maximum.max(g.state_bytes());
        }
        assert!(g.outcome().is_some(), "seed{seed} failed to terminate");
    }
    if games >= 100 {
        for (i, rows) in casts.iter().enumerate() {
            for (p, n) in rows.iter().enumerate() {
                assert!(*n > 0, "cast {} seat{p}", NAMES[i]);
                assert!(hard_effects[i][p] > 0, "effect {} seat{p}", NAMES[i]);
            }
        }
        for rows in [
            &warpcasts,
            &exiles,
            &recasts,
            &warp_effects,
            &recast_effects,
        ] {
            assert!(rows.iter().flatten().all(|n| *n > 0), "Warp route {rows:?}");
        }
    }
    println!(
        "games={games} decisions={decisions} replays={replays} parent_closed={closed} denied={denied} frames={frames} initial={initial} max={maximum} casts={casts:?} effects={effects:?} hard_effects={hard_effects:?} warp_effects={warp_effects:?} recast_effects={recast_effects:?} warp={warpcasts:?} exiles={exiles:?} recasts={recasts:?}"
    );
    Ok(())
}
