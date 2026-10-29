//! Legal Standard games with restricted activation timing and typed recovery.
use rustmage::{
    Action, Card, Config, DeckRules, Game, Keywords, ObjectRef, Observation, Player, StackKind,
    Target, Zone,
};
use std::collections::BTreeSet;
const ZEI: &str = "Professor Zei, Anthropologist";
const RESCUE: &str = "Rescue Girl, First Responder";
const DOWSER: &str = "Shipwreck Dowser";
const LORE: &str = "Zealous Lorecaster";
const NEW: [&str; 4] = [ZEI, RESCUE, DOWSER, LORE];
fn deck() -> Vec<Card> {
    let mut d = vec![Card::Mountain; 8];
    d.extend([Card::Island; 8]);
    d.extend([Card::Plains; 8]);
    for name in NEW.into_iter().chain([
        "Shock",
        "Ancestral Reminiscence",
        "Marauding Mako",
        "Masked Meower",
        "Gold Pan",
    ]) {
        d.extend([Card::from_name(name).unwrap(); 4]);
    }
    assert_eq!(d.len(), 60);
    assert!(d.iter().all(|c| c.is_supported() && c.is_standard_legal()));
    d
}
fn restricted(name: &str, ability: u16) -> bool {
    name == RESCUE || name == ZEI && ability == 1
}
fn choose(
    g: &Game,
    o: &Observation,
    actions: &[Action],
    used: &BTreeSet<(ObjectRef, u16)>,
    seed: usize,
) -> usize {
    let fallback = rustmage::policy::smoke_index(g, actions);
    let preserve_token = seed.is_multiple_of(3)
        && o.objects.iter().any(|b| {
            b.owner == o.actor.unwrap() && b.zone == Zone::Battlefield && b.card.name() == RESCUE
        })
        && o.objects.iter().any(|b| {
            b.owner == o.actor.unwrap() && b.zone == Zone::Battlefield && b.card.is_token()
        });

    actions
        .iter()
        .enumerate()
        .max_by_key(|(i, a)| match a {
            Action::Concede => -1000,
            Action::Pass
                if o.stack.last().is_some_and(|s| {
                    s.source()
                        .is_some_and(|source| source.card.name() == RESCUE)
                        && matches!(s.kind, StackKind::ActivatedAbility { .. })
                }) =>
            {
                260
            }
            Action::PlayLand { .. } => 230,
            Action::Cast { .. } | Action::AnnounceCast { .. } if preserve_token => -100,
            Action::Cast { card, target, .. }
                if o.objects
                    .iter()
                    .any(|b| b.id == *card && b.card.name() == "Shock") =>
            {
                if *target == Some(Target::Player(o.actor.unwrap().other())) {
                    190
                } else {
                    -100
                }
            }
            Action::Cast { .. } | Action::AnnounceCast { .. } => 190,
            Action::AnnounceActivate { card, ability, .. } => {
                let b = o.objects.iter().find(|b| b.id == *card).unwrap();
                if preserve_token && b.card.name() != RESCUE
                    || b.card.name() == RESCUE
                        && o.stack.iter().any(|s| {
                            s.source()
                                .is_some_and(|source| source.card.name() == "Gold Pan")
                        })
                {
                    -100
                } else if !used.contains(&(b.reference, *ability)) {
                    if b.card.name() == RESCUE
                        && o.objects.iter().any(|t| {
                            t.owner == o.actor.unwrap()
                                && t.zone == Zone::Battlefield
                                && t.card.is_token()
                        })
                    {
                        245
                    } else {
                        180
                    }
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
            Action::ChooseTarget {
                target: Target::Card(id),
            } => {
                let b = o.objects.iter().find(|b| b.id == *id).unwrap();
                if b.zone == Zone::Graveyard {
                    if (b.card.name() == "Ancestral Reminiscence") == seed.is_multiple_of(2) {
                        220
                    } else {
                        210
                    }
                } else if b.owner == o.actor.unwrap() {
                    if b.card.is_token() && seed.is_multiple_of(3) {
                        225
                    } else if b.card.is_land() {
                        if seed.is_multiple_of(5) { 220 } else { 200 }
                    } else {
                        215
                    }
                } else {
                    0
                }
            }
            Action::ChooseTarget {
                target: Target::Player(p),
            } if *p == o.actor.unwrap().other() => 220,
            Action::ChooseAdditionalCost { object } => {
                if o.objects
                    .iter()
                    .any(|b| b.reference == *object && b.card.is_land())
                {
                    210
                } else {
                    200
                }
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
    let (
        mut decisions,
        mut replays,
        mut timing_checks,
        mut recovery_checks,
        mut loot_checks,
        mut prowess_checks,
        mut max_bytes,
    ) = (0, 0, 0, 0, 0, 0, 0);
    let mut entered = BTreeSet::new();
    let mut recovered = BTreeSet::new();
    let mut recovered_types = BTreeSet::new();
    let mut activated = BTreeSet::new();
    let mut permanent_routes = BTreeSet::new();
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
            for a in &actions {
                if let Action::AnnounceActivate { card, ability, .. }
                | Action::Activate { card, ability, .. }
                | Action::BeginActivate { card, ability, .. } = a
                {
                    let b = before.objects.iter().find(|b| b.id == *card).unwrap();
                    if restricted(b.card.name(), *ability) {
                        assert_eq!(before.active, actor);
                        timing_checks += 1;
                    }
                }
            }
            let a = actions[choose(&g, &before, &actions, &used, seed)].clone();
            if let Action::AnnounceActivate { card, ability, .. } = a {
                let b = before.objects.iter().find(|b| b.id == card).unwrap();
                used.insert((b.reference, ability));
                if NEW.contains(&b.card.name()) {
                    activated.insert((b.card.name(), ability));
                }
            }
            let cost = before
                .announcing
                .as_ref()
                .filter(|_| matches!(a, Action::PayCosts { .. }))
                .cloned();
            let root = before
                .stack
                .last()
                .filter(|s| {
                    s.source()
                        .is_some_and(|source| NEW.contains(&source.card.name()))
                        && s.kind != StackKind::Spell
                })
                .cloned();
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
            if let Some(s) = cost
                && let StackKind::ActivatedAbility { ability } = s.kind
            {
                if s.source().is_some_and(|source| source.card.name() == ZEI) && ability == 1 {
                    assert!(after.objects.iter().any(|b| {
                        b.id == s
                            .source()
                            .expect("example expects an object-backed packet")
                            .id
                            && b.zone == Zone::Graveyard
                    }));
                }
                if s.source()
                    .is_some_and(|source| source.card.name() == RESCUE)
                {
                    assert!(after.objects.iter().any(|b| {
                        b.reference
                            == s.source()
                                .expect("example expects an object-backed packet")
                                .object
                            && b.tapped
                    }));
                }
            }
            if let Some(s) = root
                && after.stack.iter().all(|n| n.id != s.id)
                && after.resolving.is_none()
            {
                if let Some(target) = s.targets.first().and_then(|t| t.object) {
                    if let Some(old) = before.objects.iter().find(|b| b.reference == target) {
                        let zone = if s
                            .source()
                            .is_some_and(|source| source.card.name() == RESCUE)
                        {
                            Zone::Battlefield
                        } else {
                            Zone::Graveyard
                        };
                        if old.zone == zone && old.owner == s.controller {
                            if zone == Zone::Graveyard {
                                assert!(matches!(old.card.info().type_line, "Instant" | "Sorcery"));
                                recovered_types.insert(old.card.info().type_line);
                            } else {
                                assert_ne!(
                                    old.reference,
                                    s.source()
                                        .expect("example expects an object-backed packet")
                                        .object
                                );
                            }
                            if old.card.is_token() {
                                assert!(!after.objects.iter().any(|b| b.id == target.card));
                                permanent_routes.insert("token");
                            } else {
                                let owner_view = g.observe(s.controller);
                                assert!(
                                    owner_view
                                        .objects
                                        .iter()
                                        .any(|b| b.id == target.card && b.zone == Zone::Hand)
                                );
                                if zone == Zone::Battlefield {
                                    permanent_routes.insert(if old.card.is_land() {
                                        "land"
                                    } else {
                                        "nonland"
                                    });
                                }
                            }
                            recovery_checks += 1;
                            recovered.insert(
                                s.source()
                                    .expect("example expects an object-backed packet")
                                    .card
                                    .name(),
                            );
                        }
                    }
                } else if s.source().is_some_and(|source| source.card.name() == ZEI) {
                    let p = s.controller.index();
                    assert_eq!(
                        after.players[p].cards_drawn_this_turn,
                        before.players[p].cards_drawn_this_turn
                            + u64::from(before.players[p].library_count > 0)
                    );
                    loot_checks += 1;
                } else if s
                    .source()
                    .is_some_and(|source| source.card.name() == DOWSER)
                    && let Some(b) = after.objects.iter().find(|b| {
                        b.reference
                            == s.source()
                                .expect("example expects an object-backed packet")
                                .object
                            && b.zone == Zone::Battlefield
                    })
                {
                    let old = before
                        .objects
                        .iter()
                        .find(|b| {
                            b.reference
                                == s.source()
                                    .expect("example expects an object-backed packet")
                                    .object
                        })
                        .unwrap();
                    assert_eq!(b.power, old.power + 1);
                    assert_eq!(b.toughness, old.toughness + 1);
                    prowess_checks += 1;
                }
            }
            for b in after
                .objects
                .iter()
                .filter(|b| b.zone == Zone::Battlefield && b.card.name() == RESCUE)
            {
                assert!(b.keywords.contains(Keywords::FLYING));
            }
            decisions += 1;
            max_bytes = max_bytes.max(g.state_bytes());
        }
        assert!(g.outcome().is_some());
    }
    if games >= 100 {
        for name in NEW {
            assert!(entered.contains(name), "missing entry {name}");
            assert!(recovered.contains(name), "missing recovery {name}");
        }
        for p in [(ZEI, 0), (ZEI, 1), (RESCUE, 0)] {
            assert!(activated.contains(&p));
        }
        assert!(loot_checks > 0 && prowess_checks > 0);
        assert_eq!(recovered_types, BTreeSet::from(["Instant", "Sorcery"]));
        assert_eq!(
            permanent_routes,
            BTreeSet::from(["land", "nonland", "token"])
        );
    }
    println!(
        "{}",
        serde_json::json!({"games":games,"decisions":decisions,"timing_checks":timing_checks,"recovery_checks":recovery_checks,"loot_checks":loot_checks,"prowess_checks":prowess_checks,"clone_restore_replays":replays,"max_initialized_state_bytes":max_bytes,"entries":entered,"recovered_bodies":recovered,"recovered_types":recovered_types,"activated":activated,"permanent_return_routes":permanent_routes})
    );
}
