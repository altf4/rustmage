use super::tests::{act, cast, scenario, spawn};
use super::*;
const NEW: [&str; 10] = [
    "Camera Launcher",
    "Draconautics Engineer",
    "Greenbelt Guardian",
    "Hazard of the Dunes",
    "Keen Buccaneer",
    "Loot, the Pathfinder",
    "Pacesetter Paragon",
    "Prowcatcher Specialist",
    "Skystreak Engineer",
    "Stampeding Scurryfoot",
];
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let c = Card::from_name(name).unwrap();
    assert!(c.is_supported(), "{name}");
    spawn(g, p, c, zone)
}
fn replay(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let revision = g.revision();
    let p = g.actor().unwrap();
    let mut f = g.clone();
    act(&mut f, a.clone());
    act(g, a.clone());
    let e = g.snapshot();
    assert_eq!(f.snapshot(), e);
    g.restore(&saved).unwrap();
    assert_eq!(g.apply_index(p, revision, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), e);
}
fn settle(g: &mut Game) {
    for _ in 0..200 {
        g.prepare_triggers();
        if g.state.pending == Pending::Priority && g.state.stack.is_empty() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass | Action::PutTrigger { .. } | Action::Discard { .. }
                )
            })
            .expect("explicit fixture choice");
        replay(g, a);
    }
    panic!("did not settle");
}
fn activation(g: &Game, id: CardId, index: u16, target: Option<Target>) -> Option<Action> {
    g.legal_actions().into_iter().find(|a| match a {
        Action::Activate {
            card,
            ability,
            target: t,
            ..
        } => *card == id && *ability == index && *t == target,
        Action::ActivateMana { card, ability, .. } => *card == id && *ability == index,
        _ => false,
    })
}
fn reject(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let rev = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), saved);
    assert_eq!(g.revision(), rev);
}
fn untap_with_spell(g: &mut Game, id: CardId) {
    let s = named(g, Player::P0, "Shore Up", Zone::Hand);
    g.state.players[0].mana = [50; 6];
    cast(g, s, Some(Target::Card(id)));
    settle(g);
    assert!(!g.object_view(id).tapped);
}
#[test]
fn exhaustive_masks_match_immutable_abilities_and_do_not_alias_ordinary_per_turn_uses() {
    let mut count = 0;
    for c in Card::catalogue() {
        let expected = c.activations().iter().enumerate().fold(0, |mask, (i, a)| {
            mask | if a.exhaust { 1u64 << i } else { 0 }
        });
        assert_eq!(c.exhaust_mask(), expected);
        if expected != 0 {
            assert!(NEW.contains(&c.name()) || c.name() == "Rough Rhino Cavalry");
            count += 1;
        }
    }
    assert_eq!(count, 11);
    let mut g = scenario();
    let id = named(&mut g, Player::P0, "Greenbelt Guardian", Zone::Battlefield);
    g.state.players[0].mana = [50; 6];
    for _ in 0..2 {
        let a = activation(&g, id, 0, Some(Target::Card(id))).unwrap();
        replay(&mut g, a);
        settle(&mut g);
    }
    let a = activation(&g, id, 1, None).unwrap();
    replay(&mut g, a);
    settle(&mut g);
    assert_eq!(g.object_view(id).exhausted_abilities, 2);
    assert_eq!(g.object_view(id).used_abilities, 3);
    g.finish_cleanup();
    g.finish_cleanup();
    assert_eq!(g.object_view(id).used_abilities, 2);
    g.state.players[0].mana = [50; 6];
    assert!(activation(&g, id, 0, Some(Target::Card(id))).is_some());
    assert!(activation(&g, id, 1, None).is_none());
}
#[test]
fn every_exhaust_limit_commits_before_effects_survives_turns_and_rejects_all_start_paths() {
    for name in NEW {
        let mut g = scenario();
        let id = named(&mut g, Player::P0, name, Zone::Battlefield);
        g.state.players[0].mana = [50; 6];
        let index = if name == "Greenbelt Guardian" { 1 } else { 0 };
        let a = activation(&g, id, index, None).unwrap();
        let (payment, production) = match &a {
            Action::Activate { payment, .. } => (*payment, None),
            Action::ActivateMana {
                payment,
                production,
                ..
            } => (*payment, Some(*production)),
            _ => unreachable!(),
        };
        replay(&mut g, a);
        assert_eq!(g.object_view(id).exhausted_abilities, 1u64 << index);
        settle(&mut g);
        for _ in 0..2 {
            assert!(activation(&g, id, index, None).is_none());
            reject(
                &mut g,
                Action::AnnounceActivate {
                    card: id,
                    ability: index,
                    counter_payment: Counters::default(),
                },
            );
            reject(
                &mut g,
                Action::BeginActivate {
                    card: id,
                    ability: index,
                    payment,
                    counter_payment: Counters::default(),
                },
            );
            reject(
                &mut g,
                if let Some(production) = production {
                    Action::ActivateMana {
                        card: id,
                        ability: index,
                        payment,
                        counter_payment: Counters::default(),
                        production,
                    }
                } else {
                    Action::Activate {
                        card: id,
                        ability: index,
                        target: None,
                        payment,
                        counter_payment: Counters::default(),
                    }
                },
            );
            if let Some(production) = production {
                reject(
                    &mut g,
                    Action::AnnounceMana {
                        card: id,
                        ability: index,
                        counter_payment: Counters::default(),
                        production,
                    },
                );
            }
            g.finish_cleanup();
            g.finish_cleanup();
            g.state.players[0].mana = [50; 6];
        }
        assert_eq!(g.object_view(id).exhausted_abilities, 1u64 << index);
    }
}
#[test]
fn staged_payment_marks_exhaust_only_at_completion_and_rollback_preserves_the_unspent_choice() {
    let mut g = scenario();
    let id = named(&mut g, Player::P0, "Camera Launcher", Zone::Battlefield);
    for _ in 0..3 {
        named(&mut g, Player::P0, "Forest", Zone::Battlefield);
    }
    replay(
        &mut g,
        Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    assert_eq!(g.object_view(id).exhausted_abilities, 0);
    assert_eq!(g.observe(Player::P0).decision, "pay_costs");
    reject(&mut g, Action::PayCosts { payment: [0; 6] });
    for _ in 0..3 {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::TapMana { .. }))
            .unwrap();
        replay(&mut g, a);
        assert_eq!(g.object_view(id).exhausted_abilities, 0);
    }
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    replay(&mut g, a);
    assert_eq!(g.object_view(id).exhausted_abilities, 1);
    assert_eq!(g.object_view(id).counters.plus_one, 0);
    settle(&mut g);
    assert_eq!(g.object_view(id).counters.plus_one, 1);
}
#[test]
fn independent_engineer_limits_create_exact_tokens_and_haste_excludes_the_source() {
    let mut g = scenario();
    let id = named(
        &mut g,
        Player::P0,
        "Draconautics Engineer",
        Zone::Battlefield,
    );
    let ally = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let enemy = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    g.state.players[0].mana = [50; 6];
    let a = activation(&g, id, 0, None).unwrap();
    replay(&mut g, a);
    settle(&mut g);
    assert!(g.keywords(ally).contains(Keywords::HASTE));
    assert!(!g.keywords(id).contains(Keywords::HASTE));
    assert!(!g.keywords(enemy).contains(Keywords::HASTE));
    assert_eq!(g.object_view(id).counters.plus_one, 1);
    assert!(activation(&g, id, 1, None).is_some());
    let a = activation(&g, id, 1, None).unwrap();
    replay(&mut g, a);
    settle(&mut g);
    assert_eq!(g.object_view(id).exhausted_abilities, 3);
    let token = g
        .observe(Player::P0)
        .objects
        .into_iter()
        .find(|b| b.card.is_token())
        .unwrap();
    assert_eq!((token.power, token.toughness), (4, 4));
    assert_eq!(token.card.colors(), 8);
    assert!(token.card.has_subtype("Dinosaur") && token.card.has_subtype("Dragon"));
    assert!(token.keywords.contains(Keywords::FLYING));
    assert!(!token.card.is_artifact());
    assert!(!token.keywords.contains(Keywords::HASTE));
    g.finish_cleanup();
    assert!(!g.keywords(ally).contains(Keywords::HASTE));
    assert_eq!(g.object_view(id).exhausted_abilities, 3);
}
#[test]
fn leaving_and_reentering_resets_only_the_new_incarnation_and_old_effects_do_not_follow_it() {
    let mut g = scenario();
    let id = named(&mut g, Player::P0, "Camera Launcher", Zone::Battlefield);
    g.state.players[0].mana = [50; 6];
    let a = activation(&g, id, 0, None).unwrap();
    replay(&mut g, a);
    let old = g.object_ref(id);
    g.move_object(id, Zone::Hand);
    g.move_object(id, Zone::Battlefield);
    assert_ne!(g.object_ref(id), old);
    assert_eq!(g.object_view(id).exhausted_abilities, 0);
    settle(&mut g);
    assert_eq!(g.object_view(id).counters.plus_one, 0);
    let tokens: Vec<_> = g
        .observe(Player::P0)
        .objects
        .into_iter()
        .filter(|b| b.card.is_token())
        .collect();
    assert_eq!(tokens.len(), 1);
    let t = &tokens[0];
    assert_eq!((t.power, t.toughness), (1, 1));
    assert_eq!(t.card.colors(), 0);
    assert!(
        t.card.is_artifact()
            && t.card.has_subtype("Thopter")
            && t.keywords.contains(Keywords::FLYING)
    );
    let a = activation(&g, id, 0, None).unwrap();
    replay(&mut g, a);
    settle(&mut g);
    assert_eq!(g.object_view(id).counters.plus_one, 1);
    assert_eq!(g.object_view(id).exhausted_abilities, 1);
}
#[test]
fn countered_effect_does_not_refund_exhaust_and_a_real_return_spell_allows_new_activation() {
    let mut g = scenario();
    let id = named(
        &mut g,
        Player::P0,
        "Prowcatcher Specialist",
        Zone::Battlefield,
    );
    g.state.players[0].mana = [50; 6];
    let a = activation(&g, id, 0, None).unwrap();
    replay(&mut g, a);
    // Structural countering fixture: this pool has no admitted ability-counter
    // spell. Removing the stack item cannot refund a completed activation.
    g.state.stack.pop();
    assert_eq!(g.object_view(id).counters.plus_one, 0);
    assert!(activation(&g, id, 0, None).is_none());
    let spell = named(&mut g, Player::P0, "Unsummon", Zone::Hand);
    cast(&mut g, spell, Some(Target::Card(id)));
    settle(&mut g);
    assert_eq!(g.object_view(id).zone, Zone::Hand);
    assert_eq!(g.object_view(id).exhausted_abilities, 0);
    g.state.step = Step::Main1;
    g.state.active = Player::P0;
    cast(&mut g, id, None);
    settle(&mut g);
    assert!(activation(&g, id, 0, None).is_some());
}
#[test]
fn private_keen_discard_resumes_after_draw_without_refunding_or_repeating_its_limit() {
    let mut g = scenario();
    let id = named(&mut g, Player::P0, "Keen Buccaneer", Zone::Battlefield);
    g.state.players[0].mana = [50; 6];
    let a = activation(&g, id, 0, None).unwrap();
    replay(&mut g, a);
    replay(&mut g, Action::Pass);
    replay(&mut g, Action::Pass);
    assert_eq!(g.observe(Player::P0).decision, "effect_discard");
    assert_eq!(g.object_view(id).exhausted_abilities, 1);
    assert_eq!(g.object_view(id).counters.plus_one, 0);
    assert!(g.observe(Player::P1).actions.is_empty());
    assert!(g.observe(Player::P1).effect_discards.is_empty());
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::Discard { .. }))
        .unwrap();
    replay(&mut g, a);
    assert_eq!(g.object_view(id).counters.plus_one, 1);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
    assert_eq!(g.state.players[0].cards_discarded_this_turn, 1);
    assert!(activation(&g, id, 0, None).is_none());
}
#[test]
fn loot_mana_draw_and_damage_are_independently_exhausted_even_after_real_untaps() {
    let mut g = scenario();
    let id = named(
        &mut g,
        Player::P0,
        "Loot, the Pathfinder",
        Zone::Battlefield,
    );
    g.state.objects[id as usize].summoning_sick = true;
    g.state.players[0].mana = [50; 6];
    assert!(g.keywords(id).contains(Keywords::HASTE));
    assert!(g.keywords(id).contains(Keywords::DOUBLE_STRIKE));
    assert!(g.keywords(id).contains(Keywords::VIGILANCE));
    let choices: Vec<_> = g
        .legal_actions()
        .into_iter()
        .filter_map(|a| match a {
            Action::ActivateMana {
                card,
                ability: 0,
                production,
                ..
            } if card == id => Some(production),
            _ => None,
        })
        .collect();
    assert_eq!(choices.len(), 5);
    for production in choices {
        assert_eq!(production.iter().sum::<u32>(), 3);
        assert_eq!(production.iter().filter(|&&n| n == 3).count(), 1);
        assert_eq!(production[5], 0);
    }
    reject(
        &mut g,
        Action::ActivateMana {
            card: id,
            ability: 0,
            payment: [0, 0, 0, 0, 1, 0],
            counter_payment: Counters::default(),
            production: [1, 1, 0, 1, 0, 0],
        },
    );
    let a=g.legal_actions().into_iter().find(|a|matches!(a,Action::ActivateMana{card,ability:0,production:[0,0,0,0,3,0],..}if *card==id)).unwrap();
    replay(&mut g, a);
    assert_eq!(g.state.players[0].mana[4], 52);
    assert!(g.state.stack.is_empty());
    assert_eq!(g.object_view(id).exhausted_abilities, 1);
    untap_with_spell(&mut g, id);
    assert!(activation(&g, id, 0, None).is_none());
    let a = activation(&g, id, 1, None).unwrap();
    replay(&mut g, a);
    settle(&mut g);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 3);
    assert_eq!(g.object_view(id).exhausted_abilities, 3);
    untap_with_spell(&mut g, id);
    let life = g.state.players[1].life;
    let a = activation(&g, id, 2, Some(Target::Player(Player::P1))).unwrap();
    replay(&mut g, a);
    settle(&mut g);
    assert_eq!(g.state.players[1].life, life - 3);
    assert_eq!(g.object_view(id).exhausted_abilities, 7);
    g.finish_cleanup();
    g.finish_cleanup();
    assert_eq!(g.object_view(id).exhausted_abilities, 7);
}
#[test]
fn remaining_full_bodies_have_exact_counters_keywords_and_elephant_factory() {
    for (name, n) in [
        ("Greenbelt Guardian", 3),
        ("Hazard of the Dunes", 3),
        ("Pacesetter Paragon", 1),
        ("Prowcatcher Specialist", 2),
        ("Skystreak Engineer", 2),
        ("Stampeding Scurryfoot", 1),
    ] {
        let mut g = scenario();
        let id = named(&mut g, Player::P0, name, Zone::Battlefield);
        g.state.players[0].mana = [50; 6];
        let index = if name == "Greenbelt Guardian" { 1 } else { 0 };
        let a = activation(&g, id, index, None).unwrap();
        replay(&mut g, a);
        settle(&mut g);
        assert_eq!(g.object_view(id).counters.plus_one, n);
        if name == "Pacesetter Paragon" {
            assert!(g.keywords(id).contains(Keywords::DOUBLE_STRIKE));
            g.finish_cleanup();
            assert!(!g.keywords(id).contains(Keywords::DOUBLE_STRIKE));
        }
        if name == "Stampeding Scurryfoot" {
            let token = g
                .observe(Player::P0)
                .objects
                .into_iter()
                .find(|b| b.card.is_token())
                .unwrap();
            assert_eq!((token.power, token.toughness), (3, 3));
            assert_eq!(token.card.colors(), 16);
            assert!(token.card.has_subtype("Elephant"));
            assert!(!token.card.is_artifact());
        }
    }
}
