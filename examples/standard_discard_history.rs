//! Complete Standard games checking public discard totals and resolution-time draws.
use rustmage::{
    Action, Card, Config, DeckRules, Game, ObjectRef, Observation, Player, StackKind, Zone,
};
use std::collections::BTreeSet;
const MISTY: &str = "Misty Knight, Hero for Hire";
const GOBLIN: &str = "Green Goblin, Revenant";
fn deck() -> Vec<Card> {
    let mut d = vec![Card::Mountain; 12];
    d.extend([Card::Swamp; 12]);
    for name in [
        MISTY,
        GOBLIN,
        "Charging Strifeknight",
        "Marauding Mako",
        "Masked Meower",
        "Murmuring Volume",
        "Thrill of Possibility",
        "Shock",
        "Gold Pan",
    ] {
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
    used: &BTreeSet<(ObjectRef, u16)>,
) -> usize {
    let fallback = rustmage::policy::smoke_index(g, actions);
    actions
        .iter()
        .enumerate()
        .max_by_key(|(i, a)| match a {
            Action::Concede => -1000,
            Action::PlayLand { .. } => 220,
            Action::Cast { .. } | Action::AnnounceCast { .. } => 190,
            Action::AnnounceActivate { card, ability, .. } => {
                let b = o.objects.iter().find(|b| b.id == *card).unwrap();
                if !used.contains(&(b.reference, *ability)) {
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
                if o.objects
                    .iter()
                    .any(|b| b.reference == *object && b.card.is_land())
                {
                    210
                } else {
                    200
                }
            }
            Action::Attack { card, .. } => {
                if o.objects
                    .iter()
                    .any(|b| b.id == *card && b.card.name() == GOBLIN)
                    || o.turn >= 14
                {
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
    let (mut decisions, mut checks, mut replays, mut max_bytes, mut draws) = (0, 0, 0, 0, 0);
    let mut entered = BTreeSet::new();
    let mut bodies = BTreeSet::new();
    let mut amounts = BTreeSet::new();
    let mut causes = BTreeSet::new();
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [deck(), deck()],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })
        .unwrap();
        let mut used = BTreeSet::new();
        let mut model = [0u64; 2];
        for step in 0..100_000 {
            if g.outcome().is_some() {
                break;
            }
            assert!(step < 99_999, "game must terminate");
            let actor = g.actor().unwrap();
            let before = g.observe(actor);
            let actions = g.legal_actions();
            let a = actions[choose(&g, &before, &actions, &used)].clone();
            if let Action::AnnounceActivate { card, ability, .. } = a {
                let b = before.objects.iter().find(|b| b.id == card).unwrap();
                used.insert((b.reference, ability));
            }
            let root = before
                .stack
                .last()
                .filter(|s| {
                    s.source().is_some_and(|source| source.card.name() == MISTY)
                        && matches!(s.kind, StackKind::ActivatedAbility { .. })
                        || s.source()
                            .is_some_and(|source| source.card.name() == GOBLIN)
                            && matches!(s.kind, StackKind::TriggeredAbility { .. })
                })
                .cloned();
            let resolving = before
                .resolving
                .as_ref()
                .filter(|s| {
                    s.source()
                        .is_some_and(|source| source.card.name() == GOBLIN)
                })
                .cloned();
            let discard_action = matches!(a, Action::PayCosts { .. } | Action::Discard { .. });
            let saved = g.snapshot();
            let expected = if step % 4 == 0
                || before.announcing.is_some()
                || before.resolving.is_some()
                || !before.stack.is_empty()
            {
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
                g.apply(actor, a.clone()).unwrap();
                assert_eq!(g.snapshot(), expected);
            }
            let after = g.observe(actor);
            if discard_action {
                let discarded = before
                    .objects
                    .iter()
                    .filter(|b| {
                        b.owner == actor
                            && b.zone == Zone::Hand
                            && after
                                .objects
                                .iter()
                                .any(|n| n.id == b.id && n.zone == Zone::Graveyard)
                    })
                    .count() as u64;
                model[actor.index()] += discarded;
                if discarded > 0 {
                    causes.insert(if matches!(a, Action::PayCosts { .. }) {
                        "cost"
                    } else if before.decision == "discard" {
                        "cleanup"
                    } else {
                        "effect"
                    });
                }
            }
            if before.turn != after.turn {
                model = [0; 2];
                used.clear();
            }
            for p in [Player::P0, Player::P1] {
                let view = g.observe(p);
                for (n, expected) in model.iter().enumerate() {
                    assert_eq!(view.players[n].cards_discarded_this_turn, *expected);
                    checks += 1;
                }
            }
            for b in after
                .objects
                .iter()
                .filter(|b| b.zone == Zone::Battlefield && matches!(b.card.name(), MISTY | GOBLIN))
            {
                if before
                    .objects
                    .iter()
                    .all(|old| old.reference != b.reference)
                {
                    entered.insert(b.card.name());
                }
            }
            let completed = root
                .filter(|s| after.stack.iter().all(|n| n.id != s.id) && after.resolving.is_none())
                .map(|s| {
                    (
                        s.source()
                            .expect("example expects an object-backed packet")
                            .card
                            .name(),
                        s.controller,
                    )
                })
                .or_else(|| {
                    resolving.filter(|_| after.resolving.is_none()).map(|s| {
                        (
                            s.source()
                                .expect("example expects an object-backed packet")
                                .card
                                .name(),
                            s.controller,
                        )
                    })
                });
            if let Some((name, p)) = completed {
                let amount = after.players[p.index()].cards_discarded_this_turn;
                let expected = amount.min(before.players[p.index()].library_count as u64);
                assert_eq!(
                    after.players[p.index()].cards_drawn_this_turn,
                    before.players[p.index()].cards_drawn_this_turn + expected
                );
                draws += 1;
                bodies.insert(name);
                amounts.insert(amount);
            }
            decisions += 1;
            max_bytes = max_bytes.max(g.state_bytes());
        }
        assert!(g.outcome().is_some());
    }
    if games >= 100 {
        for name in [MISTY, GOBLIN] {
            assert!(entered.contains(name));
            assert!(bodies.contains(name));
        }
        assert!(amounts.iter().any(|n| *n > 1));
        assert_eq!(causes, BTreeSet::from(["cost", "effect", "cleanup"]));
    }
    println!(
        "{}",
        serde_json::json!({"games":games,"decisions":decisions,"history_checks":checks,"scaled_draw_checks":draws,"clone_restore_replays":replays,"max_initialized_state_bytes":max_bytes,"entries":entered,"resolved_bodies":bodies,"draw_history_amounts":amounts,"discard_causes":causes})
    );
}
