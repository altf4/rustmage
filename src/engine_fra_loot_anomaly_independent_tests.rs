use super::*;

fn anomaly(g: &mut Game, p: Player, zone: Zone) -> CardId {
    let id = put(g, p, "Bear Cub", zone);
    g.state.objects[id as usize].card = Card::from_name("Loot, the Anomaly").unwrap();
    if zone == Zone::Battlefield {
        g.register_battlefield_sources(id);
    }
    id
}

#[test]
fn anomaly_real_cast_requires_black_and_replays_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let card = anomaly(&mut g, p, Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        let before = g.snapshot();
        reject(&mut g, Action::AnnounceCast { card });
        assert_eq!(g.snapshot(), before);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 2];
        let saved = g.snapshot();
        for _ in 0..2 {
            act(&mut g, Action::AnnounceCast { card });
            pay(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            settle(&mut g);
            assert_eq!(g.object_view(card).zone, Zone::Battlefield);
            assert_eq!(stats(&g, card), (-2, 4));
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn anomaly_blocked_combat_uses_negative_magnitude_or_toughness() {
    for p in [Player::P0, Player::P1] {
        for ghalta in [false, true] {
            let mut g = scenario(p);
            let source = anomaly(&mut g, p, Zone::Battlefield);
            g.state.objects[source as usize].growth.power = -6;
            if ghalta {
                put(&mut g, p, "Ghalta the Immovable", Zone::Battlefield);
            }
            let blocker = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.state.objects[blocker as usize].growth.toughness = 20;
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            act(&mut g, Action::Attack { card: source });
            act(&mut g, Action::FinishAttackers);
            for _ in 0..32 {
                if g.state.pending == Pending::Blockers {
                    break;
                }
                act(&mut g, Action::Pass);
            }
            act(
                &mut g,
                Action::Block {
                    blocker,
                    attacker: source,
                },
            );
            act(&mut g, Action::FinishBlockers);
            let saved = g.snapshot();
            for _ in 0..2 {
                for _ in 0..32 {
                    if g.state.objects[blocker as usize].damage != 0 {
                        break;
                    }
                    act(&mut g, Action::Pass);
                }
                assert_eq!(
                    g.state.objects[blocker as usize].damage,
                    if ghalta { 4 } else { 8 }
                );
                assert_eq!(stats(&g, source), (-8, 4));
                g.restore(&saved).unwrap();
            }
        }
    }
}

#[test]
fn anomaly_threshold_counts_only_own_physical_cards_and_rejects_bad_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = anomaly(&mut g, p, Zone::Battlefield);
        let fodder = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        for _ in 0..6 {
            put(&mut g, p, "Forest", Zone::Graveyard);
        }
        put(&mut g, p.other(), "Forest", Zone::Graveyard);
        let transient = put(&mut g, p, "Forest", Zone::Graveyard);
        g.state.objects[transient as usize].copy_kind = CopyKind::Card;
        let action = Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        };
        let before = g.snapshot();
        reject(&mut g, action.clone());
        assert_eq!(g.snapshot(), before);
        put(&mut g, p, "Forest", Zone::Graveyard);
        act(&mut g, action);
        let before_choice = g.snapshot();
        let enemy_ref = g.object_ref(enemy);
        reject(&mut g, Action::ChooseAdditionalCost { object: enemy_ref });
        assert_eq!(g.snapshot(), before_choice);
        let fodder_ref = g.object_ref(fodder);
        act(&mut g, Action::ChooseAdditionalCost { object: fodder_ref });
        pay(&mut g);
        let grave = (0..g.state.objects.len())
            .map(|i| i as CardId)
            .find(|&id| {
                id != transient
                    && g.state.objects[id as usize].owner == p
                    && g.state.objects[id as usize].zone == Zone::Graveyard
                    && g.state.objects[id as usize].card == Card::from_name("Forest").unwrap()
            })
            .unwrap();
        g.move_object(grave, Zone::Exile);
        settle(&mut g);
        assert_eq!(stats(&g, source), (-4, 4));
        g.restore(&before_choice).unwrap();
        assert_eq!(g.snapshot(), before_choice);
    }
}

#[test]
fn anomaly_negative_power_deals_no_power_based_noncombat_damage() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = anomaly(&mut g, p, Zone::Battlefield);
        let target = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let spell = put(&mut g, p, "Compel Brutality", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: spell });
        act(&mut g, Action::ChooseMode { mode: 0 });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(source),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(target).damage, 0);
        assert_eq!(stats(&g, source), (-2, 4));
    }
}

#[test]
fn anomaly_queued_ability_still_affects_same_source_after_control_change() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = anomaly(&mut g, p, Zone::Battlefield);
        let fodder = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        for _ in 0..7 {
            put(&mut g, p, "Forest", Zone::Graveyard);
        }
        act(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let object = g.object_ref(fodder);
        act(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        let saved = g.snapshot();
        for _ in 0..2 {
            let reference = g.object_ref(source);
            g.change_controller(reference, p.other());
            settle(&mut g);
            assert_eq!(stats(&g, source), (-4, 4));
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn anomaly_copied_activation_affects_original_source_without_second_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = anomaly(&mut g, p, Zone::Battlefield);
        let fodder = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        for _ in 0..7 {
            put(&mut g, p, "Forest", Zone::Graveyard);
        }
        act(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let object = g.object_ref(fodder);
        act(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        let original = g.state.stack.last().unwrap().clone();
        let mut copier = original.clone();
        copier.controller = p.other();
        copier.targets = vec![BoundTarget {
            group: 0,
            target: Target::Stack(original.id),
            object: None,
        }];
        g.copy_target_ability(&copier, 0);
        settle(&mut g);
        assert_eq!(stats(&g, source), (-6, 4));
        assert_eq!(g.object_view(fodder).zone, Zone::Graveyard);
    }
}
