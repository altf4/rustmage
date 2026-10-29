//! Complete Standard decks exercising graveyard self-exile costs and replay.
use rustmage::{Action, Card, Config, DeckRules, Game, Player, SpellView, StackKind, Zone};
const NAMES: [&str; 16] = [
    "Adorned Crocodile",
    "Agent of Kotis",
    "Beetle, Legacy Criminal",
    "Bonebind Orator",
    "Cryotheory Adept",
    "Eternal Student",
    "Faerie Dreamthief",
    "Goldmeadow Nomad",
    "Gravestone Strider",
    "Leering Onlooker",
    "Rubblebelt Maverick",
    "Stoic Grove-Guide",
    "Stone Docent",
    "Suspicious Shambler",
    "Theoretical Necromancer",
    "Venom, Evil Unleashed",
];

fn index(c: Card) -> Option<usize> {
    NAMES.iter().position(|n| *n == c.name())
}
fn grave_ability(s: &SpellView) -> Option<usize> {
    let i = s.source().and_then(|source| index(source.card))?;
    matches!(s.kind,StackKind::ActivatedAbility{ability}if ability==if i==8{1}else{0}).then_some(i)
}
fn deck(family: usize) -> Vec<Card> {
    let mut d = Vec::new();
    let lands = if family == 0 {
        [("Island", 8), ("Swamp", 12), ("Plains", 4), ("Forest", 0)]
    } else {
        [("Island", 0), ("Swamp", 12), ("Plains", 4), ("Forest", 8)]
    };
    for (name, n) in lands {
        d.extend(vec![Card::from_name(name).unwrap(); n]);
    }
    for name in NAMES[family * 8..family * 8 + 8]
        .iter()
        .copied()
        .chain(["Deathcap Marionette"])
    {
        let c = Card::from_name(name).unwrap();
        assert!(c.is_supported());
        d.extend([c; 4]);
    }
    assert_eq!(d.len(), 60);
    d
}
fn live(o: &rustmage::Observation, id: u64) -> bool {
    o.stack.iter().any(|s| s.id == id)
        || o.resolving.as_ref().is_some_and(|s| s.id == id)
        || o.announcing.as_ref().is_some_and(|s| s.id == id)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse()?;
    let decks = [deck(0), deck(1)];
    let (mut decisions, mut replays, mut private, mut checks, mut max_bytes) = (0, 0, 0, 0, 0);
    let mut hand_checks = 0;
    let (mut commits, mut resolutions) = ([0; 16], [0; 16]);
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [decks[seed % 2].clone(), decks[seed % 2].clone()],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        for _ in 0..100_000 {
            if g.outcome().is_some() {
                break;
            }
            let p = g.actor().unwrap();
            let before = g.observe(p);
            let mut actions = g.legal_actions();
            // Delay combat to allow higher-cost graveyard abilities and real self-mill.
            if before.turn < 12 {
                actions.retain(|a| !matches!(a, Action::Attack { .. }));
            }
            let mut action = actions[rustmage::policy::smoke_index(&g, &actions)].clone();
            if let Some(a)=actions.iter().find(|a|matches!(a,Action::AnnounceActivate{card,..}if before.objects.iter().any(|o|o.id==*card&&o.zone==Zone::Graveyard&&index(o.card).is_some()))) {action=a.clone();}
            if before.library_choice.is_some() {
                assert!(g.observe(p.other()).library_choice.is_none());
                assert!(g.observe(p.other()).actions.is_empty());
                private += 1;
            }
            let boundary = before.announcing.is_some()
                || before.resolving.is_some()
                || before.stack.iter().any(|s| grave_ability(s).is_some())
                || matches!(action, Action::AnnounceActivate { .. });
            let saved = g.snapshot();
            let mut fork = boundary.then(|| g.clone());
            g.apply(p, action.clone())?;
            if let Some(f) = &mut fork {
                f.apply(p, action.clone())?;
            }
            let after = g.observe(p);
            // Public-to-hand identities survive in a memoryless opponent view.
            // Validate every real public departure, including targeted recovery,
            // while rejecting duplicate and stale ledger entries.
            let opponent = g.observe(p.other());
            assert_eq!(after.known_hand, opponent.known_hand);
            for (i, reference) in after.known_hand.iter().enumerate() {
                assert!(!after.known_hand[..i].contains(reference));
                assert!(
                    after
                        .objects
                        .iter()
                        .any(|o| o.reference == *reference && o.zone == Zone::Hand)
                );
                assert!(
                    opponent
                        .objects
                        .iter()
                        .any(|o| o.reference == *reference && o.zone == Zone::Hand)
                );
            }
            for old in &before.objects {
                if matches!(
                    old.zone,
                    Zone::Battlefield | Zone::Graveyard | Zone::Stack | Zone::Exile
                ) && !old.card.is_token()
                    && !old.card.is_emblem()
                    && let Some(new) = after.objects.iter().find(|o| {
                        o.id == old.id && o.zone == Zone::Hand && o.reference != old.reference
                    })
                {
                    assert!(after.known_hand.contains(&new.reference));
                    hand_checks += 1;
                }
            }
            for old in &before.objects {
                if old.zone != Zone::Graveyard {
                    continue;
                }
                let Some(i) = index(old.card) else {
                    continue;
                };
                if let Some(new) = after.objects.iter().find(|o| o.id == old.id)
                    && new.zone == Zone::Exile
                    && after.stack.iter().any(|s| {
                        s.source()
                            .expect("example expects an object-backed packet")
                            .object
                            == old.reference
                            && grave_ability(s) == Some(i)
                    })
                {
                    commits[i] += 1;
                    assert_ne!(new.reference, old.reference);
                    assert_eq!(
                        before.creatures_died_this_turn,
                        after.creatures_died_this_turn
                    );
                    assert_eq!(
                        before.players[old.owner.index()].spells_cast_this_turn,
                        after.players[old.owner.index()].spells_cast_this_turn
                    );
                    checks += 1;
                }
            }
            for s in before.stack.iter().chain(before.resolving.iter()) {
                if let Some(i) = grave_ability(s)
                    && !live(&after, s.id)
                {
                    resolutions[i] += 1;
                    let source = after
                        .objects
                        .iter()
                        .find(|o| {
                            o.id == s
                                .source()
                                .expect("example expects an object-backed packet")
                                .id
                        })
                        .unwrap();
                    assert_eq!(source.zone, Zone::Exile);
                    checks += 1;
                }
            }
            if let Some(f) = fork {
                let expected = g.snapshot();
                assert_eq!(f.snapshot(), expected);
                g.restore(&saved)?;
                assert!(g.apply_index(p, before.revision, 0).is_err());
                g.apply(p, action)?;
                assert_eq!(g.snapshot(), expected);
                replays += 1;
            }
            decisions += 1;
            max_bytes = max_bytes.max(g.state_bytes());
        }
        assert!(g.outcome().is_some(), "game {seed} did not finish");
    }
    if games >= 100 {
        assert!(commits.iter().all(|n| *n > 0), "commits {commits:?}");
        assert!(
            resolutions.iter().all(|n| *n > 0),
            "resolutions {resolutions:?}"
        );
        assert!(private > 0);
        assert!(hand_checks > 0);
    }
    println!(
        "Completed {games} Standard graveyard activation games, {decisions} decisions, commits {commits:?}, resolutions {resolutions:?}, {checks} exact cost/source checks, {hand_checks} public-to-hand checks, {private} private library choices, {replays} cloned/replayed boundaries, {max_bytes} maximum initialized state bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
