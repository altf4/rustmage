//! Focused printed outcomes; shared engine invariants live in core tests.
//! Paid rules outcomes for the shared land/Saga cohort.
use super::*;
fn card(index: usize) -> Card {
    Card::catalogue().nth(index).unwrap()
}

fn finish(g: &mut Game) {
    for _ in 0..64 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
        {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("land/Saga resolution budget {:?}", g.state.pending);
}
fn tokens(g: &Game, p: Player, identity: usize) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| {
            o.zone == Zone::Battlefield && o.controller == p && o.card == card(identity)
        })
        .map(|(id, _)| id as CardId)
        .collect()
}
fn start(g: &mut Game, id: CardId) {
    act(
        g,
        Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
}
fn saga(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "The Misty Mountains Cold", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 2];
    cast(g, id, None);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn until_trigger(g: &mut Game) {
    for _ in 0..12 {
        if g.state
            .stack
            .last()
            .is_some_and(|s| matches!(s.kind, StackKind::TriggeredAbility { .. }))
        {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("chapter never stacked");
}
#[test]
fn lonely_actual_land_entry_equipment_control_and_intrinsic_red_mana() {
    {
        let p = Player::P0;
        for (own, token, ready) in [
            (false, false, false),
            (true, false, true),
            (true, true, true),
        ] {
            let mut g = scenario(p);
            let owner = if own { p } else { p.other() };
            if token {
                g.create_tokens(owner, card(5406), 1, false);
                finish(&mut g);
            } else {
                put(&mut g, owner, "Goldvein Pick", Zone::Battlefield);
            }
            let id = put(&mut g, p, "The Lonely Mountain", Zone::Hand);
            act(&mut g, Action::PlayLand { card: id });
            assert_eq!(g.object_view(id).tapped, !ready);
            assert!(g.has_subtype(id, "Mountain"));
            g.state.players[p.index()].mana = [0; 6];
            if ready {
                act(&mut g, Action::TapMana { card: id });
                assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 1, 0, 0]);
            } else {
                reject(&mut g, Action::TapMana { card: id });
            }
        }
        let mut g = scenario(p);
        put(&mut g, p, "Thrór's Map", Zone::Battlefield);
        let id = put(&mut g, p, "The Lonely Mountain", Zone::Hand);
        act(&mut g, Action::PlayLand { card: id });
        assert!(
            g.object_view(id).tapped,
            "an ordinary artifact is not Equipment"
        );
    }
}
#[test]
fn lonely_paid_activation_live_equipment_count_generic_floor_and_exact_dwarf() {
    {
        let p = Player::P0;
        for n in [0, 1, 3, 4, 6] {
            let mut g = scenario(p);
            let id = put(&mut g, p, "The Lonely Mountain", Zone::Battlefield);
            for _ in 0..n {
                put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
            }
            put(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
            let generic = 4u32.saturating_sub(n);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, generic + 1];
            reject(
                &mut g,
                Action::AnnounceActivate {
                    card: id,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            g.state.players[p.index()].mana = [0, 0, 0, 1, 0, generic];
            start(&mut g, id);

            reject(
                &mut g,
                Action::PayCosts {
                    payment: [0, 0, 0, 0, 0, generic + 1],
                },
            );
            pay(&mut g);
            assert!(g.object_view(id).tapped);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            finish(&mut g);
            let dwarves = tokens(&g, p, 5407);
            assert_eq!(dwarves.len(), 1);
            let dwarf = g.object_view(dwarves[0]);
            assert_eq!(stats(&g, dwarves[0]), (2, 2));
            assert_eq!(dwarf.colors, 8);
            assert!(dwarf.has_subtype("Dwarf"));
            assert_eq!(dwarf.keywords, Keywords::NONE);
            assert!(!dwarf.tapped);
        }
    }
}

#[test]
fn lonely_announced_reduction_locks_before_equipment_sacrificed_for_mana() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let id = put(&mut g, p, "The Lonely Mountain", Zone::Battlefield);
        let e = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        let object = g.object_ref(e);
        g.change_object_form(
            object,
            crate::cards::AnimationSpec {
                artifact: true,
                creature_types: crate::cards::AnimationCreatureTypes::Named(&["Goblin"]),
                base: Some((2, 2)),
                keywords: Keywords::NONE,
                duration: crate::cards::AnimationDuration::EndOfTurn,
            },
            characteristic_layers::AnimationTypeMode::AddCreature,
        );
        assert!(g.has_subtype(e, "Goblin") && g.has_subtype(e, "Equipment"));
        let bolg = put(&mut g, p, "Bolg's Company", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        start(&mut g, id);
        act(
            &mut g,
            Action::AnnounceMana {
                card: bolg,
                ability: 0,
                counter_payment: Counters::default(),
                production: [0, 0, 1, 1, 0, 0],
            },
        );
        act(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        assert_eq!(g.object_view(e).zone, Zone::Graveyard);

        pay(&mut g);
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 1, 0, 0, 0]);
        assert_eq!(tokens(&g, p, 5407).len(), 1);
    }
}

#[test]
fn misty_sacrifice_competing_replacements_pause_replay_and_owned_continuation() {
    for effect in [
        ZoneReplacementKind::Exile,
        ZoneReplacementKind::RevealShuffle,
    ] {
        let p = Player::P0;
        let mut g = scenario(p);
        put(&mut g, p.other(), "Rest in Peace", Zone::Battlefield);
        g.create_tokens(p, card(5302), 3, false);
        finish(&mut g);
        let id = saga(&mut g, p);
        until_trigger(&mut g);
        // An in-place copy retains this exact source incarnation and the queued
        // chapter, while adding the Colossus's actual self-shuffle replacement.
        g.apply_temporary_copy(
            g.object_ref(id),
            CopyRecipe::new(Card::from_name("Darksteel Colossus").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        for _ in 0..8 {
            if g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseZoneReplacement { .. }))
            {
                break;
            }
            choose(&mut g, |a| matches!(a, Action::Pass));
        }
        assert!(
            g.legal_actions()
                .contains(&Action::ChooseZoneReplacement { effect })
        );
        assert!(tokens(&g, p, 5416).is_empty());

        act(&mut g, Action::ChooseZoneReplacement { effect });
        finish(&mut g);
        assert_eq!(
            g.object_view(id).zone,
            if effect == ZoneReplacementKind::Exile {
                Zone::Exile
            } else {
                Zone::Library
            }
        );
        assert_eq!(
            tokens(&g, p, 5302).len(),
            4,
            "replay must not duplicate Treasure"
        );
        assert_eq!(tokens(&g, p, 5416).len(), 1);
    }
}
