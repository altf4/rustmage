//! Reproducible Standard games with public, effect-specific flashback witnesses.
use rustmage::{
    Action, Card, CastFace, CastInfo, CastMethod, Config, DeckRules, Error, Game, Keywords,
    ObjectView, Observation, Player, SpellView, StackKind, Target, Zone,
};
use std::collections::{BTreeMap, BTreeSet};
const NAMES: [&str; 9] = [
    "Auron's Inspiration",
    "Bestial Incursion",
    "Dig Site Inventory",
    "Dreams of Laguna",
    "Laughing Mad",
    "Predictive Preparations",
    "Revenge of the Rats",
    "Think Twice",
    "Tome Blast",
];
fn idx(c: Card) -> Option<usize> {
    NAMES.iter().position(|n| *n == c.name())
}
fn deck(f: usize) -> Vec<Card> {
    let mut d = Vec::new();
    for n in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        d.extend([Card::from_name(n).unwrap(); 6]);
    }
    for n in [
        NAMES[f],
        "Bear Cub",
        "Swab Goblin",
        "Three Tree Mascot",
        "Gigantoad",
        "Stormcatch Mentor",
        "Opt",
    ] {
        d.extend([Card::from_name(n).unwrap(); 4]);
    }
    d.extend([Card::from_name("Shock").unwrap(); 2]);
    assert_eq!(d.len(), 60);
    d
}
fn object(o: &Observation, r: rustmage::ObjectRef) -> Option<&ObjectView> {
    o.objects.iter().find(|v| v.reference == r)
}
fn count_tokens(o: &Observation, p: Player, kind: &str) -> usize {
    o.objects
        .iter()
        .filter(|v| {
            v.zone == Zone::Battlefield
                && v.owner == p
                && v.card.is_token()
                && v.card.has_subtype(kind)
        })
        .count()
}
fn witness(s: &SpellView, b: &Observation, a: &Observation) -> u64 {
    let i = idx(s.source_card).unwrap();
    let p = s.controller.index();
    let spell = a.objects.iter().find(|v| v.id == s.card).unwrap();
    assert_eq!(
        spell.zone,
        if s.cast.unwrap().method == CastMethod::Flashback {
            Zone::Exile
        } else {
            Zone::Graveyard
        }
    );
    let targets: Vec<_> = s
        .targets
        .iter()
        .filter_map(|t| t.object)
        .filter(|r| object(a, *r).is_some())
        .collect();
    match i {
        0 => {
            let mut n = 0;
            for v in b.objects.iter().filter(|v| {
                v.zone == Zone::Battlefield && v.card.is_creature() && b.attackers.contains(&v.id)
            }) {
                if let Some(now) = object(a, v.reference) {
                    assert_eq!((now.power, now.toughness), (v.power + 2, v.toughness));
                    n += 1;
                }
            }
            assert!(n > 0);
            n
        }
        1 => {
            assert_eq!(
                count_tokens(a, s.controller, "Beast"),
                count_tokens(b, s.controller, "Beast") + 1
            );
            let token = a
                .objects
                .iter()
                .find(|v| {
                    v.card.is_token()
                        && v.card.has_subtype("Beast")
                        && !b.objects.iter().any(|old| old.reference == v.reference)
                })
                .unwrap();
            assert_eq!(
                (token.power, token.toughness, token.keywords),
                (4, 4, Keywords::TRAMPLE)
            );
            1
        }
        2 | 5 => {
            for r in &targets {
                let old = object(b, *r).unwrap();
                let now = object(a, *r).unwrap();
                assert_eq!(now.counters.plus_one, old.counters.plus_one + 1);
                assert_eq!(
                    (now.power, now.toughness),
                    (old.power + 1, old.toughness + 1)
                );
                if i == 2 {
                    assert!(now.keywords.contains(Keywords::VIGILANCE));
                }
            }
            assert!(!targets.is_empty());
            targets.len() as u64
        }
        3 | 7 => {
            assert_eq!(
                a.players[p].cards_drawn_this_turn,
                b.players[p].cards_drawn_this_turn + 1
            );
            1
        }
        4 => {
            assert_eq!(
                a.players[p].cards_drawn_this_turn,
                b.players[p].cards_drawn_this_turn + 2
            );
            1
        }
        6 => {
            let expected = b
                .objects
                .iter()
                .filter(|v| {
                    v.zone == Zone::Graveyard
                        && v.owner == s.controller
                        && v.card.is_creature()
                        && !v.card.is_token()
                })
                .count();
            assert_eq!(
                count_tokens(a, s.controller, "Rat"),
                count_tokens(b, s.controller, "Rat") + expected
            );
            for token in a.objects.iter().filter(|v| {
                v.card.is_token()
                    && v.card.has_subtype("Rat")
                    && !b.objects.iter().any(|old| old.reference == v.reference)
            }) {
                assert_eq!(
                    (token.power, token.toughness, token.keywords, token.tapped),
                    (1, 1, Keywords::NONE, true)
                );
            }
            expected as u64
        }
        8 => {
            let Target::Player(target) = s.targets[0].target else {
                panic!("policy must target a player")
            };
            assert_eq!(
                a.players[target.index()].life,
                b.players[target.index()].life - 2
            );
            1
        }
        _ => unreachable!(),
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games = std::env::args()
        .nth(1)
        .map_or(Ok(100), |n| n.parse::<u64>())?;
    let mut casts = [[0u64; 2]; 9];
    let mut effects = [[0u64; 2]; 9];
    let mut units = 0;
    let mut decisions = 0;
    let mut replays = 0;
    let mut private = 0;
    let mut discards = 0;
    let mut max_bytes = 0;
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [deck(seed as usize % 9), deck((seed as usize + 4) % 9)],
            seed: seed + 1000,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        let mut seen = BTreeSet::new();
        let mut pending: BTreeMap<u64, (SpellView, Observation)> = BTreeMap::new();
        let mut local = 0;
        while g.outcome().is_none() {
            local += 1;
            assert!(local < 30000, "nonterminating fixture seed{seed}");
            let actor = g.actor().unwrap();
            let b = g.observe(actor);
            let actions = g.legal_actions();
            let fallback = rustmage::policy::smoke_index(&g, &actions);
            let selected =
                actions
                    .iter()
                    .enumerate()
                    .max_by_key(|(j, a)| match a {
                        Action::AnnounceFlashback { card }
                        | Action::AnnounceCast { card }
                        | Action::Cast { card, .. } => {
                            let v = b.objects.iter().find(|v| v.id == *card).unwrap();
                            let blocked = matches!(a, Action::Cast { .. }) && idx(v.card).is_some()
                                || idx(v.card) == Some(0) && b.attackers.is_empty()
                                || idx(v.card) == Some(6)
                                    && !b.objects.iter().any(|v| {
                                        v.owner == actor
                                            && v.zone == Zone::Graveyard
                                            && v.card.is_creature()
                                    });
                            if blocked {
                                -100
                            } else if matches!(a, Action::AnnounceFlashback { .. }) {
                                310
                            } else if idx(v.card).is_some() {
                                300
                            } else {
                                if *j == fallback { 100 } else { 0 }
                            }
                        }
                        Action::ChooseTarget {
                            target: Target::Player(p),
                        } if b
                            .announcing
                            .as_ref()
                            .is_some_and(|s| s.source().and_then(|source| idx(source.card)) == Some(8)) =>
                        {
                            if *p == actor.other() { 500 } else { -100 }
                        }
                        Action::PayCosts { .. } => 600,
                        Action::TapMana { .. } => {
                            if b.decision == "pay_costs" {
                                200
                            } else {
                                -100
                            }
                        }
                        Action::Activate { .. }
                        | Action::BeginActivate { .. }
                        | Action::AnnounceActivate { .. } => -100,
                        Action::Concede => -1000,
                        _ => {
                            if *j == fallback {
                                100
                            } else {
                                0
                            }
                        }
                    })
                    .unwrap()
                    .0;
            let action = actions[selected].clone();
            if b.library_choice.is_some() {
                assert!(g.observe(actor.other()).library_choice.is_none());
                assert!(g.observe(actor.other()).actions.is_empty());
                private += 1;
            }
            if let Action::AnnounceCast { card } | Action::AnnounceFlashback { card } = action {
                let v = b.objects.iter().find(|v| v.id == card).unwrap();
                assert_eq!(
                    v.zone,
                    if matches!(action, Action::AnnounceFlashback { .. }) {
                        Zone::Graveyard
                    } else {
                        Zone::Hand
                    }
                );
            }
            let boundary = b.announcing.is_some()
                || b.resolving.is_some()
                || b.stack.iter().any(|s| s.source().and_then(|source| idx(source.card)).is_some());
            let saved = g.snapshot();
            let mut fork = boundary.then(|| g.clone());
            g.apply(actor, action.clone())?;
            if let Some(f) = &mut fork {
                f.apply(actor, action.clone())?;
            }
            let a = g.observe(actor);
            if matches!(action, Action::PayCosts { .. })
                && b.announcing
                    .as_ref()
                    .is_some_and(|s| s.source().and_then(|source| idx(source.card)) == Some(4))
            {
                assert_eq!(
                    a.players[actor.index()].cards_discarded_this_turn,
                    b.players[actor.index()].cards_discarded_this_turn + 1
                );
                discards += 1;
            }
            for s in &a.stack {
                if s.kind != StackKind::Spell || !seen.insert(s.id) {
                    continue;
                }
                if let Some(i) = s.source().and_then(|source| idx(source.card)) {
                    let info = s.cast.unwrap();
                    assert_eq!(
                        info,
                        CastInfo {
                            spend_any_mana: false,
                            exile_on_graveyard: false,
                            caster: s.controller,
                            gift: None,
                            physical_card: None,
                            flash_surcharge: false,
                            sneak_defender: None,
                            face: CastFace::Normal,
                            payment: info.payment,
                            treasure_mana_spent: info.treasure_mana_spent,
                            offspring: None,
                            kicker: None,
                            teamwork: None,
                            blight: None,
                            bargain: None,
                            evidence: None,
                            origin: if info.method == CastMethod::Flashback {
                                Zone::Graveyard
                            } else {
                                Zone::Hand
                            },
                            stash_origin: false,
                            method: info.method
                        }
                    );
                    casts[i][usize::from(info.method == CastMethod::Flashback)] += 1;
                }
            }
            if matches!(action, Action::Pass)
                && let Some(s) = b.stack.last()
                && s.kind == StackKind::Spell
                && s.source().and_then(|source| idx(source.card)).is_some()
                && !a.stack.iter().any(|x| x.id == s.id)
                && a.outcome.is_none()
            {
                pending.insert(s.id, (s.clone(), b.clone()));
            }
            let done: Vec<_> = pending
                .keys()
                .copied()
                .filter(|id| {
                    !a.stack.iter().any(|s| s.id == *id)
                        && a.resolving.as_ref().is_none_or(|s| s.id != *id)
                })
                .collect();
            if a.outcome.is_none() {
                for id in done {
                    let (s, before) = pending.remove(&id).unwrap();
                    let n = witness(&s, &before, &a);
                    if n > 0 {
                        effects[idx(s.source_card).unwrap()]
                            [usize::from(s.cast.unwrap().method == CastMethod::Flashback)] += 1;
                    }
                    units += n;
                }
            }
            assert_eq!(a.schema, rustmage::OBSERVATION_SCHEMA);
            if let Some(f) = fork {
                let expected = g.snapshot();
                assert_eq!(f.snapshot(), expected);
                g.restore(&saved)?;
                assert_eq!(
                    g.apply_index(actor, b.revision, 0),
                    Err(Error::StaleRevision)
                );
                g.apply(actor, action)?;
                assert_eq!(g.snapshot(), expected);
                replays += 1;
            }
            decisions += 1;
            max_bytes = max_bytes.max(g.state_bytes());
        }
    }
    if games >= 100 {
        assert!(casts.iter().flatten().all(|n| *n > 0), "casts{casts:?}");
        assert!(
            effects.iter().flatten().all(|n| *n > 0),
            "effects{effects:?}"
        );
        assert!(private > 0 && discards > 0 && replays > 0);
    }
    println!(
        "Completed {games} Standard flashback games, {decisions} decisions, casts [normal,flashback] {casts:?}, verified effects {effects:?}, {units} independent effect units, {discards} paid discards, {private} private decisions, {replays} replay boundaries, {max_bytes} maximum initialized logical bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
