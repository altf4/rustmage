//! Whole admitted Standard walker games through the public in-process API.
use rustmage::{Action, Card, Config, DeckRules, Game, Observation, Player, StackKind, Zone};
use std::collections::BTreeSet;
fn deck() -> Vec<Card> {
    let mut result = vec![Card::Forest; 12];
    result.extend([Card::Swamp; 12]);
    for name in [
        "Professor Dellian Fel",
        "Bear Cub",
        "Llanowar Elves",
        "Moment of Craving",
        "Murder",
        "Giant Growth",
        "Grasping Longneck",
        "Gigantosaurus",
        "Druid of the Cowl",
    ] {
        let card = Card::from_name(name).unwrap();
        assert!(card.is_supported() && card.is_standard_legal(), "{name}");
        result.extend([card; 4]);
    }
    assert_eq!(result.len(), 60);
    result
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
    let mut resolved = BTreeSet::new();
    let mut walker_attacks = 0;
    let mut emblems = 0;
    let mut emblem_triggers = 0;
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [deck(), deck()],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })
        .unwrap();
        for count in 0..100_000 {
            if g.outcome().is_some() {
                break;
            }
            assert!(count < 99_999, "did not end naturally");
            let actor = g.actor().unwrap();
            let before = g.observe(actor);
            let actions = g.legal_actions();
            let wanted = (seed % 4) as u16;
            let is_professor = |id| {
                before
                    .objects
                    .iter()
                    .any(|o| o.id == id && o.card.name() == "Professor Dellian Fel")
            };
            let chosen=actions.iter().position(|a|matches!(a,Action::Activate{card,ability,..}if is_professor(*card)&&*ability==wanted))
                .or_else(||if wanted==3{actions.iter().position(|a|matches!(a,Action::Activate{card,ability:0,..}if is_professor(*card)))}else{None})
                .or_else(||actions.iter().position(|a|matches!(a,Action::AttackPlaneswalker{..})))
                .unwrap_or_else(||rustmage::policy::smoke_index(&g,&actions));
            let action = actions[chosen].clone();
            let saved = g.snapshot();
            let expected = if count % 4 == 0 {
                let mut fork = g.clone();
                fork.apply(actor, action.clone()).unwrap();
                branches += 1;
                Some(fork.snapshot())
            } else {
                None
            };
            g.apply(actor, action.clone()).unwrap();
            if let Some(expected) = expected {
                assert_eq!(g.snapshot(), expected);
                g.restore(&saved).unwrap();
                assert_eq!(
                    g.apply_index(actor, before.revision, chosen),
                    Err(rustmage::Error::StaleRevision)
                );
                g.apply(actor, action.clone()).unwrap();
                assert_eq!(g.snapshot(), expected);
            }
            let after = g.observe(actor);
            if let Action::AttackPlaneswalker { card, defender } = action {
                assert!(after.attack_declarations.iter().any(|a| a.card == card
                    && a.defender == Some(rustmage::CombatDefender::Planeswalker(defender))));
                checks += 1;
                walker_attacks += 1;
            }
            if let Action::Activate { card, ability, .. } = action
                && is_professor(card)
            {
                let old = before.objects.iter().find(|o| o.id == card).unwrap();
                let now = after.objects.iter().find(|o| o.id == card).unwrap();
                let expected = i128::from(old.counters.loyalty) + [2, 0, -3, -6][ability as usize];
                if expected == 0 {
                    assert_eq!(now.zone, Zone::Graveyard);
                } else {
                    assert_eq!(now.counters.loyalty as i128, expected);
                    assert_eq!(now.used_abilities, 1 << ability);
                }
                assert_eq!(
                    before.players[actor.index()].life,
                    after.players[actor.index()].life
                );
                checks += 2;
            }
            if matches!(action, Action::Pass)
                && before.stack.len() == after.stack.len() + 1
                && before.resolving.is_none()
                && after.resolving.is_none()
            {
                let item = before.stack.last().unwrap();
                let p = item.controller.index();
                if item.source().is_some_and(|source| source.card.name() == "Professor Dellian Fel")
                    && let StackKind::ActivatedAbility { ability } = item.kind
                {
                    resolved.insert(ability);
                    match ability {
                        0 => assert_eq!(after.players[p].life, before.players[p].life + 3),
                        1 => {
                            assert_eq!(after.players[p].life, before.players[p].life - 1);
                            assert_eq!(
                                after.players[p].library_count,
                                before.players[p].library_count.saturating_sub(1)
                            );
                        }
                        2 => {
                            if let Some(t) = item.targets.first().and_then(|t| t.object)
                                && before
                                    .objects
                                    .iter()
                                    .any(|o| o.reference == t && o.zone == Zone::Battlefield)
                            {
                                assert_eq!(
                                    after.objects.iter().find(|o| o.id == t.card).unwrap().zone,
                                    Zone::Graveyard
                                );
                            }
                        }
                        3 => {
                            assert_eq!(emblem_count(&after), emblem_count(&before) + 1);
                            emblems += 1;
                        }
                        _ => unreachable!(),
                    }
                    checks += 1;
                }
                if item.source_card.is_emblem() {
                    emblem_triggers += 1;
                    let amount = item.life_event.unwrap().amount;
                    let opponent = item.controller.other().index();
                    assert_eq!(
                        after.players[opponent].life,
                        before.players[opponent].life - amount
                    );
                    assert!(
                        after
                            .objects
                            .iter()
                            .any(|o| o.reference == item.object && o.zone == Zone::Command)
                    );
                    checks += 2;
                }
            }
            decisions += 1;
            max_bytes = max_bytes.max(g.state_bytes());
        }
        assert!(g.outcome().is_some());
    }
    assert!(!resolved.is_empty());
    assert!(walker_attacks > 0);
    assert!(emblems > 0);
    assert!(emblem_triggers > 0);
    println!(
        "Completed {games} whole-walker Standard games, {decisions} decisions, {checks} independently checked public facts, {branches} cloned/replayed boundaries, {max_bytes} max initialized state bytes; resolved abilities {resolved:?}, {walker_attacks} planeswalker attacks, {emblems} emblems, {emblem_triggers} emblem triggers; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
}
fn emblem_count(obs: &Observation) -> usize {
    obs.objects
        .iter()
        .filter(|o| o.zone == Zone::Command && o.emblem.is_some())
        .count()
}
