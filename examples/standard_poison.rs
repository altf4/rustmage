//! Full Standard games exercising persistent poison, alternate losses and owned replay.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Observation, Outcome, Player, StackKind, Zone,
};
use std::collections::BTreeMap;
const NAMES: [&str; 4] = [
    "Fynn, the Fangbearer",
    "Persuasive Interrogators",
    "The Serpent Society",
    "Virulent Silencer",
];
fn index(c: Card) -> Option<usize> {
    NAMES.iter().position(|n| *n == c.name())
}
fn deck(family: usize) -> Vec<Card> {
    let mut d = Vec::new();
    for (n, amount) in if family == 0 {
        vec![("Swamp", 12), ("Forest", 12)]
    } else {
        vec![("Swamp", 10), ("Forest", 8), ("Island", 6)]
    } {
        d.extend(vec![Card::from_name(n).unwrap(); amount]);
    }
    let extra = if family == 0 {
        vec![
            "Pirate Peddlers",
            "Vampiric Rites",
            "Bear Cub",
            "Mephitic Draught",
            "Fell",
        ]
    } else {
        vec!["Deduce", "Negate", "Bear Cub", "Mephitic Draught", "Fell"]
    };
    for n in NAMES.into_iter().chain(extra) {
        let c = Card::from_name(n).unwrap();
        assert!(c.is_supported() && c.is_standard_legal());
        d.extend([c; 4]);
    }
    assert_eq!(d.len(), 60);
    d
}
fn choose(g: &Game, o: &Observation, actions: &[Action], accept_ward: bool) -> usize {
    let fallback = rustmage::policy::smoke_index(g, actions);
    actions
        .iter()
        .enumerate()
        .max_by_key(|(i, a)| match a {
            Action::Concede => -1000,
            Action::PlayLand { .. } => 250,
            Action::Cast { .. } | Action::AnnounceCast { .. } => 230,
            Action::AnnounceActivate { card, .. } => {
                let b = o.objects.iter().find(|b| b.id == *card).unwrap();
                if !o.stack.is_empty() {
                    -100
                } else if b.token.is_some() || matches!(b.card.name(), "Vampiric Rites") {
                    220
                } else {
                    -100
                }
            }
            Action::Activate { .. }
            | Action::BeginActivate { .. }
            | Action::AnnounceMana { .. } => -100,
            Action::TapMana { .. } | Action::ActivateMana { .. } => {
                if o.decision == "pay_costs" {
                    180
                } else {
                    -100
                }
            }
            Action::ChooseAdditionalCost { object } => {
                let b = o.objects.iter().find(|b| b.reference == *object).unwrap();
                if b.keywords.contains(rustmage::Keywords::DEATHTOUCH)
                    && o.objects.iter().any(|o| {
                        o.owner == b.owner
                            && o.zone == Zone::Battlefield
                            && o.card.name() == "The Serpent Society"
                    })
                {
                    250
                } else if b.token.is_some() {
                    240
                } else if b.card.name() == "Bear Cub" {
                    230
                } else {
                    200 - b.power.clamp(0, 100) as i32
                }
            }
            Action::ChooseTarget {
                target: rustmage::Target::Card(id),
            } => {
                let b = o.objects.iter().find(|b| b.id == *id).unwrap();
                if b.owner == o.actor.unwrap().other() {
                    if b.card.name() == "The Serpent Society" {
                        260
                    } else {
                        240
                    }
                } else {
                    -100
                }
            }
            Action::ChooseOptional { accept }
                if o.resolving.as_ref().is_some_and(|s| {
                    s.source().is_some_and(|source| source.card.name() == "The Serpent Society")
                        && s.kind == (StackKind::TriggeredAbility { ability: 0 })
                }) =>
            {
                if *accept == accept_ward {
                    260
                } else {
                    -100
                }
            }
            Action::ChooseTarget {
                target: rustmage::Target::Player(p),
            } => {
                if *p == o.actor.unwrap().other() {
                    240
                } else {
                    -100
                }
            }
            Action::Sacrifice { .. } => 220,
            Action::Attack { card } => {
                let b = o.objects.iter().find(|b| b.id == *card).unwrap();
                if matches!(b.card.name(), "Fynn, the Fangbearer" | "Virulent Silencer") {
                    180
                } else {
                    -100
                }
            }
            _ => {
                if *i == fallback {
                    150
                } else {
                    0
                }
            }
        })
        .unwrap()
        .0
}
fn live(o: &Observation, id: u64) -> bool {
    o.stack.iter().any(|s| s.id == id)
        || o.resolving.as_ref().is_some_and(|s| s.id == id)
        || o.announcing.as_ref().is_some_and(|s| s.id == id)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse()?;
    let decks = [deck(0), deck(1)];
    let (mut decisions, mut replays, mut private, mut history_checks, mut max_bytes) =
        (0, 0, 0, 0, 0);
    let (mut casts, mut triggers, mut resolutions) = ([0; 4], [0; 4], [0; 4]);
    let (mut ward_payments, mut poison_wins, mut poison_checks) = ([0; 2], 0, 0);
    let mut positive_life_poison_wins = 0;
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [decks[seed % 2].clone(), decks[seed % 2].clone()],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        let mut tracked = BTreeMap::new();

        for _ in 0..100_000 {
            if g.outcome().is_some() {
                break;
            }
            let p = g.actor().unwrap();
            let before = g.observe(p);
            let actions = g.legal_actions();
            let action = actions[choose(&g, &before, &actions, seed % 2 == 0)].clone();
            if before.library_choice.is_some() || before.search_choice.is_some() {
                let other = g.observe(p.other());
                assert!(
                    other.library_choice.is_none()
                        && other.search_choice.is_none()
                        && other.actions.is_empty()
                );
                private += 1;
            }
            let boundary = before.announcing.is_some()
                || before.resolving.is_some()
                || before.stack.iter().any(|s| index(s.source_card).is_some());
            if let Action::ChooseOptional { accept } = action
                && before.resolving.as_ref().is_some_and(|s| {
                    s.source().is_some_and(|source| source.card.name() == "The Serpent Society")
                        && s.kind == (StackKind::TriggeredAbility { ability: 0 })
                })
            {
                ward_payments[usize::from(accept)] += 1;
            }
            let saved = g.snapshot();
            let mut fork = boundary.then(|| g.clone());
            g.apply(p, action.clone())?;
            if let Some(f) = &mut fork {
                f.apply(p, action.clone())?;
            }
            let after = g.observe(p);
            for s in &after.stack {
                if before.stack.iter().any(|old| old.id == s.id)
                    || before.resolving.as_ref().is_some_and(|old| old.id == s.id)
                {
                    continue;
                }
                let Some(i) = index(s.source_card) else {
                    continue;
                };
                if s.kind == StackKind::Spell {
                    casts[i] += 1;
                } else if s.kind
                    == (StackKind::TriggeredAbility {
                        ability: if matches!(i, 1 | 2) { 1 } else { 0 },
                    })
                {
                    assert!(!tracked.contains_key(&s.id));
                    tracked.insert(s.id, i);
                    triggers[i] += 1;
                }
            }
            let completed: Vec<_> = tracked
                .iter()
                .filter(|(id, _)| !live(&after, **id))
                .map(|(&id, &i)| (id, i))
                .collect();
            for (id, i) in completed {
                tracked.remove(&id);
                resolutions[i] += 1;
            }
            let public = g.observe(p.other());
            for i in 0..2 {
                assert_eq!(
                    public.players[i].poison_counters,
                    after.players[i].poison_counters
                );
                assert!(
                    after.players[i].poison_counters >= before.players[i].poison_counters,
                    "poison reset or removal"
                );
            }
            for o in public
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Hand && o.owner == p)
            {
                assert!(
                    public.known_hand.contains(&o.reference),
                    "private hand leaked"
                );
            }
            assert_eq!(after.schema, rustmage::OBSERVATION_SCHEMA);
            poison_checks += 1;
            if before.turn == after.turn {
                // This deck has no exile/bounce or zero-toughness incoming
                // creatures. Departed creature tokens cease before observation.
                let died = before
                    .objects
                    .iter()
                    .filter(|b| b.zone == Zone::Battlefield && b.card.is_creature())
                    .filter(|b| match after.objects.iter().find(|o| o.id == b.id) {
                        Some(o) => o.zone == Zone::Graveyard && o.reference != b.reference,
                        None => b.token.is_some(),
                    })
                    .count() as u64;
                assert_eq!(
                    after.creatures_died_this_turn,
                    before.creatures_died_this_turn + died
                );
                history_checks += 1;
            }
            if let Some(f) = fork {
                let expected = g.snapshot();
                assert_eq!(f.snapshot(), expected);
                g.restore(&saved)?;
                assert_eq!(
                    g.apply_index(p, before.revision, 0),
                    Err(Error::StaleRevision)
                );
                g.apply(p, action)?;
                assert_eq!(g.snapshot(), expected);
                replays += 1;
            }
            decisions += 1;
            max_bytes = max_bytes.max(g.state_bytes());
        }
        assert!(g.outcome().is_some(), "game {seed} unfinished");
        let end = g.observe(Player::P0);
        if let Some(loser) = end.players.iter().position(|p| p.poison_counters >= 10) {
            poison_wins += 1;
            let winner = if loser == 0 { Player::P1 } else { Player::P0 };
            assert_eq!(end.outcome, Some(Outcome::Winner(winner)));
            if end.players[loser].life > 0 {
                positive_life_poison_wins += 1;
            }
        }
    }
    if games >= 100 {
        assert!(casts.iter().all(|n| *n > 0), "casts {casts:?}");
        assert!(triggers.iter().all(|n| *n > 0), "triggers {triggers:?}");
        assert!(
            resolutions.iter().all(|n| *n > 0),
            "resolutions {resolutions:?}"
        );
        assert!(
            poison_wins > 0,
            "no poison-ended games; ward {ward_payments:?}"
        );
        assert!(ward_payments.iter().all(|n| *n > 0));
        assert!(positive_life_poison_wins > 0);
    }
    println!(
        "Completed {games} Standard poison games, {decisions} decisions, casts {casts:?}, poison/death triggers {triggers:?}, finished triggers {resolutions:?}, ward declined/paid {ward_payments:?}, {poison_wins} poison-ended games ({positive_life_poison_wins} with positive losing life), {poison_checks} public persistent-poison checks, {history_checks} creature-history checks, {private} private choices, {replays} replay boundaries, {max_bytes} maximum initialized logical bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
