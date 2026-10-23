//! Travelers creates independent, fully captured triggers rather than stack copies.
use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::*;
fn count(g: &Game, id: CardId) -> usize {
    g.state
        .pending_triggers
        .iter()
        .chain(&g.state.ordering_triggers)
        .chain(&g.state.stack)
        .filter(|item| item.card == id)
        .count()
}
#[test]
fn ecl_twinflame_paid_body_additive_providers_and_frozen_counts_after_departure() {
    for p in [Player::P0, Player::P1] {
        for amount in 0..=2 {
            let mut g = setup(p);
            let mut providers = Vec::new();
            for _ in 0..amount {
                let id = named(&mut g, p, "Twinflame Travelers", Zone::Hand);
                let before = g.state.players[p.index()].mana.iter().sum::<u32>();
                cast(&mut g, id, None, &[]);
                settle(&mut g);
                assert_eq!(
                    g.state.players[p.index()].mana.iter().sum::<u32>(),
                    before - 4
                );
                assert_eq!(
                    g.stats(id),
                    Stats {
                        power: 3,
                        toughness: 3
                    }
                );
                assert!(g.keywords(id).contains(Keywords::FLYING));
                providers.push(id);
            }
            let id = named(&mut g, p, "Shimmercreep", Zone::Hand);
            g.move_object(id, Zone::Battlefield);
            let before = g.state.players[p.other().index()].life;
            let view = g.observe(p);
            assert_eq!(
                view.pending_trigger_multiplicities.len(),
                usize::from(amount != 0)
            );
            if amount != 0 {
                assert_eq!(view.pending_trigger_multiplicities[0].additional, amount);
            }
            for provider in providers {
                g.move_object(provider, Zone::Exile);
            }
            g.prepare_triggers();
            assert_eq!(count(&g, id), amount as usize + 1);
            let items: Vec<_> = g
                .state
                .ordering_triggers
                .iter()
                .chain(&g.state.stack)
                .filter(|i| i.card == id)
                .collect();
            assert!(
                items
                    .iter()
                    .all(|i| !i.is_copy && i.entering_object == Some(g.object_ref(id)))
            );
            let mut ids: Vec<_> = items.iter().map(|i| i.id).collect();
            ids.sort_unstable();
            ids.dedup();
            assert_eq!(ids.len(), amount as usize + 1);
            assert!(g.observe(p).pending_trigger_multiplicities.is_empty());
            // No re-expansion when the same batch is prepared again.
            g.prepare_triggers();
            assert_eq!(count(&g, id), amount as usize + 1);
            settle(&mut g);
            // Providers departed, so vivid sees only black at resolution.
            assert_eq!(
                g.state.players[p.other().index()].life,
                before - i128::from(amount + 1)
            );
        }
    }
}
fn grant_elemental_fixture(g: &mut Game, source: CardId) {
    // Raw typed layer-four boundary: this may also apply to a Kindred noncreature.
    let timestamp = g.next_role_timestamp();
    let recipient = g.object_ref(source);
    g.state
        .roles
        .get_or_insert_with(Default::default)
        .creature_types
        .push(subtypes::CreatureTypesEffectView {
            recipient,
            source: recipient,
            types: &[],
            additional: false,
            all_creature_types: true,
            expires_turn: None,
            timestamp,
        });
    assert!(g.has_subtype(source, "Elemental"));
}
#[test]
fn ecl_twinflame_independent_paid_trigger_targets_and_invalid_choice_rollback() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Twinflame Travelers", Zone::Battlefield);
        let victims = [
            named(&mut g, p.other(), "Skybeast Tracker", Zone::Battlefield),
            named(&mut g, p.other(), "Skybeast Tracker", Zone::Battlefield),
        ];
        let source = named(&mut g, p, "Explosive Prodigy", Zone::Hand);
        cast(&mut g, source, None, &[]);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert_eq!(count(&g, source), 2);
        let id = g.state.ordering_triggers[0].id;
        checked(&mut g, Action::PutTrigger { id });
        let before = g.snapshot();
        let revision = g.revision();
        assert_eq!(
            g.apply(
                p,
                Action::ChooseTarget {
                    target: Target::Card(source)
                }
            ),
            Err(Error::IllegalAction)
        );
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.revision(), revision);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(victims[0]),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(victims[1]),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        assert_eq!(g.state.stack.len(), 2);
        assert_ne!(g.state.stack[0].targets, g.state.stack[1].targets);
        settle(&mut g);
        for victim in victims {
            assert_eq!(g.object_view(victim).damage, 2);
        }
    }
}
#[test]
fn ecl_twinflame_source_control_kindred_noncreature_changeling_and_ability_removal() {
    for p in [Player::P0, Player::P1] {
        for source_name in ["Morcant's Eyes", "Flying Octobot"] {
            let mut g = setup(p);
            let provider = named(&mut g, p, "Twinflame Travelers", Zone::Battlefield);
            named(&mut g, p.other(), "Twinflame Travelers", Zone::Battlefield);
            let source = named(&mut g, p, source_name, Zone::Battlefield);
            grant_elemental_fixture(&mut g, source);
            assert_eq!(g.is_creature(source), source_name != "Morcant's Eyes");
            assert!(g.queue_trigger(&g.object_view(source), 0));
            g.prepare_triggers();
            assert_eq!(
                count(&g, source),
                if source_name == "Flying Octobot" {
                    1
                } else {
                    2
                }
            );
            if source_name == "Flying Octobot" {
                assert!(!g.queue_trigger(&g.object_view(source), 0));
            }
            // Restore a new event with provider's printed rule removed.
            let mut h = setup(p);
            let provider2 = named(&mut h, p, "Twinflame Travelers", Zone::Battlefield);
            h.remove_abilities_exact(
                &[h.object_ref(provider2)],
                crate::cards::AnimationDuration::Permanent,
            );
            let changeling = named(&mut h, p, "Sizzling Changeling", Zone::Battlefield);
            h.move_object(changeling, Zone::Graveyard);
            h.prepare_triggers();
            assert_eq!(count(&h, changeling), 1);
            assert!(g.state.static_sources.contains(&provider));
        }
    }
}
#[test]
fn ecl_twinflame_simultaneous_deaths_freeze_provider_source_and_complete_zone_packets() {
    for p in [Player::P0, Player::P1] {
        for reverse in [false, true] {
            let mut g = setup(p);
            let provider = named(&mut g, p, "Twinflame Travelers", Zone::Battlefield);
            let source = named(&mut g, p, "Sizzling Changeling", Zone::Battlefield);
            let original = g.object_ref(source);
            let observers = g.death_trigger_sources();
            let ids = if reverse {
                [source, provider]
            } else {
                [provider, source]
            };
            let events = g.move_objects_raw(&ids, Zone::Graveyard);
            g.finish_zone_events(&events, &observers);
            assert_eq!(g.observe(p).pending_trigger_multiplicities[0].additional, 1);
            g.prepare_triggers();
            assert_eq!(count(&g, source), 2);
            for item in &g.state.ordering_triggers {
                assert_eq!(item.object, original);
                assert_eq!(
                    item.source_move.as_ref().unwrap().after,
                    g.object_ref(source)
                );
            }
            let before = g.state.players[p.index()].library.len();
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].library.len(), before - 2);
        }
    }
}
#[test]
fn ecl_twinflame_simultaneous_non_graveyard_departures_preserve_lookback_and_new_incarnations() {
    for destination in [Zone::Hand, Zone::Exile] {
        let mut g = setup(Player::P0);
        let provider = named(&mut g, Player::P0, "Twinflame Travelers", Zone::Battlefield);
        let source = named(
            &mut g,
            Player::P0,
            "Champion of the Path",
            Zone::Battlefield,
        );
        let old = g.object_ref(source);
        let events = g.move_objects_raw(&[provider, source], destination);
        g.finish_zone_events(&events, &[]);
        let after = g.object_ref(source);
        g.move_object(source, Zone::Battlefield);
        g.prepare_triggers();
        assert_eq!(count(&g, source), 2);
        assert!(g.state.ordering_triggers.iter().all(|i| i.object == old
            && match destination {
                Zone::Hand => i.source_move.is_none(),
                _ => i.source_move.as_ref().is_some_and(|m| m.after == after),
            }));
        settle(&mut g);
        assert!(g.on_battlefield(source));
        assert_ne!(g.object_ref(source), old);
    }
}
#[test]
fn ecl_twinflame_granted_ward_is_owned_by_recipient_and_another_excludes_provider_itself() {
    for amount in 1..=2 {
        let mut g = setup(Player::P0);
        let source = named(&mut g, Player::P0, "Twinflame Travelers", Zone::Battlefield);
        if amount == 2 {
            named(&mut g, Player::P0, "Twinflame Travelers", Zone::Battlefield);
        }
        named(&mut g, Player::P0, "Hexing Squelcher", Zone::Battlefield);
        let shock = named(&mut g, Player::P1, "Shock", Zone::Hand);
        g.state.active = Player::P1;
        g.state.priority = Player::P1;
        cast(&mut g, shock, None, &[Target::Card(source)]);
        assert_eq!(count(&g, source), amount);
        let items: Vec<_> = g
            .state
            .stack
            .iter()
            .chain(&g.state.ordering_triggers)
            .filter(|i| i.card == source)
            .collect();
        assert!(
            items
                .iter()
                .all(
                    |i| i.attachment_trigger.as_ref().unwrap().recipient == g.object_ref(source)
                        && i.related_stack.is_some()
                )
        );
        settle(&mut g);
    }
}
#[test]
fn ecl_twinflame_intrinsic_copy_end_step_triggers_double_but_delayed_and_reflexive_do_not() {
    let mut g = setup(Player::P0);
    named(&mut g, Player::P0, "Twinflame Travelers", Zone::Battlefield);
    let source = named(&mut g, Player::P0, "Sizzling Changeling", Zone::Battlefield);
    let kindle = named(&mut g, Player::P0, "Kindle the Inner Flame", Zone::Hand);
    cast(&mut g, kindle, None, &[Target::Card(source)]);
    settle(&mut g);
    let token = g
        .state
        .objects
        .iter()
        .enumerate()
        .find_map(|(id, o)| (o.zone == Zone::Battlefield && o.is_token()).then_some(id as CardId))
        .unwrap();
    g.queue_copy_end_step_abilities();
    g.prepare_triggers();
    assert_eq!(count(&g, token), 2);
    assert!(
        g.state
            .ordering_triggers
            .iter()
            .all(|i| i.kind == StackKind::CopyEndStepSacrifice && !i.is_copy)
    );
    settle(&mut g);
    let nuisance = named(
        &mut g,
        Player::P0,
        "Flitterwing Nuisance",
        Zone::Battlefield,
    );
    grant_elemental_fixture(&mut g, nuisance);
    let item = g.activation_item(Player::P0, nuisance, 0);
    g.queue_reflexive(&item, 0, None);
    g.prepare_triggers();
    assert_eq!(count(&g, nuisance), 1);
    settle(&mut g);
    let item = g.activation_item(Player::P0, nuisance, 0);
    g.remember_combat_draw_observer(&item, 0);
    g.delayed_combat_draw_triggers(&[DamageEvent {
        preventable: true,
        source: g.damage_source(nuisance),
        target: Target::Player(Player::P1),
        amount: 1,
        combat: true,
    }]);
    g.prepare_triggers();
    assert_eq!(count(&g, nuisance), 1);
    assert!(
        g.observe(Player::P0)
            .pending_trigger_multiplicities
            .is_empty()
    );
    settle(&mut g);
}
#[test]
fn ecl_twinflame_frozen_cast_event_source_control_and_removal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let provider = named(&mut g, p, "Twinflame Travelers", Zone::Battlefield);
        let source = named(&mut g, p, "Kulrath Mystic", Zone::Battlefield);
        let second = named(&mut g, p, "Twinflame Travelers", Zone::Hand);
        cast(&mut g, second, None, &[]);
        let items: Vec<_> = g
            .state
            .ordering_triggers
            .iter()
            .chain(&g.state.stack)
            .filter(|i| i.card == source)
            .collect();
        assert_eq!(items.len(), 2);
        assert!(items.iter().all(|i| i.cast_event.is_none() && !i.is_copy));
        assert_eq!(items[0].cast_event, items[1].cast_event);
        settle(&mut g);
        assert_eq!(g.stats(source).power, 6);
        // Providers arriving after this cast event cannot retroactively add one.
        assert!(g.on_battlefield(second));
        let eyes = named(&mut g, p, "Morcant's Eyes", Zone::Battlefield);
        grant_elemental_fixture(&mut g, eyes);
        g.record_control(g.object_ref(provider), p.other(), None);
        g.record_control(g.object_ref(second), p.other(), None);
        assert!(g.queue_trigger(&g.object_view(eyes), 0));
        // Returning control before publication cannot alter the captured one.
        g.record_control(g.object_ref(provider), p, None);
        g.prepare_triggers();
        assert_eq!(count(&g, eyes), 1);
        settle(&mut g);
        g.remove_abilities_exact(
            &[g.object_ref(source)],
            crate::cards::AnimationDuration::Permanent,
        );
        assert!(!g.queue_trigger(&g.object_view(source), 0));
    }
}
#[test]
fn ecl_twinflame_shared_triggered_mana_count_and_frozen_bonus_boundary() {
    use super::ability_world::CharacteristicWorld;
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let provider = named(&mut g, p, "Twinflame Travelers", Zone::Battlefield);
        let source = named(&mut g, p, "Morcant's Eyes", Zone::Battlefield);
        grant_elemental_fixture(&mut g, source);
        assert_eq!(g.trigger_occurrences(source, false), 2);
        assert_eq!(
            g.characteristic_query().trigger_occurrences(source, false),
            2
        );
        assert_eq!(g.trigger_occurrences(provider, false), 1);
        g.remove_abilities_exact(
            &[g.object_ref(provider)],
            crate::cards::AnimationDuration::Permanent,
        );
        assert_eq!(g.trigger_occurrences(source, false), 1);
        assert_eq!(g.trigger_occurrences(source, true), 2);
        g.move_object(provider, Zone::Exile);
        assert_eq!(g.trigger_occurrences(source, true), 1);
        named(&mut g, p.other(), "Twinflame Travelers", Zone::Battlefield);
        assert_eq!(g.trigger_occurrences(source, false), 1);
        // A future virtual control-source sacrifice can restore shared control.
        assert_eq!(g.trigger_occurrences(source, true), 2);
    }
}
#[test]
fn ecl_twinflame_failed_expansion_restores_owned_counts_packets_views_and_revision() {
    let mut g = setup(Player::P0);
    named(&mut g, Player::P0, "Twinflame Travelers", Zone::Battlefield);
    let source = named(&mut g, Player::P0, "Shimmercreep", Zone::Hand);
    g.move_object(source, Zone::Battlefield);
    g.state.next_stack_id = u64::MAX;
    let before = g.snapshot();
    let revision = g.revision();
    let views = [
        serde_json::to_value(g.observe(Player::P0)).unwrap(),
        serde_json::to_value(g.observe(Player::P1)).unwrap(),
    ];
    assert!(matches!(
        g.apply(Player::P0, Action::Pass),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
    for (index, p) in [Player::P0, Player::P1].into_iter().enumerate() {
        assert_eq!(serde_json::to_value(g.observe(p)).unwrap(), views[index]);
    }
    // A retained clone owns the original captured count after parent disposal.
    let mut branch = {
        let parent = g.clone();
        parent.clone()
    };
    branch.state.next_stack_id = 1000;
    checked(&mut branch, Action::Pass);
    assert_eq!(count(&branch, source), 2);
}
