//! Exact source identity for a craft ability's exile-cost return.
use super::*;

#[test]
fn craft_exile_event_is_owned_by_the_departing_permanents_controller() {
    for player in [Player::P0, Player::P1] {
        for (origin, destination, actor, expected) in [
            (Zone::Battlefield, Zone::Exile, player, 1),
            (Zone::Battlefield, Zone::Exile, player.other(), 0),
            (Zone::Battlefield, Zone::Graveyard, player, 0),
            (Zone::Graveyard, Zone::Exile, player, 0),
        ] {
            let mut g = scenario(player);
            let id = put(&mut g, player, "@test/manifest-observer", origin);
            let checkpoint = g.snapshot();
            let event = g.move_object(id, destination);
            g.craft_exile_triggers(&event, actor);
            assert_eq!(g.state.pending_triggers.len(), expected);
            if expected == 1 {
                let movement = g.state.pending_triggers[0].source_move.as_ref().unwrap();
                assert_eq!(movement.before, event.before.reference);
                assert_eq!(movement.after, event.after);
            }
            let after = g.snapshot();
            g.restore(&checkpoint).unwrap();
            let event = g.move_object(id, destination);
            g.craft_exile_triggers(&event, actor);
            assert_eq!(g.snapshot(), after);
        }
    }
}

#[test]
fn jewel_mana_pays_ability_costs_but_not_spells() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        g.state.players[player.index()].mana = [0; 6];
        let jewel = put(&mut g, player, "The Enigma Jewel", Zone::Hand);
        let entry = g.entry_move(jewel, Zone::Battlefield, false);
        g.stage_entries(vec![entry], None);
        assert!(g.object_view(jewel).tapped);
        g.untap_objects(&[jewel], untap_rules::UntapCause::Effect);
        let bow = put(&mut g, player, "Short Bow", Zone::Battlefield);
        let creature = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, player, "Short Bow", Zone::Hand);
        choose(&mut g, |action| {
            matches!(action,
            Action::ActivateMana { card, production, .. }
                if *card == jewel && *production == [0, 0, 0, 0, 0, 2])
        });
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: spell })
        );
        big_helpers::checked(
            &mut g,
            Action::AnnounceActivate {
                card: bow,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(creature),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(bow).attached_to, Some(g.object_ref(creature)));
    }
}

#[test]
fn locus_copies_your_nonmana_activation_and_allows_retargeting() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let locus = put(&mut g, player, "Locus of Enlightenment", Zone::Battlefield);
        let bow = put(&mut g, player, "Short Bow", Zone::Battlefield);
        let first = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let second = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        big_helpers::checked(
            &mut g,
            Action::AnnounceActivate {
                card: bow,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(first),
            },
        );
        pay(&mut g);
        assert!(
            g.state
                .pending_triggers
                .iter()
                .chain(g.state.stack.iter())
                .any(|trigger| {
                    trigger.source.is_some_and(|source| source.id == locus)
                        && trigger.copy_event.is_some()
                })
        );
        for _ in 0..30 {
            if matches!(g.state.pending, Pending::Retarget { .. }) {
                break;
            }
            big_helpers::checked(&mut g, Action::Pass);
        }
        assert!(matches!(g.state.pending, Pending::Retarget { .. }));
        let alternative = g.legal_actions().into_iter().find(|action| {
            matches!(action, Action::SetStackTarget { target: Target::Card(card), .. } if *card == second)
        }).expect("copy offers a new target");
        big_helpers::checked(&mut g, alternative);
        big_helpers::checked(&mut g, Action::FinishStackTargets);
        assert!(g.state.stack.iter().any(|packet| packet.is_copy));
        settle(&mut g);
        assert_eq!(g.object_view(bow).attached_to, Some(g.object_ref(first)));
    }
}

pub(super) fn begin_craft(g: &mut Game, source: CardId, ability: u16) {
    big_helpers::checked(
        g,
        Action::AnnounceActivate {
            card: source,
            ability,
            counter_payment: Counters::default(),
        },
    );
    big_helpers::checked(g, Action::BeginOrderedCosts);
}
pub(super) fn craft_clause(g: &mut Game, clause: CastCostClause) {
    let index = g
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|row| row.clause == clause && !row.paid)
        .unwrap() as u16;
    big_helpers::checked(g, Action::ChooseCastCost { clause: index });
}
pub(super) fn craft_material(g: &mut Game, material: CardId) {
    craft_clause(g, CastCostClause::Additional);
    let object = g.object_ref(material);
    big_helpers::checked(g, Action::ChooseAdditionalCost { object });
}
pub(super) fn craft_mana(g: &mut Game) {
    craft_clause(g, CastCostClause::Mana);
    pay(g);
}
pub(super) fn craft_source(g: &mut Game) {
    craft_clause(g, CastCostClause::CraftSource);
}

