//! Public supported Standard batch checks. The smoke policy delays attacks and
//! limits optional token growth; these are policy choices, never engine limits.
use rustmage::{
    Action, Card, Config, DeckRules, Game, Keywords, Observation, Player, SpellView, StackKind,
    Zone,
};
use std::collections::{BTreeSet, HashSet};
fn deck(which: usize) -> Vec<Card> {
    let (lands, spells): (&[(&str, usize)], &[&str]) = match which {
        0 => (
            &[("Plains", 24)],
            &[
                "Ajani, Caller of the Pride",
                "Gather the White Lotus",
                "Germinate Recruits",
                "Moogles' Valor",
                "Sally Pride, Lioness Leader",
                "Clachan Festival",
                "Hinterland Sanctifier",
                "Brightblade Stoat",
                "Ajani's Pridemate",
            ],
        ),
        1 => (
            &[("Mountain", 24)],
            &[
                "Krenko, Mob Boss",
                "Brass's Bounty",
                "Dragon Fodder",
                "Elder Auntie",
                "Swab Goblin",
                "Redcap Thief",
                "Raging Redcap",
                "Fire Elemental",
                "Quaketusk Boar",
            ],
        ),
        _ => (
            &[("Forest", 12), ("Swamp", 12)],
            &[
                "Aatchik, Emerald Radian",
                "Morcant's Eyes",
                "The Unbeatable Squirrel Girl",
                "Llanowar Elves",
                "Three Tree Mascot",
                "Professor Dellian Fel",
                "Grasping Longneck",
                "Bear Cub",
                "Murder",
            ],
        ),
    };
    let mut result = Vec::new();
    for &(name, n) in lands {
        result.extend(vec![Card::from_name(name).unwrap(); n]);
    }
    for &name in spells {
        let c = Card::from_name(name).unwrap();
        assert!(c.is_supported() && c.is_standard_legal(), "{name}");
        result.extend([c; 4]);
    }
    assert_eq!(result.len(), 60);
    result
}
fn count(obs: &Observation, p: Player, filter: impl Fn(&rustmage::ObjectView) -> bool) -> u128 {
    obs.objects
        .iter()
        .filter(|o| o.owner == p && filter(o))
        .count() as u128
}
fn creation(obs: &Observation, item: &SpellView) -> Option<(u128, &'static str, i128, i128)> {
    let p = item.controller;
    let n = item.source().expect("example expects an object-backed packet").card.name();
    let bf = |o: &rustmage::ObjectView| o.zone == Zone::Battlefield;
    match (n, item.kind) {
        ("Ajani, Caller of the Pride", StackKind::ActivatedAbility { ability: 2 }) => {
            Some((obs.players[p.index()].life.max(0) as u128, "Cat", 2, 2))
        }
        ("Brass's Bounty", StackKind::Spell) => Some((
            count(obs, p, |o| bf(o) && o.card.is_land()),
            "Treasure",
            0,
            0,
        )),
        ("Gather the White Lotus", StackKind::Spell) => Some((
            count(obs, p, |o| bf(o) && o.card.has_subtype("Plains")),
            "Ally",
            1,
            1,
        )),
        ("Germinate Recruits", StackKind::Spell) => Some((
            obs.players[p.index()].life_gained_this_turn,
            "Wizard Soldier",
            2,
            2,
        )),
        ("Krenko, Mob Boss", StackKind::ActivatedAbility { .. }) => Some((
            count(obs, p, |o| bf(o) && o.card.has_subtype("Goblin")),
            "Goblin",
            1,
            1,
        )),
        ("Moogles' Valor", StackKind::Spell) => Some((
            count(obs, p, |o| bf(o) && o.card.is_creature()),
            "Moogle",
            1,
            2,
        )),
        ("Morcant's Eyes", StackKind::ActivatedAbility { .. }) => Some((
            count(obs, p, |o| {
                o.zone == Zone::Graveyard && o.token.is_none() && o.card.has_subtype("Elf")
            }),
            "Elf",
            2,
            2,
        )),
        ("Aatchik, Emerald Radian", StackKind::TriggeredAbility { ability: 0 }) => Some((
            count(obs, p, |o| {
                o.zone == Zone::Graveyard
                    && o.token.is_none()
                    && (o.card.is_creature() || o.card.is_artifact())
            }),
            "Insect",
            1,
            1,
        )),
        ("Sally Pride, Lioness Leader", StackKind::TriggeredAbility { ability: 0 }) => Some((
            count(obs, p, |o| {
                bf(o) && o.card.is_creature() && o.token.is_none()
            }),
            "Mutant",
            2,
            2,
        )),
        ("Clachan Festival", StackKind::TriggeredAbility { ability: 0 }) => {
            Some((2, "Kithkin", 1, 1))
        }
        ("Clachan Festival", StackKind::ActivatedAbility { .. }) => Some((1, "Kithkin", 1, 1)),
        ("The Unbeatable Squirrel Girl", StackKind::TriggeredAbility { .. }) => {
            Some((1, "Squirrel", 1, 1))
        }
        ("The Unbeatable Squirrel Girl", StackKind::ActivatedAbility { .. }) => Some((
            count(obs, p, |o| bf(o) && o.card.has_subtype("Squirrel")),
            "Squirrel",
            1,
            1,
        )),
        _ => None,
    }
}
fn check(before: &Observation, after: &Observation, item: &SpellView) -> usize {
    let Some((amount, subtypes, power, toughness)) = creation(before, item) else {
        return 0;
    };
    let old: HashSet<_> = before
        .objects
        .iter()
        .map(|o| (o.reference.card, o.reference.generation))
        .collect();
    let created: Vec<_> = after
        .objects
        .iter()
        .filter(|o| {
            o.zone == Zone::Battlefield
                && o.owner == item.controller
                && o.token.is_some()
                && !old.contains(&(o.reference.card, o.reference.generation))
        })
        .collect();
    assert_eq!(
        created.len() as u128,
        amount,
        "{} {:?}",
        item.source().expect("example expects an object-backed packet").card.name(),
        item.kind
    );
    for o in &created {
        assert_eq!(o.token.unwrap().subtypes, subtypes);
        assert_eq!((o.power, o.toughness), (power, toughness));
        assert!(o.summoning_sick);
    }
    if subtypes == "Moogle" {
        assert!(
            created
                .iter()
                .all(|o| o.keywords.contains(Keywords::LIFELINK))
        );
        assert!(
            after
                .objects
                .iter()
                .filter(|o| o.owner == item.controller
                    && o.zone == Zone::Battlefield
                    && o.card.is_creature())
                .all(|o| o.keywords.contains(Keywords::INDESTRUCTIBLE))
        );
    }
    1 + created.len() * 3
}
fn main() {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse()
        .unwrap();
    assert!(games > 0);
    let mut decisions = 0;
    let mut checks = 0;
    let mut branches = 0;
    let mut max_bytes = 0;
    let mut batches = 0;
    let mut seen = BTreeSet::new();
    for seed in 0..games {
        let mut game = Game::new(Config {
            decks: [deck(seed % 3), deck((seed + 1) % 3)],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })
        .unwrap();
        let mut checked = HashSet::new();
        for step in 0..100_000 {
            if game.outcome().is_some() {
                break;
            }
            assert!(step < 99_999, "did not end naturally");
            let actor = game.actor().unwrap();
            let before = game.observe(actor);
            let mut actions = game.legal_actions();
            let creatures = count(&before, actor, |o| {
                o.zone == Zone::Battlefield && o.card.is_creature()
            });
            actions.retain(|a| {
                if before.turn < 16
                    && matches!(a, Action::Attack { .. } | Action::AttackPlaneswalker { .. })
                {
                    return false;
                }
                if matches!(a, Action::Block { .. }) {
                    return false;
                }
                if let Action::Activate { card, ability, .. }
                | Action::BeginActivate { card, ability, .. }
                | Action::AnnounceActivate { card, ability, .. } = a
                {
                    let o = before.objects.iter().find(|o| o.id == *card).unwrap();
                    if [
                        "Krenko, Mob Boss",
                        "The Unbeatable Squirrel Girl",
                        "Clachan Festival",
                    ]
                    .contains(&o.card.name())
                        && (creatures >= 16 || o.used_abilities & (1 << ability) != 0)
                    {
                        return false;
                    }
                }
                if let Action::Cast { card, .. }
                | Action::AnnounceCast { card, .. }
                | Action::BeginCast { card, .. } = a
                {
                    let c = before.objects.iter().find(|o| o.id == *card).unwrap().card;
                    if c.name() == "Germinate Recruits"
                        && !(1..=16).contains(&before.players[actor.index()].life_gained_this_turn)
                    {
                        return false;
                    }
                    if c.name() == "Moogles' Valor" && creatures >= 24 {
                        return false;
                    }
                }
                true
            });
            let index=actions.iter().position(|a|matches!(a,Action::Activate{card,ability:2,..}if before.objects.iter().any(|o|o.id==*card&&o.card.name()=="Ajani, Caller of the Pride")))
                .or_else(|| actions.iter().position(|a| matches!(a,Action::Activate{card,ability:0,..} if before.objects.iter().any(|o|o.id==*card&&o.card.name()=="Ajani, Caller of the Pride"))))
                .unwrap_or_else(||rustmage::policy::smoke_index(&game,&actions));
            let action = actions[index].clone();
            let saved = game.snapshot();
            let expected = if step % 4 == 0 {
                let mut fork = game.clone();
                fork.apply(actor, action.clone()).unwrap();
                branches += 1;
                Some(fork.snapshot())
            } else {
                None
            };
            game.apply(actor, action.clone()).unwrap();
            if let Some(expected) = expected {
                assert_eq!(game.snapshot(), expected);
                game.restore(&saved).unwrap();
                assert_eq!(
                    game.apply_index(actor, before.revision, 0),
                    Err(rustmage::Error::StaleRevision)
                );
                game.apply(actor, action.clone()).unwrap();
                assert_eq!(game.snapshot(), expected);
            }
            let after = game.observe(actor);
            if matches!(action, Action::Pass)
                && let Some(item) = before.stack.last()
                && after.stack.iter().all(|i| i.id != item.id)
                && checked.insert(item.id)
            {
                let n = check(&before, &after, item);
                if n > 0 {
                    checks += n;
                    batches += 1;
                    seen.insert((item.source().expect("example expects an object-backed packet").card.name(), format!("{:?}", item.kind)));
                }
            }
            decisions += 1;
            max_bytes = max_bytes.max(game.state_bytes());
        }
        assert!(game.outcome().is_some());
    }
    assert!(batches > 0);
    println!(
        "Completed {games} supported Standard scaled-token games, {decisions} decisions, {checks} independently checked public facts, {batches} batches, {branches} cloned/replayed boundaries, {max_bytes} max initialized state bytes; resolved {seen:?}; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
}
