//! Public cost/rule checks and branch replay over legal supported Standard decks.
use rustmage::{
    Action, Card, Config, DeckRules, Game, Keywords, Mana, ManaCost, Observation, Player,
    SpellView, StackKind, Target, Zone,
};
use std::collections::{BTreeSet, HashMap};
const NEW: [&str; 20] = [
    "Arcane Epiphany",
    "Archmage of Runes",
    "Ballyrush Banneret",
    "Dragonlord's Servant",
    "Focus the Mind",
    "Geist of Saint Thalia",
    "Geyser Drake",
    "Gigastorm Titan",
    "Highspire Bell-Ringer",
    "Honest Rutstein",
    "Mistmeadow Council",
    "Mocking Sprite",
    "Orysa, Tide Choreographer",
    "Pearl of Wisdom",
    "Saved by the Shell",
    "Stormcatch Mentor",
    "Uthros Psionicist",
    "Venom's Hunger",
    "Visions of Villainy",
    "Voyager Quickwelder",
];
fn deck(which: usize) -> Vec<Card> {
    let (lands, spells): (&[(&str, usize)], &[&str]) = match which {
        0 => (
            &[("Island", 18), ("Mountain", 6)],
            &[
                "Mocking Sprite",
                "Archmage of Runes",
                "Stormcatch Mentor",
                "Geyser Drake",
                "Uthros Psionicist",
                "Highspire Bell-Ringer",
                "Arcane Epiphany",
                "Focus the Mind",
                "Gigastorm Titan",
            ],
        ),
        1 => (
            &[("Plains", 8), ("Island", 8), ("Forest", 8)],
            &[
                "Ballyrush Banneret",
                "Geist of Saint Thalia",
                "Mistmeadow Council",
                "Three Tree Mascot",
                "Saved by the Shell",
                "Pearl of Wisdom",
                "Orysa, Tide Choreographer",
                "Aegis Turtle",
                "Ordinary Bear",
            ],
        ),
        2 => (
            &[("Swamp", 8), ("Mountain", 8), ("Plains", 8)],
            &[
                "Dragonlord's Servant",
                "Voyager Quickwelder",
                "Venom's Hunger",
                "Visions of Villainy",
                "Three Tree Mascot",
                "Goldvein Pick",
                "Shivan Dragon",
                "Reputable Merchant",
                "Bishop's Soldier",
            ],
        ),
        _ => (
            &[("Forest", 8), ("Swamp", 8), ("Plains", 8)],
            &[
                "Honest Rutstein",
                "Ballyrush Banneret",
                "Three Tree Mascot",
                "Reputable Merchant",
                "Venom's Hunger",
                "Visions of Villainy",
                "Ordinary Bear",
                "Saved by the Shell",
                "Duty Beyond Death",
            ],
        ),
    };
    let mut result = Vec::new();
    for &(name, n) in lands {
        result.extend(vec![Card::from_name(name).unwrap(); n]);
    }
    for &name in spells {
        result.extend([Card::from_name(name).unwrap(); 4]);
    }
    assert_eq!(result.len(), 60);
    assert!(
        result
            .iter()
            .all(|c| c.is_supported() && c.is_standard_legal())
    );
    result
}
fn base(card: Card) -> ManaCost {
    let mut cost = ManaCost {
        colored: [0; 6],
        generic: 0,
    };
    for s in card
        .info()
        .mana_cost
        .split('}')
        .filter_map(|p| p.strip_prefix('{'))
    {
        if let Ok(n) = s.parse::<u32>() {
            cost.generic += n;
        } else if s.starts_with("2/") {
            cost.generic += 2;
        } else {
            assert_eq!(s.len(), 1);
            cost.colored["WUBRGC".find(s).unwrap()] += 1;
        }
    }
    cost
}
fn kind(card: Card, wanted: &str) -> bool {
    card.info()
        .type_line
        .split(" — ")
        .next()
        .unwrap()
        .split_whitespace()
        .any(|w| w == wanted)
}
fn costs(
    obs: &Observation,
    p: Player,
    card: Card,
    mut cost: ManaCost,
) -> (ManaCost, BTreeSet<&'static str>) {
    let own = || {
        obs.objects
            .iter()
            .filter(|o| o.owner == p && o.zone == Zone::Battlefield)
    };
    let subtype = |s| own().any(|o| o.card.has_subtype(s));
    let count = obs.players[p.index()].spells_cast_this_turn;
    let mut applied = BTreeSet::new();
    let mut n = 0u32;
    let own_discount = match card.name() {
        "Arcane Epiphany" => u32::from(subtype("Wizard")),
        "Mistmeadow Council" => u32::from(subtype("Kithkin")),
        "Pearl of Wisdom" => u32::from(subtype("Otter")),
        "Saved by the Shell" => u32::from(subtype("Turtle")),
        "Visions of Villainy" => u32::from(subtype("Villain")),
        "Venom's Hunger" => 2 * u32::from(subtype("Villain")),
        "Focus the Mind" => 2 * u32::from(count > 0),
        "Gigastorm Titan" => 3 * u32::from(count > 0),
        "Orysa, Tide Choreographer" => {
            3 * u32::from(
                own()
                    .filter(|o| o.card.is_creature())
                    .map(|o| o.toughness)
                    .sum::<i128>()
                    >= 10,
            )
        }
        _ => 0,
    };
    if own_discount > 0 {
        n += own_discount;
        applied.insert(card.name());
    }
    for source in own() {
        let amount = match source.card.name() {
            "Dragonlord's Servant" => u32::from(card.has_subtype("Dragon")),
            "Voyager Quickwelder" => u32::from(card.is_artifact()),
            "Geist of Saint Thalia" => u32::from(!card.is_creature()),
            "Archmage of Runes" | "Mocking Sprite" | "Stormcatch Mentor" => {
                u32::from(kind(card, "Instant") || kind(card, "Sorcery"))
            }
            "Honest Rutstein" => u32::from(card.is_creature()),
            "Ballyrush Banneret" => {
                u32::from(card.has_subtype("Kithkin") || card.has_subtype("Soldier"))
            }
            "Geyser Drake" => u32::from(obs.active != p),
            "Uthros Psionicist" => 2 * u32::from(count == 1),
            "Highspire Bell-Ringer" => u32::from(count == 1),
            _ => 0,
        };
        if amount > 0 {
            n += amount;
            applied.insert(source.card.name());
        }
    }
    if cost.generic == 0 {
        applied.clear();
    }
    cost.generic = cost.generic.saturating_sub(n);
    (cost, applied)
}
fn payment(cost: ManaCost, paid: Mana) {
    assert!(paid.iter().zip(cost.colored).all(|(n, c)| *n >= c));
    assert_eq!(
        paid.iter().sum::<u32>(),
        cost.colored.iter().sum::<u32>() + cost.generic
    );
}
fn score(obs: &Observation, a: &Action, hold_drake: bool) -> i32 {
    let hold = hold_drake
        && obs.actor == Some(obs.active)
        && obs.objects.iter().any(|o| {
            Some(o.owner) == obs.actor
                && o.zone == Zone::Battlefield
                && o.card.name() == "Geyser Drake"
        })
        && obs.objects.iter().any(|o| {
            Some(o.owner) == obs.actor && o.zone == Zone::Hand && o.card.name() == "Arcane Epiphany"
        });
    if hold && matches!(a, Action::AnnounceCast { .. } | Action::Cast { .. }) {
        return -20;
    }
    match a {
        Action::Concede | Action::Mulligan => -1000,
        Action::AnnounceCast { card } => {
            if obs.objects.iter().find(|o| o.id == *card).is_some_and(|o| {
                NEW.contains(&o.card.name())
                    || o.card.name() == "Reputable Merchant"
                    || o.card.name() == "Duty Beyond Death"
            }) {
                160
            } else {
                80
            }
        }
        Action::Cast { card, .. } => {
            if obs
                .objects
                .iter()
                .find(|o| o.id == *card)
                .is_some_and(|o| NEW.contains(&o.card.name()))
            {
                90
            } else {
                80
            }
        }
        Action::ChooseSpellCost { cost } => 200 - before_generic(obs, *cost) as i32,
        Action::ChooseAdditionalCost { object } => {
            if obs
                .objects
                .iter()
                .find(|o| o.reference == *object)
                .is_some_and(|o| o.card.name() == "Geist of Saint Thalia")
            {
                190
            } else {
                80
            }
        }
        Action::TapMana { .. } => {
            if obs.decision == "pay_costs"
                || (!hold
                    && obs.actor == Some(obs.active)
                    && matches!(obs.step, rustmage::Step::Main1 | rustmage::Step::Main2))
            {
                120
            } else {
                -100
            }
        }
        Action::Activate { .. } | Action::AnnounceActivate { .. } => -20,
        Action::PlayLand { .. } => 130,
        Action::PutTrigger { .. }
        | Action::FinishTargets
        | Action::PayCosts { .. }
        | Action::Keep
        | Action::Bottom { .. }
        | Action::Discard { .. }
        | Action::KeepLegend { .. } => 100,
        Action::ChooseTarget {
            target: Target::Card(id),
        } => {
            let target = obs.objects.iter().find(|o| o.id == *id).unwrap();
            if obs
                .announcing
                .as_ref()
                .is_some_and(|s| s.source().is_some_and(|source| source.card.name() == "Venom's Hunger"))
                && target.owner == obs.actor.unwrap()
            {
                -100
            } else {
                90
            }
        }
        Action::Attack { .. } | Action::AttackPlaneswalker { .. } => {
            if obs.turn >= 18 {
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
fn before_generic(obs: &Observation, index: u16) -> u32 {
    obs.cost_choices
        .iter()
        .find(|c| c.index == index)
        .unwrap()
        .cost
        .generic
}
fn body(before: &Observation, after: &Observation, item: &SpellView) -> usize {
    let Some(source) = item.source() else { return 0; };
    let name = source.card.name();
    let p = item.controller.index();
    let target = item.targets.first().and_then(|b| {
        if let Target::Card(id) = b.target {
            before
                .objects
                .iter()
                .find(|o| o.id == id && Some(o.reference) == b.object)
        } else {
            None
        }
    });
    if item.kind == StackKind::Spell && item.source().is_some_and(|source| source.card.is_creature()) {
        let stats = match name {
            "Archmage of Runes" => (3, 6),
            "Ballyrush Banneret" => (2, 1),
            "Dragonlord's Servant" => (1, 3),
            "Geist of Saint Thalia" => (1, 2),
            "Geyser Drake" => (2, 3),
            "Gigastorm Titan" => (4, 4),
            "Highspire Bell-Ringer" => (1, 4),
            "Honest Rutstein" => (3, 2),
            "Mistmeadow Council" => (4, 3),
            "Mocking Sprite" => (2, 1),
            "Orysa, Tide Choreographer" => (2, 2),
            "Stormcatch Mentor" => (1, 1),
            "Uthros Psionicist" => (2, 4),
            "Voyager Quickwelder" => (2, 4),
            _ => return 0,
        };
        let source = after.objects.iter().find(|o| o.id == item.source().expect("example expects an object-backed packet").id).unwrap();
        assert_eq!(source.zone, Zone::Battlefield);
        assert_eq!((source.power, source.toughness), stats);
        return 3;
    }
    let trigger = matches!(item.kind, StackKind::TriggeredAbility { .. });
    let draw = match (name, trigger) {
        ("Arcane Epiphany", false) => (3, false),
        ("Focus the Mind", false) => (3, true),
        ("Pearl of Wisdom" | "Visions of Villainy", false) => (2, false),
        ("Archmage of Runes" | "Mistmeadow Council", true) => (1, false),
        ("Orysa, Tide Choreographer", true) => (2, false),
        _ => (0, false),
    };
    if draw.0 > 0 {
        if before.players[p].library_count < draw.0 {
            return 0;
        }
        assert_eq!(
            after.players[p].library_count,
            before.players[p].library_count - draw.0
        );
        assert_eq!(
            after.players[p].cards_drawn_this_turn,
            before.players[p].cards_drawn_this_turn + draw.0 as u64
        );
        assert_eq!(
            after.players[p].hand_count,
            before.players[p].hand_count + draw.0 - usize::from(draw.1)
        );
        if name == "Visions of Villainy" {
            assert_eq!(after.players[p].life, before.players[p].life - 2);
        }
        return 3;
    }
    if name == "Stormcatch Mentor" && trigger {
        if let Some(old) = before
            .objects
            .iter()
            .find(|o| o.reference == item.source().expect("example expects an object-backed packet").object && o.zone == Zone::Battlefield)
        {
            let new = after
                .objects
                .iter()
                .find(|o| o.reference == item.source().expect("example expects an object-backed packet").object)
                .unwrap();
            assert_eq!(
                (new.power, new.toughness),
                (old.power + 1, old.toughness + 1)
            );
            return 2;
        }
        return 0;
    }
    if name == "Honest Rutstein" && trigger {
        if target.is_some_and(|o| o.zone == Zone::Graveyard) {
            assert_eq!(
                after.players[p].hand_count,
                before.players[p].hand_count + 1
            );
            return 1;
        }
        return 0;
    }
    if let Some(old) = target.filter(|o| o.zone == Zone::Battlefield) {
        if name == "Venom's Hunger" {
            let new = after.objects.iter().find(|o| o.id == old.id).unwrap();
            assert_eq!(
                new.zone,
                if old.keywords.contains(Keywords::INDESTRUCTIBLE) {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(after.players[p].life, before.players[p].life + 2);
            return 2;
        }
        if name == "Saved by the Shell" {
            let new = after
                .objects
                .iter()
                .find(|o| o.reference == old.reference)
                .unwrap();
            assert_eq!(new.counters.plus_one, old.counters.plus_one + 1);
            assert!(
                new.keywords.contains(
                    Keywords::TRAMPLE
                        .union(Keywords::HEXPROOF)
                        .union(Keywords::INDESTRUCTIBLE)
                )
            );
            return 2;
        }
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
    let (mut decisions, mut checks, mut branches, mut max_bytes, mut payments) = (0, 0, 0, 0, 0);
    let mut applied = BTreeSet::new();
    let mut casted = BTreeSet::new();
    let mut resolved = BTreeSet::new();
    for seed in 0..games {
        let mut game = Game::new(Config {
            decks: [deck(seed % 4), deck((seed + 1) % 4)],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })
        .unwrap();
        let mut locked = HashMap::new();
        let mut locked_rules = HashMap::new();
        let mut origins = HashMap::new();
        for step in 0..100_000 {
            if game.outcome().is_some() {
                break;
            }
            assert!(step < 99_999, "game must terminate");
            let actor = game.actor().unwrap();
            let before = game.observe(actor);
            let actions = game.legal_actions();
            if matches!(before.decision, "pay_costs" | "additional_cost")
                && let Some(root) = before
                    .announcing
                    .as_ref()
                    .filter(|s| s.kind == StackKind::Spell)
            {
                locked.entry(root.id).or_insert_with(|| {
                    let (cost, rules) = costs(
                        &before,
                        root.controller,
                        root.source().expect("example expects an object-backed packet").card,
                        before
                            .announcing_cost
                            .unwrap_or_else(|| base(root.source().expect("example expects an object-backed packet").card)),
                    );
                    locked_rules.insert(root.id, rules);
                    cost
                });
            }
            for a in &actions {
                if let Action::Cast {
                    card,
                    payment: paid,
                    ..
                } = a
                {
                    let card = before.objects.iter().find(|o| o.id == *card).unwrap().card;
                    if !card.info().mana_cost.contains('/') {
                        let (cost, _) = costs(&before, actor, card, base(card));
                        payment(cost, *paid);
                        checks += 1;
                    }
                }
            }
            let a = actions
                .iter()
                .enumerate()
                .max_by_key(|(i, a)| {
                    (
                        score(&before, a, !applied.contains("Geyser Drake")),
                        std::cmp::Reverse(*i),
                    )
                })
                .unwrap()
                .1
                .clone();
            if let Action::PayCosts { payment: paid } = a
                && let Some(root) = before
                    .announcing
                    .as_ref()
                    .filter(|s| s.kind == StackKind::Spell)
            {
                let cost = locked[&root.id];
                let request = before.payment_request.as_ref().unwrap();
                assert_eq!(request.required, cost.colored);
                assert_eq!(request.generic, cost.generic);
                payment(cost, paid);
                applied.extend(locked_rules.remove(&root.id).unwrap_or_default());
                payments += 1;
                checks += 3;
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
            for item in &after.stack {
                if item.kind == StackKind::Spell && item.source().is_some_and(|source| NEW.contains(&source.card.name())) {
                    casted.insert(item.source().expect("example expects an object-backed packet").card.name());
                }
            }
            if let Some(item) = before.stack.last()
                && after.stack.iter().all(|s| s.id != item.id)
                && item.source().is_some_and(|source| NEW.contains(&source.card.name()))
            {
                origins.entry(item.id).or_insert_with(|| before.clone());
            }
            let complete = before
                .resolving
                .as_ref()
                .filter(|s| after.resolving.as_ref().is_none_or(|a| a.id != s.id))
                .or_else(|| {
                    before.stack.last().filter(|s| {
                        after.stack.iter().all(|a| a.id != s.id)
                            && after.resolving.as_ref().is_none_or(|a| a.id != s.id)
                    })
                });
            if let Some(item) = complete
                && item.source().is_some_and(|source| NEW.contains(&source.card.name()))
            {
                let origin = origins.remove(&item.id).unwrap_or_else(|| before.clone());
                let n = body(&origin, &after, item);
                checks += n;
                if n > 0 {
                    resolved.insert((item.source().expect("example expects an object-backed packet").card.name(), format!("{:?}", item.kind)));
                }
            }
            decisions += 1;
            max_bytes = max_bytes.max(game.state_bytes());
        }
        assert!(game.outcome().is_some());
    }
    if games >= 100 {
        assert_eq!(casted.len(), 20);
        assert_eq!(applied, NEW.into_iter().collect());
    }
    println!(
        "Completed {games} legal Standard cost-reduction games, {decisions} decisions, {payments} staged payments, {checks} independent public checks, {branches} cloned/replayed boundaries, {max_bytes} maximum initialized state bytes; applied {applied:?}; cast {casted:?}; resolved {resolved:?}; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
}
