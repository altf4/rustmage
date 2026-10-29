//! Independent Emrakul interaction tests; link under assurance for validation.
use super::*;

fn emrakul_permanent(g: &mut Game, owner: Player) -> CardId {
    let id = put(g, owner, "Bear Cub", Zone::Battlefield);
    g.state.objects[id as usize].card = Card::test_index(1324);
    g.register_battlefield_sources(id);
    id
}

fn wait_for_ward_payment(g: &mut Game, payer: Player) {
    for _ in 0..48 {
        if matches!(g.state.pending, Pending::EffectChoice { player } if player == payer) {
            return;
        }
        choose(g, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("missing ward payment: {:?}", g.state.pending);
}

#[test]
fn uncounterable_eject_survives_emrakul_ward_whether_paid_or_declined() {
    for owner in [Player::P0, Player::P1] {
        for pay_ward in [false, true] {
            let mut g = scenario(owner);
            let emrakul = emrakul_permanent(&mut g, owner);
            let payer = owner.other();
            let sacrifices: Vec<_> = (0..3)
                .map(|_| put(&mut g, payer, "Forest", Zone::Battlefield))
                .collect();
            let eject = put(&mut g, payer, "Eject", Zone::Hand);
            g.state.active = payer;
            g.state.priority = payer;
            cast(&mut g, eject, Some(Target::Card(emrakul)));
            wait_for_ward_payment(&mut g, payer);
            let checkpoint = g.snapshot();
            for _ in 0..2 {
                if pay_ward {
                    for &id in &sacrifices {
                        let object = g.object_ref(id);
                        act(&mut g, Action::Sacrifice { object });
                    }
                } else {
                    act(&mut g, Action::DeclineEffectPayment);
                }
                settle(&mut g);
                assert_eq!(g.object_view(emrakul).zone, Zone::Hand);
                assert_eq!(g.object_view(eject).zone, Zone::Graveyard);
                for &id in &sacrifices {
                    assert_eq!(
                        g.object_view(id).zone,
                        if pay_ward {
                            Zone::Graveyard
                        } else {
                            Zone::Battlefield
                        }
                    );
                }
                g.restore(&checkpoint).unwrap();
            }
        }
    }
}

#[test]
fn echoed_ward_survives_emrakul_departure_and_each_instance_can_be_paid() {
    for owner in [Player::P0, Player::P1] {
        let mut g = scenario(owner);
        let emrakul = emrakul_permanent(&mut g, owner);
        let echo = put(&mut g, owner, "Echo, Perceptive Prodigy", Zone::Battlefield);
        let payer = owner.other();
        let sacrifices: Vec<_> = (0..6)
            .map(|_| put(&mut g, payer, "Forest", Zone::Battlefield))
            .collect();
        let shock = put(&mut g, payer, "Shock", Zone::Hand);
        g.state.active = payer;
        g.state.priority = payer;
        cast(&mut g, shock, Some(Target::Card(emrakul)));
        let ward = g
            .state
            .stack
            .iter()
            .find(|item| {
                matches!(item.kind, StackKind::TriggeredAbility { .. })
                    && item.source.is_some_and(|source| source.id == emrakul)
            })
            .unwrap()
            .id;
        if g.actor() != Some(owner) {
            act(&mut g, Action::Pass);
        }
        act(
            &mut g,
            Action::AnnounceActivate {
                card: echo,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Stack(ward),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        act(&mut g, Action::Pass);
        act(&mut g, Action::Pass);
        assert!(
            g.state
                .stack
                .iter()
                .any(|item| item.is_copy && item.source.is_some_and(|source| source.id == emrakul))
        );
        // Model an intervening legal departure; it does not remove either ward packet.
        g.move_object(emrakul, Zone::Graveyard);
        let checkpoint = g.snapshot();
        for _ in 0..2 {
            for group in sacrifices.chunks_exact(3) {
                wait_for_ward_payment(&mut g, payer);
                for &id in group {
                    let object = g.object_ref(id);
                    act(&mut g, Action::Sacrifice { object });
                }
            }
            settle(&mut g);
            assert!(
                sacrifices
                    .iter()
                    .all(|&id| g.object_view(id).zone == Zone::Graveyard)
            );
            assert_eq!(g.object_view(shock).zone, Zone::Graveyard);
            g.restore(&checkpoint).unwrap();
        }
    }
}

#[test]
fn echoed_wards_offer_payment_even_after_targeting_spell_leaves() {
    for owner in [Player::P0, Player::P1] {
        let mut g = scenario(owner);
        let emrakul = emrakul_permanent(&mut g, owner);
        let echo = put(&mut g, owner, "Echo, Perceptive Prodigy", Zone::Battlefield);
        let payer = owner.other();
        let sacrifices: Vec<_> = (0..6)
            .map(|_| put(&mut g, payer, "Forest", Zone::Battlefield))
            .collect();
        let shock = put(&mut g, payer, "Shock", Zone::Hand);
        g.state.active = payer;
        g.state.priority = payer;
        cast(&mut g, shock, Some(Target::Card(emrakul)));
        let ward = g
            .state
            .stack
            .iter()
            .find(|item| {
                matches!(item.kind, StackKind::TriggeredAbility { .. })
                    && item.source.is_some_and(|source| source.id == emrakul)
            })
            .unwrap()
            .id;
        if g.actor() != Some(owner) {
            act(&mut g, Action::Pass);
        }
        act(
            &mut g,
            Action::AnnounceActivate {
                card: echo,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Stack(ward),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        act(&mut g, Action::Pass);
        act(&mut g, Action::Pass);
        assert!(
            g.state
                .stack
                .iter()
                .any(|item| item.is_copy && item.source.is_some_and(|source| source.id == emrakul))
        );
        // Model a counter effect before either ward resolves. Ward has no target,
        // so both instances still offer their optional payment.
        let shock_id = g
            .state
            .stack
            .iter()
            .find(|item| {
                item.kind == StackKind::Spell
                    && item.source.is_some_and(|source| source.id == shock)
            })
            .unwrap()
            .id;
        assert!(g.counter_stack_item(shock_id, false));
        assert_eq!(g.object_view(shock).zone, Zone::Graveyard);
        assert_eq!(
            g.state
                .stack
                .iter()
                .filter(
                    |item| matches!(item.kind, StackKind::TriggeredAbility { .. })
                        && item.source.is_some_and(|source| source.id == emrakul)
                )
                .count(),
            2
        );
        let checkpoint = g.snapshot();
        for _ in 0..2 {
            for group in sacrifices.chunks_exact(3) {
                wait_for_ward_payment(&mut g, payer);
                for &id in group {
                    let object = g.object_ref(id);
                    act(&mut g, Action::Sacrifice { object });
                }
            }
            settle(&mut g);
            assert!(
                sacrifices
                    .iter()
                    .all(|&id| g.object_view(id).zone == Zone::Graveyard)
            );
            assert_eq!(g.object_view(shock).zone, Zone::Graveyard);
            g.restore(&checkpoint).unwrap();
        }
    }
}

#[test]
fn emrakul_hand_ability_fizzles_when_target_loses_land_type() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Bear Cub", Zone::Hand);
        g.state.objects[source as usize].card = Card::test_index(1324);
        let land = put(&mut g, player, "Forest", Zone::Battlefield);
        g.state.players[player.index()].mana = [0, 0, 0, 0, 0, 3];
        act(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        pay(&mut g);
        let reference = g.object_ref(land);
        let timestamp = g.next_role_timestamp();
        // Synthetic Food-only layer removes land without changing incarnation.
        g.state
            .roles
            .get_or_insert_with(Default::default)
            .animations
            .push(AnimationView {
                object: reference,
                timestamp,
                artifact: false,
                land: false,
                preserve_card_types: false,
                food_only: true,
                artifact_noncreature: false,
                colors: None,
                creature_types: None,
                all_creature_types: false,
                base: None,
                keywords: Keywords::NONE,
                expires_turn: None,
            });
        assert_eq!(g.current_kinds(land) & 2, 0);
        let checkpoint = g.snapshot();
        for _ in 0..2 {
            settle(&mut g);
            assert!(!g.can_play_exiled(player, source));
            assert!(!g.legal_actions().iter().any(
                |action| matches!(action, Action::TapGrantedMana { card, .. } if *card == land)
            ));
            g.restore(&checkpoint).unwrap();
        }
    }
}
