//! Reproducible legal Standard Mobilize games and independent public effect ledgers.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Keywords, ObjectRef, Observation, Player,
    StackKind, Target, Zone,
};
const NAMES: [&str; 8] = [
    "Avenger of the Fallen",
    "Dalkovan Packbeasts",
    "Dragonback Lancer",
    "Nightblade Brigade",
    "Reigning Victor",
    "Shock Brigade",
    "Stadium Headliner",
    "Zurgo's Vanguard",
];
fn deck(i: usize, focal: bool) -> Vec<Card> {
    let mut d = Vec::new();
    for (n, k) in [("Plains", 8), ("Mountain", 8), ("Swamp", 8), ("Forest", 4)] {
        d.extend(vec![Card::from_name(n).unwrap(); k]);
    }
    for n in [
        if focal { NAMES[i] } else { "Llanowar Elves" },
        "Savannah Lions",
        "Bear Cub",
        "Thrill of Possibility",
        "Shock",
        "Cartographer's Companion",
        "Resolute Reinforcements",
        "Novice Inspector",
    ] {
        d.extend([Card::from_name(n).unwrap(); 4]);
    }
    assert_eq!(d.len(), 60);
    d
}
#[derive(Default)]
struct Pilot {
    paid: bool,
    source: Option<u32>,
    attacked: bool,
    tokens: Vec<ObjectRef>,
    queued: bool,
    sacrificed: bool,
    done: bool,
    surveil: bool,
    victor: bool,
    activation: bool,
    damage_done: bool,
    target: Option<ObjectRef>,
    expected_damage: i128,
}
fn score(o: &Observation, a: &Action, i: usize, p: Player, s: &Pilot) -> i32 {
    let actor = o.actor.unwrap();
    let obj = |id| o.objects.iter().find(|v| v.id == id).unwrap();
    let grave = o
        .objects
        .iter()
        .any(|v| v.owner == p && v.zone == Zone::Graveyard && v.card.is_creature());
    let friend = o
        .objects
        .iter()
        .filter(|v| {
            v.owner == p
                && v.zone == Zone::Battlefield
                && v.card.is_creature()
                && Some(v.id) != s.source
        })
        .count();
    match a {
        Action::AnnounceCast { card }
        | Action::Cast { card, .. }
        | Action::BeginCast { card, .. } => {
            let v = obj(*card);
            let n = v.card.name();
            if actor == p && n == NAMES[i] {
                return if !s.paid
                    && (i != 0 || grave)
                    && (i != 6 || friend > 0)
                    && matches!(a, Action::AnnounceCast { .. })
                {
                    2000
                } else if s.done {
                    700
                } else {
                    -1000
                };
            }
            if n == "Thrill of Possibility" {
                return if actor == p && i == 0 && !grave {
                    1800
                } else {
                    -1000
                };
            }
            if n == "Shock" {
                return if s.done { 800 } else { -1000 };
            }
            if v.card.is_creature() {
                if s.done {
                    900
                } else if actor == p && friend < 2 {
                    1000
                } else if actor != p
                    && o.objects
                        .iter()
                        .filter(|v| {
                            v.owner == actor && v.zone == Zone::Battlefield && v.card.is_creature()
                        })
                        .count()
                        < 3
                {
                    900
                } else {
                    -1000
                }
            } else {
                -1000
            }
        }
        Action::AnnounceActivate {
            card, ability: 0, ..
        } if actor == p
            && i == 6
            && Some(*card) == s.source
            && !s.tokens.is_empty()
            && !s.activation
            && o.objects
                .iter()
                .any(|v| v.owner != p && v.zone == Zone::Battlefield && v.card.is_creature()) =>
        {
            2200
        }
        Action::ChooseTarget {
            target: Target::Card(id),
        } => {
            let v = obj(*id);
            if i == 4
                && o.announcing.as_ref().is_some_and(|a| {
                    a.source()
                        .is_some_and(|source| source.card.name() == NAMES[i])
                })
            {
                if Some(*id) == s.source { 2200 } else { 1000 }
            } else if v.owner != actor && v.card.is_creature() {
                2000
            } else {
                500
            }
        }
        Action::ChooseTarget {
            target: Target::Player(t),
        } => {
            if *t == actor.other() {
                1500
            } else {
                -1000
            }
        }
        Action::ChooseSpellCost { cost } => 2100 - i32::from(*cost),
        Action::ChooseAdditionalCost { object } => {
            if obj(object.card).card.is_creature() {
                2100
            } else {
                1000
            }
        }
        Action::PayCosts { .. } => 2100,
        Action::TapMana { .. } => {
            if o.decision == "pay_costs" {
                2000
            } else {
                -1000
            }
        }
        Action::Attack { card } => {
            if s.done || (actor == p && Some(*card) == s.source && !s.attacked) {
                1500
            } else {
                -1000
            }
        }
        Action::PlayLand { card } => {
            1200 - 20
                * o.objects
                    .iter()
                    .filter(|v| {
                        v.owner == actor && v.zone == Zone::Battlefield && v.card == obj(*card).card
                    })
                    .count() as i32
        }
        Action::ArrangeLibrary {
            destination: rustmage::LibraryDestination::Graveyard,
            ..
        } => 2100,
        Action::AssignDamage { amount, .. } => 1000 + (*amount).min(100) as i32,
        Action::Keep => 2500,
        Action::Mulligan | Action::Concede => -10000,
        Action::Pass => 0,
        Action::FinishAttackers | Action::FinishBlockers => 1000,
        Action::PutTrigger { .. }
        | Action::Discard { .. }
        | Action::ArrangeLibrary { .. }
        | Action::FinishTargets => 1800,
        _ => -1000,
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse::<usize>()?;
    let (mut casts, mut effects, mut delayed) = ([[0u64; 2]; 8], [[0u64; 2]; 8], [[0u64; 2]; 8]);
    let (mut decisions, mut replays, mut tokens, mut rejects, mut stadium) =
        (0u64, 0u64, 0u64, 0u64, 0u64);
    let (mut initial, mut maximum) = (0, 0);
    for seed in 0..games {
        let i = seed % 8;
        let p = if (seed / 8) % 2 == 0 {
            Player::P0
        } else {
            Player::P1
        };
        let a = deck(i, true);
        let b = deck(i, false);
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
        while g.outcome().is_none() {
            let actor = g.actor().unwrap();
            let o = g.observe(actor);
            let quiet = o.stack.is_empty()
                && o.announcing.is_none()
                && o.resolving.is_none()
                && o.decision == "priority";
            if s.paid && !s.attacked && quiet {
                let v = o
                    .objects
                    .iter()
                    .find(|v| Some(v.id) == s.source && v.zone == Zone::Battlefield)
                    .unwrap();
                match i {
                    0 | 3 => assert!(v.keywords.contains(Keywords::DEATHTOUCH)),
                    1 => assert!(v.keywords.contains(Keywords::VIGILANCE)),
                    2 => assert!(v.keywords.contains(Keywords::FLYING)),
                    5 => assert!(v.keywords.contains(Keywords::MENACE)),
                    _ => {}
                }
                if i == 3 {
                    assert!(s.surveil);
                }
                if i == 4 {
                    assert!(s.victor);
                }
            }
            if s.queued && quiet && !s.sacrificed {
                assert!(o.delayed_sacrifices.is_empty());
                assert!(s.tokens.iter().all(|r| {
                    o.objects
                        .iter()
                        .all(|v| v.reference != *r || v.zone != Zone::Battlefield)
                }));
                assert!(o.creatures_died_this_turn >= s.tokens.len() as u64);
                s.sacrificed = true;
            }
            if s.sacrificed && !s.done && (i != 6 || s.damage_done) {
                s.done = true;
                effects[i][p.index()] += 1;
            }
            let actions = g.legal_actions();
            let (index, action) = actions
                .iter()
                .enumerate()
                .max_by_key(|(_, a)| score(&o, a, i, p, &s))
                .unwrap();
            assert!(
                score(&o, action, i, p, &s) >= 0,
                "unhandled {} {} {actions:?}",
                NAMES[i],
                o.decision
            );
            let action = action.clone();
            if matches!(action, Action::ArrangeLibrary { .. }) && i == 3 && s.paid {
                s.surveil = true;
            }
            if let Action::Attack { card } = action
                && Some(card) == s.source
            {
                s.attacked = true;
            }
            if matches!(action, Action::PayCosts { .. })
                && let Some(ann) = &o.announcing
            {
                if ann
                    .source()
                    .is_some_and(|source| source.card.name() == NAMES[i])
                    && ann.kind == StackKind::Spell
                    && !s.paid
                {
                    s.paid = true;
                    s.source = Some(
                        ann.source()
                            .expect("example expects an object-backed packet")
                            .id,
                    );
                    casts[i][p.index()] += 1;
                    if i == 4 {
                        assert_eq!(ann.mana_value(), 6);
                    }
                }
                if i == 6
                    && ann
                        .source()
                        .is_some_and(|source| source.card.name() == NAMES[i])
                    && matches!(ann.kind, StackKind::ActivatedAbility { .. })
                {
                    s.activation = true;
                    stadium += 1;
                    s.target = ann.targets.iter().find_map(|t| {
                        if let Target::Card(id) = t.target {
                            o.objects.iter().find(|v| v.id == id).map(|v| v.reference)
                        } else {
                            None
                        }
                    });
                    s.expected_damage = o
                        .objects
                        .iter()
                        .filter(|v| {
                            v.owner == p
                                && v.zone == Zone::Battlefield
                                && v.card.is_creature()
                                && Some(v.id) != s.source
                        })
                        .count() as i128;
                }
            }
            let saved = g.snapshot();
            let rev = g.revision();
            let mut f = g.clone();
            f.apply(actor, action.clone())?;
            g.apply_index(actor, rev, index)?;
            let expected = g.snapshot();
            assert_eq!(f.snapshot(), expected);
            g.restore(&saved)?;
            assert_eq!(g.apply_index(actor, rev, index), Err(Error::StaleRevision));
            g.apply(actor, action)?;
            assert_eq!(g.snapshot(), expected);
            decisions += 1;
            replays += 1;
            rejects += 1;
            maximum = maximum.max(g.state_bytes());
            let after = g.observe(p);
            if i == 4
                && s.paid
                && after.objects.iter().any(|v| {
                    Some(v.id) == s.source
                        && v.zone == Zone::Battlefield
                        && v.keywords.contains(Keywords::INDESTRUCTIBLE)
                })
            {
                s.victor = true;
            }
            if s.tokens.is_empty()
                && let Some(batch) = after.delayed_sacrifices.first()
            {
                assert_eq!(Some(batch.source.card), s.source);
                let amount = if i == 1 {
                    3
                } else if i == 0 {
                    o.objects
                        .iter()
                        .filter(|v| {
                            v.owner == p && v.zone == Zone::Graveyard && v.card.is_creature()
                        })
                        .count()
                } else {
                    1
                };
                assert_eq!(batch.tokens.len(), amount);
                assert!(amount > 0);
                assert_eq!(
                    after.creatures_attacked_this_turn,
                    o.creatures_attacked_this_turn
                );
                assert_eq!(
                    after.players[p.index()].nonland_permanents_entered_this_turn,
                    o.players[p.index()].nonland_permanents_entered_this_turn + amount as u64
                );
                for r in &batch.tokens {
                    let v = after.objects.iter().find(|v| v.reference == *r).unwrap();
                    assert!(v.tapped && v.attacking);
                    assert_eq!((v.power, v.toughness), (1, 1));
                    let t = v.token.unwrap();
                    assert_eq!((t.colors, t.subtypes), (8, "Warrior"));
                }
                if i == 1 {
                    assert!(
                        !after
                            .objects
                            .iter()
                            .find(|v| Some(v.id) == s.source)
                            .unwrap()
                            .tapped
                    );
                }
                if i == 7 {
                    assert_eq!(
                        after
                            .objects
                            .iter()
                            .find(|v| Some(v.id) == s.source)
                            .unwrap()
                            .power,
                        after
                            .objects
                            .iter()
                            .filter(|v| v.owner == p
                                && v.zone == Zone::Battlefield
                                && v.card.is_creature())
                            .count() as i128
                    );
                }
                tokens += amount as u64;
                s.tokens = batch.tokens.clone();
            }
            if !s.queued
                && after
                    .stack
                    .iter()
                    .any(|a| a.kind == StackKind::DelayedSacrifice)
            {
                assert!(after.delayed_sacrifices.is_empty());
                s.queued = true;
                delayed[i][p.index()] += 1;
            }
            if s.activation
                && !s.damage_done
                && after.stack.iter().all(|a| {
                    !matches!(a.kind, StackKind::ActivatedAbility { .. })
                        || a.source()
                            .is_none_or(|source| source.card.name() != NAMES[i])
                })
                && let Some(r) = s.target
            {
                let before = o.objects.iter().find(|v| v.reference == r);
                let target = after.objects.iter().find(|v| v.reference == r);
                if let Some(b) = before {
                    assert!(
                        target.is_none_or(|v| v.zone == Zone::Graveyard
                            || v.damage >= b.damage + s.expected_damage)
                    );
                    s.damage_done = true;
                }
            }
        }
        assert!(s.done, "incomplete {} seed{seed}", NAMES[i]);
    }
    if games >= 100 {
        assert!(casts.iter().flatten().all(|v| *v > 0));
        assert!(effects.iter().flatten().all(|v| *v > 0));
        assert!(delayed.iter().flatten().all(|v| *v > 0));
        assert!(stadium > 0);
    }
    println!(
        "games={games} decisions={decisions} replays={replays} stale_rejections={rejects} tokens={tokens} stadium={stadium} initial_bytes={initial} max_bytes={maximum} casts={casts:?} effects={effects:?} delayed={delayed:?}"
    );
    Ok(())
}
