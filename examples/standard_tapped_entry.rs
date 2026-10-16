//! Native creature-entry replacements, stun untaps, flash, and replay workload.
use rustmage::{Action, Card, Config, DeckRules, Game, Player, StackKind, Zone};
const NAMES: [&str; 5] = [
    "Diregraf Ghoul",
    "Piranha Fly",
    "Daring Thunder-Thief",
    "Sleep-Cursed Faerie",
    "Eddymurk Crab",
];
fn index(c: Card) -> Option<usize> {
    NAMES.iter().position(|n| *n == c.name())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse()?;
    let mut deck = Vec::new();
    for (name, n) in [("Island", 14), ("Swamp", 8), ("Mountain", 2)] {
        deck.extend(vec![Card::from_name(name).unwrap(); n]);
    }
    for name in NAMES
        .into_iter()
        .chain(["Unsummon", "Zombify", "Shore Up", "Shock"])
    {
        let c = Card::from_name(name).unwrap();
        assert!(c.is_supported());
        deck.extend([c; 4]);
    }
    assert_eq!(deck.len(), 60);
    let (
        mut decisions,
        mut checks,
        mut replays,
        mut removals,
        mut untaps,
        mut activations,
        mut noncast,
        mut max_bytes,
    ) = (0, 0, 0, 0, 0, 0, 0, 0);
    let (mut entries, mut crab_turns) = ([0; 5], [0; 2]);
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [deck.clone(), deck.clone()],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        for turn in 0..100_000 {
            if g.outcome().is_some() {
                break;
            }
            let actor = g.actor().unwrap();
            let before = g.observe(actor);
            let mut actions = g.legal_actions();
            // Deliberately hold Crab for own/opposing turn entry on alternating seeds.
            actions.retain(|a|!matches!(a,Action::Cast{card,..}|Action::AnnounceCast{card} if before.objects.iter().any(|o|o.id==*card&&o.card.name()=="Eddymurk Crab") && ((actor==before.active)!=(seed%2==0))));
            // Preserve mana for the held flash spell instead of spending it on
            // other spells or untaps before the opposing turn arrives.
            if seed % 2 == 1
                && actor == before.active
                && before.decision == "priority"
                && before.objects.iter().any(|o| {
                    o.owner == actor && o.zone == Zone::Hand && o.card.name() == "Eddymurk Crab"
                })
            {
                actions.retain(|a| {
                    !matches!(
                        a,
                        Action::TapMana { .. }
                            | Action::Cast { .. }
                            | Action::AnnounceCast { .. }
                            | Action::Activate { .. }
                            | Action::AnnounceActivate { .. }
                            | Action::BeginActivate { .. }
                    )
                });
            }
            let mut action = actions[rustmage::policy::smoke_index(&g, &actions)].clone();
            if let Some(a)=actions.iter().find(|a|matches!(a,Action::Activate{card,..} if before.objects.iter().any(|o|o.id==*card&&o.card.name()=="Sleep-Cursed Faerie"&&o.tapped))){action=a.clone();}
            activations += usize::from(
                matches!(action,Action::Activate{card,..}if before.objects.iter().any(|o|o.id==card&&o.card.name()=="Sleep-Cursed Faerie")),
            );
            let saved = g.snapshot();
            let boundary = before.announcing.is_some()
                || before.resolving.is_some()
                || !before.stack.is_empty()
                || turn % 19 == 0;
            let mut fork = boundary.then(|| g.clone());
            if let Some(f) = &mut fork {
                f.apply(actor, action.clone())?;
            }
            g.apply(actor, action.clone())?;
            let after = g.observe(actor);
            for o in &after.objects {
                if o.zone != Zone::Battlefield {
                    continue;
                }
                if let Some(i) = index(o.card) {
                    checks += 1;
                    if let Some(old) = before
                        .objects
                        .iter()
                        .find(|p| p.reference == o.reference && p.zone == Zone::Battlefield)
                    {
                        if i == 3 && old.counters.stun > o.counters.stun {
                            assert_eq!(old.counters.stun, o.counters.stun + 1);
                            assert!(o.tapped);
                            removals += 1;
                        }
                        if i == 3 && old.tapped && !o.tapped {
                            assert_eq!(old.counters.stun, 0);
                            untaps += 1;
                        }
                    } else {
                        entries[i] += 1;
                        assert_eq!(o.counters.stun, if i == 3 { 3 } else { 0 });
                        assert_eq!(o.tapped, i != 4 || o.owner != after.active);
                        if i == 4 {
                            crab_turns[usize::from(o.owner != after.active)] += 1;
                        }
                        if !before
                            .stack
                            .iter()
                            .any(|s| s.kind == StackKind::Spell && s.card == o.id)
                        {
                            noncast += 1;
                        }
                    }
                }
            }
            if let Some(f) = fork {
                let expected = g.snapshot();
                assert_eq!(f.snapshot(), expected);
                g.restore(&saved)?;
                assert!(g.apply_index(actor, before.revision, 0).is_err());
                g.apply(actor, action)?;
                assert_eq!(g.snapshot(), expected);
                replays += 1;
            }
            decisions += 1;
            max_bytes = max_bytes.max(g.state_bytes());
        }
        assert!(g.outcome().is_some(), "unfinished game {seed}");
    }
    if games >= 100 {
        assert!(entries.iter().all(|n| *n > 0));
        assert!(
            crab_turns.iter().all(|n| *n > 0),
            "Crab entries {crab_turns:?}"
        );
        assert!(removals > 0 && untaps > 0 && activations > 0 && noncast > 0 && replays > 0);
    }
    println!(
        "Completed {games} Standard tapped-entry games: {decisions} decisions, {checks} body/entry checks, entries {entries:?}, Crab own/opposing {crab_turns:?}, {removals} stun removals, {untaps} completed Faerie untaps, {activations} paid untap activations, {noncast} noncasting entries, {replays} clone/restore replays, {max_bytes} maximum initialized bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
