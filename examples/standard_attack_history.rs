//! Public cost/rule checks and branch replay over legal supported Standard decks.
use rustmage::{
    Action, Card, Config, DeckRules, Game, Keywords, Mana, ManaCost, Observation, Player,
    SpellView, StackKind, Target, Zone,
};
use std::collections::{BTreeSet, HashMap};
const NEW: [&str; 7] = [
    "Chart a Course",
    "Fire Nation Raider",
    "Gorehorn Raider",
    "Rowdy Research",
    "Skyship Buccaneer",
    "Storm Fleet Spy",
    "Witchstalker Frenzy",
];
const RULES: [&str; 2] = ["Rowdy Research", "Witchstalker Frenzy"];
fn deck(which: usize) -> Vec<Card> {
    let (lands, spells): (&[(&str, usize)], &[&str]) = match which {
        0 => (
            &[("Island", 14), ("Mountain", 10)],
            &[
                "Rowdy Research",
                "Chart a Course",
                "Storm Fleet Spy",
                "Skyship Buccaneer",
                "Fire Nation Raider",
                "Gorehorn Raider",
                "Witchstalker Frenzy",
                "Three Tree Mascot",
                "Strix Lookout",
            ],
        ),
        1 => (
            &[("Island", 12), ("Forest", 12)],
            &[
                "Rowdy Research",
                "Chart a Course",
                "Storm Fleet Spy",
                "Skyship Buccaneer",
                "Ordinary Bear",
                "Three Tree Mascot",
                "Wary Thespian",
                "Opt",
                "Quick Study",
            ],
        ),
        2 => (
            &[("Mountain", 16), ("Forest", 8)],
            &[
                "Witchstalker Frenzy",
                "Fire Nation Raider",
                "Gorehorn Raider",
                "Three Tree Mascot",
                "Ordinary Bear",
                "Fire Elemental",
                "Gold Pan",
                "Shock",
                "Shivan Dragon",
            ],
        ),
        _ => (
            &[("Island", 10), ("Mountain", 8), ("Forest", 6)],
            &[
                "Chart a Course",
                "Fire Nation Raider",
                "Gorehorn Raider",
                "Rowdy Research",
                "Skyship Buccaneer",
                "Storm Fleet Spy",
                "Witchstalker Frenzy",
                "Ordinary Bear",
                "Three Tree Mascot",
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
fn costs(
    obs: &Observation,
    p: Player,
    card: Card,
    mut cost: ManaCost,
) -> (ManaCost, BTreeSet<&'static str>) {
    let mut applied = BTreeSet::new();
    let n = if RULES.contains(&card.name()) {
        u32::try_from(obs.creatures_attacked_this_turn).unwrap()
    } else {
        0
    };
    if n > 0 {
        applied.insert(card.name());
    }
    let _ = p;
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
fn score(obs: &Observation, a: &Action, prefer_raid: bool) -> i32 {
    let actor = obs.actor.unwrap();
    let raid = obs.players[actor.index()].attacked_this_turn;
    match a {
        Action::Concede | Action::Mulligan => -1000,
        Action::AnnounceCast { card } | Action::Cast { card, .. } => {
            let name = obs
                .objects
                .iter()
                .find(|o| o.id == *card)
                .unwrap()
                .card
                .name();
            let has_attacker = obs.objects.iter().any(|o| {
                o.owner == actor
                    && o.zone == Zone::Battlefield
                    && o.card.is_creature()
                    && !o.summoning_sick
            });
            // Reserve main-phase resources for raid entries in half the games.
            if prefer_raid
                && actor == obs.active
                && obs.turn >= 10
                && has_attacker
                && matches!(obs.step, rustmage::Step::Main1)
            {
                return -50;
            }
            if prefer_raid
                && actor == obs.active
                && RULES.contains(&name)
                && !matches!(obs.step, rustmage::Step::Main2)
            {
                return -50;
            }
            if RULES.contains(&name) && obs.creatures_attacked_this_turn == 0 {
                return -50;
            }
            if name == "Chart a Course" && prefer_raid && !raid {
                return -50;
            }
            if [
                "Fire Nation Raider",
                "Gorehorn Raider",
                "Skyship Buccaneer",
                "Storm Fleet Spy",
            ]
            .contains(&name)
                && !raid
            {
                if prefer_raid && has_attacker {
                    return -50;
                }
                return 70;
            }
            if matches!(a, Action::AnnounceCast { .. }) && NEW.contains(&name) {
                160
            } else {
                80
            }
        }
        Action::TapMana { .. } | Action::ActivateMana { .. } => {
            if obs.decision == "pay_costs" {
                140
            } else {
                -100
            }
        }
        Action::PlayLand { .. } => 150,
        Action::Attack { .. } | Action::AttackPlaneswalker { .. } => {
            if obs.turn >= 10 {
                60
            } else {
                -50
            }
        }
        Action::Block { .. } => 60,
        Action::ChooseTarget { target } => {
            let root = obs.announcing.as_ref().unwrap();
            match target {
                Target::Player(p) => {
                    if *p != actor {
                        180
                    } else {
                        20
                    }
                }
                Target::Card(id) => {
                    let o = obs.objects.iter().find(|o| o.id == *id).unwrap();
                    let harm = matches!(
                        root.source_card.name(),
                        "Witchstalker Frenzy" | "Gorehorn Raider" | "Shock"
                    );
                    if harm {
                        if o.owner != actor { 120 } else { -50 }
                    } else {
                        100
                    }
                }
                _ => 50,
            }
        }
        Action::AssignDamage { amount, .. } => {
            100 + i32::try_from(*amount).unwrap_or(i32::MAX - 100)
        }
        Action::PutTrigger { .. }
        | Action::FinishTargets
        | Action::PayCosts { .. }
        | Action::Keep
        | Action::Discard { .. }
        | Action::Bottom { .. }
        | Action::KeepLegend { .. } => 100,
        Action::Activate { .. } | Action::AnnounceActivate { .. } => -20,
        Action::Pass | Action::FinishAttackers | Action::FinishBlockers => 0,
        _ => 40,
    }
}
fn body(before: &Observation, after: &Observation, item: &SpellView) -> usize {
    let name = item.source_card.name();
    let p = item.controller.index();
    let trigger = matches!(item.kind, StackKind::TriggeredAbility { .. });
    if item.kind == StackKind::Spell && item.source_card.is_creature() {
        let stats = match name {
            "Fire Nation Raider" => (4, 2),
            "Gorehorn Raider" => (4, 4),
            "Skyship Buccaneer" => (4, 3),
            "Storm Fleet Spy" => (2, 2),
            _ => return 0,
        };
        let o = after.objects.iter().find(|o| o.id == item.card).unwrap();
        assert_eq!(o.zone, Zone::Battlefield);
        assert_eq!((o.power, o.toughness), stats);
        return 3;
    }
    if trigger {
        assert!(before.players[p].attacked_this_turn);
    }
    let draw = match (name, trigger) {
        ("Rowdy Research", false) => 3,
        ("Chart a Course", false) => 2,
        ("Skyship Buccaneer" | "Storm Fleet Spy", true) => 1,
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
        let discarded =
            usize::from(name == "Chart a Course" && !before.players[p].attacked_this_turn);
        assert_eq!(
            after.players[p].hand_count,
            before.players[p].hand_count + draw - discarded
        );
        return 3;
    }
    if name == "Fire Nation Raider" && trigger {
        let count = |o: &Observation| {
            o.objects
                .iter()
                .filter(|o| {
                    o.owner == item.controller
                        && o.zone == Zone::Battlefield
                        && o.card.token_info().is_some_and(|i| i.name == "Clue Token")
                })
                .count()
        };
        assert_eq!(count(after), count(before) + 1);
        return 1;
    }
    if name == "Witchstalker Frenzy" || name == "Gorehorn Raider" && trigger {
        let amount = if name == "Witchstalker Frenzy" { 5 } else { 2 };
        if let Some(bound) = item.targets.first() {
            match bound.target {
                Target::Player(p) => {
                    assert_eq!(
                        after.players[p.index()].life,
                        before.players[p.index()].life - amount
                    );
                    return 1;
                }
                Target::Card(id) => {
                    if let Some(old) = before.objects.iter().find(|o| {
                        o.id == id
                            && Some(o.reference) == bound.object
                            && o.zone == Zone::Battlefield
                    }) {
                        let new = after.objects.iter().find(|o| o.id == id).unwrap();
                        if old.damage + amount >= old.toughness
                            && !old.keywords.contains(Keywords::INDESTRUCTIBLE)
                        {
                            assert!(matches!(new.zone, Zone::Graveyard | Zone::Outside));
                        } else {
                            assert_eq!(new.zone, Zone::Battlefield);
                            assert_eq!(new.damage, old.damage + amount);
                        }
                        return 1;
                    }
                }
                _ => {}
            }
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
    let mut chart_branches = BTreeSet::new();
    let mut raid_casts = BTreeSet::new();
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
        let mut historical = BTreeSet::new();
        let mut raid_flags = [false; 2];
        for step in 0..100_000 {
            if game.outcome().is_some() {
                break;
            }
            assert!(step < 99_999, "game must terminate");
            let actor = game.actor().unwrap();
            let before = game.observe(actor);
            assert_eq!(before.creatures_attacked_this_turn, historical.len());
            assert_eq!(before.players[0].attacked_this_turn, raid_flags[0]);
            assert_eq!(before.players[1].attacked_this_turn, raid_flags[1]);
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
                .max_by_key(|(i, a)| (score(&before, a, seed % 2 == 0), std::cmp::Reverse(*i)))
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
            if matches!(a, Action::FinishAttackers) {
                for attack in &before.attack_declarations {
                    historical.insert(
                        before
                            .objects
                            .iter()
                            .find(|o| o.id == attack.card)
                            .unwrap()
                            .reference,
                    );
                }
                raid_flags[actor.index()] |= !before.attack_declarations.is_empty();
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
            if after.turn != before.turn {
                historical.clear();
                raid_flags = [false; 2];
            }
            assert_eq!(after.creatures_attacked_this_turn, historical.len());
            assert_eq!(
                game.observe(actor.other()).creatures_attacked_this_turn,
                historical.len()
            );
            checks += 5;

            for item in &after.stack {
                if item.kind == StackKind::Spell && NEW.contains(&item.source_card.name()) {
                    casted.insert(item.source_card.name());
                    if [
                        "Fire Nation Raider",
                        "Gorehorn Raider",
                        "Skyship Buccaneer",
                        "Storm Fleet Spy",
                    ]
                    .contains(&item.source_card.name())
                    {
                        raid_casts.insert((
                            item.source_card.name(),
                            after.players[item.controller.index()].attacked_this_turn,
                        ));
                    }
                }
            }
            if let Some(item) = before.stack.last()
                && after.stack.iter().all(|s| s.id != item.id)
                && NEW.contains(&item.source_card.name())
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
                && NEW.contains(&item.source_card.name())
            {
                let origin = origins.remove(&item.id).unwrap_or_else(|| before.clone());
                let n = body(&origin, &after, item);
                if n > 0 && item.source_card.name() == "Chart a Course" {
                    chart_branches
                        .insert(origin.players[item.controller.index()].attacked_this_turn);
                }
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
        assert_eq!(casted.len(), 7);
        assert!(
            RULES.into_iter().all(|name| applied.contains(name)),
            "missing rule from {applied:?}"
        );
        assert_eq!(chart_branches.len(), 2);
        for name in [
            "Fire Nation Raider",
            "Gorehorn Raider",
            "Skyship Buccaneer",
            "Storm Fleet Spy",
        ] {
            assert!(
                raid_casts.contains(&(name, false)) && raid_casts.contains(&(name, true)),
                "missing raid cast branch for {name}"
            );
            assert!(
                resolved
                    .iter()
                    .any(|(n, k)| *n == name && k.contains("TriggeredAbility")),
                "missing raid resolution for {name}"
            );
        }
    }
    println!(
        "Completed {games} legal Standard attack-history games, {decisions} decisions, {payments} staged payments, {checks} independent public checks, {branches} cloned/replayed boundaries, {max_bytes} maximum initialized state bytes; applied {applied:?}; cast {casted:?}; resolved {resolved:?}; Chart raid branches {chart_branches:?}; raid casts {raid_casts:?}; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
}
