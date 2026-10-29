//! Exact two-brid choices and complete card bodies in legal native Standard games.
use rustmage::{
    Action, Card, Config, DeckRules, Game, Observation, Player, StackKind, Target, Zone,
};
use std::collections::{BTreeSet, HashMap};
const CARDS: [&str; 3] = [
    "Kin-Tree Severance",
    "Reputable Merchant",
    "Temur Tawnyback",
];
fn deck(blue: bool) -> Vec<Card> {
    let lands: &[(&str, usize)] = if blue {
        &[("Island", 12), ("Forest", 12)]
    } else {
        &[
            ("Plains", 6),
            ("Swamp", 6),
            ("Forest", 6),
            ("Island", 3),
            ("Mountain", 3),
        ]
    };
    let mut deck = Vec::new();
    for &(name, n) in lands {
        deck.extend(vec![Card::from_name(name).unwrap(); n]);
    }
    for name in [
        "Kin-Tree Severance",
        "Reputable Merchant",
        "Temur Tawnyback",
        "Bear Cub",
        "Hinterland Sanctifier",
        "Three Tree Mascot",
        "Goldvein Pick",
        "Ordinary Bear",
        "Murder",
    ] {
        deck.extend([Card::from_name(name).unwrap(); 4]);
    }
    assert_eq!(deck.len(), 60);
    assert!(
        deck.iter()
            .all(|c| c.is_supported() && c.is_standard_legal())
    );
    deck
}
fn score(obs: &Observation, a: &Action, generic: u32) -> i64 {
    let object = |id| obs.objects.iter().find(|o| o.id == id).unwrap();
    match a {
        Action::Concede | Action::Mulligan => -1000,
        Action::ChooseSpellCost { cost } => {
            let c = &obs
                .cost_choices
                .iter()
                .find(|c| c.index == *cost)
                .unwrap()
                .cost;
            200 - i64::from(c.generic.abs_diff(generic))
        }
        Action::AnnounceCast { card } | Action::Cast { card, .. }
            if CARDS.contains(&object(*card).card.name()) =>
        {
            let available = obs.players[obs.actor.unwrap().index()]
                .mana
                .iter()
                .map(|n| u64::from(*n))
                .sum::<u64>()
                + obs
                    .objects
                    .iter()
                    .filter(|o| {
                        o.owner == obs.actor.unwrap()
                            && o.zone == Zone::Battlefield
                            && !o.tapped
                            && o.card.is_land()
                    })
                    .count() as u64;
            if generic == 6 && available < 6 {
                -100
            } else if matches!(a, Action::AnnounceCast { .. }) {
                150
            } else {
                80
            }
        }
        Action::TapMana { .. } => {
            if obs.decision == "pay_costs"
                || (obs.actor == Some(obs.active)
                    && matches!(obs.step, rustmage::Step::Main1 | rustmage::Step::Main2))
            {
                120
            } else {
                -100
            }
        }
        Action::Cast {
            target: Some(Target::Card(id)),
            ..
        } => {
            if object(*id).owner != obs.actor.unwrap() {
                90
            } else {
                -100
            }
        }
        Action::AnnounceCast { .. } | Action::Cast { .. } => 80,
        Action::Activate { .. } | Action::AnnounceActivate { .. } => -20,
        Action::ChooseTarget {
            target: Target::Card(id),
        } => {
            if obs
                .announcing
                .as_ref()
                .is_some_and(|s| s.source().is_some_and(|source| source.card.name() == "Kin-Tree Severance"))
                && object(*id).owner == obs.actor.unwrap()
            {
                -50
            } else {
                90
            }
        }
        Action::PutTrigger { .. }
        | Action::FinishTargets
        | Action::PayCosts { .. }
        | Action::Keep
        | Action::Bottom { .. }
        | Action::Discard { .. } => 100,
        Action::PlayLand { .. } => 130,
        Action::Attack { .. } | Action::AttackPlaneswalker { .. } => {
            if obs.turn >= 16 {
                60
            } else {
                -50
            }
        }
        Action::Block { .. } => 60,
        Action::Pass | Action::FinishAttackers | Action::FinishBlockers => 0,
        _ => 40,
    }
}
fn check(origin: &Observation, after: &Observation, item: &rustmage::SpellView) -> usize {
    let n = item.source_card.name();
    let target = item.targets.first().and_then(|b| {
        if let Target::Card(id) = b.target {
            origin.objects.iter().find(|o| {
                o.id == id && Some(o.reference) == b.object && o.zone == Zone::Battlefield
            })
        } else {
            None
        }
    });
    if n == "Kin-Tree Severance" && item.kind == StackKind::Spell {
        if let Some(target) = target {
            assert_eq!(
                after
                    .objects
                    .iter()
                    .find(|o| o.id == target.id)
                    .unwrap()
                    .zone,
                Zone::Exile
            );
            return 1;
        }
    } else if n == "Reputable Merchant" && matches!(item.kind, StackKind::TriggeredAbility { .. }) {
        if let Some(target) = target {
            let new = after
                .objects
                .iter()
                .find(|o| o.reference == target.reference)
                .unwrap();
            assert_eq!(new.counters.plus_one, target.counters.plus_one + 1);
            assert_eq!(new.power, target.power + 1);
            assert_eq!(new.toughness, target.toughness + 1);
            return 3;
        }
    } else if n == "Temur Tawnyback" && matches!(item.kind, StackKind::TriggeredAbility { .. }) {
        let p = item.controller.index();
        assert_eq!(
            after.players[p].library_count,
            origin.players[p].library_count - 1
        );
        assert_eq!(after.players[p].hand_count, origin.players[p].hand_count);
        assert_eq!(
            after.players[p].cards_drawn_this_turn,
            origin.players[p].cards_drawn_this_turn + 1
        );
        return 3;
    }
    0
}
fn main() {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse()
        .unwrap();
    assert!(games > 0);
    let (mut decisions, mut checks, mut branches, mut max_bytes, mut choices) = (0, 0, 0, 0, 0);
    let mut seen = BTreeSet::new();
    let mut generic_choices = BTreeSet::new();
    let mut paid_totals = BTreeSet::new();
    for seed in 0..games {
        let mut game = Game::new(Config {
            decks: [deck(false), deck(true)],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })
        .unwrap();
        let mut origins = HashMap::new();
        for step in 0..100_000 {
            if game.outcome().is_some() {
                break;
            }
            assert!(step < 99_999, "game must end naturally");
            let actor = game.actor().unwrap();
            let before = game.observe(actor);
            let actions = game.legal_actions();
            let generic = if actor == Player::P1 {
                6
            } else {
                (seed % 4) as u32 * 2
            };
            let a = actions
                .iter()
                .enumerate()
                .max_by_key(|(i, a)| (score(&before, a, generic), std::cmp::Reverse(*i)))
                .unwrap()
                .1
                .clone();
            if let Action::ChooseSpellCost { cost } = a {
                let spec = before
                    .cost_choices
                    .iter()
                    .find(|c| c.index == cost)
                    .unwrap()
                    .cost;
                assert!(matches!(spec.generic, 0 | 2 | 4 | 6));
                assert_eq!(spec.colored.iter().sum::<u32>(), 3 - spec.generic / 2);
                let name = before.announcing.as_ref().unwrap().source_card.name();
                let colors = if name == "Temur Tawnyback" {
                    [1, 3, 4]
                } else {
                    [0, 2, 4]
                };
                assert!(
                    spec.colored
                        .iter()
                        .enumerate()
                        .all(|(i, n)| *n <= 1 && (*n == 0 || colors.contains(&i)))
                );
                choices += 1;
                checks += 2;
                generic_choices.insert(spec.generic);
            }
            if let Action::PayCosts { payment } = a
                && let Some(cost) = before.announcing_cost
                && before
                    .announcing
                    .as_ref()
                    .is_some_and(|s| CARDS.contains(&s.source_card.name()))
            {
                let total = payment.iter().sum::<u32>();
                assert_eq!(total, cost.generic + cost.colored.iter().sum::<u32>());
                assert!(
                    payment
                        .iter()
                        .zip(cost.colored)
                        .all(|(paid, required)| *paid >= required)
                );
                paid_totals.insert(total);
                checks += 2;
            }
            let saved = game.snapshot();
            let fork = if step % 4 == 0 || before.announcing.is_some() || before.resolving.is_some()
            {
                let mut fork = game.clone();
                fork.apply(actor, a.clone()).unwrap();
                branches += 1;
                Some(fork.snapshot())
            } else {
                None
            };
            game.apply(actor, a.clone()).unwrap();
            if let Some(expected) = fork {
                assert_eq!(game.snapshot(), expected);
                game.restore(&saved).unwrap();
                assert_eq!(
                    game.apply_index(actor, before.revision, 0),
                    Err(rustmage::Error::StaleRevision)
                );
                game.apply(actor, a).unwrap();
                assert_eq!(game.snapshot(), expected);
            }
            let after = game.observe(actor);
            if let Some(item) = before.stack.last()
                && after.stack.iter().all(|s| s.id != item.id)
                && CARDS.contains(&item.source_card.name())
            {
                origins.entry(item.id).or_insert_with(|| before.clone());
            }
            let completed = before
                .resolving
                .as_ref()
                .filter(|s| after.resolving.as_ref().is_none_or(|a| a.id != s.id))
                .or_else(|| {
                    before.stack.last().filter(|s| {
                        after.stack.iter().all(|a| a.id != s.id)
                            && after.resolving.as_ref().is_none_or(|a| a.id != s.id)
                    })
                });
            if let Some(item) = completed
                && CARDS.contains(&item.source_card.name())
            {
                let origin = origins.remove(&item.id).unwrap_or_else(|| before.clone());
                let n = check(&origin, &after, item);
                checks += n;
                if n > 0 {
                    seen.insert(item.source_card.name());
                }
            }
            max_bytes = max_bytes.max(game.state_bytes());
            decisions += 1;
        }
        assert!(game.outcome().is_some());
    }
    if games >= 20 {
        assert_eq!(seen.len(), 3);
        assert_eq!(generic_choices, [0, 2, 4, 6].into_iter().collect());
        assert_eq!(paid_totals, [3, 4, 5, 6].into_iter().collect());
    }
    println!(
        "Completed {games} legal Standard two-brid games, {decisions} decisions, {choices} committed choices, {checks} independent public checks, {branches} cloned/replayed boundaries, {max_bytes} maximum initialized state bytes; bodies {seen:?}; generic choices {generic_choices:?}; mana paid {paid_totals:?}; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
}
