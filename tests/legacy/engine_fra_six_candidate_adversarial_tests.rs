use super::*;

#[test]
fn living_library_sacrifice_stays_paid_when_target_reenters_or_changes_control() {
    for p in [Player::P0, Player::P1] {
        for reentry in [false, true] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Living Library", Zone::Battlefield);
            let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 6];
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
                    target: Target::Card(target),
                },
            );
            pay(&mut g);
            assert_eq!(g.object_view(source).zone, Zone::Graveyard);
            if reentry {
                g.move_object(target, Zone::Exile);
                g.move_object(target, Zone::Battlefield);
            } else {
                g.change_controller(g.object_ref(target), p);
            }
            let before = g.snapshot();
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&before).unwrap();
                }
                settle(&mut g);
                assert_eq!(g.object_view(target).zone, Zone::Battlefield);
                assert_eq!(g.object_view(source).zone, Zone::Graveyard);
            }
        }
    }
}

#[test]
fn twinned_vision_paid_exile_cast_draws_two_without_flashback() {
    for p in [Player::P0, Player::P1] {
        for mana in [[0, 1, 0, 0, 0, 1], [0, 0, 0, 1, 0, 1]] {
            let mut g = scenario(p);
            let spell = put(&mut g, p, "Twinned Vision", Zone::Exile);
            g.grant_exile_play_permission(p, g.object_ref(spell), false);
            g.state.players[p.index()].mana = mana;
            let initial = g.state.players[p.index()].library.len();
            act(&mut g, Action::AnnounceExile { card: spell });
            choose(&mut g, |a| matches!(a, Action::ChooseSpellCost { .. }));
            pay(&mut g);
            let saved = g.snapshot();
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&saved).unwrap();
                }
                settle(&mut g);
                assert_eq!(g.state.players[p.index()].library.len(), initial - 2);
                assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            }
        }
    }
}

#[test]
fn light_realms_returns_simultaneous_creatures_with_entry_choice_and_aura() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let host = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let automaton = put(&mut g, p, "Adaptive Automaton", Zone::Graveyard);
        let bear = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        let aura = put(&mut g, p, "Pacifism", Zone::Graveyard);
        let spell = put(&mut g, p, "Return to the Light Realms", Zone::Hand);
        g.state.players[p.index()].mana = [2, 0, 0, 0, 0, 7];
        cast(&mut g, spell, None);
        let mut chosen_type = false;
        let mut chosen_aura = false;
        for _ in 0..96 {
            if g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
                && g.state.pending_triggers.is_empty()
                && g.state.resolving.is_none()
            {
                break;
            }
            if matches!(g.state.pending, Pending::EntryValue { .. }) {
                choose(
                    &mut g,
                    |a| matches!(a, Action::ChooseEntryValue { value: EntryChoiceValue::CreatureType(t) } if t.name() == "Dwarf"),
                );
                chosen_type = true;
            } else if matches!(g.state.pending, Pending::AuraEntry { .. }) {
                let object = g.object_ref(host);
                act(&mut g, Action::ChooseAuraAttachment { object });
                chosen_aura = true;
            } else {
                choose(&mut g, |a| {
                    matches!(a, Action::Pass | Action::PutTrigger { .. })
                });
            }
        }
        assert!(chosen_type && chosen_aura);
        for id in [automaton, bear, aura] {
            assert_eq!(g.object_view(id).zone, Zone::Battlefield);
        }
        assert_eq!(g.object_view(aura).attached_to, Some(g.object_ref(host)));
        assert!(
            matches!(g.entry_value(g.object_ref(automaton)), Some(EntryChoiceValue::CreatureType(t)) if t.name() == "Dwarf")
        );
    }
}
