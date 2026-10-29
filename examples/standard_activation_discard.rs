//! Legal Standard self-play with owned discard costs, exact effects and replay checks.
use rustmage::{
    Action, Card, Config, DeckRules, Game, Keywords, ObjectRef, Observation, Player, StackKind,
    Zone,
};
use std::collections::BTreeSet;
const NEW: [&str; 7] = [
    "Charging Strifeknight",
    "Flamecache Gecko",
    "Inner Demons Gangsters",
    "Iron-Shield Elf",
    "Masked Meower",
    "Murmuring Volume",
    "Thunderhead Gunner",
];
fn deck() -> Vec<Card> {
    let mut d = vec![Card::Mountain; 12];
    d.extend([Card::Swamp; 12]);
    for name in NEW.into_iter().chain(["Marauding Mako", "Shock"]) {
        d.extend([Card::from_name(name).unwrap(); 4]);
    }
    assert_eq!(d.len(), 60);
    assert!(d.iter().all(|c| c.is_supported() && c.is_standard_legal()));
    d
}
fn is_new(name: &str, ability: u16) -> bool {
    NEW.contains(&name) && ability == u16::from(name == "Murmuring Volume")
}
fn choose(
    g: &Game,
    o: &Observation,
    actions: &[Action],
    used: &BTreeSet<(ObjectRef, u16)>,
    seed: usize,
) -> usize {
    let fallback = rustmage::policy::smoke_index(g, actions);
    actions
        .iter()
        .enumerate()
        .max_by_key(|(i, a)| match a {
            Action::Concede => -1000,
            Action::PlayLand { .. } => 220,
            Action::Cast { card, target, .. }
                if o.objects
                    .iter()
                    .any(|b| b.id == *card && b.card.name() == "Shock") =>
            {
                if *target == Some(rustmage::Target::Player(o.actor.unwrap().other())) {
                    if seed % 2 == 1
                        && o.players[o.actor.unwrap().other().index()].life_lost_this_turn == 0
                    {
                        250
                    } else {
                        170
                    }
                } else {
                    -100
                }
            }
            Action::AnnounceCast { card }
                if o.objects
                    .iter()
                    .any(|b| b.id == *card && b.card.name() == "Shock") =>
            {
                if seed % 2 == 1
                    && o.players[o.actor.unwrap().other().index()].life_lost_this_turn == 0
                    && o.objects
                        .iter()
                        .filter(|b| {
                            b.owner == o.actor.unwrap()
                                && b.zone == Zone::Battlefield
                                && b.card.is_land()
                        })
                        .count()
                        >= 3
                {
                    250
                } else {
                    180
                }
            }
            Action::ChooseTarget {
                target: rustmage::Target::Player(p),
            } if *p == o.actor.unwrap().other() => 230,
            Action::Cast { card, .. } | Action::AnnounceCast { card }
                if seed % 2 == 1
                    && o.objects
                        .iter()
                        .any(|b| b.id == *card && b.card.name() == "Flamecache Gecko")
                    && o.players[o.actor.unwrap().other().index()].life_lost_this_turn == 0 =>
            {
                -100
            }
            Action::Cast { .. } | Action::AnnounceCast { .. } => 190,
            Action::AnnounceActivate { card, ability, .. } => {
                let b = o.objects.iter().find(|b| b.id == *card).unwrap();
                if is_new(b.card.name(), *ability) && !used.contains(&(b.reference, *ability)) {
                    180
                } else {
                    -100
                }
            }
            Action::Activate { .. }
            | Action::BeginActivate { .. }
            | Action::AnnounceMana { .. } => -100,
            Action::ActivateMana { .. } | Action::TapMana { .. } => {
                if o.decision == "pay_costs" {
                    170
                } else {
                    -100
                }
            }
            Action::ChooseAdditionalCost { object } => {
                let b = o.objects.iter().find(|b| b.reference == *object).unwrap();
                if b.card.is_land() { 210 } else { 200 }
            }
            Action::Attack { .. } => {
                if o.turn >= 14 {
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
    let (mut decisions, mut payments, mut checks, mut replays, mut max_bytes) = (0, 0, 0, 0, 0);
    let mut entries = BTreeSet::new();
    let mut bodies = BTreeSet::new();
    let mut entry_branches = BTreeSet::new();
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [deck(), deck()],
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
            if let Action::AnnounceActivate { card, ability, .. } = a {
                let b = before.objects.iter().find(|b| b.id == card).unwrap();
                if is_new(b.card.name(), ability) {
                    used.insert((b.reference, ability));
                }
            }
            let paid = if let Some(root) = before.announcing.as_ref()
                && let StackKind::ActivatedAbility { ability } = root.kind
                && is_new(root.source().expect("example expects an object-backed packet").card.name(), ability)
                && let Action::PayCosts { payment } = a
            {
                let v = before.additional_cost.as_ref().unwrap();
                assert!(!v.paid);
                let selected = v.selected.unwrap();
                let b = before
                    .objects
                    .iter()
                    .find(|b| b.reference == selected)
                    .unwrap();
                assert_eq!(b.owner, actor);
                assert_eq!(b.zone, Zone::Hand);
                let request = before.payment_request.as_ref().unwrap();
                assert_eq!(
                    payment.iter().sum::<u32>(),
                    request.required.iter().sum::<u32>() + request.generic
                );
                assert!(
                    payment
                        .iter()
                        .zip(request.required)
                        .all(|(paid, need)| *paid >= need)
                );
                assert!(g.observe(actor.other()).additional_cost.is_none());
                payments += 1;
                Some((selected, root.clone()))
            } else {
                None
            };
            let saved = g.snapshot();
            let expected =
                if step % 4 == 0 || before.announcing.is_some() || !before.stack.is_empty() {
                    let mut f = g.clone();
                    f.apply(actor, a.clone()).unwrap();
                    replays += 1;
                    Some(f.snapshot())
                } else {
                    None
                };
            g.apply(actor, a.clone()).unwrap();
            if let Some(expected) = expected {
                assert_eq!(g.snapshot(), expected);
                g.restore(&saved).unwrap();
                assert_eq!(
                    g.apply_index(actor, before.revision, 0),
                    Err(rustmage::Error::StaleRevision)
                );
                g.apply(actor, a).unwrap();
                assert_eq!(g.snapshot(), expected);
            }
            let after = g.observe(actor);
            if after.turn != before.turn {
                used.clear();
            }
            if let Some((selected, root)) = paid {
                assert!(
                    after
                        .objects
                        .iter()
                        .any(|b| b.id == selected.card && b.zone == Zone::Graveyard)
                );
                if root.source().is_some_and(|source| source.card.name() == "Masked Meower") {
                    assert!(
                        after
                            .objects
                            .iter()
                            .any(|b| b.id == root.card && b.zone == Zone::Graveyard)
                    );
                }
                if matches!(
                    root.source().expect("example expects an object-backed packet").card.name(),
                    "Charging Strifeknight" | "Murmuring Volume"
                ) {
                    assert!(
                        after
                            .objects
                            .iter()
                            .any(|b| b.reference == root.object && b.tapped)
                    );
                }
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
                    entries.insert(b.card.name());
                    if b.card.name() == "Flamecache Gecko" {
                        entry_branches.insert(
                            before.players[b.owner.other().index()].life_lost_this_turn > 0,
                        );
                    }
                }
            }
            if let Some(root) = before.stack.last()
                && let StackKind::ActivatedAbility { ability } = root.kind
                && is_new(root.source().expect("example expects an object-backed packet").card.name(), ability)
                && after.stack.iter().all(|s| s.id != root.id)
                && after.resolving.is_none()
            {
                let name = root.source().expect("example expects an object-backed packet").card.name();
                let p = root.controller.index();
                if matches!(name, "Inner Demons Gangsters" | "Iron-Shield Elf") {
                    if let Some(b) = after
                        .objects
                        .iter()
                        .find(|b| b.reference == root.object && b.zone == Zone::Battlefield)
                    {
                        let old = before
                            .objects
                            .iter()
                            .find(|b| b.reference == root.object)
                            .unwrap();
                        if name == "Inner Demons Gangsters" {
                            assert_eq!(b.power, old.power + 1);
                            assert_eq!(b.toughness, old.toughness);
                            assert!(b.keywords.contains(Keywords::MENACE));
                        } else {
                            assert!(b.keywords.contains(Keywords::INDESTRUCTIBLE));
                            assert!(b.tapped);
                        }
                        checks += 1;
                        bodies.insert(name);
                    }
                } else {
                    assert_eq!(
                        after.players[p].cards_drawn_this_turn,
                        before.players[p].cards_drawn_this_turn
                            + u64::from(before.players[p].library_count > 0)
                    );
                    checks += 1;
                    bodies.insert(name);
                }
            }
            // Gecko's entry-trigger mana resolves on the stack, independently of its draw activation.
            if let Some(root) = before.stack.last()
                && root.source().is_some_and(|source| source.card.name() == "Flamecache Gecko")
                && matches!(root.kind, StackKind::TriggeredAbility { .. })
                && after.stack.iter().all(|s| s.id != root.id)
                && after.resolving.is_none()
            {
                let p = root.controller.index();
                let mut expected = before.players[p].mana;
                expected[2] += 1;
                expected[3] += 1;
                assert_eq!(after.players[p].mana, expected);
            }
            max_bytes = max_bytes.max(g.state_bytes());
            decisions += 1;
        }
        assert!(g.outcome().is_some());
    }
    if games >= 100 {
        for name in NEW {
            assert!(entries.contains(name), "missing entry {name}");
            assert!(bodies.contains(name), "missing body {name}");
        }
        assert_eq!(entry_branches, BTreeSet::from([false, true]));
    }
    println!(
        "{}",
        serde_json::json!({"games":games,"decisions":decisions,"activation_payments":payments,"body_checks":checks,"clone_restore_replays":replays,"max_initialized_state_bytes":max_bytes,"entries":entries,"resolved_abilities":bodies,"gecko_entry_branches":entry_branches})
    );
}
