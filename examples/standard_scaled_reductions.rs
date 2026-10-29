//! Public cost/rule checks and branch replay over legal supported Standard decks.
use rustmage::{
    Action, Card, Config, DeckRules, Game, Keywords, Mana, ManaCost, Observation, Player,
    SpellView, StackKind, Target, Zone,
};
use std::collections::{BTreeSet, HashMap};
const NEW: [&str; 14] = [
    "Chitin Gravestalker",
    "Claws Out",
    "Gargantuan Leech",
    "Huskburster Swarm",
    "Memory Guardian",
    "Salt Road Packbeast",
    "Sami, Wildcat Captain",
    "Scales of Shale",
    "Thrumming Hivepool",
    "Tolarian Terror",
    "Travel the Overworld",
    "Valkyrie Aerial Unit",
    "Wildvine Pummeler",
    "Witherbloom, the Balancer",
];
fn deck(which: usize) -> Vec<Card> {
    let (lands, spells): (&[(&str, usize)], &[&str]) = match which {
        0 => (
            &[("Island", 24)],
            &[
                "Memory Guardian",
                "Valkyrie Aerial Unit",
                "Tolarian Terror",
                "Three Tree Mascot",
                "Goldvein Pick",
                "Opt",
                "Strix Lookout",
                "A.I.M. Synthoids",
                "Quick Study",
            ],
        ),
        1 => (
            &[("Swamp", 10), ("Forest", 10), ("Captivating Cave", 4)],
            &[
                "Chitin Gravestalker",
                "Huskburster Swarm",
                "Gargantuan Leech",
                "Witherbloom, the Balancer",
                "Wildvine Pummeler",
                "Three Tree Mascot",
                "Ordinary Bear",
                "Murder",
                "Wary Thespian",
            ],
        ),
        2 => (
            &[("Plains", 12), ("Mountain", 8), ("Swamp", 4)],
            &[
                "Sami, Wildcat Captain",
                "Salt Road Packbeast",
                "Claws Out",
                "Scales of Shale",
                "Three Tree Mascot",
                "Thrumming Hivepool",
                "Goldvein Pick",
                "Bishop's Soldier",
                "Disenchant",
            ],
        ),
        _ => (
            &[
                ("Island", 10),
                ("Forest", 6),
                ("Swamp", 4),
                ("Gohn, Town of Ruin", 4),
            ],
            &[
                "Travel the Overworld",
                "Memory Guardian",
                "Valkyrie Aerial Unit",
                "Wildvine Pummeler",
                "Witherbloom, the Balancer",
                "Huskburster Swarm",
                "Chitin Gravestalker",
                "Three Tree Mascot",
                "Ordinary Bear",
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
    let count = |zone, predicate: &dyn Fn(Card) -> bool| {
        obs.objects
            .iter()
            .filter(|o| o.owner == p && o.zone == zone && predicate(o.card))
            .count() as u32
    };
    let artifacts = own().filter(|o| o.card.is_artifact()).count() as u32;
    let creatures = own().filter(|o| o.card.is_creature()).count() as u32;
    let mut applied = BTreeSet::new();
    let own_discount = match card.name() {
        "Memory Guardian" | "Valkyrie Aerial Unit" => artifacts,
        "Salt Road Packbeast" | "Witherbloom, the Balancer" => creatures,
        "Claws Out" => own().filter(|o| o.card.has_subtype("Cat")).count() as u32,
        "Scales of Shale" => own().filter(|o| o.card.has_subtype("Lizard")).count() as u32,
        "Thrumming Hivepool" => own().filter(|o| o.card.has_subtype("Sliver")).count() as u32,
        "Travel the Overworld" => own().filter(|o| o.card.has_subtype("Town")).count() as u32,
        "Gargantuan Leech" => {
            own().filter(|o| o.card.has_subtype("Cave")).count() as u32
                + count(Zone::Graveyard, &|c| !c.is_token() && c.has_subtype("Cave"))
        }
        "Chitin Gravestalker" => count(Zone::Graveyard, &|c| {
            !c.is_token() && (c.is_artifact() || c.is_creature())
        }),
        "Tolarian Terror" => count(Zone::Graveyard, &|c| {
            !c.is_token() && (kind(c, "Instant") || kind(c, "Sorcery"))
        }),
        "Huskburster Swarm" => {
            count(Zone::Graveyard, &|c| !c.is_token() && c.is_creature())
                + count(Zone::Exile, &|c| !c.is_token() && c.is_creature())
        }
        "Wildvine Pummeler" => own()
            .fold(0u8, |mask, o| mask | o.card.colors())
            .count_ones(),
        _ => 0,
    };
    let mut n = own_discount;
    if n > 0 {
        applied.insert(card.name());
    }
    for source in own() {
        let amount = match source.card.name() {
            "Sami, Wildcat Captain" => artifacts,
            "Witherbloom, the Balancer" if kind(card, "Instant") || kind(card, "Sorcery") => {
                creatures
            }
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
                .is_some_and(|s| matches!(s.source_card.name(), "Murder" | "Disenchant"))
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
            "Chitin Gravestalker" => (5, 4),
            "Gargantuan Leech" => (5, 5),
            "Huskburster Swarm" => (6, 6),
            "Memory Guardian" => (3, 4),
            "Salt Road Packbeast" => (4, 3),
            "Sami, Wildcat Captain" => (4, 4),
            "Tolarian Terror" => (5, 5),
            "Valkyrie Aerial Unit" => (5, 4),
            "Wildvine Pummeler" => (6, 5),
            "Witherbloom, the Balancer" => (5, 5),
            _ => return 0,
        };
        let o = after.objects.iter().find(|o| o.id == item.card).unwrap();
        assert_eq!(o.zone, Zone::Battlefield);
        assert_eq!((o.power, o.toughness), stats);
        return 3;
    }
    let trigger = matches!(item.kind, StackKind::TriggeredAbility { .. });
    let draw = match (name, trigger) {
        ("Travel the Overworld", false) => 4,
        ("Salt Road Packbeast", true) => 1,
        _ => 0,
    };
    if draw > 0 && before.players[p].library_count >= draw {
        assert_eq!(
            after.players[p].library_count,
            before.players[p].library_count - draw
        );
        assert_eq!(
            after.players[p].cards_drawn_this_turn,
            before.players[p].cards_drawn_this_turn + draw as u64
        );
        return 2;
    }
    if name == "Thrumming Hivepool" && trigger {
        let count = |obs: &Observation| {
            obs.objects
                .iter()
                .filter(|o| {
                    o.owner == item.controller
                        && o.zone == Zone::Battlefield
                        && o.card.is_token()
                        && o.card.has_subtype("Sliver")
                })
                .count()
        };
        assert_eq!(count(after), count(before) + 2);
        // A queued upkeep trigger survives its source. Tokens still enter,
        // but the keyword grant requires a live owned Hivepool.
        if after.objects.iter().any(|o| {
            o.owner == item.controller
                && o.zone == Zone::Battlefield
                && o.card.name() == "Thrumming Hivepool"
        }) {
            for o in after.objects.iter().filter(|o| {
                o.owner == item.controller
                    && o.zone == Zone::Battlefield
                    && o.card.has_subtype("Sliver")
            }) {
                assert!(
                    o.keywords
                        .contains(Keywords::DOUBLE_STRIKE.union(Keywords::HASTE))
                );
            }
        }
        return 2;
    }
    if name == "Valkyrie Aerial Unit" && trigger {
        assert!(after.resolving.is_none());
        return 1;
    }
    let targets: Vec<_> = if name == "Claws Out" {
        before
            .objects
            .iter()
            .filter(|o| {
                o.owner == item.controller && o.zone == Zone::Battlefield && o.card.is_creature()
            })
            .collect()
    } else if name == "Scales of Shale" {
        item.targets
            .iter()
            .filter_map(|b| {
                before
                    .objects
                    .iter()
                    .find(|o| Some(o.reference) == b.object && o.zone == Zone::Battlefield)
            })
            .collect()
    } else {
        vec![]
    };
    let mut checked = 0;
    for old in targets {
        if let Some(new) = after
            .objects
            .iter()
            .find(|o| o.reference == old.reference && o.zone == Zone::Battlefield)
        {
            assert_eq!(new.power, old.power + 2);
            assert_eq!(
                new.toughness,
                old.toughness + if name == "Claws Out" { 2 } else { 0 }
            );
            if name == "Scales of Shale" {
                assert!(
                    new.keywords
                        .contains(Keywords::LIFELINK.union(Keywords::INDESTRUCTIBLE))
                );
            }
            checked += 2;
        }
    }
    checked
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
                        root.source_card,
                        before
                            .announcing_cost
                            .unwrap_or_else(|| base(root.source_card)),
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
                    casted.insert(item.source_card.name());
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
                    resolved.insert((item.source_card.name(), format!("{:?}", item.kind)));
                }
            }
            decisions += 1;
            max_bytes = max_bytes.max(game.state_bytes());
        }
        assert!(game.outcome().is_some());
    }
    if games >= 100 {
        assert_eq!(casted.len(), 14);
        assert_eq!(applied, NEW.into_iter().collect());
    }
    println!(
        "Completed {games} legal Standard scaled-cost games, {decisions} decisions, {payments} staged payments, {checks} independent public checks, {branches} cloned/replayed boundaries, {max_bytes} maximum initialized state bytes; applied {applied:?}; cast {casted:?}; resolved {resolved:?}; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
}