fn exile_packet(g: &mut Game, source: CardId, controller: Player) -> SpellView {
    let mut packet = g.activation_item(controller, source, 0);
    let event = g.move_object(source, Zone::Exile);
    assert!(event.moved);
    packet.source_move = Some(Box::new(TriggerSourceMoveView {
        before: event.before.reference,
        after: event.after,
        origin: event.before.zone,
        destination: event.destination,
    }));
    packet
}

#[test]
fn craft_return_uses_the_exact_exiled_source_and_owner() {
    for owner in [Player::P0, Player::P1] {
        let mut g = scenario(owner);
        let source = put(&mut g, owner, "Oko, Lorwyn Liege", Zone::Battlefield);
        g.state.objects[source as usize].controller = owner.other();
        let packet = exile_packet(&mut g, source, owner.other());
        let checkpoint = g.snapshot();
        let mut branch = g.clone();
        g.return_crafted_source_transformed(&packet);
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        assert_eq!(g.object_view(source).controller, owner);
        assert_eq!(g.object_view(source).card.name(), "Oko, Shadowmoor Scion");
        branch.return_crafted_source_transformed(&packet);
        assert_eq!(g.snapshot(), branch.snapshot());
        g.restore(&checkpoint).unwrap();
        g.return_crafted_source_transformed(&packet);
        assert_eq!(g.snapshot(), branch.snapshot());
    }
}

#[test]
fn craft_return_rejects_a_later_exile_incarnation() {
    for owner in [Player::P0, Player::P1] {
        let mut g = scenario(owner);
        let source = put(&mut g, owner, "Oko, Lorwyn Liege", Zone::Battlefield);
        let packet = exile_packet(&mut g, source, owner);
        g.move_object(source, Zone::Graveyard);
        g.move_object(source, Zone::Exile);
        let before = g.snapshot();
        g.return_crafted_source_transformed(&packet);
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.object_view(source).zone, Zone::Exile);
    }
}

#[test]
fn craft_artifact_cost_uses_control_on_battlefield_and_ownership_in_graveyard() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Short Bow", Zone::Battlefield);
        let own = put(&mut g, player, "Short Bow", Zone::Battlefield);
        let grave = put(&mut g, player, "Short Bow", Zone::Graveyard);
        let theirs = put(&mut g, player.other(), "Short Bow", Zone::Battlefield);
        let their_grave = put(&mut g, player.other(), "Short Bow", Zone::Graveyard);
        let creature = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let stolen = put(&mut g, player.other(), "Short Bow", Zone::Battlefield);
        g.state.objects[stolen as usize].controller = player;
        let cost = AdditionalCost {
            kind: EffectChoiceKind::ExileControlledOrGraveyard,
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 16,
                nonland: false,
                other: true,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: None,
            },
        };
        let src = g.object_ref(source);
        for valid in [own, grave, stolen] {
            assert!(g.additional_candidate(cost, player, src, g.object_ref(valid)));
        }
        for invalid in [source, theirs, their_grave, creature] {
            assert!(!g.additional_candidate(cost, player, src, g.object_ref(invalid)));
        }
        let old = g.object_ref(own);
        g.move_object(own, Zone::Exile);
        g.move_object(own, Zone::Battlefield);
        assert!(!g.additional_candidate(cost, player, src, old));
    }
}

#[test]
fn craft_graveyard_material_cannot_be_exiled_through_a_prohibition() {
    use crate::engine::ability_world::TEST_EXILE_PROHIBITOR;
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            TEST_EXILE_PROHIBITOR.with(|p| p.set(None));
        }
    }
    let _reset = Reset;
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Short Bow", Zone::Battlefield);
        let grave = put(&mut g, player, "Short Bow", Zone::Graveyard);
        let provider = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        let cost = AdditionalCost {
            kind: EffectChoiceKind::ExileControlledOrGraveyard,
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 16,
                nonland: false,
                other: true,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: None,
            },
        };
        let grave_ref = g.object_ref(grave);
        let goal = FundingGoal {
            actor: player,
            required: [0; 6],
            generic: 1,
            reserve: Some((grave_ref, cost)),
            source: None,
        };
        assert!(g.can_fund(goal.clone()));
        TEST_EXILE_PROHIBITOR.with(|p| p.set(Some(g.object_ref(provider))));
        assert!(!g.additional_candidate(cost, player, g.object_ref(source), grave_ref));
        assert!(!g.can_fund(goal));
        TEST_EXILE_PROHIBITOR.with(|p| p.set(None));
    }
}

