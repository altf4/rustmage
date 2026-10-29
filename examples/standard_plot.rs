//! Legal Standard Plot/hard-cast games with independently checked full effects.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Keywords, ObjectRef, Observation, Player,
    StackKind, Target, Zone,
};
const NAMES: [&str; 13] = [
    "Beastbond Outcaster",
    "Blacksnag Buzzard",
    "Cunning Coyote",
    "Djinn of Fool's Fall",
    "Irascible Wolverine",
    "Loan Shark",
    "Slickshot Show-Off",
    "Spinewoods Paladin",
    "Stagecoach Security",
    "Stingerback Terror",
    "Demonic Ruckus",
    "Rictus Robber",
    "Rise of the Varmints",
];
const PREFERRED: [&str; 13] = [
    "Forest", "Swamp", "Mountain", "Island", "Mountain", "Island", "Mountain", "Forest", "Plains",
    "Mountain", "Mountain", "Swamp", "Forest",
];
fn deck(i: usize, focal: bool) -> Vec<Card> {
    let mut d = Vec::new();
    for (n, k) in [
        ("Forest", 8),
        ("Mountain", 8),
        (PREFERRED[i], 8),
        ("Plains", 4),
    ] {
        d.extend(vec![Card::from_name(n).unwrap(); k]);
    }
    for n in [
        if focal { NAMES[i] } else { "Chrome Companion" },
        "Ambush Wolf",
        "Bear Cub",
        "Llanowar Elves",
        "Shock",
        "Scorching Shot",
        "Cartographer's Companion",
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
    plotted_turn: Option<u32>,
    paid: bool,
    done: bool,
    death_turn: Option<u32>,
    loan_turn: Option<u32>,
    pump: bool,
    aura_death: bool,
    library: usize,
    life: i128,
    rights: usize,
    grave: usize,
    death_library: usize,
    mate: Option<ObjectRef>,
}
fn on_field(o: &Observation, p: Player, n: &str) -> bool {
    o.objects
        .iter()
        .any(|v| v.owner == p && v.zone == Zone::Battlefield && v.card.name() == n)
}
fn ready(o: &Observation, i: usize, p: Player, s: &Pilot, plotted: bool) -> bool {
    (i != 0 || on_field(o, p, "Ambush Wolf"))
        && (![2, 8, 10].contains(&i) || on_field(o, p, "Bear Cub"))
        && (![1, 11, 12].contains(&i) || s.death_turn == Some(o.turn))
        && (i != 5 || s.loan_turn == Some(o.turn))
        && (i != 9 || o.players[p.index()].hand_count <= if plotted { 6 } else { 7 })
}
fn score(
    o: &Observation,
    a: &Action,
    i: usize,
    p: Player,
    s: &Pilot,
    plotted: bool,
    available: bool,
) -> i32 {
    let actor = o.actor.unwrap();
    let obj = |id| o.objects.iter().find(|v| v.id == id).unwrap();
    let lands = o
        .objects
        .iter()
        .filter(|v| v.owner == p && v.zone == Zone::Battlefield && v.card.is_land() && !v.tapped)
        .count();
    let later = s.plotted_turn.is_some_and(|t| o.turn > t);
    match a {
        Action::Cast { .. }
        | Action::BeginCast { .. }
        | Action::AnnounceExile { .. }
        | Action::AnnounceFlashback { .. } => -1000,
        Action::AnnouncePlot { card } => {
            if actor == p
                && plotted
                && s.plotted_turn.is_none()
                && !s.paid
                && obj(*card).card.name() == NAMES[i]
            {
                1850
            } else {
                -1000
            }
        }
        Action::AnnouncePlotted { card } => {
            if actor == p && Some(*card) == s.source && !s.paid && ready(o, i, p, s, plotted) {
                1950
            } else {
                -1000
            }
        }
        Action::AnnounceCast { card } => {
            let v = obj(*card);
            let n = v.card.name();
            if s.done {
                return if v.card.is_creature() {
                    1200
                } else if n == "Shock" || n == "Scorching Shot" {
                    1100
                } else {
                    -1000
                };
            }
            if actor != p {
                return -1000;
            }
            if n == NAMES[i] {
                return if !plotted && !s.paid && ready(o, i, p, s, plotted) {
                    1900
                } else {
                    -1000
                };
            }
            if n == "Ambush Wolf" && i == 0 && !on_field(o, p, n) {
                return 1750;
            }
            if n == "Bear Cub" {
                let mate = [1, 2, 8, 10, 11, 12].contains(&i)
                    && !on_field(o, p, n)
                    && s.death_turn.is_none();
                let loan = i == 5
                    && s.loan_turn != Some(o.turn)
                    && lands >= 7
                    && available
                    && (!plotted || later);
                if mate || loan {
                    return 1750;
                }
            }
            if i == 9 && v.card.is_creature() && o.players[p.index()].hand_count > 6 {
                return 1750;
            }
            if n == "Shock" {
                let death = [1, 11, 12].contains(&i)
                    && on_field(o, p, "Bear Cub")
                    && s.death_turn.is_none()
                    && lands >= 6
                    && available
                    && (!plotted || later);
                let pump = i == 6
                    && s.paid
                    && !s.pump
                    && s.source.is_some_and(|id| {
                        o.objects
                            .iter()
                            .any(|v| v.id == id && v.zone == Zone::Battlefield)
                    });
                let shed_hand = i == 9 && o.players[p.index()].hand_count > 6;
                if death || pump || shed_hand {
                    return 2000;
                }
            }
            if n == "Scorching Shot"
                && i == 10
                && s.paid
                && !s.aura_death
                && s.source.is_some_and(|id| {
                    o.objects
                        .iter()
                        .any(|v| v.id == id && v.zone == Zone::Battlefield)
                })
            {
                return 2000;
            }
            -1000
        }
        Action::ChooseTarget {
            target: Target::Card(id),
        } => {
            let v = obj(*id);
            if !s.done && actor == p {
                if [1, 2, 8, 10, 11, 12].contains(&i)
                    && v.owner == p
                    && v.zone == Zone::Battlefield
                    && v.card.name() == "Bear Cub"
                {
                    return 2300;
                }
                return 1000;
            }
            if v.owner != actor { 1800 } else { 800 }
        }
        Action::ChooseTarget {
            target: Target::Player(who),
        } => {
            if *who != actor {
                2000
            } else {
                700
            }
        }
        Action::PayCosts { .. } | Action::PayPlot { .. } | Action::ChooseSpellCost { .. } => 2400,
        Action::TapMana { .. } | Action::ActivateMana { .. } | Action::AnnounceMana { .. } => {
            if ["pay_costs", "plot_payment", "resolution_mana"].contains(&o.decision) {
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
            if n == PREFERRED[i] && count < 2 {
                1650
            } else if n == "Forest" && count < 2 {
                1620
            } else {
                1550 - 20 * count as i32
            }
        }
        Action::Attack { .. } => {
            if s.done {
                1700
            } else {
                -1000
            }
        }
        Action::Block { .. } => {
            if s.done {
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
                && !s.done
                && ((n == NAMES[i] && s.source.is_none())
                    || (n == "Shock" && ([1, 6, 11, 12].contains(&i)))
                    || (n == "Scorching Shot" && i == 10)
                    || (n == "Ambush Wolf" && i == 0)
                    || (n == "Bear Cub" && [1, 2, 5, 8, 10, 11, 12].contains(&i)));
            if keep && count == 1 { -100 } else { 2500 }
        }
        Action::Keep => 3000,
        Action::Mulligan | Action::Concede => -10000,
        Action::Pass => 0,
        Action::FinishAttackers | Action::FinishBlockers => 1000,
        Action::PutTrigger { .. } | Action::FinishTargets | Action::ArrangeLibrary { .. } => 2200,
        _ => {
            if s.done {
                500
            } else {
                100
            }
        }
    }
}
fn verify(o: &Observation, i: usize, p: Player, s: &Pilot) -> bool {
    let source = o.objects.iter().find(|v| Some(v.id) == s.source).unwrap();
    let own: Vec<_> = o
        .objects
        .iter()
        .filter(|v| v.owner == p && v.zone == Zone::Battlefield)
        .collect();
    let me = &o.players[p.index()];
    match i {
        0 => {
            assert_eq!(me.library_count, s.library - 1);
            assert_eq!((source.power, source.toughness), (3, 3));
        }
        1 => {
            assert_eq!((source.power, source.toughness), (3, 2));
            assert!(source.keywords.contains(Keywords::FLYING));
        }
        2 => {
            assert!(source.keywords.contains(Keywords::HASTE));
            assert!(own.iter().any(|v| v.card.name() == "Bear Cub"
                && v.power == 3
                && v.toughness == 3
                && v.keywords.contains(Keywords::HASTE)));
        }
        3 => {
            assert_eq!((source.power, source.toughness), (4, 3));
            assert!(source.keywords.contains(Keywords::FLYING));
        }
        4 => {
            assert_eq!(me.library_count, s.library - 1);
            assert_eq!(o.exile_permissions.len(), s.rights + 1);
        }
        5 => {
            assert_eq!(me.library_count, s.library - 1);
            assert_eq!((source.power, source.toughness), (3, 4));
        }
        6 => {
            assert!(
                source.keywords.contains(Keywords::FLYING)
                    && source.keywords.contains(Keywords::HASTE)
            );
            if !s.pump {
                return false;
            }
            assert_eq!((source.power, source.toughness), (3, 2));
        }
        7 => {
            assert_eq!(me.life, s.life + 3);
            assert!(source.keywords.contains(Keywords::TRAMPLE));
            assert_eq!((source.power, source.toughness), (5, 4));
        }
        8 => {
            assert_eq!((source.power, source.toughness), (5, 6));
            assert!(source.keywords.contains(Keywords::VIGILANCE));
            assert!(own.iter().any(|v| v.card.name() == "Bear Cub"
                && v.power == 3
                && v.toughness == 3
                && v.keywords.contains(Keywords::VIGILANCE)));
        }
        9 => {
            let n = 7 - me.hand_count as i128;
            assert_eq!((source.power, source.toughness), (n, n));
            assert!(n > 0);
            assert!(
                source.keywords.contains(Keywords::FLYING)
                    && source.keywords.contains(Keywords::TRAMPLE)
            );
        }
        10 => {
            if !s.aura_death {
                assert!(own.iter().any(|v| v.card.name() == "Bear Cub"
                    && v.power == 3
                    && v.toughness == 3
                    && v.keywords.contains(Keywords::MENACE)
                    && v.keywords.contains(Keywords::TRAMPLE)));
                assert_eq!(source.attached_to, s.mate);
                return false;
            }
            assert_eq!(source.zone, Zone::Graveyard);
            assert_eq!(me.library_count, s.death_library - 1);
            assert!(o.objects.iter().any(|v| {
                s.mate
                    .is_some_and(|old| v.id == old.card && v.reference.generation > old.generation)
                    && v.zone == Zone::Graveyard
            }));
        }
        11 | 12 => {
            let tokens: Vec<_> = own
                .iter()
                .filter(|v| {
                    v.token.is_some_and(|t| {
                        t.subtypes == if i == 11 { "Zombie Rogue" } else { "Varmint" }
                    })
                })
                .collect();
            assert_eq!(tokens.len(), if i == 11 { 1 } else { s.grave });
            assert!(!tokens.is_empty());
            for v in tokens {
                assert_eq!((v.power, v.toughness), (2, if i == 11 { 2 } else { 1 }));
                assert_eq!(v.token.unwrap().colors, if i == 11 { 6 } else { 16 });
            }
            if i == 11 {
                assert_eq!((source.power, source.toughness), (4, 3));
            }
        }
        _ => unreachable!(),
    }
    if i != 10 {
        assert_eq!(
            source.zone,
            if i == 12 {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
    }
    assert!(
        !o.plotted_cards
            .iter()
            .any(|v| Some(v.object.card) == s.source)
    );
    true
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse::<usize>()?;
    let (mut hard, mut plotted_casts, mut hard_effects, mut plotted_effects, mut plots) = (
        [[0u64; 2]; 13],
        [[0u64; 2]; 13],
        [[0u64; 2]; 13],
        [[0u64; 2]; 13],
        [[0u64; 2]; 13],
    );
    let (mut decisions, mut replays, mut frames, mut closed, mut rejected) =
        (0u64, 0u64, 0u64, 0u64, 0u64);
    let (mut initial, mut maximum) = (0, 0);
    for seed in 0..games {
        let i = seed % 13;
        let p = if (seed / 13) % 2 == 0 {
            Player::P0
        } else {
            Player::P1
        };
        let plotted = (seed / 26) % 2 == 1;
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
            if s.paid && !s.done && quiet && verify(&o, i, p, &s) {
                s.done = true;
                if plotted {
                    plotted_effects[i][p.index()] += 1;
                } else {
                    hard_effects[i][p.index()] += 1;
                }
            }
            let actions = g.legal_actions();
            let available = actions.iter().any(|a| match a {
                Action::AnnounceCast { card } => {
                    !plotted
                        && o.objects
                            .iter()
                            .any(|v| v.id == *card && v.card.name() == NAMES[i])
                }
                Action::AnnouncePlotted { card } => plotted && Some(*card) == s.source,
                _ => false,
            });
            let (index, action) = actions
                .iter()
                .enumerate()
                .max_by_key(|(_, a)| score(&o, a, i, p, &s, plotted, available))
                .unwrap();
            assert!(
                score(&o, action, i, p, &s, plotted, available) >= 0,
                "unhandled {} seed{seed} step{step} {} {actions:?}",
                NAMES[i],
                o.decision
            );
            let action = action.clone();
            if let Action::AnnouncePlot { card } = action {
                s.source = Some(card);
            }
            if let Action::AnnounceCast { card } | Action::AnnouncePlotted { card } = action
                && actor == p
                && o.objects
                    .iter()
                    .any(|v| v.id == card && v.card.name() == NAMES[i])
                && !s.paid
            {
                s.source = Some(card);
            }
            if let Action::ChooseTarget {
                target: Target::Card(id),
            } = action
                && actor == p
                && i == 10
                && !s.paid
                && o.objects.iter().any(|v| {
                    v.id == id
                        && v.owner == p
                        && v.zone == Zone::Battlefield
                        && v.card.name() == "Bear Cub"
                })
            {
                s.mate = Some(o.objects.iter().find(|v| v.id == id).unwrap().reference);
            }
            if matches!(action, Action::PayCosts { .. })
                && let Some(a) = &o.announcing
                && a.kind == StackKind::Spell
                && a.controller == p
                && !s.done
            {
                let n = a.source_card.name();
                if n == NAMES[i] && !s.paid {
                    s.paid = true;
                    s.library = o.players[p.index()].library_count;
                    s.life = o.players[p.index()].life;
                    s.rights = o.exile_permissions.len();
                    s.grave = o
                        .objects
                        .iter()
                        .filter(|v| {
                            v.owner == p
                                && v.zone == Zone::Graveyard
                                && !v.card.is_token()
                                && v.card.is_creature()
                        })
                        .count();
                    if plotted {
                        plotted_casts[i][p.index()] += 1;
                    } else {
                        hard[i][p.index()] += 1;
                    }
                }
                if n == "Bear Cub" && i == 5 {
                    s.loan_turn = Some(o.turn);
                }
                if n == "Shock" && [1, 11, 12].contains(&i) {
                    s.death_turn = Some(o.turn);
                }
                if n == "Shock" && i == 6 {
                    s.pump = true;
                }
                if n == "Scorching Shot" && i == 10 {
                    s.aura_death = true;
                    s.death_library = o.players[p.index()].library_count;
                }
            }
            let frame = o.plot_payment;
            if frame.is_some() {
                frames += 1;
                assert_eq!(actor, p);
                assert!(g.observe(p.other()).plot_payment.is_none());
                assert!(o.stack.is_empty());
                let before = g.snapshot();
                let rev = g.revision();
                assert!(g.apply(p.other(), action.clone()).is_err());
                assert_eq!(g.snapshot(), before);
                assert_eq!(g.revision(), rev);
                assert_eq!(g.apply(p, Action::Pass), Err(Error::IllegalAction));
                assert_eq!(g.snapshot(), before);
                assert_eq!(g.revision(), rev);
                rejected += 2;
            }
            let saved = g.snapshot();
            let rev = g.revision();
            let mut fork = g.clone();
            fork.apply(actor, action.clone())?;
            let expected = fork.snapshot();
            let child = if frame.is_some() || !o.plotted_cards.is_empty() {
                let parent = g.clone();
                let child = parent.clone();
                drop(parent);
                Some(child)
            } else {
                None
            };
            g.apply_index(actor, rev, index)?;
            assert_eq!(g.snapshot(), expected);
            if let Some(mut child) = child {
                child.apply(actor, action.clone())?;
                assert_eq!(child.snapshot(), expected);
                closed += 1;
            }
            for viewer in [Player::P0, Player::P1] {
                assert_eq!(
                    serde_json::to_value(fork.observe(viewer))?,
                    serde_json::to_value(g.observe(viewer))?
                );
            }
            g.restore(&saved)?;
            assert_eq!(g.apply_index(actor, rev, index), Err(Error::StaleRevision));
            rejected += 1;
            g.apply(actor, action.clone())?;
            assert_eq!(g.snapshot(), expected);
            decisions += 1;
            replays += 1;
            maximum = maximum.max(g.state_bytes());
            if let Action::PayPlot { payment } = action {
                let f = frame.unwrap();
                let after = g.observe(p);
                s.plotted_turn = Some(o.turn);
                plots[i][p.index()] += 1;
                assert_eq!(after.actor, Some(p));
                assert_eq!(
                    after.players[p.index()].spells_cast_this_turn,
                    o.players[p.index()].spells_cast_this_turn
                );
                assert!(after.stack.is_empty());
                assert_eq!(after.plotted_cards, g.observe(p.other()).plotted_cards);
                let permission = after
                    .plotted_cards
                    .iter()
                    .find(|v| Some(v.object.card) == s.source)
                    .unwrap();
                assert_eq!(permission.object.generation, f.object.generation + 1);
                assert_eq!(permission.plotted_turn, o.turn);
                assert!(
                    after
                        .objects
                        .iter()
                        .any(|v| v.reference == permission.object
                            && v.zone == Zone::Exile
                            && v.card == f.source_card)
                );
                for (j, n) in payment.iter().enumerate() {
                    assert_eq!(
                        after.players[p.index()].mana[j],
                        o.players[p.index()].mana[j] - n
                    );
                }
            }
            if matches!(action, Action::PayCosts { .. })
                && let Some(a) = &o.announcing
                && a.kind == StackKind::Spell
                && a.controller == p
                && a.source().is_some_and(|source| source.card.name() == NAMES[i])
                && !s.done
            {
                let after = g.observe(p);
                let spell = after
                    .stack
                    .iter()
                    .find(|v| v.card == a.card && v.kind == StackKind::Spell)
                    .unwrap();
                let cast = serde_json::to_value(spell.cast)?;
                assert_eq!(cast["method"], if plotted { "Plot" } else { "Normal" });
                assert_eq!(cast["origin"], if plotted { "Exile" } else { "Hand" });
                if plotted {
                    assert!(o.turn > s.plotted_turn.unwrap());
                }
            }
        }
        if !s.done {
            let o = g.observe(p);
            eprintln!(
                "failed seed={seed} source={:?} plotted_turn={:?} hand={:?} battlefield={:?} rights={:?}",
                s.source,
                s.plotted_turn,
                o.objects
                    .iter()
                    .filter(|v| v.owner == p && v.zone == Zone::Hand)
                    .map(|v| v.card.name())
                    .collect::<Vec<_>>(),
                o.objects
                    .iter()
                    .filter(|v| v.owner == p && v.zone == Zone::Battlefield)
                    .map(|v| (v.card.name(), v.power, v.toughness))
                    .collect::<Vec<_>>(),
                o.plotted_cards
            );
        }
        assert!(
            s.done && g.outcome().is_some(),
            "incomplete {} seed{seed} plotted{plotted} paid{} done{} turn{}",
            NAMES[i],
            s.paid,
            s.done,
            g.observe(p).turn
        );
    }
    if games >= 100 {
        for a in [
            &hard,
            &plotted_casts,
            &hard_effects,
            &plotted_effects,
            &plots,
        ] {
            assert!(a.iter().flatten().all(|n| *n > 0));
        }
        assert!(frames > 0 && closed > 0);
    }
    println!(
        "games={games} decisions={decisions} replays={replays} frames={frames} parent_closed={closed} rejected={rejected} initial_bytes={initial} max_bytes={maximum} hard={hard:?} plotted={plotted_casts:?} plots={plots:?} hard_effects={hard_effects:?} plotted_effects={plotted_effects:?}"
    );
    Ok(())
}
