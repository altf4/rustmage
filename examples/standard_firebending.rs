//! Full Standard games exercising combat-retained red mana, costs and native replay.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Observation, Player, StackKind, Zone,
};
use std::collections::BTreeMap;
const NAMES: [&str; 8] = [
    "Azula, On the Hunt",
    "Fire Sages",
    "Namazu Trader",
    "Rough Rhino Cavalry",
    "Sun Warriors",
    "Uncle Iroh",
    "Vindictive Warden",
    "Zhao, Ruthless Admiral",
];
fn index(c: Card) -> Option<usize> {
    NAMES.iter().position(|n| *n == c.name())
}
fn deck(family: usize) -> Vec<Card> {
    let (lands, spells) = if family == 0 {
        (
            vec![("Swamp", 10), ("Mountain", 14)],
            vec![
                NAMES[0],
                NAMES[1],
                NAMES[2],
                NAMES[3],
                NAMES[6],
                NAMES[7],
                "Shock",
                "Reckless Lackey",
                "Seize the Spoils",
            ],
        )
    } else {
        (
            vec![("Mountain", 10), ("Plains", 6), ("Forest", 8)],
            vec![
                NAMES[4],
                NAMES[5],
                NAMES[1],
                NAMES[3],
                "Shared Roots",
                "Bear Cub",
                "Shock",
                "Reckless Lackey",
                "Llanowar Elves",
            ],
        )
    };
    let mut d = Vec::new();
    for (n, amount) in lands {
        let c = Card::from_name(n).unwrap();
        assert!(c.is_supported() && c.is_standard_legal());
        d.extend(vec![c; amount]);
    }
    for n in spells {
        let c = Card::from_name(n).unwrap();
        assert!(c.is_supported() && c.is_standard_legal(), "{n}");
        d.extend([c; 4]);
    }
    assert_eq!(d.len(), 60);
    d
}
fn choose(g: &Game, o: &Observation, actions: &[Action], _: bool) -> usize {
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
                if o.stack.is_empty()
                    && o.players[o.actor.unwrap().index()].combat_red_mana > 0
                    && (b.token.is_some() || index(b.card).is_some())
                {
                    240
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
            Action::ChooseAdditionalCost { object } | Action::Sacrifice { object } => {
                let b = o.objects.iter().find(|b| b.reference == *object).unwrap();
                if b.token.is_some() {
                    260
                } else if b.card.is_land() {
                    220
                } else {
                    200 - b.power.clamp(0, 100) as i32
                }
            }
            Action::ChooseTarget {
                target: rustmage::Target::Card(id),
            } => {
                let b = o.objects.iter().find(|b| b.id == *id).unwrap();
                if b.owner == o.actor.unwrap().other() {
                    240
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
            Action::Attack { card } => {
                let b = o.objects.iter().find(|b| b.id == *card).unwrap();
                if index(b.card).is_some() { 200 } else { 180 }
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
    let (mut casts, mut triggers, mut resolutions, mut activations) =
        ([0; 8], [0; 8], [0; 8], [0; 8]);
    let (mut received, mut spent, mut expiries, mut retained_checks) = (0u64, 0u64, 0u64, 0u64);
    let mut ending = 0u64;
    let mut other_triggers = [0u64; 3];
    let mut lesson_discounts = 0u64;
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
                || before
                    .stack
                    .iter()
                    .any(|s| s.source().and_then(|source| index(source.card)).is_some());
            if matches!(action, Action::PayCosts { .. })
                && before.announcing.as_ref().is_some_and(|s| {
                    s.source()
                        .is_some_and(|source| source.card.name() == "Shared Roots")
                })
                && before.objects.iter().any(|o| {
                    o.zone == Zone::Battlefield && o.owner == p && o.card.name() == "Uncle Iroh"
                })
            {
                lesson_discounts += 1;
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
                let Some(i) = s.source().and_then(|source| index(source.card)) else {
                    continue;
                };
                if s.kind == StackKind::Spell {
                    casts[i] += 1;
                } else if matches!(s.kind, StackKind::ActivatedAbility { .. }) {
                    activations[i] += 1;
                } else if matches!(s.kind, StackKind::TriggeredAbility { .. }) {
                    assert!(!tracked.contains_key(&s.id));
                    tracked.insert(s.id, i);
                    triggers[i] += 1;
                    if s.kind == (StackKind::TriggeredAbility { ability: 1 })
                        && let Some(j) = [0, 2, 7].iter().position(|k| *k == i)
                    {
                        other_triggers[j] += 1;
                    }
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
                let old = before.players[i].combat_red_mana;
                let new = after.players[i].combat_red_mana;
                assert!(new <= after.players[i].mana[3]);
                assert_eq!(new, public.players[i].combat_red_mana);
                assert_eq!(after.players[i].mana, public.players[i].mana);
                if new > old {
                    received += u64::from(new - old);
                }
                if new < old {
                    if before.step != after.step && after.step == rustmage::Step::Main2 {
                        expiries += u64::from(old - new);
                    } else {
                        spent += u64::from(old - new);
                    }
                }
                if !matches!(
                    after.step,
                    rustmage::Step::BeginCombat
                        | rustmage::Step::Attackers
                        | rustmage::Step::Blockers
                        | rustmage::Step::FirstStrikeDamage
                        | rustmage::Step::CombatDamage
                        | rustmage::Step::EndCombat
                ) {
                    assert_eq!(new, 0);
                }
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
            retained_checks += 1;
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
        ending += g
            .observe(Player::P0)
            .players
            .iter()
            .map(|p| u64::from(p.combat_red_mana))
            .sum::<u64>();
    }
    if games >= 100 {
        assert!(casts.iter().all(|n| *n > 0), "casts {casts:?}");
        assert!(triggers.iter().all(|n| *n > 0), "triggers {triggers:?}");
        assert!(
            resolutions.iter().all(|n| *n > 0),
            "resolutions {resolutions:?}"
        );
        for i in [1, 3, 4, 6] {
            assert!(activations[i] > 0, "activation {i}: {activations:?}");
        }
        assert!(other_triggers.iter().all(|n| *n > 0));
        assert!(lesson_discounts > 0);
        assert!(received > 0 && spent > 0 && expiries > 0 && private > 0);
    }
    assert_eq!(received, spent + expiries + ending);
    println!(
        "Completed {games} Standard firebending games, {decisions} decisions, casts {casts:?}, triggers {triggers:?}, finished triggers {resolutions:?}, activations {activations:?}, other attack/sacrifice triggers {other_triggers:?}, {lesson_discounts} discounted Lessons, retained red received/spent/expired/held at game end {received}/{spent}/{expiries}/{ending}, {retained_checks} public subset checks, {history_checks} creature-history checks, {private} private choices, {replays} replay boundaries, {max_bytes} maximum initialized logical bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
