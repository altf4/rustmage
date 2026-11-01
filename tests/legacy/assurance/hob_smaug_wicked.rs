//! Treasure-source mana provenance and the actual cast event.
use super::*;
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..120 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Smaug decision budget {:?}", g.state.pending)
}
fn finish(g: &mut Game) {
    until(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
            && g.state.resolving.is_none()
    });
}
fn treasures(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(i, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && g.has_subtype(i as CardId, "Treasure"))
            .then_some(i as CardId)
        })
        .collect()
}
fn mana(g: &mut Game, source: CardId, production: Mana) {
    let action = g.legal_actions().into_iter().find(|a| matches!(a, Action::ActivateMana {card, production: m, ..} if *card == source && *m == production) || matches!(a, Action::TapMana { card } if *card == source && production == [0, 0, 0, 1, 0, 0]))
        .unwrap_or_else(|| panic!("missing mana ability for source={source} view={:?} actions={:?}",g.object_view(source),g.legal_actions()));
    act(g, action);
}

fn paid_smaug(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Smaug, Wicked Worm", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 1, 1, 0, 3];
    cast(g, id, None);
    finish(g);
    id
}
#[test]
fn wicked_smaug_paid_etb_counts_opponents_and_tapped_token_replacement() {
    for p in [Player::P0, Player::P1] {
        for doubled in [false, true] {
            let mut g = scenario(p);
            put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
            put(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
            put(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
            if doubled {
                put(&mut g, p, "Doubling Season", Zone::Battlefield);
            }
            let smaug = paid_smaug(&mut g, p);
            assert_eq!(stats(&g, smaug), (5, 5));
            assert!(g.object_view(smaug).keywords.contains(Keywords::FLYING));
            let found = treasures(&g, p);
            assert_eq!(found.len(), if doubled { 4 } else { 2 });
            assert!(found.iter().all(|&id| g.object_view(id).tapped));
        }
    }
}
#[test]
fn wicked_smaug_treasure_mana_source_survives_sacrifice_and_triggers_cast() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let smaug = paid_smaug(&mut g, p);
        put(&mut g, p, "Forest", Zone::Library);
        let treasure = put(
            &mut g,
            p,
            "@token/b60f6cd3b44b22bd289fe4e2",
            Zone::Battlefield,
        );
        mana(&mut g, treasure, [0, 0, 0, 1, 0, 0]);
        assert_ne!(g.object_view(treasure).zone, Zone::Battlefield);
        assert!(
            g.observe(p)
                .mana_lots
                .iter()
                .any(|lot| lot.player == p && lot.treasure_source)
        );
        contracts::branch_contract(g.clone());
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, Some(Target::Player(p.other())));
        g.move_object(smaug, Zone::Graveyard);
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].life, 19);
        assert!(
            g.state
                .objects
                .iter()
                .any(|o| o.owner == p && o.card == Card::Forest && o.zone == Zone::Hand)
        );
    }
}

