//! Native Standard external activation sacrifice costs and resource reservation.
use rustmage::{Action, Card, Config, DeckRules, EffectChoiceKind, Game, Player, StackKind, Zone};
const NAMES: [&str; 19] = [
    "Acolyte of Aclazotz",
    "Ahriman",
    "Cauldron of Essence",
    "Fountainport",
    "Hardened Tactician",
    "Hungry Ghoul",
    "Kingpin's Enforcers",
    "Legion Extruder",
    "Marwyn, the Clearcutter",
    "Ripchain Razorkin",
    "Seismic Monstrosaur",
    "Slagdrill Scrapper",
    "Sweettooth Witch",
    "Umbral Collar Zealot",
    "Unburied Earthcarver",
    "Unrooted Ancestor",
    "Vampiric Rites",
    "Vanguard of the Rose",
    "Vito's Inquisitor",
];
fn index(card: Card) -> Option<usize> {
    NAMES.iter().position(|n| *n == card.name())
}
fn external(s: &rustmage::SpellView) -> Option<usize> {
    let i = index(s.source_card)?;
    matches!(s.kind,StackKind::ActivatedAbility{ability}if ability==if i==3{1}else{0}).then_some(i)
}
fn deck(kind: usize) -> Vec<Card> {
    let (mut cards, lands) = (
        Vec::new(),
        match kind {
            0 => [("Plains", 8), ("Swamp", 8), ("Mountain", 8), ("Forest", 0)],
            1 => [("Swamp", 8), ("Forest", 8), ("Mountain", 8), ("Plains", 0)],
            _ => [("Plains", 8), ("Swamp", 8), ("Forest", 4), ("Mountain", 4)],
        },
    );
    for (name, n) in lands {
        cards.extend(vec![Card::from_name(name).unwrap(); n]);
    }
    let ids = match kind {
        0 => [3, 4, 12, 7, 0, 1, 6, 13, 18],
        1 => [2, 8, 11, 9, 10, 5, 14, 15, 16],
        _ => [17, 15, 8, 5, 16, 3, 4, 12, 7],
    };
    for i in ids {
        let c = Card::from_name(NAMES[i]).unwrap();
        assert!(c.is_supported());
        cards.extend([c; 4]);
    }
    assert_eq!(cards.len(), 60);
    cards
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse()?;
    let decks = [deck(0), deck(1), deck(2)];
    let (
        mut decisions,
        mut choices,
        mut commits,
        mut self_sacrifices,
        mut checks,
        mut private,
        mut replays,
        mut max_bytes,
    ) = (0, 0, 0, 0, 0, 0, 0, 0);
    let (mut uses, mut resolves) = ([0; 19], [0; 19]);
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [decks[seed % 3].clone(), decks[(seed + 1) % 3].clone()],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        for turn in 0..150_000 {
            if g.outcome().is_some() {
                break;
            }
            let actor = g.actor().unwrap();
            let before = g.observe(actor);
            let mut actions = g.legal_actions();
            if before.turn < 12 {
                actions.retain(|a| {
                    !matches!(a, Action::Attack { .. } | Action::AttackPlaneswalker { .. })
                });
            }
            if seed % 2 == 0 {
                actions.retain(|a|!matches!(a,Action::Activate{card,ability:1,..}|Action::AnnounceActivate{card,ability:1,..}|Action::BeginActivate{card,ability:1,..}if before.objects.iter().any(|o|o.id==*card&&o.zone==Zone::Hand&&o.card.name()=="Seismic Monstrosaur")));
            }
            let mut action = actions[rustmage::policy::smoke_index(&g, &actions)].clone();
            // Ensure the Food is used by Witch at least once before competing
            // sacrifice outlets consume every available Food in these decks.
            if uses[12]==0 && let Some(a)=actions.iter().find(|a|matches!(a,Action::AnnounceActivate{card,ability:0,..}if before.objects.iter().any(|o|o.id==*card&&o.card.name()=="Sweettooth Witch"))){action=a.clone();}
            if let Some(cost) = &before.additional_cost {
                assert_eq!(cost.kind, EffectChoiceKind::Sacrifice);
                assert!(g.observe(actor.other()).additional_cost.is_none());
                checks += 1;
                if seed % 2 == 0
                    && before
                        .announcing
                        .as_ref()
                        .is_some_and(|s| s.source_card.name() == "Kingpin's Enforcers")
                {
                    let source = before.announcing.as_ref().unwrap().object;
                    if cost.candidates.contains(&source) {
                        action = Action::ChooseAdditionalCost { object: source };
                    }
                }
            }
            let root = before.announcing.as_ref().filter(|s| external(s).is_some());
            let selected = if matches!(action, Action::PayCosts { .. }) {
                before
                    .additional_cost
                    .as_ref()
                    .filter(|c| c.kind == EffectChoiceKind::Sacrifice)
                    .and_then(|c| c.selected)
            } else {
                None
            };
            choices += usize::from(matches!(action, Action::ChooseAdditionalCost { .. }));
            private += usize::from(before.library_choice.is_some());
            let saved = g.snapshot();
            let boundary = before.announcing.is_some()
                || before.resolving.is_some()
                || !before.stack.is_empty()
                || turn % 23 == 0;
            let mut fork = boundary.then(|| g.clone());
            if let Some(f) = &mut fork {
                f.apply(actor, action.clone())?;
            }
            g.apply(actor, action.clone())?;
            let after = g.observe(actor);
            if let (Some(root), Some(selected)) = (root, selected) {
                let i = external(root).unwrap();
                uses[i] += 1;
                commits += 1;
                self_sacrifices += usize::from(selected == root.object);
                assert!(
                    !after
                        .objects
                        .iter()
                        .any(|o| o.reference == selected && o.zone == Zone::Battlefield)
                );
                if let Action::PayCosts { payment } = action {
                    for (c, (old, new)) in payment.into_iter().zip(
                        before.players[actor.index()]
                            .mana
                            .into_iter()
                            .zip(after.players[actor.index()].mana),
                    ) {
                        assert_eq!(old - c, new);
                    }
                }
                if [0, 2, 3, 7, 8, 11].contains(&i)
                    && let Some(o) = after
                        .objects
                        .iter()
                        .find(|o| o.reference == root.object && o.zone == Zone::Battlefield)
                {
                    assert!(o.tapped);
                }
                checks += 1;
            }
            if matches!(action, Action::Pass) && before.decision == "priority" {
                for item in &before.stack {
                    if after.stack.iter().any(|s| s.id == item.id)
                        || after.resolving.as_ref().is_some_and(|s| s.id == item.id)
                    {
                        continue;
                    }
                    if let Some(i) = external(item) {
                        resolves[i] += 1;
                    }
                }
            }
            if let Some(item) = &before.resolving
                && after.resolving.as_ref().is_none_or(|s| s.id != item.id)
                && let Some(i) = external(item)
            {
                resolves[i] += 1;
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
        assert!(g.outcome().is_some(), "game {seed} unfinished");
    }
    if games >= 100 {
        assert!(uses.iter().all(|n| *n > 0), "missing commits {uses:?}");
        assert!(
            resolves.iter().all(|n| *n > 0),
            "missing bodies {resolves:?}"
        );
        assert!(self_sacrifices > 0 && private > 0 && replays > 0);
    }
    println!(
        "Completed {games} Standard sacrifice-activation games: {decisions} decisions, {choices} owned selections, {commits} committed sacrifices, {self_sacrifices} source sacrifices, {checks} cost/privacy checks, {private} private surveil boundaries, {replays} clone/restore replays, {max_bytes} maximum initialized bytes; uses {uses:?}; resolutions {resolves:?}; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
