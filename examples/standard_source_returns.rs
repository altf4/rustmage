//! Native Standard source-return effects, graveyard origin, entry and replay workload.
use rustmage::{Action, Card, Config, DeckRules, Game, Player, SpellView, StackKind, Zone};
const NAMES: [&str; 11] = [
    "Abzan Devotee",
    "Arcanis the Omnipotent",
    "Bestial Bloodline",
    "Blessed Ghoul",
    "Merchant of Many Hats",
    "Project Deathlok Soldier",
    "Reassembling Skeleton",
    "Summoned Dromedary",
    "Teacher's Pest",
    "Tunnel Rats",
    "Wingspan Stride",
];

fn index(c: Card) -> Option<usize> {
    NAMES.iter().position(|n| *n == c.name())
}
fn returning(s: &SpellView) -> Option<usize> {
    let i = s.source().and_then(|source| index(source.card))?;
    matches!(s.kind,StackKind::ActivatedAbility{ability}if ability==if i<=1{1}else{0}).then_some(i)
}
fn deck(family: usize) -> Vec<Card> {
    let mut d = Vec::new();
    let lands = if family == 2 {
        [("Island", 12), ("Swamp", 8), ("Plains", 2), ("Forest", 2)]
    } else {
        [("Island", 0), ("Swamp", 10), ("Plains", 6), ("Forest", 8)]
    };
    for (n, count) in lands {
        d.extend(vec![Card::from_name(n).unwrap(); count]);
    }
    let ids = match family {
        0 => [0, 2, 3, 4, 5, 6, 7],
        1 => [8, 9, 6, 0, 5, 4, 3],
        _ => [1, 10, 0, 3, 5, 6, 9],
    };
    for n in ids
        .into_iter()
        .map(|i| NAMES[i])
        .chain(["Deathcap Marionette", "Bear Cub"])
    {
        let c = Card::from_name(n).unwrap();
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
    let decks = [deck(0), deck(1), deck(2)];
    let (mut decisions, mut checks, mut replays, mut max_bytes, mut arcanis_draws) =
        (0, 0, 0, 0, 0);
    let (mut commits, mut moved, mut resolved) = ([0; 11], [0; 11], [0; 11]);
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [decks[seed % 3].clone(), decks[seed % 3].clone()],
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
            if before.turn < 12 {
                actions.retain(|a| !matches!(a, Action::Attack { .. }));
            }
            // On odd seeds retain Arcanis until its draw activation can be paid; then
            // returning it in response also exercises the independent lower ability.
            if seed % 2 == 1 {
                actions.retain(|a|!matches!(a,Action::Activate{card,ability:1,..}|Action::AnnounceActivate{card,ability:1,..}if before.objects.iter().any(|o|o.id==*card&&o.card.name()=="Arcanis the Omnipotent"&&o.used_abilities&1==0)));
            }
            let mut action = actions[rustmage::policy::smoke_index(&g, &actions)].clone();
            if let Some(a)=actions.iter().find(|a|matches!(a,Action::AnnounceActivate{card,ability,..}if before.objects.iter().any(|o|o.id==*card&&index(o.card).is_some_and(|i|*ability==if i<=1{1}else{0})))){action=a.clone();}
            let boundary = before.announcing.is_some()
                || before.resolving.is_some()
                || before.stack.iter().any(|s| returning(s).is_some())
                || matches!(action, Action::AnnounceActivate { .. });
            let saved = g.snapshot();
            let mut f = boundary.then(|| g.clone());
            g.apply(p, action.clone())?;
            if let Some(f) = &mut f {
                f.apply(p, action.clone())?;
            }
            let after = g.observe(p);
            for s in &after.stack {
                if let Some(i) = returning(s)
                    && !before.stack.iter().any(|old| old.id == s.id)
                    && !before.resolving.as_ref().is_some_and(|old| old.id == s.id)
                {
                    commits[i] += 1;
                    let owner = g.observe(s.controller);
                    let source = owner.objects.iter().find(|o| o.id == s.source().expect("example expects an object-backed packet").id).unwrap();
                    assert_eq!(source.reference, s.source().expect("example expects an object-backed packet").object);
                    assert_eq!(
                        source.zone,
                        if matches!(i, 1 | 10) {
                            Zone::Battlefield
                        } else {
                            Zone::Graveyard
                        }
                    );
                    checks += 1;
                }
            }
            for s in before.stack.iter().chain(before.resolving.iter()) {
                if let Some(i) = returning(s)
                    && !live(&after, s.id)
                {
                    resolved[i] += 1;
                    let owner = g.observe(s.controller);
                    if let Some(new) = owner.objects.iter().find(|o| o.id == s.source().expect("example expects an object-backed packet").id)
                        && before
                            .objects
                            .iter()
                            .any(|old| old.id == s.source().expect("example expects an object-backed packet").id && old.reference == s.source().expect("example expects an object-backed packet").object)
                        && new.reference != s.source().expect("example expects an object-backed packet").object
                        && matches!(new.zone, Zone::Hand | Zone::Battlefield)
                    {
                        moved[i] += 1;
                        if new.zone == Zone::Hand {
                            assert!(
                                g.observe(s.controller.other())
                                    .known_hand
                                    .contains(&new.reference)
                            );
                        } else {
                            assert!(new.tapped && new.summoning_sick);
                        }
                        checks += 1;
                    }
                }
                if s.source().is_some_and(|source| source.card.name() == "Arcanis the Omnipotent")
                    && matches!(s.kind, StackKind::ActivatedAbility { ability: 0 })
                    && !live(&after, s.id)
                {
                    arcanis_draws += 1;
                }
            }
            if let Some(f) = f {
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
        assert!(moved.iter().all(|n| *n > 0), "moves {moved:?}");
        assert!(resolved.iter().all(|n| *n > 0));
        assert!(arcanis_draws > 0);
    }
    println!(
        "Completed {games} Standard source-return games, {decisions} decisions, commits {commits:?}, moves {moved:?}, resolutions {resolved:?}, {arcanis_draws} Arcanis draw resolutions, {checks} exact cost/source/public-knowledge checks, {replays} cloned/replayed boundaries, {max_bytes} maximum initialized state bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