#[test]
fn wicked_smaug_selects_actual_mixed_mana_and_preserves_unspent_treasure() {
    for p in [Player::P0, Player::P1] {
        for use_treasure in [false, true] {
            let mut g = scenario(p);
            paid_smaug(&mut g, p);
            put(&mut g, p, "Forest", Zone::Library);
            let token = put(
                &mut g,
                p,
                "@token/b60f6cd3b44b22bd289fe4e2",
                Zone::Battlefield,
            );
            mana(&mut g, token, [0, 0, 0, 1, 0, 0]);
            g.state.players[p.index()].mana[3] += 1;
            let shock = put(&mut g, p, "Shock", Zone::Hand);
            cast(&mut g, shock, Some(Target::Player(p.other())));
            assert!(matches!(g.state.pending, Pending::ManaLotSpend { .. }));
            contracts::branch_contract(g.clone());
            act(
                &mut g,
                Action::ChooseManaLotSpend {
                    amount: u32::from(use_treasure),
                },
            );
            finish(&mut g);
            assert_eq!(
                g.state.players[p.index()].life,
                if use_treasure { 19 } else { 20 }
            );
            assert_eq!(
                g.observe(p)
                    .mana_lots
                    .iter()
                    .any(|lot| lot.player == p && lot.treasure_source),
                !use_treasure
            );
        }
    }
}
#[test]
fn wicked_smaug_uses_actual_source_type_not_token_or_printed_name() {
    for p in [Player::P0, Player::P1] {
        for is_treasure in [false, true] {
            let mut g = scenario(p);
            paid_smaug(&mut g, p);
            put(&mut g, p, "Forest", Zone::Library);
            let producer = if is_treasure {
                put(&mut g, p, "Goldvein Pick", Zone::Battlefield)
            } else {
                put(
                    &mut g,
                    p,
                    "@token/b60f6cd3b44b22bd289fe4e2",
                    Zone::Battlefield,
                )
            };
            let face = if is_treasure {
                "@token/b60f6cd3b44b22bd289fe4e2"
            } else {
                "Mountain"
            };
            g.apply_temporary_copy(
                g.object_ref(producer),
                CopyRecipe::new(Card::from_name(face).unwrap()),
                CopyDeadline::EndOfTurn,
            );
            assert_eq!(g.has_subtype(producer, "Treasure"), is_treasure);
            mana(&mut g, producer, [0, 0, 0, 1, 0, 0]);
            let shock = put(&mut g, p, "Shock", Zone::Hand);
            cast(&mut g, shock, Some(Target::Player(p.other())));
            finish(&mut g);
            assert_eq!(
                g.state.players[p.index()].life,
                if is_treasure { 19 } else { 20 }
            );
        }
    }
}

#[path = "hob_smaug_wicked_rollback.rs"]
mod rollback;

#[test]
fn wicked_smaug_granted_mana_belongs_to_its_actual_source() {
    use crate::cards::{AnimationCreatureTypes, AnimationDuration, AnimationSpec};
    for p in [Player::P0, Player::P1] {
        for is_treasure in [false, true] {
            let mut g = scenario(p);
            paid_smaug(&mut g, p);
            put(&mut g, p, "Forest", Zone::Library);
            let source = if is_treasure {
                let id = put(
                    &mut g,
                    p,
                    "@token/b60f6cd3b44b22bd289fe4e2",
                    Zone::Battlefield,
                );
                g.change_object_form(
                    g.object_ref(id),
                    AnimationSpec {
                        base: Some((2, 2)),
                        artifact: true,
                        creature_types: AnimationCreatureTypes::Named(&[]),
                        keywords: Keywords::NONE,
                        duration: AnimationDuration::Permanent,
                    },
                    characteristic_layers::AnimationTypeMode::AddCreature,
                );
                id
            } else {
                put(&mut g, p, "Bear Cub", Zone::Battlefield)
            };
            let ring = put(&mut g, p, "Lotus Ring", Zone::Battlefield);
            assert!(g.attach_equipment(g.object_ref(ring), g.object_ref(source)));
            let grantor = g.object_ref(ring);
            act(
                &mut g,
                Action::TapGrantedMana {
                    card: source,
                    grantor,
                    production: [0, 0, 3, 0, 0, 0],
                },
            );
            assert_ne!(g.object_view(source).zone, Zone::Battlefield);
            let victim = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let spell = put(&mut g, p, "Stab", Zone::Hand);
            cast(&mut g, spell, Some(Target::Card(victim)));
            finish(&mut g);
            assert_eq!(
                g.state.players[p.index()].life,
                if is_treasure { 19 } else { 20 }
            );
        }
    }
}

