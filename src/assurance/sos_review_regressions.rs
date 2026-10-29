//! Paid regressions for the independently reviewed SOS continuation fixes.
use super::*;

fn pause(g: &mut Game) {
    for _ in 0..20 {
        if g.state.pending != Pending::Priority {
            return;
        }
        big_helpers::checked(g, Action::Pass);
    }
    panic!("expected a resolution choice");
}

fn checked_pay(g: &mut Game) {
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    big_helpers::checked(g, action);
}

fn spirits(g: &Game, player: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, object)| {
            (object.zone == Zone::Battlefield
                && object.controller == player
                && object.is_token()
                && g.has_subtype(id as CardId, "Spirit"))
            .then_some(id as CardId)
        })
        .collect()
}

#[test]
fn sos_review_henchbots_daydream_preserves_choice_bearing_departure_return() {
    for p in [Player::P0, Player::P1] {
        // Establish the held creature through its actual paid, choice-bearing entry.
        let mut g = scenario(p.other());
        let automaton = put(&mut g, p.other(), "Adaptive Automaton", Zone::Hand);
        big_helpers::cast(&mut g, automaton, None, &[]);
        pause(&mut g);
        let robot =
            EntryChoiceValue::CreatureType(crate::cards::CreatureType::from_name("Robot").unwrap());
        assert!(matches!(g.state.pending, Pending::EntryValue { .. }));
        big_helpers::checked(&mut g, Action::ChooseEntryValue { value: robot });
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(automaton).entry_choice, Some(robot));

        g.state.active = p;
        g.state.priority = p;
        g.state.step = Step::Main1;
        g.state.pending = Pending::Priority;
        g.state.objects[automaton as usize].tapped = true;
        let henchbots = put(&mut g, p, "Henchbots", Zone::Hand);
        big_helpers::cast(&mut g, henchbots, None, &[]);
        pause(&mut g);
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(automaton),
            },
        );
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(automaton).zone, Zone::Exile);
        let old_henchbots = g.object_ref(henchbots);
        let daydream = put(&mut g, p, "Daydream", Zone::Hand);
        big_helpers::cast(&mut g, daydream, None, &[Target::Card(henchbots)]);
        pause(&mut g);
        assert!(matches!(g.state.pending, Pending::EntryValue { player } if player == p.other()));
        assert_eq!(
            g.observe(p).entry_value.as_ref().unwrap().object.card,
            automaton
        );
        assert_eq!(g.object_view(henchbots).zone, Zone::Exile);
        assert_ne!(g.object_ref(henchbots), old_henchbots);
        let continuation = g.observe(p).immediate_blink_return.unwrap();
        assert_eq!(continuation.len(), 1);
        assert_eq!(continuation[0].object, g.object_ref(henchbots));
        assert_eq!(continuation[0].counter, Some((CounterKind::PlusOne, 1)));
        // This selected action checks both raw/index owned branches and restore
        // while both the linked entry and immediate blink continuation are live.
        let human =
            EntryChoiceValue::CreatureType(crate::cards::CreatureType::from_name("Human").unwrap());
        big_helpers::checked(&mut g, Action::ChooseEntryValue { value: human });
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(automaton).zone, Zone::Battlefield);
        assert_eq!(g.object_view(automaton).controller, p.other());
        assert_eq!(g.object_view(automaton).entry_choice, Some(human));
        assert!(g.has_subtype(automaton, "Human"));
        assert_eq!(g.object_view(henchbots).zone, Zone::Battlefield);
        assert_eq!(g.counter_count(henchbots, CounterKind::PlusOne), 1);
        assert_eq!(g.object_view(daydream).zone, Zone::Graveyard);
        assert!(g.observe(p).immediate_blink_return.is_none());
    }
}

#[test]
fn sos_review_pox_second_chooser_observation_does_not_disclose_prior_hand_selections() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let common = put(&mut g, p, "Forest", Zone::Hand);
        let left_card = put(&mut g, p, "Mountain", Zone::Hand);
        let right_card = put(&mut g, p, "Island", Zone::Hand);
        put(&mut g, p, "Shock", Zone::Hand);
        let their_card = put(&mut g, p.other(), "Forest", Zone::Hand);
        put(&mut g, p.other(), "Shock", Zone::Hand);
        let spell = put(&mut g, p, "Pox Plague", Zone::Hand);
        big_helpers::cast(&mut g, spell, None, &[]);
        pause(&mut g);
        assert_eq!(g.actor(), Some(p));
        big_helpers::checked(&mut g, Action::Discard { card: common });
        assert_eq!(g.actor(), Some(p));
        let own_view = g.observe(p).effect_choice.unwrap();
        assert_eq!(own_view.selected[p.index()], vec![g.object_ref(common)]);
        assert!(g.observe(p.other()).effect_choice.is_none());
        let mut left = g.clone();
        let mut right = g.clone();
        drop(g);
        big_helpers::checked(&mut left, Action::Discard { card: left_card });
        big_helpers::checked(&mut right, Action::Discard { card: right_card });
        for branch in [&left, &right] {
            assert_eq!(branch.actor(), Some(p.other()));
            let view = branch.observe(p.other());
            assert!(view.effect_choice.as_ref().unwrap().selected[p.index()].is_empty());
            assert!(
                view.objects
                    .iter()
                    .all(|o| ![common, left_card, right_card].contains(&o.id))
            );
            for card in [common, left_card, right_card] {
                assert_eq!(
                    branch.object_view(card).zone,
                    Zone::Hand,
                    "choices remain private until the combined discard event"
                );
            }
        }
        assert_eq!(
            serde_json::to_value(left.observe(p.other())).unwrap(),
            serde_json::to_value(right.observe(p.other())).unwrap(),
            "the complete next chooser observation must not identify the previous chooser's selected hand cards"
        );
        big_helpers::checked(&mut left, Action::Discard { card: their_card });
        big_helpers::checked(&mut right, Action::Discard { card: their_card });
        big_helpers::settle(&mut left);
        big_helpers::settle(&mut right);
        assert_eq!(left.object_view(left_card).zone, Zone::Graveyard);
        assert_eq!(left.object_view(right_card).zone, Zone::Hand);
        assert_eq!(right.object_view(left_card).zone, Zone::Hand);
        assert_eq!(right.object_view(right_card).zone, Zone::Graveyard);
    }
}

