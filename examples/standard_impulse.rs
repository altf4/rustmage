//! Standard-only public acceptance for paid exile spells and ordinary land plays.
use rustmage::{
    Action, Card, CastMethod, Config, DeckRules, Error, Game, Keywords, ObjectRef, Observation,
    Player, SpellView, StackKind, Target, Zone,
};
use std::collections::{BTreeMap, BTreeSet};
const NAMES: [&str; 15] = [
    "Alania's Pathmaker",
    "Blazing Crescendo",
    "Clockwork Percussionist",
    "Cori Mountain Monastery",
    "Count on Luck",
    "Crimson Operative",
    "Diversion Specialist",
    "Emberheart Challenger",
    "Equilibrium Adept",
    "Gundabad Opportunist",
    "Kulrath Zealot",
    "Outlaws' Fury",
    "Seize Opportunity",
    "Sizzling Changeling",
    "Zuko, Exiled Prince",
];
fn family(c: Card) -> Option<usize> {
    NAMES.iter().position(|n| *n == c.name())
}
fn deck(f: usize) -> Vec<Card> {
    let mut d = vec![Card::Mountain; 20];
    d.extend([Card::Forest; 6]);
    d.extend([Card::Swamp; 6]);
    let mut names = vec![NAMES[f]];
    for n in [
        "Swab Goblin",
        "Bear Cub",
        "Sizzling Changeling",
        "Clockwork Percussionist",
        "Corrupted Conviction",
        "Giant Growth",
        "Shock",
    ] {
        if names.len() < 7 && !names.contains(&n) {
            names.push(n);
        }
    }
    for n in names {
        d.extend([Card::from_name(n).unwrap(); 4]);
    }
    assert_eq!(d.len(), 60);
    d
}
fn obj(o: &Observation, r: ObjectRef) -> Option<&rustmage::ObjectView> {
    o.objects.iter().find(|v| v.reference == r)
}
fn outlaw(c: Card) -> bool {
    let line = c.info().type_line.split('—').nth(1).unwrap_or("");
    c.info().keywords.contains(Keywords::CHANGELING)
        || ["Assassin", "Mercenary", "Pirate", "Rogue", "Warlock"]
            .iter()
            .any(|n| line.split_whitespace().any(|s| s == *n))
}
fn granting(s: &SpellView) -> Option<usize> {
    let i = family(s.source_card)?;
    let is = match s.kind {
        StackKind::Spell => matches!(i, 1 | 11) || i == 12 && s.mode == Some(0),
        StackKind::TriggeredAbility { ability } => match i {
            0 | 2 | 4 | 8 | 9 | 10 | 13 => ability == 0,
            5 | 7 => ability == 1,
            _ => false,
        },
        StackKind::ActivatedAbility { ability } => match i {
            3 => ability == 1,
            6 | 14 => ability == 0,
            _ => false,
        },
    };
    is.then_some(i)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games = std::env::args()
        .nth(1)
        .map_or(Ok(100), |s| s.parse::<u64>())?;
    let mut grants = [0u64; 15];
    let mut entries = [0u64; 15];
    let mut residuals = [0u64; 5];
    let mut plays = [0u64; 2];
    let mut owners = [0u64; 2];
    let mut expired = 0;
    let mut left_source = 0;
    let mut decisions = 0;
    let mut replays = 0;
    let mut max_bytes = 0;
    let mut initial_bytes = 0;
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [deck(seed as usize % 15), deck((seed as usize + 7) % 15)],
            seed: seed + 20700,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        initial_bytes = g.state_bytes();
        let mut held = None;
        let mut used = BTreeSet::new();
        let mut seen = BTreeSet::new();
        let mut exiled_casts = BTreeMap::new();
        let mut source_for = BTreeMap::new();
        let mut local = 0;
        while g.outcome().is_none() {
            local += 1;
            assert!(local < 40000, "fixture did not terminate seed{seed}");
            let p = g.actor().unwrap();
            let b = g.observe(p);
            let actions = g.legal_actions();
            let fallback = rustmage::policy::smoke_index(&g, &actions);
            let selected = actions
                .iter()
                .enumerate()
                .max_by_key(|(j, a)| {
                    let ordinary = if *j == fallback { 100 } else { 0 };
                    match a {
                        Action::AnnounceExile { card } => {
                            if Some(*card) != held {
                                1000
                            } else {
                                -100
                            }
                        }
                        Action::PlayLand { card } => {
                            if Some(*card) == held {
                                -100
                            } else if b.exile_permissions.iter().any(|r| r.object.card == *card) {
                                990
                            } else {
                                350
                            }
                        }
                        Action::AnnounceCast { card } | Action::Cast { card, .. } => {
                            let v = b.objects.iter().find(|v| v.id == *card).unwrap();
                            let live = b.objects.iter().any(|x| {
                                x.owner == p && x.zone == Zone::Battlefield && x.card == v.card
                            });
                            if matches!(a, Action::Cast { .. }) && family(v.card).is_some() {
                                -100
                            } else if v.card.name() == "Corrupted Conviction" {
                                if b.objects.iter().any(|x| {
                                    x.owner == p
                                        && x.zone == Zone::Battlefield
                                        && x.card.is_creature()
                                }) {
                                    310
                                } else {
                                    -100
                                }
                            } else if v.card.name() == "Giant Growth" {
                                if b.objects.iter().any(|x| {
                                    x.owner == p
                                        && x.zone == Zone::Battlefield
                                        && x.card.name() == "Emberheart Challenger"
                                }) {
                                    420
                                } else {
                                    -100
                                }
                            } else if family(v.card).is_some() {
                                if live && matches!(family(v.card), Some(3 | 4 | 6 | 7 | 14)) {
                                    -100
                                } else {
                                    400
                                }
                            } else {
                                ordinary
                            }
                        }
                        Action::AnnounceActivate { card, ability, .. }
                        | Action::Activate { card, ability, .. }
                        | Action::BeginActivate { card, ability, .. } => {
                            let v = b.objects.iter().find(|v| v.id == *card).unwrap();
                            if matches!(
                                (family(v.card), *ability),
                                (Some(3), 1) | (Some(6 | 14), 0)
                            ) && !used.contains(&(b.turn, *card))
                            {
                                430
                            } else {
                                -100
                            }
                        }
                        Action::ChooseAdditionalCost { object } => {
                            let v = obj(&b, *object).unwrap();
                            if matches!(family(v.card), Some(2 | 13)) {
                                700
                            } else if family(v.card).is_some() {
                                650
                            } else {
                                600
                            }
                        }
                        Action::ChooseTarget {
                            target: Target::Card(id),
                        } => {
                            let v = b.objects.iter().find(|v| v.id == *id).unwrap();
                            if b.announcing.as_ref().is_some_and(|s| {
                                s.source_card.name() == "Giant Growth"
                                    || s.source_card.name() == "Blazing Crescendo"
                            }) {
                                if v.owner == p && v.card.name() == "Emberheart Challenger" {
                                    750
                                } else if v.owner == p {
                                    700
                                } else {
                                    -100
                                }
                            } else {
                                ordinary
                            }
                        }
                        Action::ChooseMode { mode } => {
                            if *mode == (if seed % 3 == 0 { 1 } else { 0 }) {
                                700
                            } else {
                                650
                            }
                        }
                        Action::PayCosts { .. } => 1100,
                        Action::TapMana { .. } => {
                            if b.decision == "pay_costs" {
                                500
                            } else {
                                -100
                            }
                        }
                        Action::AnnounceMana { .. } | Action::ActivateMana { .. } => -100,
                        Action::Concede => -1000,
                        _ => ordinary,
                    }
                })
                .unwrap()
                .0;
            let action = actions[selected].clone();
            if let Action::AnnounceActivate { card, .. }
            | Action::Activate { card, .. }
            | Action::BeginActivate { card, .. } = action
            {
                used.insert((b.turn, card));
            }
            let saved = g.snapshot();
            let rev = g.revision();
            let mut fork = g.clone();
            fork.apply(p, action.clone())?;
            g.apply(p, action.clone())?;
            let expected = g.snapshot();
            assert_eq!(fork.snapshot(), expected);
            g.restore(&saved)?;
            assert_eq!(g.apply_index(p, rev, selected), Err(Error::StaleRevision));
            g.apply(p, action.clone())?;
            assert_eq!(g.snapshot(), expected);
            replays += 1;
            let a = g.observe(p);
            assert_eq!(a.schema, 57);
            assert_eq!(a.exile_permissions, g.observe(p.other()).exile_permissions);
            for r in &a.exile_permissions {
                let v = obj(&a, r.object).expect("exact public exile object");
                assert_eq!(v.zone, Zone::Exile);
                assert_eq!(v.owner, r.player);
                assert!(r.expires_turn >= a.turn);
            }
            let new: Vec<_> = a
                .exile_permissions
                .iter()
                .filter(|r| !b.exile_permissions.iter().any(|old| old.object == r.object))
                .collect();
            if !new.is_empty() {
                let s = b.stack.last().expect("grant resolves on stack");
                let i = granting(s).expect("only complete impulse clause grants");
                let expected =
                    if i == 12 { 2 } else { 1 }.min(b.players[s.controller.index()].library_count);
                assert_eq!(new.len(), expected);
                assert_eq!(
                    a.players[s.controller.index()].library_count,
                    b.players[s.controller.index()].library_count - new.len()
                );
                assert_eq!(
                    a.players[s.controller.index()].cards_drawn_this_turn,
                    b.players[s.controller.index()].cards_drawn_this_turn
                );
                grants[i] += new.len() as u64;
                for r in new {
                    assert_eq!(r.player, s.controller);
                    assert_eq!(
                        r.expires_turn,
                        b.turn
                            + if matches!(i, 4 | 6 | 7 | 14) {
                                0
                            } else if b.active == s.controller {
                                2
                            } else {
                                1
                            }
                    );
                    source_for.insert(r.object, s.object);
                    if held.is_none() {
                        held = Some(r.object.card);
                    }
                }
            }
            if matches!(action, Action::Pass)
                && let Some(s) = b.stack.last()
                && !a.stack.iter().any(|v| v.id == s.id)
                && a.resolving.as_ref().is_none_or(|v| v.id != s.id)
                && a.outcome.is_none()
            {
                if let Some(i) = granting(s) {
                    let allowed = if i == 11 {
                        b.objects.iter().any(|v| {
                            v.owner == s.controller && v.zone == Zone::Battlefield && outlaw(v.card)
                        })
                    } else {
                        true
                    };
                    let valid = s.targets.is_empty()
                        || s.targets
                            .iter()
                            .any(|t| t.object.is_none_or(|r| obj(&b, r).is_some()));
                    if allowed && valid && b.players[s.controller.index()].library_count > 0 {
                        assert!(
                            a.exile_permissions.iter().any(|r| !b
                                .exile_permissions
                                .iter()
                                .any(|old| old.object == r.object)),
                            "missing permission {s:?}"
                        );
                    }
                }
                if let Some(i) = family(s.source_card) {
                    if s.kind == StackKind::Spell
                        && s.source_card.is_creature()
                        && let Some(v) = a
                            .objects
                            .iter()
                            .find(|v| v.id == s.card && v.zone == Zone::Battlefield)
                    {
                        let stats = match i {
                            0 | 9 => (4, 2),
                            2 => (1, 1),
                            5 | 13 => (3, 2),
                            6 | 14 => (4, 3),
                            7 => (2, 2),
                            8 => (2, 4),
                            10 => (6, 5),
                            _ => unreachable!(),
                        };
                        assert_eq!((v.power, v.toughness), stats);
                        assert_eq!(v.keywords, s.source_card.info().keywords);
                        entries[i] += 1;
                    }
                    if matches!(
                        (i, s.kind),
                        (5 | 7, StackKind::TriggeredAbility { ability: 0 })
                    ) && let (Some(before), Some(after)) = (obj(&b, s.object), obj(&a, s.object))
                    {
                        assert_eq!(
                            (after.power, after.toughness),
                            (before.power + 1, before.toughness + 1)
                        );
                        residuals[0] += 1;
                    }
                    if matches!((i, s.kind), (8, StackKind::TriggeredAbility { ability: 1 }))
                        && let Some(v) = obj(&a, s.object)
                    {
                        assert!(v.keywords.contains(Keywords::DOUBLE_STRIKE));
                        residuals[1] += 1;
                    }
                    if i == 14 && matches!(s.kind, StackKind::TriggeredAbility { ability: 0 }) {
                        assert_eq!(
                            a.players[s.controller.index()].combat_red_mana,
                            b.players[s.controller.index()].combat_red_mana + 3
                        );
                        residuals[2] += 1;
                    }
                    if i == 1 || i == 12 && s.mode == Some(1) {
                        for t in &s.targets {
                            if let Some(r) = t.object
                                && let (Some(before), Some(after)) = (obj(&b, r), obj(&a, r))
                            {
                                assert_eq!(
                                    (after.power, after.toughness),
                                    (
                                        before.power + if i == 1 { 3 } else { 2 },
                                        before.toughness + 1
                                    )
                                );
                                residuals[3] += 1;
                            }
                        }
                    }
                    if i == 11 && s.kind == StackKind::Spell {
                        for v in b.objects.iter().filter(|v| {
                            v.zone == Zone::Battlefield
                                && v.owner == s.controller
                                && v.card.is_creature()
                        }) {
                            if let Some(after) = obj(&a, v.reference) {
                                assert_eq!(
                                    (after.power, after.toughness),
                                    (v.power + 2, v.toughness)
                                );
                                residuals[4] += 1;
                            }
                        }
                    }
                }
            }
            if let Action::AnnounceExile { card } | Action::PlayLand { card } = action
                && let Some(r) = b.exile_permissions.iter().find(|r| r.object.card == card)
            {
                assert_eq!(r.player, p);
                let v = a.objects.iter().find(|v| v.id == card).unwrap();
                assert_eq!(v.reference.generation, r.object.generation + 1);
                assert!(!a.exile_permissions.iter().any(|x| x.object == r.object));
                owners[p.index()] += 1;
                if let Some(source) = source_for.get(&r.object)
                    && obj(&b, *source).is_none_or(|v| v.zone != Zone::Battlefield)
                {
                    left_source += 1;
                }
                if matches!(action, Action::PlayLand { .. }) {
                    assert_eq!(v.zone, Zone::Battlefield);
                    assert_eq!(b.active, p);
                    plays[1] += 1;
                } else {
                    let s = a.announcing.as_ref().unwrap();
                    assert_eq!(s.cast.unwrap().origin, Zone::Exile);
                    assert_eq!(s.cast.unwrap().method, CastMethod::Normal);
                    plays[0] += 1;
                }
            }
            for s in &a.stack {
                if s.kind == StackKind::Spell
                    && seen.insert(s.id)
                    && s.cast.unwrap().origin == Zone::Exile
                {
                    exiled_casts.insert(s.id, s.clone());
                }
            }
            let completed: Vec<_> = exiled_casts
                .iter()
                .filter(|(id, _)| {
                    !a.stack.iter().any(|s| s.id == **id)
                        && a.resolving.as_ref().is_none_or(|s| s.id != **id)
                })
                .map(|(id, _)| *id)
                .collect();
            for id in completed {
                let s = exiled_casts.remove(&id).unwrap();
                let v = a.objects.iter().find(|v| v.id == s.card).unwrap();
                assert!(matches!(v.zone, Zone::Battlefield | Zone::Graveyard));
            }
            for r in &b.exile_permissions {
                if a.turn > r.expires_turn
                    && obj(&a, r.object).is_some_and(|v| v.zone == Zone::Exile)
                {
                    assert!(!a.exile_permissions.iter().any(|v| v.object == r.object));
                    expired += 1;
                    if a.actor == Some(r.player) {
                        let old = g.snapshot();
                        assert_eq!(
                            g.apply(
                                r.player,
                                Action::AnnounceExile {
                                    card: r.object.card
                                }
                            ),
                            Err(Error::IllegalAction)
                        );
                        assert_eq!(g.snapshot(), old);
                    }
                }
            }
            max_bytes = max_bytes.max(g.state_bytes());
            decisions += 1;
        }
    }
    if games >= 100 {
        assert!(grants.iter().all(|n| *n > 0), "{grants:?}");
        assert!(plays.iter().all(|n| *n > 0));
        assert!(owners.iter().all(|n| *n > 0));
        assert!(residuals.iter().all(|n| *n > 0), "{residuals:?}");
        assert!(expired > 0 && left_source > 0);
    }
    println!(
        "catalogue={} games={games} decisions={decisions} replays={replays} initial_bytes={initial_bytes} max_bytes={max_bytes} grants={grants:?} entries={entries:?} residuals={residuals:?} spells_lands={plays:?} owners={owners:?} expired={expired} source_left={left_source}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
