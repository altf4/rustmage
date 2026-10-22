//! Legal Standard self-play with exact searches, discarded Hand abilities and Chocobos.
//! Each decision is replayed through a clone and a restored snapshot. Arrays use NAMES order.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Observation, Player, SearchDestination,
    SearchFilter, SpellView, StackKind, Target, Zone,
};
use std::collections::{BTreeMap, BTreeSet};
const NAMES: [&str; 19] = [
    "Circuitous Route",
    "Map the Frontier",
    "Dragonstorm Forecaster",
    "Magitek Infantry",
    "Honored Knight-Captain",
    "Mystical Teachings",
    "Glimpse the Core",
    "Radioactive Spider",
    "Spinewoods Armadillo",
    "Call the Mountain Chocobo",
    "Chocobo Racetrack",
    "Gysahl Greens",
    "Harvester of Misery",
    "Outcaster Greenblade",
    "Pia, Aether Ascetic",
    "Simulacrum Shaper",
    "Solemn Simulacrum",
    "Steel Wrecking Ball",
    "Sureshot Sower",
];
fn index(c: Card) -> Option<usize> {
    NAMES.iter().position(|n| *n == c.name())
}
fn deck(i: usize) -> Vec<Card> {
    let mut d = Vec::new();
    for (n, k) in [
        ("Plains", 7),
        ("Island", 5),
        ("Swamp", 6),
        ("Mountain", 6),
        ("Forest", 8),
    ] {
        d.extend(vec![Card::from_name(n).unwrap(); k]);
    }
    let fixture = [
        "Sandstorm Verge",
        "Sandstorm Verge",
        "Boulderborn Dragon",
        "Medic's Kitesail",
        "Medic's Kitesail",
        "Cathar Commando",
        "Captivating Cave",
        "Masked Meower",
        "Sandstorm Verge",
        "Lush Portico",
        "Gysahl Greens",
        "Call the Mountain Chocobo",
        "Boulderborn Dragon",
        "Sandstorm Verge",
        "Anthem of Champions",
        "Vampiric Rites",
        "Vampiric Rites",
        "Boulderborn Dragon",
        "Boulderborn Dragon",
    ][i];
    for n in [
        NAMES[i],
        "Bear Cub",
        "Three Tree Mascot",
        "Spectral Sailor",
        fixture,
        "Shock",
        "Thrill of Possibility",
    ] {
        d.extend([Card::from_name(n).unwrap(); 4]);
    }
    assert_eq!(d.len(), 60);
    d
}
fn matches(c: Card, f: SearchFilter) -> bool {
    match f {
        SearchFilter::Any => true,
        SearchFilter::ManaValue { minimum, maximum } => {
            (minimum..=maximum).contains(&c.fixed_mana_value())
        }
        SearchFilter::AnyOf(fs) => fs.iter().any(|f| matches(c, *f)),
        SearchFilter::AllOf(fs) => fs.iter().all(|f| matches(c, *f)),
        SearchFilter::Name(n) => c.name() == n,
        SearchFilter::Subtype(t) => c.has_subtype(t),
        SearchFilter::Keyword(k) => c.keywords().contains(k),
        SearchFilter::BasicLand => c.is_land() && c.is_basic_land(),
        SearchFilter::Land => c.is_land(),
        SearchFilter::Creature => c.is_creature(),
        SearchFilter::CreatureManaValueAtMost(maximum) => {
            c.is_creature() && c.fixed_mana_value() <= maximum
        }
        SearchFilter::Artifact => c.is_artifact(),
        SearchFilter::Enchantment => c
            .info()
            .type_line
            .split(" — ")
            .next()
            .unwrap()
            .split_whitespace()
            .any(|x| x == "Enchantment"),
        SearchFilter::Kinds(k) => [
            (1, "Creature"),
            (2, "Land"),
            (4, "Sorcery"),
            (8, "Instant"),
            (16, "Artifact"),
            (32, "Enchantment"),
            (64, "Planeswalker"),
        ]
        .iter()
        .any(|(b, n)| {
            k & b != 0
                && c.info()
                    .type_line
                    .split(" — ")
                    .next()
                    .unwrap()
                    .split_whitespace()
                    .any(|x| x == *n)
        }),
    }
}
fn packets(o: &Observation) -> impl Iterator<Item = &SpellView> {
    o.stack
        .iter()
        .chain(&o.waiting_triggers)
        .chain(o.announcing.iter())
        .chain(o.resolving.iter())
}
fn score(o: &Observation, a: &Action, i: usize, seed: usize, casts: &[[u64; 2]; 19]) -> i32 {
    let p = o.actor.unwrap();
    let object = |id| o.objects.iter().find(|v| v.id == id).unwrap();
    let own = |n| {
        o.objects
            .iter()
            .any(|v| v.owner == p && v.zone == Zone::Battlefield && v.card.name() == n)
    };
    match a {
        Action::Keep => 2000,
        Action::Mulligan | Action::Concede => -10000,
        Action::PlayLand { card } => {
            let n = object(*card).card;
            let k = o
                .objects
                .iter()
                .filter(|v| v.owner == p && v.zone == Zone::Battlefield && v.card == n)
                .count();
            if i == 6 && n.name() == "Captivating Cave" {
                if o.turn > 30 { 700 } else { -1000 }
            } else {
                1400 - k as i32
            }
        }
        Action::AnnounceCast { card } | Action::Cast { card, .. } => {
            let n = object(*card).card.name();
            if n == NAMES[i] {
                if matches!(i, 8 | 12 | 17 | 18) && casts[i][p.index()] > 0 {
                    700
                } else if i == 12 && !own("Three Tree Mascot") {
                    650
                } else {
                    1100
                }
            } else if n == "Thrill of Possibility" {
                if i == 6 { 1000 } else { 200 }
            } else if n == "Shock" {
                if matches!(i, 15 | 16) && own(NAMES[i]) {
                    1050
                } else if o.turn > 35 {
                    600
                } else {
                    -1000
                }
            } else if n == "Three Tree Mascot" {
                1000
            } else if n == "Bear Cub" || n == "Spectral Sailor" {
                850
            } else {
                800
            }
        }
        Action::AnnounceFlashback { .. } => 1150,
        Action::AnnounceActivate { card, .. }
        | Action::Activate { card, .. }
        | Action::BeginActivate { card, .. } => {
            let v = object(*card);
            if v.card.name() == NAMES[i] {
                if v.zone == Zone::Hand {
                    if casts[i][p.index()] > 0 { 1250 } else { -1000 }
                } else {
                    1200
                }
            } else if v.card.name() == "Vampiric Rites" && own(NAMES[i]) {
                1300
            } else {
                -1000
            }
        }
        Action::ChooseAdditionalCost { object: r } => {
            if object(r.card).card.name() == NAMES[i] {
                1500
            } else {
                1000
            }
        }
        Action::ChooseMode { mode } => {
            if *mode == ((seed / 19) % 2) as u16 {
                1600
            } else {
                500
            }
        }
        Action::ChooseTarget {
            target: Target::Card(id),
        } => {
            let v = object(*id);
            if o.announcing
                .as_ref()
                .is_some_and(|a| a.source_card.name() == "Shock")
                && matches!(i, 15 | 16)
                && v.card.name() == NAMES[i]
                && v.owner == p
            {
                1600
            } else if v.owner != p {
                1500
            } else {
                700
            }
        }
        Action::ChooseTarget { .. } => 400,
        Action::Discard { card } => {
            let v = object(*card);
            if i == 6 && v.card.name() == "Captivating Cave" {
                1700
            } else if v.card.name() != NAMES[i] && v.card.is_land() {
                1500
            } else {
                800
            }
        }
        Action::ChooseOptional { accept } => {
            if *accept != (seed / 19).is_multiple_of(4) {
                1600
            } else {
                500
            }
        }
        Action::DeclineEffectPayment => {
            if (seed / 19).is_multiple_of(4) {
                1600
            } else {
                -1000
            }
        }
        Action::ChooseSearch { card } => {
            let f = o.search_choice.as_ref().unwrap();
            let v = f.cards.iter().find(|v| v.id == *card).unwrap();
            if (seed / 19).is_multiple_of(4) {
                -1000
            } else if v.card.name()
                == if i < 2 {
                    "Forest"
                } else if i == 7 {
                    "Three Tree Mascot"
                } else if i == 2 {
                    "Boulderborn Dragon"
                } else if i == 5 {
                    "Cathar Commando"
                } else {
                    NAMES[i]
                }
            {
                1700
            } else {
                1600
            }
        }
        Action::FinishSearch => {
            if (seed / 19).is_multiple_of(4)
                || o.search_choice
                    .as_ref()
                    .is_some_and(|f| f.selected.len() == usize::from(f.maximum))
            {
                1500
            } else {
                0
            }
        }
        Action::TapMana { .. } | Action::ActivateMana { .. } | Action::AnnounceMana { .. } => {
            if o.decision == "pay_costs" {
                1100
            } else {
                -1000
            }
        }
        Action::PayCosts { .. } => 1900,
        Action::PutTrigger { .. } => 1800,
        Action::FinishTargets => 1300,
        Action::Attack { .. } => {
            if o.turn > 38 {
                600
            } else {
                -1000
            }
        }
        Action::Block { .. } => -1000,
        Action::FinishAttackers | Action::FinishBlockers => 400,
        _ => 0,
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse::<usize>()?;
    let (
        mut decisions,
        mut replay,
        mut rejects,
        mut partials,
        mut reveals,
        mut zeros,
        mut max_bytes,
    ) = (0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0usize);
    let (mut casts, mut abilities, mut effects, mut searches, mut flashback) = (
        [[0u64; 2]; 19],
        [[0u64; 2]; 19],
        [[0u64; 2]; 19],
        [[0u64; 2]; 19],
        [[0u64; 2]; 19],
    );
    let (
        mut discarded,
        mut life3,
        mut chocobos,
        mut death_draws,
        mut cave_returns,
        mut modes,
        mut optional,
    ) = (0u64, 0u64, 0u64, 0u64, 0u64, [0u64; 2], [0u64; 2]);
    for seed in 0..games {
        let i = seed % 19;
        let mut g = Game::new(Config {
            decks: [deck(i), deck(i)],
            seed: seed as u64,
            starting_player: if seed.is_multiple_of(2) {
                Player::P0
            } else {
                Player::P1
            },
            deck_rules: DeckRules::Standard,
        })?;
        let mut active: BTreeMap<u64, (SpellView, Observation)> = BTreeMap::new();
        let mut seen = BTreeSet::new();
        for _ in 0..100000 {
            if g.outcome().is_some() {
                break;
            }
            let actor = g.actor().unwrap();
            let before = g.observe(actor);
            let actions = g.legal_actions();
            let j = (0..actions.len())
                .max_by_key(|&j| (score(&before, &actions[j], i, seed, &casts), -(j as i64)))
                .unwrap();
            let a = actions[j].clone();
            if let Some(f) = &before.search_choice {
                assert!(g.observe(actor.other()).search_choice.is_none());
                assert!(g.observe(actor.other()).actions.is_empty());
                let legal = actions
                    .iter()
                    .filter_map(|a| {
                        if let Action::ChooseSearch { card } = a {
                            Some(*card)
                        } else {
                            None
                        }
                    })
                    .collect::<BTreeSet<_>>();
                let expected = if f.selected.len() < usize::from(f.maximum) {
                    f.cards
                        .iter()
                        .filter(|v| matches(v.card, f.filter) && !f.selected.contains(&v.id))
                        .map(|v| v.id)
                        .collect()
                } else {
                    BTreeSet::new()
                };
                assert_eq!(legal, expected);
                let snap = g.snapshot();
                let rev = g.revision();
                assert_eq!(
                    g.apply(actor.other(), Action::FinishSearch),
                    Err(Error::WrongActor)
                );
                assert_eq!(
                    g.apply(actor, Action::ChooseSearch { card: u32::MAX }),
                    Err(Error::IllegalAction)
                );
                assert_eq!(g.snapshot(), snap);
                assert_eq!(g.revision(), rev);
                rejects += 2;
                if let Some(v) = f.cards.iter().find(|v| !matches(v.card, f.filter)) {
                    assert_eq!(
                        g.apply(actor, Action::ChooseSearch { card: v.id }),
                        Err(Error::IllegalAction)
                    );
                    assert_eq!(g.snapshot(), snap);
                    rejects += 1;
                }
            }
            if let Action::ChooseMode { mode } = a
                && before
                    .announcing
                    .as_ref()
                    .is_some_and(|v| v.source_card.name() == "Glimpse the Core")
            {
                modes[usize::from(mode)] += 1;
            }
            if let Action::ChooseOptional { accept } = a {
                optional[usize::from(accept)] += 1;
            }
            if let (Action::PayCosts { payment, .. }, Some(ann), Some(req)) =
                (&a, &before.announcing, before.payment_request.as_ref())
            {
                assert_eq!(
                    payment.iter().map(|x| u64::from(*x)).sum::<u64>(),
                    req.required.iter().map(|x| u64::from(*x)).sum::<u64>()
                        + u64::from(req.generic)
                );
                if let Some(k) = index(ann.source_card) {
                    if ann.kind == StackKind::Spell {
                        casts[k][actor.index()] += 1;
                        if ann.cast.unwrap().method == rustmage::CastMethod::Flashback {
                            flashback[k][actor.index()] += 1;
                        }
                    } else {
                        abilities[k][actor.index()] += 1;
                    }
                }
            }
            if matches!(a, Action::Pass)
                && let Some(packet) = before.stack.last()
                && index(packet.source_card).is_some()
                && seen.insert(packet.id)
            {
                active.insert(packet.id, (packet.clone(), before.clone()));
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
            replay += 1;
            let after = g.observe(actor);
            assert_eq!(after.schema, rustmage::OBSERVATION_SCHEMA);
            max_bytes = max_bytes.max(g.state_bytes());
            if let (Some(ann), Action::PayCosts { .. }) = (&before.announcing, &a)
                && let Some(k) = index(ann.source_card)
                && matches!(k, 8 | 12 | 17 | 18)
                && ann.kind != StackKind::Spell
            {
                assert_eq!(
                    before
                        .objects
                        .iter()
                        .find(|v| v.id == ann.card)
                        .unwrap()
                        .zone,
                    Zone::Hand
                );
                assert_eq!(
                    after
                        .objects
                        .iter()
                        .find(|v| v.id == ann.card)
                        .unwrap()
                        .zone,
                    Zone::Graveyard
                );
                assert_eq!(
                    after.players[actor.index()].cards_discarded_this_turn,
                    before.players[actor.index()].cards_discarded_this_turn + 1
                );
                assert_eq!(
                    after.players[actor.index()].spells_cast_this_turn,
                    before.players[actor.index()].spells_cast_this_turn
                );
                discarded += 1;
            }
            if let Some(f) = &before.search_choice {
                if let Action::ChooseSearch { .. } = a {
                    let next = after.search_choice.as_ref().unwrap();
                    assert_eq!(
                        before.players[actor.index()].library_count,
                        after.players[actor.index()].library_count
                    );
                    assert_eq!(f.cards, next.cards);
                    partials += 1;
                }
                if a == Action::FinishSearch {
                    let packet = before.resolving.as_ref().unwrap();
                    if let Some(k) = index(packet.source_card) {
                        searches[k][actor.index()] += 1;
                    }
                    assert_eq!(
                        after.players[actor.index()].library_count,
                        before.players[actor.index()].library_count - f.selected.len()
                    );
                    for id in &f.selected {
                        let old = f.cards.iter().find(|v| v.id == *id).unwrap();
                        let view = g.observe(actor.other());
                        let new = view.objects.iter().find(|v| v.id == *id).unwrap();
                        assert_eq!(new.reference.generation, old.reference.generation + 1);
                        assert_eq!(
                            new.zone,
                            if f.destination == SearchDestination::Hand {
                                Zone::Hand
                            } else {
                                Zone::Battlefield
                            }
                        );
                        if f.destination == SearchDestination::Battlefield {
                            assert_eq!(new.tapped, f.tapped);
                        } else {
                            assert!(after.known_hand.contains(&new.reference));
                            reveals += 1;
                        }
                    }
                    zeros += u64::from(f.selected.is_empty());
                    if packet.source_card.name() == "Spinewoods Armadillo" {
                        assert_eq!(
                            after.players[actor.index()].life,
                            before.players[actor.index()].life + 3
                        );
                        life3 += 1;
                    }
                    if packet.source_card.name() == "Call the Mountain Chocobo" {
                        let birds = |o: &Observation| {
                            o.objects
                                .iter()
                                .filter(|v| {
                                    v.owner == actor
                                        && v.zone == Zone::Battlefield
                                        && v.card.token_info().is_some_and(|t| t.subtypes == "Bird")
                                })
                                .count()
                        };
                        assert_eq!(birds(&after), birds(&before) + 1);
                        chocobos += 1;
                    }
                }
            }
            let done = active
                .keys()
                .copied()
                .filter(|id| !packets(&after).any(|v| v.id == *id))
                .collect::<Vec<_>>();
            for id in done {
                let (packet, base) = active.remove(&id).unwrap();
                let k = index(packet.source_card).unwrap();
                let p = packet.controller.index();
                effects[k][p] += 1;
                if k == 6 && packet.mode == Some(1) {
                    for target in &packet.targets {
                        if let Some(r) = target.object {
                            let v = after.objects.iter().find(|v| v.id == r.card).unwrap();
                            assert_eq!(v.zone, Zone::Battlefield);
                            assert!(v.tapped);
                            assert!(v.reference.generation > r.generation);
                            cave_returns += 1;
                        }
                    }
                }
                if matches!(k, 15 | 16)
                    && matches!(packet.kind, StackKind::TriggeredAbility { .. })
                    && base
                        .objects
                        .iter()
                        .any(|v| v.id == packet.card && v.zone == Zone::Graveyard)
                    && after.players[p].cards_drawn_this_turn
                        > base.players[p].cards_drawn_this_turn
                {
                    death_draws += 1;
                }
                if packet
                    .cast
                    .is_some_and(|c| c.method == rustmage::CastMethod::Flashback)
                {
                    assert_eq!(
                        after
                            .objects
                            .iter()
                            .find(|v| v.id == packet.card)
                            .unwrap()
                            .zone,
                        Zone::Exile
                    );
                }
            }
        }
        assert!(g.outcome().is_some(), "unfinished seed{seed}");
    }
    if games >= 100 {
        assert!(
            casts.iter().all(|v| v.iter().all(|x| *x > 0)),
            "casts{casts:?}"
        );
        assert!(
            effects.iter().all(|v| v.iter().all(|x| *x > 0)),
            "effects{effects:?}"
        );
        for i in [2, 3, 4, 7, 8, 12, 17, 18] {
            assert!(
                abilities[i].iter().all(|x| *x > 0),
                "abilities{i} {:?}",
                abilities[i]
            );
        }
        for i in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 13, 14, 15, 16] {
            assert!(
                searches[i].iter().all(|x| *x > 0),
                "search{i} {:?}",
                searches[i]
            );
        }
        for i in [5, 9, 11] {
            assert!(
                flashback[i].iter().all(|x| *x > 0),
                "flashback{i} {:?}",
                flashback[i]
            );
        }
        assert!(
            discarded > 0
                && life3 > 0
                && chocobos > 0
                && death_draws > 0
                && cave_returns > 0
                && modes.iter().all(|x| *x > 0)
                && optional.iter().all(|x| *x > 0)
        );
    }
    println!(
        "Completed {games} exact-search Standard games, {decisions} decisions/{replay} replays, casts {casts:?}, abilities {abilities:?}, effects {effects:?}, searches {searches:?}, flashback {flashback:?}, {partials} deferred selections/{reveals} revealed hand objects/{zeros} zero searches, {discarded} actual Hand discards/{life3} life-three tails/{chocobos} post-search Chocobos/{death_draws} death draws/{cave_returns} tapped Cave returns, Glimpse modes {modes:?}, optional accept {optional:?}, {rejects} rejected illegal actions, {max_bytes} max logical bytes"
    );
    Ok(())
}