#[test]
fn craft_funding_can_remove_the_prohibitor_with_a_mana_ability() {
    use crate::engine::ability_world::TEST_EXILE_PROHIBITOR;
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            TEST_EXILE_PROHIBITOR.with(|p| p.set(None));
        }
    }
    let _reset = Reset;
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let grave = put(&mut g, player, "Short Bow", Zone::Graveyard);
        g.create_tokens(player, Card::test_index(5302), 1, false);
        big_helpers::settle(&mut g);
        let provider = g
            .state
            .objects
            .iter()
            .position(|o| o.card == Card::test_index(5302) && o.zone == Zone::Battlefield)
            .unwrap() as CardId;
        TEST_EXILE_PROHIBITOR.with(|p| p.set(Some(g.object_ref(provider))));
        g.state.players[player.index()].mana = [0; 6];
        let cost = AdditionalCost {
            kind: EffectChoiceKind::ExileControlledOrGraveyard,
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 16,
                nonland: false,
                other: true,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: None,
            },
        };
        let goal = FundingGoal {
            actor: player,
            required: [1, 0, 0, 0, 0, 0],
            generic: 0,
            reserve: Some((g.object_ref(grave), cost)),
            source: None,
        };
        assert!(g.can_fund(goal));
        TEST_EXILE_PROHIBITOR.with(|p| p.set(None));
    }
}

#[test]
fn craft_return_does_not_transform_a_nondoublefaced_source_copy() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let physical = put(&mut g, player, "Short Bow", Zone::Battlefield);
        let mut packet = exile_packet(&mut g, physical, player);
        // A copied or borrowed ability retains its immutable donor definition.
        packet.source.as_mut().unwrap().card = Card::from_name("Oko, Lorwyn Liege").unwrap();
        let before = g.snapshot();
        g.return_crafted_source_transformed(&packet);
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.object_view(physical).zone, Zone::Exile);
    }
}

#[test]
fn jewel_crafts_with_four_or_more_and_links_exact_exiled_cards() {
    for player in [Player::P0, Player::P1] {
        for count in [4, 5] {
            let mut g = scenario(player);
            let jewel = put(&mut g, player, "The Enigma Jewel", Zone::Battlefield);
            let mut materials = Vec::new();
            for n in 0..count {
                materials.push(put(
                    &mut g,
                    player,
                    "Short Bow",
                    if n % 2 == 0 {
                        Zone::Battlefield
                    } else {
                        Zone::Graveyard
                    },
                ));
            }
            let invalid = put(&mut g, player, "Bear Cub", Zone::Battlefield);
            let grant_only = put(&mut g, player, "Sanctum Lurker", Zone::Battlefield);
            begin_craft(&mut g, jewel, 1);
            craft_clause(&mut g, CastCostClause::Additional);
            assert!(!g.legal_actions().contains(&Action::ChooseAdditionalCost {
                object: g.object_ref(invalid)
            }));
            assert!(!g.legal_actions().contains(&Action::ChooseAdditionalCost {
                object: g.object_ref(grant_only)
            }));
            for (index, &material) in materials.iter().enumerate() {
                let object = g.object_ref(material);
                big_helpers::checked(&mut g, Action::ChooseAdditionalCost { object });
                assert_eq!(
                    g.legal_actions().contains(&Action::FinishCraftMaterials),
                    index >= 3
                );
            }
            big_helpers::checked(&mut g, Action::FinishCraftMaterials);
            craft_source(&mut g);
            craft_mana(&mut g);
            settle(&mut g);
            assert_eq!(g.object_view(jewel).card.name(), "Locus of Enlightenment");
            for material in &materials {
                assert_eq!(g.object_view(*material).zone, Zone::Exile);
            }
            let inherited = g.borrowed_ability_views();
            assert_eq!(inherited.len(), count);
            for row in inherited {
                assert!(materials.contains(&row.exiled.card));
            }
        }
    }
}
#[test]
fn locus_copies_last_known_targets_when_original_ability_is_countered() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        put(&mut g, player, "Locus of Enlightenment", Zone::Battlefield);
        let bow = put(&mut g, player, "Short Bow", Zone::Battlefield);
        let first = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let second = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        big_helpers::checked(
            &mut g,
            Action::AnnounceActivate {
                card: bow,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(first),
            },
        );
        pay(&mut g);
        let mut changed = g.bind_target(Target::Card(second));
        changed.group = 0;
        let original = g
            .state
            .stack
            .iter_mut()
            .find(|packet| {
                packet.source.is_some_and(|source| source.id == bow)
                    && matches!(packet.kind, StackKind::ActivatedAbility { .. })
            })
            .unwrap();
        original.targets[0] = changed;
        let id = original.id;
        assert!(g.counter_stack_item(id, false));
        for _ in 0..30 {
            if matches!(g.state.pending, Pending::Retarget { .. }) {
                break;
            }
            big_helpers::checked(&mut g, Action::Pass);
        }
        assert!(matches!(g.state.pending, Pending::Retarget { .. }));
        let view = g.retarget_view().unwrap();
        assert_eq!(view.item.targets[0].target, Target::Card(second));
        big_helpers::checked(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(g.object_view(bow).attached_to, Some(g.object_ref(second)));
    }
}