#[test]
fn sos_review_mica_copy_of_flashed_back_antiquities_has_no_cast_origin_bonus() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Mica, Reader of Ruins", Zone::Battlefield);
        let artifact = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        let spell = put(&mut g, p, "Antiquities on the Loose", Zone::Graveyard);
        big_helpers::checked(&mut g, Action::AnnounceFlashback { card: spell });
        checked_pay(&mut g);
        pause(&mut g);
        let payment = g.object_ref(artifact);
        big_helpers::checked(&mut g, Action::Sacrifice { object: payment });
        assert_eq!(g.object_view(artifact).zone, Zone::Graveyard);
        let copied = g
            .state
            .stack
            .iter()
            .find(|item| {
                item.is_copy
                    && item.source.unwrap().card == Card::from_name("Antiquities on the Loose").unwrap()
            })
            .unwrap();
        let copy_id = copied.id;
        let copy_card = copied.card;
        assert_eq!(copied.cast.as_ref().unwrap().origin, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 1);
        for _ in 0..10 {
            if !g.state.stack.iter().any(|item| item.id == copy_id) {
                break;
            }
            big_helpers::checked(&mut g, Action::Pass);
        }
        assert!(!g.state.stack.iter().any(|item| item.id == copy_id));
        assert!(
            g.state
                .stack
                .iter()
                .any(|item| item.card == spell && !item.is_copy)
        );
        let made = spirits(&g, p);
        assert_eq!(made.len(), 2);
        for card in made {
            assert_eq!(g.counter_count(card, CounterKind::PlusOne), 0);
        }
        assert_eq!(g.object_view(copy_card).zone, Zone::Outside);
        big_helpers::settle(&mut g);
        let made = spirits(&g, p);
        assert_eq!(made.len(), 4);
        for card in made {
            assert_eq!(g.counter_count(card, CounterKind::PlusOne), 1);
        }
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 1);
    }
}

#[test]
fn sos_review_rendezvous_draw_trigger_ids_record_active_player_draws_first() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own_crawler = put(&mut g, p, "Scrawling Crawler", Zone::Battlefield);
        let their_crawler = put(&mut g, p.other(), "Scrawling Crawler", Zone::Battlefield);
        put(&mut g, p, "High Fae Trickster", Zone::Battlefield);
        put(&mut g, p.other(), "Forest", Zone::Hand);
        let joined = put(&mut g, p, "Joined Researchers", Zone::Hand);
        big_helpers::cast(&mut g, joined, None, &[]);
        big_helpers::settle(&mut g);
        g.begin_step(Step::End);
        big_helpers::settle(&mut g);
        assert!(g.object_view(joined).prepared);
        let copy = g
            .observe(p)
            .preparations
            .iter()
            .find(|row| row.permanent == g.object_ref(joined))
            .unwrap()
            .spell_copy
            .card;
        g.state.active = p.other();
        g.state.priority = p;
        g.state.step = Step::Main1;
        g.state.pending = Pending::Priority;
        let hands = [g.hand_count(p), g.hand_count(p.other())];
        big_helpers::checked(&mut g, Action::AnnounceExile { card: copy });
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        checked_pay(&mut g);
        pause(&mut g);
        assert!(matches!(g.state.pending, Pending::OrderTriggers { .. }));
        let mut packets: Vec<_> = g
            .state
            .ordering_triggers
            .iter()
            .chain(g.state.pending_triggers.iter())
            .chain(g.state.stack.iter())
            .filter(|item| [own_crawler, their_crawler].contains(&item.card))
            .filter_map(|item| {
                item.draw_event
                    .map(|event| (item.id, event.player, event.ordinal, item.card))
            })
            .collect();
        packets.sort_unstable_by_key(|packet| packet.0);
        assert_eq!(packets.len(), 6);
        assert_eq!(
            packets
                .iter()
                .map(|packet| (packet.1, packet.2, packet.3))
                .collect::<Vec<_>>(),
            vec![
                (p.other(), 1, own_crawler),
                (p.other(), 2, own_crawler),
                (p.other(), 3, own_crawler),
                (p, 1, their_crawler),
                (p, 2, their_crawler),
                (p, 3, their_crawler),
            ]
        );
        assert!(packets.windows(2).all(|pair| pair[0].0 < pair[1].0));
        assert_eq!(g.hand_count(p), hands[0] + 3);
        assert_eq!(g.hand_count(p.other()), hands[1] + 3);
        big_helpers::settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 17);
        assert_eq!(g.state.players[p.other().index()].life, 17);
    }
}
