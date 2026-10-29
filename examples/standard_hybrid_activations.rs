//! Legal Standard games checking committed activation equivalents and body replay.
use rustmage::{
    Action, Card, Config, DeckRules, Game, Keywords, ManaCost, Observation, Player, StackKind, Zone,
};
use std::collections::BTreeSet;
const NEW: [&str; 3] = [
    "Cindering Cutthroat",
    "Hovel Hurler",
    "Trostani, Three Whispers",
];
fn deck(n: usize) -> Vec<Card> {
    let (lands, spells): (&[(&str, usize)], &[&str]) = match n {
        0 => (
            &[("Plains", 6), ("Forest", 6), ("Mountain", 6), ("Swamp", 6)],
            &[
                "Cindering Cutthroat",
                "Hovel Hurler",
                "Trostani, Three Whispers",
                "Three Tree Mascot",
                "Ordinary Bear",
                "Highborn Vampire",
                "Hullcarver",
                "Llanowar Elves",
                "Gold Pan",
            ],
        ),
        1 => (
            &[("Plains", 12), ("Mountain", 12)],
            &[
                "Cindering Cutthroat",
                "Hovel Hurler",
                "Frilled Sparkshooter",
                "Fire Elemental",
                "Shivan Dragon",
                "Three Tree Mascot",
                "Gold Pan",
                "Hullcarver",
                "Savannah Lions",
            ],
        ),
        _ => (
            &[("Plains", 12), ("Forest", 12)],
            &[
                "Trostani, Three Whispers",
                "Hovel Hurler",
                "Three Tree Mascot",
                "Llanowar Elves",
                "Ordinary Bear",
                "Savannah Lions",
                "Hullcarver",
                "Gold Pan",
                "Wary Thespian",
            ],
        ),
    };
    let mut d = Vec::new();
    for &(name, n) in lands {
        d.extend(vec![Card::from_name(name).unwrap(); n]);
    }
    for &name in spells {
        d.extend([Card::from_name(name).unwrap(); 4]);
    }
    assert_eq!(d.len(), 60);
    assert!(d.iter().all(|c| c.is_supported() && c.is_standard_legal()));
    d
}
fn choose(
    g: &Game,
    o: &Observation,
    actions: &[Action],
    used: &BTreeSet<(rustmage::ObjectRef, u16)>,
    seed: usize,
) -> usize {
    let fallback = rustmage::policy::smoke_index(g, actions);
    let selected = o
        .cost_choices
        .get((seed + o.turn as usize) % o.cost_choices.len().max(1))
        .map(|c| c.index);
    actions
        .iter()
        .enumerate()
        .max_by_key(|(i, a)| match a {
            Action::Concede => -1000,
            Action::PlayLand { .. } => 200,
            Action::Cast { card, .. } | Action::AnnounceCast { card, .. } => {
                if NEW.contains(
                    &o.objects
                        .iter()
                        .find(|b| b.id == *card)
                        .unwrap()
                        .card
                        .name(),
                ) {
                    150
                } else {
                    100
                }
            }
            Action::AnnounceActivate { card, ability, .. }
            | Action::Activate { card, ability, .. }
            | Action::BeginActivate { card, ability, .. } => {
                let b = o.objects.iter().find(|b| b.id == *card).unwrap();
                if NEW.contains(&b.card.name()) && !used.contains(&(b.reference, *ability)) {
                    if matches!(a, Action::AnnounceActivate { .. }) {
                        180
                    } else {
                        80
                    }
                } else {
                    -100
                }
            }
            Action::ChooseActivationCost { cost } => {
                if Some(*cost) == selected {
                    200
                } else {
                    10
                }
            }
            Action::TapMana { .. } | Action::ActivateMana { .. } => {
                if o.decision == "pay_costs" {
                    140
                } else {
                    -100
                }
            }
            Action::ChooseTarget {
                target: rustmage::Target::Card(id),
            } => {
                if o.objects.iter().find(|b| b.id == *id).unwrap().owner == o.actor.unwrap() {
                    180
                } else {
                    90
                }
            }
            Action::Attack { .. } => {
                if o.turn >= 12 {
                    80
                } else {
                    -50
                }
            }
            Action::Block { .. } => 80,
            _ => {
                if *i == fallback {
                    60
                } else {
                    0
                }
            }
        })
        .unwrap()
        .0
}
fn main() {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse()
        .unwrap();
    assert!(games > 0);
    let (mut decisions, mut payments, mut replays, mut body_checks, mut max_bytes) =
        (0, 0, 0, 0, 0);
    let mut routes = BTreeSet::new();
    let mut entered = BTreeSet::new();
    let mut abilities = BTreeSet::new();
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [deck(seed % 3), deck((seed + 1) % 3)],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })
        .unwrap();
        let mut used = BTreeSet::new();
        for step in 0..100_000 {
            if g.outcome().is_some() {
                break;
            }
            assert!(step < 99_999, "game must terminate");
            let actor = g.actor().unwrap();
            let before = g.observe(actor);
            let actions = g.legal_actions();
            let a = actions[choose(&g, &before, &actions, &used, seed)].clone();
            if let Action::AnnounceActivate { card, ability, .. }
            | Action::Activate { card, ability, .. }
            | Action::BeginActivate { card, ability, .. } = a
            {
                let b = before.objects.iter().find(|b| b.id == card).unwrap();
                if NEW.contains(&b.card.name()) {
                    used.insert((b.reference, ability));
                }
            }
            if let Some(root) = before.announcing.as_ref()
                && let StackKind::ActivatedAbility { ability } = root.kind
                && root.source().is_some_and(|source| NEW.contains(&source.card.name()))
                && let Action::PayCosts { payment } = a
            {
                let request = before.payment_request.as_ref().unwrap();
                let cost = before.announcing_cost.unwrap_or(ManaCost {
                    colored: request.required,
                    generic: request.generic,
                });
                assert_eq!(request.required, cost.colored);
                assert_eq!(request.generic, cost.generic);
                assert_eq!(
                    payment.into_iter().sum::<u32>(),
                    cost.colored.into_iter().sum::<u32>() + cost.generic
                );
                assert!(
                    payment
                        .into_iter()
                        .zip(cost.colored)
                        .all(|(paid, required)| paid >= required)
                );
                routes.insert((root.source_card.name(), ability, cost.colored, cost.generic));
                payments += 1;
            }
            let saved = g.snapshot();
            let expected =
                if step % 4 == 0 || before.announcing.is_some() || !before.stack.is_empty() {
                    let mut fork = g.clone();
                    fork.apply(actor, a.clone()).unwrap();
                    replays += 1;
                    Some(fork.snapshot())
                } else {
                    None
                };
            g.apply(actor, a.clone()).unwrap();
            if let Some(e) = expected {
                assert_eq!(g.snapshot(), e);
                g.restore(&saved).unwrap();
                assert_eq!(
                    g.apply_index(actor, before.revision, 0),
                    Err(rustmage::Error::StaleRevision)
                );
                g.apply(actor, a).unwrap();
                assert_eq!(g.snapshot(), e);
            }
            let after = g.observe(actor);
            if after.turn != before.turn {
                used.clear();
            }
            for b in after
                .objects
                .iter()
                .filter(|b| b.zone == Zone::Battlefield && NEW.contains(&b.card.name()))
            {
                if before
                    .objects
                    .iter()
                    .all(|old| old.reference != b.reference)
                {
                    entered.insert(b.card.name());
                }
            }
            if let Some(root) = before.stack.last()
                && let StackKind::ActivatedAbility { ability } = root.kind
                && root.source().is_some_and(|source| NEW.contains(&source.card.name()))
                && after.stack.iter().all(|s| s.id != root.id)
                && after.resolving.is_none()
            {
                let target = if root.source().is_some_and(|source| source.card.name() == "Cindering Cutthroat") {
                    Some(root.object)
                } else {
                    root.targets.first().and_then(|t| t.object)
                };
                if let Some(reference) = target
                    && let Some(object) = after
                        .objects
                        .iter()
                        .find(|b| b.reference == reference && b.zone == Zone::Battlefield)
                {
                    let keyword = match (root.source_card.name(), ability) {
                        ("Cindering Cutthroat", 0) => Keywords::MENACE,
                        ("Hovel Hurler", 0) => Keywords::FLYING,
                        ("Trostani, Three Whispers", 0) => Keywords::DEATHTOUCH,
                        ("Trostani, Three Whispers", 1) => Keywords::VIGILANCE,
                        ("Trostani, Three Whispers", 2) => Keywords::DOUBLE_STRIKE,
                        _ => unreachable!(),
                    };
                    assert!(object.keywords.contains(keyword));
                    if root.source().is_some_and(|source| source.card.name() == "Hovel Hurler") {
                        let old = before
                            .objects
                            .iter()
                            .find(|b| b.reference == reference)
                            .unwrap();
                        assert_eq!(object.power, old.power + 1);
                        assert_eq!(object.toughness, old.toughness);
                    }
                    abilities.insert((root.source_card.name(), ability));
                    body_checks += 1;
                }
            }
            max_bytes = max_bytes.max(g.state_bytes());
            decisions += 1;
        }
        assert!(g.outcome().is_some());
    }
    if games >= 100 {
        for name in NEW {
            assert!(entered.contains(name));
        }
        for (name, index) in [
            ("Cindering Cutthroat", 0),
            ("Hovel Hurler", 0),
            ("Trostani, Three Whispers", 0),
            ("Trostani, Three Whispers", 1),
            ("Trostani, Three Whispers", 2),
        ] {
            assert!(abilities.contains(&(name, index)));
        }
        for (name, index, costs) in [
            (
                "Cindering Cutthroat",
                0,
                vec![[0, 0, 1, 0, 0, 0], [0, 0, 0, 1, 0, 0]],
            ),
            (
                "Hovel Hurler",
                0,
                vec![[2, 0, 0, 0, 0, 0], [1, 0, 0, 1, 0, 0], [0, 0, 0, 2, 0, 0]],
            ),
            (
                "Trostani, Three Whispers",
                1,
                vec![[1, 0, 0, 0, 0, 0], [0, 0, 0, 0, 1, 0]],
            ),
        ] {
            for cost in costs {
                assert!(
                    routes
                        .iter()
                        .any(|(n, i, c, _)| *n == name && *i == index && *c == cost),
                    "missing {name} {cost:?}"
                );
            }
        }
    }
    println!(
        "{}",
        serde_json::json!({"games":games,"decisions":decisions,"activation_payments":payments,"body_checks":body_checks,"clone_restore_replays":replays,"max_initialized_state_bytes":max_bytes,"entries":entered,"resolved_abilities":abilities,"paid_cost_routes":routes})
    );
}
