use super::*;

#[test]
fn fulcrum_rejects_tapped_source_and_destruction_obeys_exile_replacement() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Rest in Peace", Zone::Battlefield);
        let source = put(&mut g, p, "The Echoverse Fulcrum", Zone::Battlefield);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let activation = Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        };
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 5];
        g.state.objects[source as usize].tapped = true;
        reject(&mut g, activation.clone());
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        g.state.objects[source as usize].tapped = false;
        act(&mut g, activation);
        pay(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Exile);
        super::super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(g.object_view(own).zone, Zone::Exile);
        assert_eq!(g.object_view(enemy).zone, Zone::Exile);
    }
}

#[test]
fn fulcrum_entry_draw_discard_survives_source_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let discard = put(&mut g, p, "Bear Cub", Zone::Hand);
        let source = put(&mut g, p, "The Echoverse Fulcrum", Zone::Hand);
        let before_library = g.state.players[p.index()].library.len();
        cast(&mut g, source, None);
        for _ in 0..32 {
            if g.object_view(source).zone == Zone::Battlefield {
                break;
            }
            choose(&mut g, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        g.move_object(source, Zone::Exile);
        let mut discarded = false;
        for _ in 0..48 {
            let action = Action::Discard { card: discard };
            if g.legal_actions().contains(&action) {
                act(&mut g, action);
                discarded = true;
            } else if g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
                && g.state.pending_triggers.is_empty()
                && g.state.resolving.is_none()
            {
                break;
            } else {
                choose(&mut g, |a| {
                    matches!(a, Action::Pass | Action::PutTrigger { .. })
                });
            }
        }
        assert!(discarded);
        assert_eq!(g.state.players[p.index()].library.len(), before_library - 1);
        assert_eq!(g.object_view(discard).zone, Zone::Graveyard);
    }
}

#[test]
fn jiang_end_step_uses_trigger_controller_even_after_control_change_and_departure() {
    for p in [Player::P0, Player::P1] {
        for source_leaves in [false, true] {
            let mut g = scenario(p);
            let jiang = put(&mut g, p, "Jiang Yanggu, Never Alone", Zone::Battlefield);
            g.create_tokens(p, Card::test_index(5246), 1, false);
            g.create_tokens(p.other(), Card::test_index(5246), 1, false);
            let own = g
                .state
                .objects
                .iter()
                .enumerate()
                .find_map(|(i, o)| {
                    (o.is_token() && o.zone == Zone::Battlefield && o.owner == p)
                        .then_some(i as CardId)
                })
                .unwrap();
            let enemy = g
                .state
                .objects
                .iter()
                .enumerate()
                .find_map(|(i, o)| {
                    (o.is_token() && o.zone == Zone::Battlefield && o.owner == p.other())
                        .then_some(i as CardId)
                })
                .unwrap();
            g.state.objects[own as usize].tapped = true;
            g.state.objects[enemy as usize].tapped = true;
            g.begin_step(Step::End);
            g.prepare_triggers();
            if source_leaves {
                g.move_object(jiang, Zone::Graveyard);
            } else {
                let source = g.object_ref(jiang);
                g.change_controller(source, p.other());
            }
            let saved = g.snapshot();
            super::super::contracts::branch_contract(g.clone());
            settle(&mut g);
            assert!(!g.object_view(own).tapped);
            assert!(g.object_view(enemy).tapped);
            g.restore(&saved).unwrap();
            settle(&mut g);
            assert!(!g.object_view(own).tapped);
            assert!(g.object_view(enemy).tapped);
        }
    }
}

#[test]
fn jiang_does_not_untap_tokens_on_opponents_end_step() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p.other());
        put(&mut g, p, "Jiang Yanggu, Never Alone", Zone::Battlefield);
        g.create_tokens(p, Card::test_index(5246), 1, false);
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(i, o)| {
                (o.is_token() && o.zone == Zone::Battlefield && o.owner == p).then_some(i as CardId)
            })
            .unwrap();
        g.state.objects[token as usize].tapped = true;
        g.begin_step(Step::End);
        g.prepare_triggers();
        settle(&mut g);
        assert!(g.object_view(token).tapped);
    }
}

#[test]
fn marwyn_control_and_ability_loss_update_land_hexproof() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let marwyn = put(&mut g, p, "Marwyn, the Preserver", Zone::Battlefield);
        let own = put(&mut g, p, "Forest", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Forest", Zone::Battlefield);
        assert!(g.keywords(own).contains(Keywords::HEXPROOF));
        assert!(!g.keywords(enemy).contains(Keywords::HEXPROOF));
        let reference = g.object_ref(marwyn);
        g.change_controller(reference, p.other());
        assert!(!g.keywords(own).contains(Keywords::HEXPROOF));
        assert!(g.keywords(enemy).contains(Keywords::HEXPROOF));
        let aura = put(&mut g, p, "Witness Protection", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(reference);
        assert!(!g.printed_abilities_available(marwyn));
        assert!(!g.keywords(enemy).contains(Keywords::HEXPROOF));
        g.move_object(aura, Zone::Graveyard);
        assert!(g.keywords(enemy).contains(Keywords::HEXPROOF));
    }
}

#[test]
fn marwyn_rejects_unfunded_activation_and_recovers_land_on_opponents_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Marwyn, the Preserver", Zone::Battlefield);
        let land = put(&mut g, p, "Forest", Zone::Graveyard);
        let activation = Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        };
        g.state.active = p.other();
        g.state.step = Step::BeginCombat;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        reject(&mut g, activation.clone());
        assert_eq!(g.object_view(land).zone, Zone::Graveyard);
        g.state.players[p.index()].mana[5] = 2;
        act(&mut g, activation);
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        pay(&mut g);
        super::super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(g.object_view(land).zone, Zone::Hand);
    }
}
