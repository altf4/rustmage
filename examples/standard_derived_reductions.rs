//! Public cost/rule checks and branch replay over legal supported Standard decks.
use rustmage::{
    Action, Card, Config, DeckRules, Game, Keywords, Mana, ManaCost, Observation, Player,
    SpellView, StackKind, Target, Zone,
};
use std::collections::{BTreeSet, HashMap};
const NEW: [&str; 7] = [
    "All-Fates Scroll",
    "Fungal Colossus",
    "Ghalta the Unstoppable",
    "Ghalta, Primal Hunger",
    "Hellspur Brute",
    "The Lord of the Eagles",
    "The Scarlet Witch",
];
const RULES: [&str; 6] = [
    "Fungal Colossus",
    "Ghalta the Unstoppable",
    "Ghalta, Primal Hunger",
    "Hellspur Brute",
    "The Lord of the Eagles",
    "The Scarlet Witch",
];
fn deck(which: usize) -> Vec<Card> {
    let (lands, spells): (&[(&str, usize)], &[&str]) = match which {
        0 => (
            &[("Forest", 12), ("Plains", 8), ("Island", 4)],
            &[
                "Fungal Colossus",
                "Ghalta the Unstoppable",
                "Ghalta, Primal Hunger",
                "Regal Bunnicorn",
                "Ordinary Bear",
                "Three Tree Mascot",
                "All-Fates Scroll",
                "Gold Pan",
                "Snakeskin Veil",
            ],
        ),
        1 => (
            &[("Island", 20), ("Forest", 4)],
            &[
                "The Lord of the Eagles",
                "Memory Guardian",
                "Geyser Drake",
                "Three Tree Mascot",
                "All-Fates Scroll",
                "Quick Study",
                "Opt",
                "Ghalta, Primal Hunger",
                "Gold Pan",
            ],
        ),
        2 => (
            &[("Mountain", 12), ("Island", 8), ("Forest", 4)],
            &[
                "The Scarlet Witch",
                "Arcane Epiphany",
                "Kin-Tree Severance",
                "Hellspur Brute",
                "Three Tree Mascot",
                "Strix Lookout",
                "Fire Elemental",
                "Gold Pan",
                "Quick Study",
            ],
        ),
        _ => (
            &[("Forest", 12), ("Island", 6), ("Mountain", 6)],
            &[
                "Fungal Colossus",
                "Ghalta the Unstoppable",
                "Ghalta, Primal Hunger",
                "The Lord of the Eagles",
                "The Scarlet Witch",
                "Hellspur Brute",
                "All-Fates Scroll",
                "Three Tree Mascot",
                "Gold Pan",
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
    let creatures = || own().filter(|o| o.card.is_creature());
    let amount = match card.name() {
        "Fungal Colossus" => own()
            .filter(|o| o.card.is_land())
            .map(|o| o.card.name())
            .collect::<BTreeSet<_>>()
            .len() as i128,
        "Ghalta the Unstoppable" => creatures().map(|o| o.power).max().unwrap_or(0),
        "Ghalta, Primal Hunger" => creatures().map(|o| o.power).sum(),
        "The Lord of the Eagles" => creatures()
            .filter(|o| o.keywords.contains(Keywords::FLYING))
            .map(|o| o.power)
            .sum(),
        "Hellspur Brute" => own()
            .filter(|o| {
                ["Assassin", "Mercenary", "Pirate", "Rogue", "Warlock"]
                    .into_iter()
                    .any(|s| o.card.has_subtype(s))
            })
            .count() as i128,
        "Memory Guardian" => own().filter(|o| o.card.is_artifact()).count() as i128,
        "Arcane Epiphany" => i128::from(own().any(|o| o.card.has_subtype("Wizard"))),
        _ => 0,
    };
    let mut applied = BTreeSet::new();
    let mut n = u32::try_from(amount.max(0)).unwrap_or(u32::MAX);
    if n > 0 {
        applied.insert(card.name());
    }
    for source in own() {
        let amount = match source.card.name() {
            "The Scarlet Witch"
                if (kind(card, "Instant") || kind(card, "Sorcery"))
                    && base(card).generic + base(card).colored.iter().sum::<u32>() >= 4 =>
            {
                u32::try_from(source.power.max(0)).unwrap_or(u32::MAX)
            }
            "Geyser Drake" if obs.active != p => 1,
            _ => 0,
        };
        if amount > 0 {
            n = n.saturating_add(amount);
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
fn score(obs: &Observation, a: &Action) -> i32 {
    match a {
        Action::Concede | Action::Mulligan => -1000,
        Action::AnnounceCast { card } => {
            if obs
                .objects
                .iter()
                .find(|o| o.id == *card)
                .is_some_and(|o| NEW.contains(&o.card.name()))
            {
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
        Action::Activate {
            card, ability: 1, ..
        }
        | Action::AnnounceActivate {
            card, ability: 1, ..
        } if obs
            .objects
            .iter()
            .any(|o| o.id == *card && o.card.name() == "All-Fates Scroll") =>
        {
            150
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
                .is_some_and(|s| s.source().is_some_and(|source| source.card.name() == "Kin-Tree Severance"))
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
    if item.kind == StackKind::Spell && item.source().is_some_and(|source| source.card.is_creature()) {
        let stats = match name {
            "Fungal Colossus" => (5, 5),
            "Ghalta the Unstoppable" => (8, 8),
            "Ghalta, Primal Hunger" => (12, 12),
            "Hellspur Brute" => (5, 4),
            "The Lord of the Eagles" => (8, 8),
            "The Scarlet Witch" => (2, 3),
            _ => return 0,
        };
        let o = after.objects.iter().find(|o| o.id == item.card).unwrap();
        assert_eq!(o.zone, Zone::Battlefield);
        assert_eq!((o.power, o.toughness), stats);
        return 3;
    }
    if name == "All-Fates Scroll" {
        if item.kind == StackKind::Spell {
            let o = after.objects.iter().find(|o| o.id == item.card).unwrap();
            assert_eq!(o.zone, Zone::Battlefield);
            return 1;
        }
        if matches!(item.kind, StackKind::ActivatedAbility { ability: 1 }) {
            let n = before
                .objects
                .iter()
                .filter(|o| {
                    o.owner == item.controller && o.zone == Zone::Battlefield && o.card.is_land()
                })
                .map(|o| o.card.name())
                .collect::<BTreeSet<_>>()
                .len();
            if before.players[p].library_count < n {
                return 0;
            }
            assert_eq!(
                after.players[p].library_count,
                before.players[p].library_count - n
            );
            assert_eq!(
                after.players[p].cards_drawn_this_turn,
                before.players[p].cards_drawn_this_turn + n as u64
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
                .max_by_key(|(i, a)| (score(&before, a), std::cmp::Reverse(*i)))
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
        assert_eq!(casted.len(), 7);
        assert!(
            RULES.into_iter().all(|name| applied.contains(name)),
            "missing rule from {applied:?}"
        );
        assert!(
            resolved.iter().any(
                |(name, kind)| *name == "All-Fates Scroll" && kind.contains("ActivatedAbility")
            )
        );
    }
    println!(
        "Completed {games} legal Standard derived-cost games, {decisions} decisions, {payments} staged payments, {checks} independent public checks, {branches} cloned/replayed boundaries, {max_bytes} maximum initialized state bytes; applied {applied:?}; cast {casted:?}; resolved {resolved:?}; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
}
