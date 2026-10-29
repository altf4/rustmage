//! Complete Standard games with independent block-pair checks and full body replay.
use rustmage::{
    Action, Card, Config, DeckRules, Game, Keywords, ObjectRef, Observation, Player, StackKind,
    Zone,
};
use std::collections::BTreeSet;
const APRIL: &str = "April O'Neil, Kunoichi Trainee";
const PET: &str = "Prehistoric Pet";
const STOMPER: &str = "Cavern Stomper";
const OUTRIDER: &str = "Verdant Outrider";
const NEW: [&str; 4] = [APRIL, PET, STOMPER, OUTRIDER];
fn deck() -> Vec<Card> {
    let mut d = vec![Card::Plains; 12];
    d.extend([Card::Forest; 12]);
    for name in NEW.into_iter().chain([
        "Healer's Hawk",
        "Savannah Lions",
        "Ordinary Bear",
        "Large Bear",
        "Llanowar Elves",
    ]) {
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
            Action::PlayLand { .. } => 230,
            Action::Cast { .. } | Action::AnnounceCast { .. } => 190,
            Action::AnnounceActivate { card, ability, .. } => {
                let b = o.objects.iter().find(|b| b.id == *card).unwrap();
                if used.contains(&(b.reference, *ability)) {
                    -100
                } else if matches!(b.card.name(), STOMPER | OUTRIDER)
                    && !b.keywords.contains(Keywords::NO_POWER_LE_TWO_BLOCKERS)
                {
                    210
                } else if b.card.name() == PET && o.turn.is_multiple_of(4) {
                    200
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
                target: rustmage::Target::Card(id),
            } => {
                let b = o.objects.iter().find(|b| b.id == *id).unwrap();
                if b.owner == o.actor.unwrap() {
                    220 - b.power.clamp(0, 100) as i32
                } else {
                    -100
                }
            }
            Action::Attack { .. } => {
                if o.turn >= 10 {
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
fn pair(a: &rustmage::ObjectView, b: &rustmage::ObjectView) -> bool {
    let ak = a.keywords;
    let bk = b.keywords;
    !ak.contains(Keywords::CANT_BE_BLOCKED)
        && (!ak.contains(Keywords::NO_TOKEN_BLOCKERS) || !b.card.is_token())
        && (!ak.contains(Keywords::FLYING)
            || bk.intersects(Keywords::FLYING.union(Keywords::REACH)))
        && (!ak.intersects(Keywords::SKULK.union(Keywords::NO_GREATER_POWER_BLOCKERS))
            || b.power <= a.power)
        && (!ak.contains(Keywords::NO_POWER_LE_TWO_BLOCKERS) || b.power > 2)
        && (!ak.contains(Keywords::NO_POWER_GE_THREE_BLOCKERS) || b.power < 3)
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
        mut pair_checks,
        mut replays,
        mut grants,
        mut returns,
        mut scry_choices,
        mut rejected,
        mut max_bytes,
    ) = (0, 0, 0, 0, 0, 0, 0, 0);
    let mut entries = BTreeSet::new();
    let mut abilities = BTreeSet::new();
    let mut outcomes = BTreeSet::new();
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [deck(), deck()],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })
        .unwrap();
        let mut used = BTreeSet::new();
        for step in 0usize..100_000 {
            if g.outcome().is_some() {
                break;
            }
            assert!(step < 99_999, "game must terminate");
            let actor = g.actor().unwrap();
            let before = g.observe(actor);
            let actions = g.legal_actions();
            if before.decision == "blockers" {
                for id in &before.attackers {
                    let a = before.objects.iter().find(|b| b.id == *id).unwrap();
                    for b in before.objects.iter().filter(|b| {
                        b.owner == actor
                            && b.zone == Zone::Battlefield
                            && b.card.is_creature()
                            && !b.tapped
                            && !b.keywords.contains(Keywords::CANT_BLOCK)
                            && before.blocks.iter().all(|(id, _)| *id != b.id)
                    }) {
                        // This workload has no menace/minimum/maximum blocker abilities.
                        let expected = pair(a, b);
                        let block = Action::Block {
                            blocker: b.id,
                            attacker: a.id,
                        };
                        assert_eq!(actions.contains(&block), expected);
                        pair_checks += 1;
                        if NEW.contains(&a.card.name())
                            && (matches!(a.card.name(), APRIL | PET)
                                || a.keywords.contains(Keywords::NO_POWER_LE_TWO_BLOCKERS))
                        {
                            outcomes.insert((a.card.name(), expected));
                        }
                        if !expected && step.is_multiple_of(17) {
                            let saved = g.snapshot();
                            let revision = g.revision();
                            assert_eq!(g.apply(actor, block), Err(rustmage::Error::IllegalAction));
                            assert_eq!(g.snapshot(), saved);
                            assert_eq!(g.revision(), revision);
                            rejected += 1;
                        }
                    }
                }
            }
            let a = actions[choose(&g, &before, &actions, &used)].clone();
            if let Action::AnnounceActivate { card, ability, .. } = a {
                let b = before.objects.iter().find(|b| b.id == card).unwrap();
                used.insert((b.reference, ability));
            }
            if matches!(a, Action::ArrangeLibrary { .. })
                && before
                    .resolving
                    .as_ref()
                    .is_some_and(|s| s.source().is_some_and(|source| matches!(source.card.name(), APRIL | STOMPER)))
            {
                scry_choices += 1;
                assert!(g.observe(actor.other()).library_choice.is_none());
            }
            let root = before
                .stack
                .last()
                .filter(|s| {
                    s.source().is_some_and(|source| NEW.contains(&source.card.name()))
                        && matches!(s.kind, StackKind::ActivatedAbility { .. })
                })
                .cloned();
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
                    entries.insert(b.card.name());
                }
                if b.card.name() == PET {
                    assert!(!b.keywords.contains(Keywords::SKULK));
                    assert!(b.keywords.contains(Keywords::NO_GREATER_POWER_BLOCKERS));
                }
                if b.card.name() == APRIL {
                    assert!(b.keywords.contains(Keywords::NO_POWER_GE_THREE_BLOCKERS));
                }
            }
            if let Some(root) = root
                && after.stack.iter().all(|s| s.id != root.id)
                && after.resolving.is_none()
            {
                if root.source().is_some_and(|source| matches!(source.card.name(), STOMPER | OUTRIDER)) {
                    if let Some(b) = after
                        .objects
                        .iter()
                        .find(|b| b.reference == root.object && b.zone == Zone::Battlefield)
                    {
                        assert!(b.keywords.contains(Keywords::NO_POWER_LE_TWO_BLOCKERS));
                        abilities.insert(root.source().expect("example expects an object-backed packet").card.name());
                        grants += 1;
                    }
                } else if root.source().is_some_and(|source| source.card.name() == PET)
                    && let Some(target) = root.targets.first().and_then(|t| t.object)
                    && before
                        .objects
                        .iter()
                        .any(|b| b.reference == target && b.zone == Zone::Battlefield)
                {
                    let owner = g.observe(root.controller);
                    assert!(
                        owner
                            .objects
                            .iter()
                            .any(|b| b.id == target.card && b.zone == Zone::Hand)
                    );
                    abilities.insert(PET);
                    returns += 1;
                }
            }
            decisions += 1;
            max_bytes = max_bytes.max(g.state_bytes());
        }
        assert!(g.outcome().is_some());
    }
    if games >= 100 {
        for name in NEW {
            assert!(entries.contains(name));
            for expected in [false, true] {
                assert!(
                    outcomes.contains(&(name, expected)),
                    "missing {name} pair {expected}"
                );
            }
        }
        for name in [PET, STOMPER, OUTRIDER] {
            assert!(abilities.contains(name));
        }
        assert!(scry_choices > 0 && rejected > 0);
    }
    println!(
        "{}",
        serde_json::json!({"games":games,"decisions":decisions,"block_pair_checks":pair_checks,"rejected_illegal_blocks":rejected,"granted_restrictions":grants,"pet_returns":returns,"scry_choices":scry_choices,"clone_restore_replays":replays,"max_initialized_state_bytes":max_bytes,"entries":entries,"resolved_abilities":abilities,"restricted_pair_outcomes":outcomes})
    );
}