#[test]
fn wicked_smaug_counts_opponent_artifacts_at_resolution_after_source_leaves() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let old = put(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
        let smaug = put(&mut g, p, "Smaug, Wicked Worm", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 1, 1, 0, 3];
        cast(&mut g, smaug, None);
        until(&mut g, |g| {
            g.state
                .stack
                .iter()
                .any(|item| matches!(item.kind, StackKind::TriggeredAbility { ability: 0 }))
        });
        g.move_object(old, Zone::Graveyard);
        put(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
        put(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
        g.move_object(smaug, Zone::Graveyard);
        finish(&mut g);
        assert_eq!(treasures(&g, p).len(), 2);
        assert!(treasures(&g, p).iter().all(|&id| g.object_view(id).tapped));
    }
}
#[test]
fn wicked_smaug_mana_spent_on_non_treasure_mana_ability_is_not_cast_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        paid_smaug(&mut g, p);
        let treasure = put(
            &mut g,
            p,
            "@token/b60f6cd3b44b22bd289fe4e2",
            Zone::Battlefield,
        );
        mana(&mut g, treasure, [0, 0, 0, 1, 0, 0]);
        let faun = put(&mut g, p, "Rootrider Faun", Zone::Battlefield);
        act(
            &mut g,
            Action::ActivateMana {
                card: faun,
                ability: 1,
                payment: [0, 0, 0, 1, 0, 0],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 1, 0, 0],
            },
        );
        assert!(
            !g.observe(p)
                .mana_lots
                .iter()
                .any(|lot| lot.player == p && lot.treasure_source)
        );
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, Some(Target::Player(p.other())));
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].life, 20);
    }
}

#[test]
fn wicked_smaug_opponents_treasure_casts_and_spell_copies_do_not_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let smaug = paid_smaug(&mut g, p);
        put(&mut g, p, "Forest", Zone::Library);
        g.state.active = p.other();
        g.state.priority = p.other();
        g.state.players[p.other().index()].mana = [0; 6];
        let enemy_treasure = put(
            &mut g,
            p.other(),
            "@token/b60f6cd3b44b22bd289fe4e2",
            Zone::Battlefield,
        );
        mana(&mut g, enemy_treasure, [0, 0, 0, 1, 0, 0]);
        let enemy_shock = put(&mut g, p.other(), "Shock", Zone::Hand);
        cast(&mut g, enemy_shock, Some(Target::Player(p)));
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].life, 18);
        g.state.active = p;
        g.state.priority = p;
        let own_treasure = put(
            &mut g,
            p,
            "@token/b60f6cd3b44b22bd289fe4e2",
            Zone::Battlefield,
        );
        mana(&mut g, own_treasure, [0, 0, 0, 1, 0, 0]);
        g.state.players[p.index()].mana[5] = 1;
        let original = put(&mut g, p, "Goldvein Pick", Zone::Hand);
        cast(&mut g, original, None);
        let template = g
            .state
            .stack
            .iter()
            .find(|item| item.source.unwrap().id == original && item.kind == StackKind::Spell)
            .unwrap()
            .clone();
        let triggers = |g: &Game| {
            g.state
                .pending_triggers
                .iter()
                .chain(&g.state.ordering_triggers)
                .chain(g.state.stack.iter())
                .filter(|item| {
                    item.source.unwrap().id == smaug
                        && matches!(item.kind, StackKind::TriggeredAbility { ability: 1 })
                })
                .count()
        };
        let before = triggers(&g);
        assert_eq!(before, 1);
        let mut copy_effect = template.clone();
        copy_effect.copy_event = Some(Box::new(CopyEvent {
            spell: template,
            amount: 1,
        }));
        assert!(!g.copy_captured_spell(&copy_effect));
        assert!(g.state.stack.last().unwrap().is_copy);
        assert!(g.state.stack.last().unwrap().actual_cast_event().is_none());
        assert_eq!(triggers(&g), before);
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].life, 17);
    }
}

#[test]
fn wicked_smaug_departed_mana_source_uses_latest_exact_incarnation() {
    for p in [Player::P0, Player::P1] {
        for treasure_at_departure in [false, true] {
            let mut g = scenario(p);
            let token = put(
                &mut g,
                p,
                "@token/b60f6cd3b44b22bd289fe4e2",
                Zone::Battlefield,
            );
            let item = g.activation_item(p, token, 0);
            g.state.stack.push(item);
            if !treasure_at_departure {
                g.apply_temporary_copy(
                    g.object_ref(token),
                    CopyRecipe::new(Card::from_name("Mountain").unwrap()),
                    CopyDeadline::EndOfTurn,
                );
            }
            g.move_object(token, Zone::Exile);
            let item = g.state.stack.last().unwrap();
            let quality = g.item_mana_quality(item, ManaProduction::Fixed([0, 0, 0, 1, 0, 0]));
            assert_eq!(
                quality.is_some_and(|q| q.treasure_source),
                treasure_at_departure
            );
        }
    }
}
