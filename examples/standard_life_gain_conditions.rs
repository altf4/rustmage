//! Standard games exercising conditional history, continuous bonuses and branches.
use rustmage::{
    Action, Card, Config, DeckRules, Game, Keywords, ObjectRef, Observation, Player, StackKind,
    Target, Zone,
};
use std::collections::BTreeSet;
const NEW: [&str; 13] = [
    "Ambush Gigapede",
    "Burrog Befuddler",
    "Cogwork Wrestler",
    "Efflorescence",
    "Front Porch Sentries",
    "Grim Bauble",
    "Humbling Elder",
    "Old-Growth Educator",
    "Poisoner's Apprentice",
    "Tenured Concocter",
    "Thornfist Striker",
    "Ulna Alley Shopkeep",
    "Withering Curse",
];
fn deck() -> Vec<Card> {
    let mut d = Vec::new();
    for name in ["Plains", "Island", "Swamp", "Forest"] {
        d.extend([Card::from_name(name).unwrap(); 6]);
    }
    for name in NEW {
        d.extend([Card::from_name(name).unwrap(); 2]);
    }
    for name in ["Moment of Triumph", "Healer's Hawk"] {
        d.extend([Card::from_name(name).unwrap(); 4]);
    }
    d.extend([Card::from_name("Giant Growth").unwrap(); 2]);
    assert_eq!(d.len(), 60);
    assert!(d.iter().all(|c| c.is_supported() && c.is_standard_legal()));
    d
}
fn choose(
    g: &Game,
    o: &Observation,
    actions: &[Action],
    used: &BTreeSet<(ObjectRef, u16)>,
    prefer_gain: bool,
) -> usize {
    let fallback = rustmage::policy::smoke_index(g, actions);
    actions
        .iter()
        .enumerate()
        .max_by_key(|(i, a)| match a {
            Action::Concede => -1000,
            Action::PlayLand { .. } => 300,
            Action::Cast { card, .. } | Action::AnnounceCast { card } => {
                let name = o
                    .objects
                    .iter()
                    .find(|b| b.id == *card)
                    .unwrap()
                    .card
                    .name();
                if name == "Moment of Triumph"
                    && prefer_gain
                    && o.players[o.actor.unwrap().index()].life_gained_this_turn == 0
                {
                    260
                } else if prefer_gain
                    && matches!(
                        name,
                        "Efflorescence"
                            | "Withering Curse"
                            | "Old-Growth Educator"
                            | "Poisoner's Apprentice"
                    )
                {
                    if o.players[o.actor.unwrap().index()].life_gained_this_turn > 0 {
                        250
                    } else {
                        -100
                    }
                } else if matches!(name, "Efflorescence" | "Withering Curse") && !prefer_gain {
                    240
                } else {
                    210
                }
            }
            Action::AnnounceActivate { card, ability, .. } => {
                let b = o.objects.iter().find(|b| b.id == *card).unwrap();
                if b.card.name() == "Grim Bauble" && !used.contains(&(b.reference, *ability)) {
                    230
                } else {
                    -100
                }
            }
            Action::Activate { .. }
            | Action::ActivateMana { .. }
            | Action::AnnounceMana { .. }
            | Action::BeginActivate { .. } => -100,
            Action::TapMana { .. } => {
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
                let root = o.announcing.as_ref().unwrap();
                let benefit = matches!(
                    root.source_card.name(),
                    "Moment of Triumph" | "Giant Growth" | "Efflorescence"
                );
                if root.source().is_some_and(|source| source.card.name() == "Giant Growth")
                    && b.card.name() == "Tenured Concocter"
                    && b.owner != root.controller
                {
                    270
                } else if (b.owner == root.controller) == benefit {
                    250
                } else {
                    100
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
            Action::ChooseOptional { accept: true } => 150,
            _ => {
                if *i == fallback {
                    120
                } else {
                    0
                }
            }
        })
        .unwrap()
        .0
}
fn conditional_power(o: &Observation, b: &rustmage::ObjectView) -> i128 {
    if o.players[b.owner.index()].life_gained_this_turn == 0 {
        return 0;
    }
    let own = if matches!(b.card.name(), "Tenured Concocter" | "Ulna Alley Shopkeep") {
        2
    } else {
        0
    };
    own + o
        .objects
        .iter()
        .filter(|s| {
            s.zone == Zone::Battlefield
                && s.owner == b.owner
                && s.card.name() == "Thornfist Striker"
        })
        .count() as i128
}
fn main() {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse()
        .unwrap();
    assert!(games > 0);
    let (mut decisions, mut checks, mut replays, mut private_choices, mut max_bytes) =
        (0, 0, 0, 0, 0);
    let mut entries = BTreeSet::new();
    let mut effects = BTreeSet::new();
    let mut branches = BTreeSet::new();
    let mut debuffs = BTreeSet::new();
    let mut surveils = 0;
    let mut gained_deltas = 0;
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [deck(), deck()],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })
        .unwrap();
        let mut used = BTreeSet::new();
        let mut explicit_trample = BTreeSet::new();
        for step in 0usize..100_000 {
            if g.outcome().is_some() {
                break;
            }
            assert!(step < 99_999);
            let actor = g.actor().unwrap();
            let before = g.observe(actor);
            let actions = g.legal_actions();
            let a = actions[choose(&g, &before, &actions, &used, seed.is_multiple_of(2))].clone();
            if let Action::AnnounceActivate { card, ability, .. } = a {
                let b = before.objects.iter().find(|b| b.id == card).unwrap();
                used.insert((b.reference, ability));
            }
            if matches!(a, Action::ArrangeLibrary { .. }) {
                assert!(g.observe(actor.other()).library_choice.is_none());
                private_choices += 1;
                if before
                    .resolving
                    .as_ref()
                    .is_some_and(|s| s.source().is_some_and(|source| source.card.name() == "Grim Bauble"))
                {
                    surveils += 1;
                }
            }
            let root = before
                .stack
                .last()
                .filter(|_| {
                    before.decision == "priority"
                        && before.resolving.is_none()
                        && matches!(a, Action::Pass)
                })
                .cloned();
            let saved = g.snapshot();
            let replay = step.is_multiple_of(4)
                || before.announcing.is_some()
                || before.resolving.is_some()
                || !before.stack.is_empty();
            let expected = if replay {
                let mut f = g.clone();
                f.apply(actor, a.clone()).unwrap();
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
                replays += 1;
            }
            let after = g.observe(actor);
            if after.turn != before.turn {
                used.clear();
                explicit_trample.clear();
            }
            // Explicit Efflorescence grants use the caster's history even
            // when their recipient is an opponent's creature.
            if let Some(s) = root.as_ref()
                && s.kind == StackKind::Spell
                && s.source().is_some_and(|source| source.card.name() == "Efflorescence")
                && before.players[s.controller.index()].life_gained_this_turn > 0
                && after.stack.iter().all(|t| t.id != s.id)
                && after.resolving.is_none()
                && let Some(reference) = s.targets.first().and_then(|t| t.object)
            {
                explicit_trample.insert(reference);
            }
            for b in after
                .objects
                .iter()
                .filter(|b| b.zone == Zone::Battlefield && b.card.is_creature())
            {
                if NEW.contains(&b.card.name()) {
                    entries.insert(b.card.name());
                }
                let gain = after.players[b.owner.index()].life_gained_this_turn > 0;
                let thorn = after.objects.iter().any(|s| {
                    s.owner == b.owner
                        && s.zone == Zone::Battlefield
                        && s.card.name() == "Thornfist Striker"
                });
                if !gain && !explicit_trample.contains(&b.reference) {
                    assert!(!b.keywords.contains(Keywords::TRAMPLE));
                } else if gain && thorn {
                    assert!(b.keywords.contains(Keywords::TRAMPLE));
                }
                checks += 1;
            }
            if after
                .objects
                .iter()
                .any(|b| b.card.name() == "Grim Bauble" && b.zone == Zone::Battlefield)
            {
                entries.insert("Grim Bauble");
            }
            if let Some(root) = root
                && after.stack.iter().all(|s| s.id != root.id)
                && after.resolving.is_none()
                && before.turn == after.turn
            {
                let name = root.source_card.name();
                let gain = before.players[root.controller.index()].life_gained_this_turn > 0;
                let old_target = root.targets.first().and_then(|t| t.object).and_then(|r| {
                    before
                        .objects
                        .iter()
                        .find(|b| b.reference == r && b.zone == Zone::Battlefield)
                });
                if name == "Moment of Triumph"
                    && !gain
                    && after.players[root.controller.index()].life_gained_this_turn > 0
                {
                    let thorns = before
                        .objects
                        .iter()
                        .filter(|b| {
                            b.zone == Zone::Battlefield
                                && b.owner == root.controller
                                && b.card.name() == "Thornfist Striker"
                        })
                        .count() as i128;
                    for old in before.objects.iter().filter(|b| {
                        b.zone == Zone::Battlefield
                            && b.owner == root.controller
                            && b.card.is_creature()
                    }) {
                        let b = after
                            .objects
                            .iter()
                            .find(|b| b.reference == old.reference)
                            .unwrap();
                        let target =
                            i128::from(old_target.is_some_and(|t| t.reference == old.reference))
                                * 2;
                        let source = if matches!(
                            old.card.name(),
                            "Tenured Concocter" | "Ulna Alley Shopkeep"
                        ) {
                            2
                        } else {
                            0
                        };
                        assert_eq!(b.power - old.power, target + source + thorns);
                        gained_deltas += 1;
                    }
                }
                if NEW.contains(&name) {
                    effects.insert(name);
                }
                if name == "Efflorescence"
                    && let Some(old) = old_target
                {
                    let b = after
                        .objects
                        .iter()
                        .find(|b| b.reference == old.reference && b.zone == Zone::Battlefield)
                        .unwrap();
                    assert_eq!(b.counters.plus_one, old.counters.plus_one + 2);
                    assert_eq!(
                        b.keywords.contains(Keywords::INDESTRUCTIBLE),
                        gain || old.keywords.contains(Keywords::INDESTRUCTIBLE)
                    );
                    branches.insert((name, gain));
                    checks += 1;
                }
                if name == "Withering Curse" {
                    for old in before
                        .objects
                        .iter()
                        .filter(|b| b.zone == Zone::Battlefield && b.card.is_creature())
                    {
                        let b = after.objects.iter().find(|b| b.id == old.id).unwrap();
                        let survives = if gain {
                            old.keywords.contains(Keywords::INDESTRUCTIBLE)
                        } else {
                            old.toughness > 2
                        };
                        assert_eq!(b.zone == Zone::Battlefield, survives);
                        if survives {
                            assert_eq!(b.toughness, old.toughness - if gain { 0 } else { 2 });
                        }
                        checks += 1;
                    }
                    branches.insert((name, gain));
                }
                if matches!(root.kind, StackKind::TriggeredAbility { .. }) {
                    if name == "Old-Growth Educator"
                        && let Some(old) = before
                            .objects
                            .iter()
                            .find(|b| b.reference == root.object && b.zone == Zone::Battlefield)
                        && let Some(b) = after
                            .objects
                            .iter()
                            .find(|b| b.reference == root.object && b.zone == Zone::Battlefield)
                    {
                        assert_eq!(
                            b.counters.plus_one,
                            old.counters.plus_one + if gain { 2 } else { 0 }
                        );
                        branches.insert((name, gain));
                        checks += 1;
                    }
                    let change = match name {
                        "Ambush Gigapede" | "Grim Bauble" => Some((-2, -2)),
                        "Burrog Befuddler" => Some((-1, 0)),
                        "Cogwork Wrestler" | "Humbling Elder" => Some((-2, 0)),
                        "Front Porch Sentries" => Some((-1, -1)),
                        "Poisoner's Apprentice" => Some(if gain { (-4, -4) } else { (0, 0) }),
                        _ => None,
                    };
                    if let Some((p, t)) = change
                        && let Some(old) = old_target
                    {
                        let b = after.objects.iter().find(|b| b.id == old.id).unwrap();
                        if b.zone == Zone::Battlefield {
                            assert_eq!(
                                b.power,
                                old.power + p + conditional_power(&after, b)
                                    - conditional_power(&before, old)
                            );
                            assert_eq!(b.toughness, old.toughness + t);
                        }
                        // Combat can resume after a death trigger resolves;
                        // source removal and lifelink may occur in that action.
                        // Native fixtures separately isolate the lethal debuff.

                        debuffs.insert(name);
                        checks += 1;
                        if name == "Poisoner's Apprentice" {
                            branches.insert((name, gain));
                        }
                    }
                }
            }
            decisions += 1;
            max_bytes = max_bytes.max(g.state_bytes());
        }
        assert!(g.outcome().is_some());
    }
    if games >= 100 {
        for name in NEW {
            assert!(
                entries.contains(name) || matches!(name, "Efflorescence" | "Withering Curse"),
                "entry {name}"
            );
            assert!(
                effects.contains(name) || matches!(name, "Ulna Alley Shopkeep"),
                "effect {name}"
            );
        }
        for name in [
            "Efflorescence",
            "Withering Curse",
            "Old-Growth Educator",
            "Poisoner's Apprentice",
        ] {
            for gain in [false, true] {
                assert!(branches.contains(&(name, gain)), "branch {name} {gain}");
            }
        }
        assert_eq!(debuffs.len(), 7);
        assert!(surveils > 0 && gained_deltas > 0);
    }
    println!(
        "{games} Standard games; {decisions} decisions; {checks} history/body checks; {gained_deltas} gain-transition stat checks; {private_choices} private choices ({surveils} Grim surveil); {replays} clone/restore replays; {max_bytes} maximum initialized state bytes"
    );
}
