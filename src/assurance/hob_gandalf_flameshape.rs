//! Private persistent exile rights and ordinary cast-trigger outcomes for both faces.
use super::*;
const GANDALF: &str = "Gandalf, Goblins' Bane // Flameshape";
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..80 {
        if ready(g) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass | Action::PutTrigger { .. } | Action::FinishStackTargets
            )
        });
    }
    panic!("Gandalf budget {:?}", g.state.pending);
}
fn finish(g: &mut Game) {
    until(g, |g| {
        g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending == Pending::Priority
    });
}
fn wizard(g: &mut Game, p: Player) -> CardId {
    put(g, p, GANDALF, Zone::Battlefield)
}
fn top(g: &mut Game, p: Player, names: &[&str]) -> Vec<CardId> {
    let old = g.state.players[p.index()].library.clone();
    for id in old {
        g.move_object(id, Zone::Graveyard);
    }
    names.iter().map(|n| put(g, p, n, Zone::Library)).collect()
}
fn expose(g: &mut Game, p: Player, names: &[&str]) -> (CardId, Vec<CardId>) {
    let cards = top(g, p, names);
    let id = put(g, p, GANDALF, Zone::Hand);
    g.state.priority = p;
    g.state.active = p;
    g.state.step = Step::Main1;
    g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 1];
    act(g, Action::AnnounceAdventure { card: id });
    pay(g);
    finish(g);
    (id, cards)
}
fn private(g: &Game, p: Player, cards: &[CardId]) {
    for id in cards {
        assert!(
            g.observe(p)
                .objects
                .iter()
                .any(|o| o.id == *id && o.zone == Zone::Exile)
        );
        assert!(g.observe(p.other()).objects.iter().all(|o| o.id != *id));
        assert!(
            g.observe(p.other())
                .exile_permissions
                .iter()
                .all(|r| r.object.card != *id)
        );
    }
    assert!(
        g.observe(p.other())
            .linked_exile
            .iter()
            .all(|l| !cards.contains(&l.exiled.card))
    );
}
fn own(g: &mut Game, p: Player) {
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Main1;
}
fn shock(g: &mut Game, p: Player) -> CardId {
    g.state.priority = p;
    let id = put(g, p, "Shock", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
    cast(g, id, Some(Target::Player(p.other())));
    id
}
#[test]
fn hob_gandalf_paid_body_and_noncreature_cast_trigger_is_temporary() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let s = put(&mut g, p, GANDALF, Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 2];
        cast(&mut g, s, None);
        finish(&mut g);
        assert_eq!(stats(&g, s), (2, 3));
        assert!(g.has_subtype(s, "Avatar") && g.has_subtype(s, "Wizard"));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        let pick = put(&mut g, p, "Goldvein Pick", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        cast(&mut g, pick, None);
        let spell = g.state.stack.iter().find(|v| v.card == pick).unwrap().id;
        g.counter_stack_item(spell, false);
        finish(&mut g);
        assert_eq!(stats(&g, s), (3, 4));
        assert_eq!(g.state.players[p.other().index()].life, 19);
        let turn = g.state.turn;
        for _ in 0..100 {
            if g.state.turn != turn {
                break;
            }
            choose(&mut g, |a| {
                matches!(
                    a,
                    Action::Pass | Action::FinishAttackers | Action::FinishBlockers
                )
            });
        }
        assert_ne!(g.state.turn, turn);
        assert_eq!(stats(&g, s), (2, 3));
    }
}
#[test]
fn hob_gandalf_trigger_ignores_own_creature_and_opponent_noncreature() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let s = wizard(&mut g, p);
        let bear = put(&mut g, p, "Bear Cub", Zone::Hand);
        cast(&mut g, bear, None);
        finish(&mut g);
        own(&mut g, p.other());
        let pick = put(&mut g, p.other(), "Goldvein Pick", Zone::Hand);
        cast(&mut g, pick, None);
        finish(&mut g);
        assert_eq!(stats(&g, s), (2, 3));
        assert_eq!(g.state.players[p.other().index()].life, 20);
    }
}
#[test]
fn hob_gandalf_departed_source_still_deals_damage_and_does_not_grow_reentry() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let s = wizard(&mut g, p);
        shock(&mut g, p);
        g.move_object(s, Zone::Hand);
        g.move_object(s, Zone::Battlefield);
        finish(&mut g);
        assert_eq!(stats(&g, s), (2, 3));
        assert_eq!(g.state.players[p.other().index()].life, 17);
    }
}
#[test]
fn hob_gandalf_staff_doubles_whole_trigger_and_spell_copies_are_not_casts() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let s = wizard(&mut g, p);
        let staff = put(&mut g, p, "Wizard's Staff", Zone::Battlefield);
        assert!(g.attach_equipment(g.object_ref(staff), g.object_ref(s)));
        shock(&mut g, p);
        finish(&mut g);
        assert_eq!(stats(&g, s), (6, 7));
        assert_eq!(g.state.players[p.other().index()].life, 16);
        let mut g = scenario(p);
        let s = wizard(&mut g, p);
        let original = shock(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 2, 0, 0];
        let sparks = put(&mut g, p, "Choreographed Sparks", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: sparks });
        act(&mut g, Action::ChooseMode { mode: 0 });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Spell(original),
            },
        );
        pay(&mut g);
        finish(&mut g);
        assert_eq!(stats(&g, s), (4, 5));
        assert_eq!(g.state.players[p.other().index()].life, 14);
    }
}
#[test]
fn hob_gandalf_ability_suppression_stops_new_packets_and_control_uses_current_caster() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let s = wizard(&mut g, p);
        let aura = put(&mut g, p.other(), "Witness Protection", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(s));
        shock(&mut g, p);
        finish(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        g.move_object(aura, Zone::Graveyard);
        g.change_controller(g.object_ref(s), p.other());
        own(&mut g, p.other());
        shock(&mut g, p.other());
        finish(&mut g);
        assert_eq!(stats(&g, s), (3, 4));
        assert_eq!(g.state.players[p.index()].life, 17);
    }
}
#[test]
fn hob_flameshape_paid_face_exiles_zero_one_or_two_privately_without_wizard() {
    for p in [Player::P0, Player::P1] {
        for names in [
            &[][..],
            &["Shock"][..],
            &["Forest", "Shock"][..],
            &["Goldvein Pick", "Forest", "Shock"][..],
        ] {
            let mut g = scenario(p);
            let (source, all) = expose(&mut g, p, names);
            let cards = &all[all.len().saturating_sub(2)..];
            assert_eq!(
                g.state.players[p.index()].library.len(),
                all.len() - cards.len()
            );
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(g.object_view(source).zone, Zone::Exile);
            assert!(g.can_play_adventure(p, source));
            assert_eq!(g.observe(p).exile_permissions.len(), cards.len());
            private(&g, p, cards);
            for &id in cards {
                let row = g
                    .observe(p)
                    .exile_permissions
                    .into_iter()
                    .find(|r| r.object.card == id)
                    .unwrap();
                assert_eq!(row.expiry, ExilePlayExpiry::WhileExiled);
                assert_eq!(row.requires_subtype, Some("Wizard"));
                reject(&mut g, Action::AnnounceExile { card: id });
                reject(&mut g, Action::PlayLand { card: id });
            }
        }
    }
}
#[test]
fn hob_flameshape_real_adventure_body_enables_paid_spell_and_land_then_control_revokes() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let (source, cards) = expose(&mut g, p, &["Forest", "Shock"]);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 2];
        act(&mut g, Action::AnnounceExile { card: source });
        pay(&mut g);
        finish(&mut g);
        assert_eq!(stats(&g, source), (2, 3));
        assert!(g.can_play_exiled(p, cards[1]));
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
        act(&mut g, Action::AnnounceExile { card: cards[1] });
        assert!(
            g.observe(p.other())
                .objects
                .iter()
                .any(|o| o.id == cards[1] && o.zone == Zone::Stack)
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        pay(&mut g);
        finish(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 17);
        assert_eq!(stats(&g, source), (3, 4));
        g.change_controller(g.object_ref(source), p.other());
        reject(&mut g, Action::PlayLand { card: cards[0] });
        private(&g, p, &cards[..1]);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let recipient = g.object_ref(bear);
        let timestamp = g.next_role_timestamp();
        g.state.roles.get_or_insert_default().creature_types.push(
            subtypes::CreatureTypesEffectView {
                recipient,
                source: recipient,
                types: &["Wizard"],
                additional: true,
                timestamp,
            },
        );
        act(&mut g, Action::PlayLand { card: cards[0] });
        assert!(
            g.observe(p.other())
                .objects
                .iter()
                .any(|o| o.id == cards[0] && o.zone == Zone::Battlefield)
        );
        assert!(g.state.land_played);
    }
}
#[test]
fn hob_flameshape_condition_uses_current_type_and_does_not_restrict_look() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let (_, cards) = expose(&mut g, p, &["Goldvein Pick"]);
        let s = wizard(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        assert!(g.can_play_exiled(p, cards[0]));
        let aura = put(&mut g, p.other(), "Witness Protection", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(s));
        assert!(!g.has_subtype(s, "Wizard"));
        assert!(!g.can_play_exiled(p, cards[0]));
        private(&g, p, &cards);
        g.move_object(aura, Zone::Graveyard);
        assert!(g.can_play_exiled(p, cards[0]));
        act(&mut g, Action::AnnounceExile { card: cards[0] });
        pay(&mut g);
        finish(&mut g);
        assert_eq!(g.object_view(cards[0]).zone, Zone::Battlefield);
    }
}
#[test]
fn hob_flameshape_timing_cost_land_allowance_and_exact_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let (_, cards) = expose(&mut g, p, &["Forest", "Goldvein Pick"]);
        wizard(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        reject(&mut g, Action::AnnounceExile { card: cards[1] });
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        g.state.active = p.other();
        reject(&mut g, Action::AnnounceExile { card: cards[1] });
        reject(&mut g, Action::PlayLand { card: cards[0] });
        own(&mut g, p);
        let handland = put(&mut g, p, "Forest", Zone::Hand);
        act(&mut g, Action::PlayLand { card: handland });
        reject(&mut g, Action::PlayLand { card: cards[0] });
        let old = g.object_ref(cards[1]);
        g.move_object(cards[1], Zone::Hand);
        g.move_object(cards[1], Zone::Exile);
        assert_ne!(old, g.object_ref(cards[1]));
        assert!(!g.can_play_exiled(p, cards[1]));
        assert!(
            g.observe(p.other())
                .objects
                .iter()
                .any(|o| o.id == cards[1])
        );
    }
}
#[test]
fn hob_flameshape_source_leaves_and_hidden_rights_survive_turn_cleanup() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let (source, cards) = expose(&mut g, p, &["Goldvein Pick", "Shock"]);
        g.move_object(source, Zone::Graveyard);
        for _ in 0..4 {
            put(&mut g, p, "Forest", Zone::Library);
        }
        let start = g.state.turn;
        for _ in 0..130 {
            if g.state.turn >= start + 2 {
                break;
            }
            choose(&mut g, |a| {
                matches!(
                    a,
                    Action::Pass | Action::FinishAttackers | Action::FinishBlockers
                )
            });
        }
        assert!(g.state.turn >= start + 2);
        private(&g, p, &cards);
        assert_eq!(g.observe(p).exile_permissions.len(), 2);
    }
}
#[test]
fn hob_flameshape_actual_adventure_cast_triggers_front_and_countered_adventure_exiles_nothing() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let s = wizard(&mut g, p);
        let before = g.state.players[p.index()].library.clone();
        let id = put(&mut g, p, GANDALF, Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 1];
        act(&mut g, Action::AnnounceAdventure { card: id });
        pay(&mut g);
        let spell = g.state.stack.iter().find(|s| s.source.is_some_and(|source| source.id == id)).unwrap().id;
        g.counter_stack_item(spell, false);
        finish(&mut g);
        assert_eq!(stats(&g, s), (3, 4));
        assert_eq!(g.state.players[p.other().index()].life, 19);
        assert_eq!(g.state.players[p.index()].library, before);
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        assert!(g.observe(p).exile_permissions.is_empty());
    }
}
#[test]
fn hob_flameshape_mana_sacrificing_last_wizard_preserves_approved_cast_and_locked_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let (_, cards) = expose(&mut g, p, &["Goldvein Pick"]);
        let s = wizard(&mut g, p);
        let ring = put(&mut g, p, "Lotus Ring", Zone::Battlefield);
        assert!(g.attach_equipment(g.object_ref(ring), g.object_ref(s)));
        g.state.players[p.index()].mana = [0; 6];
        act(&mut g, Action::AnnounceExile { card: cards[0] });
        assert_eq!(
            g.state
                .announcing
                .as_ref()
                .unwrap()
                .total_cost
                .unwrap()
                .generic,
            2
        );
        let grantor = g.object_ref(ring);
        act(
            &mut g,
            Action::TapGrantedMana {
                card: s,
                grantor,
                production: [0, 0, 3, 0, 0, 0],
            },
        );
        assert_eq!(g.object_view(s).zone, Zone::Graveyard);
        act(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 2, 0, 0, 0],
            },
        );
        finish(&mut g);
        assert_eq!(g.object_view(cards[0]).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 1, 0, 0, 0]);
    }
}
#[test]
fn hob_flameshape_owned_private_permission_raw_index_restore_after_parent_disposal() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let (_, cards) = expose(&mut g, p, &["Forest", "Goldvein Pick"]);
        wizard(&mut g, p);
        private(&g, p, &cards);
        contracts::branch_contract(g.clone());
        let action = Action::PlayLand { card: cards[0] };
        let saved = g.snapshot();
        let mut raw = g.clone();
        let mut indexed = g.clone();
        drop(g);
        let index = indexed
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        indexed.apply_index(p, indexed.revision(), index).unwrap();
        act(&mut raw, action.clone());
        assert_eq!(raw.snapshot(), indexed.snapshot());
        private(&raw, p, &cards[1..]);
        let expected = raw.snapshot();
        raw.restore(&saved).unwrap();
        private(&raw, p, &cards);
        act(&mut raw, action);
        assert_eq!(raw.snapshot(), expected);
    }
}
#[test]
fn hob_gandalf_trigger_is_damage_with_source_lifelink_after_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = wizard(&mut g, p);
        let collar = put(&mut g, p, "Basilisk Collar", Zone::Battlefield);
        assert!(g.attach_equipment(g.object_ref(collar), g.object_ref(source)));
        let pick = put(&mut g, p, "Goldvein Pick", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        cast(&mut g, pick, None);
        g.move_object(source, Zone::Graveyard);
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].life, 21);
        assert_eq!(g.state.players[p.other().index()].life, 19);
    }
}

#[test]
fn gandalf_front_and_combined_names_resolve_to_same_physical_card() {
    let front = Card::from_name("Gandalf, Goblins' Bane").unwrap();
    let combined = Card::from_name("Gandalf, Goblins' Bane // Flameshape").unwrap();
    assert_eq!(front, combined);
    assert_eq!(front.name(), "Gandalf, Goblins' Bane");
}
