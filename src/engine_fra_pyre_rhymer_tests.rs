use super::*;

// Keep the catalogue gated while using checked actions against authored rules.
fn pyre_fixture(g: &mut Game, player: Player, zone: Zone) -> CardId {
    let id = put(g, player, "Bear Cub", zone);
    g.state.objects[id as usize].card = Card::from_name("Pyre Rhymer // Molten Tide").unwrap();
    if zone == Zone::Battlefield { g.register_battlefield_sources(id); }
    id
}

// Draft-only front face coverage: Molten Tide remains unsupported and gated.
#[test]
fn pyre_rhymer_casts_as_three_three_and_enters_prepared() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = pyre_fixture(&mut g, player, Zone::Hand);
        g.state.players[player.index()].mana = [0, 0, 0, 2, 0, 1];
        let saved = g.snapshot();
        for _ in 0..2 {
            act(&mut g, Action::AnnounceCast { card: source });
            pay(&mut g);
            assert_eq!(g.state.players[player.index()].mana, [0; 6]);
            settle(&mut g);
            assert_eq!(stats(&g, source), (3, 3));
            let reference = g.object_ref(source);
            assert!(g.is_prepared(reference));
            let row = g.state.preparations.as_ref().unwrap().iter()
                .find(|row| row.permanent == reference).unwrap();
            assert_eq!(g.state.objects[row.spell_copy.card as usize].card, Card::test_index(5461));
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn pyre_rhymer_prowess_responds_only_to_own_noncreature_cast() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = pyre_fixture(&mut g, player, Zone::Battlefield);
        let creature = put(&mut g, player, "Bear Cub", Zone::Hand);
        cast(&mut g, creature, None);
        settle(&mut g);
        assert_eq!(stats(&g, source), (3, 3));
        let spell = put(&mut g, player, "Shock", Zone::Hand);
        cast(&mut g, spell, Some(Target::Player(player.other())));
        settle(&mut g);
        assert_eq!(stats(&g, source), (4, 4));
        g.finish_cleanup();
        assert_eq!(stats(&g, source), (3, 3));
    }
}

fn grant_molten_tide(g: &mut Game, player: Player) {
    let source = pyre_fixture(g, player, Zone::Battlefield);
    let reference = g.object_ref(source);
    g.prepare_permanent(reference);
    let copy = g.state.preparations.as_ref().unwrap().iter()
        .find(|row| row.permanent == reference).unwrap().spell_copy;
    g.state.players[player.index()].mana = [0, 0, 0, 1, 0, 0];
    act(g, Action::AnnounceExile { card: copy.card });
    pay(g);
    for _ in 0..96 {
        g.prepare_triggers();
        if g.state.pending == Pending::Priority && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty() && g.state.resolving.is_none() { return; }
        choose(g, |action| matches!(action, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Molten Tide resolution did not finish");
}

#[test]
fn molten_tide_stacks_and_only_actual_mountain_mana_taps_trigger() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        grant_molten_tide(&mut g, player);
        grant_molten_tide(&mut g, player);
        assert_eq!(g.observe(player).players[player.index()].mountain_mana_bonus, Some(2));
        assert_eq!(g.observe(player.other()).players[player.index()].mountain_mana_bonus, Some(2));
        let mountain = put(&mut g, player, "Mountain", Zone::Battlefield);
        let forest = put(&mut g, player, "Forest", Zone::Battlefield);
        let tapped_without_mana = put(&mut g, player, "Mountain", Zone::Battlefield);
        g.state.players[player.index()].mana = [0; 6];
        let saved = g.snapshot();
        for _ in 0..2 {
            g.tap_objects(&[tapped_without_mana]);
            assert_eq!(g.state.players[player.index()].mana, [0; 6]);
            act(&mut g, Action::TapMana { card: forest });
            assert_eq!(g.state.players[player.index()].mana, [0, 0, 0, 0, 1, 0]);
            act(&mut g, Action::TapMana { card: mountain });
            assert_eq!(g.state.players[player.index()].mana, [0, 0, 0, 3, 1, 0]);
            g.finish_cleanup();
            assert_eq!(g.observe(player).players[player.index()].mountain_mana_bonus, None);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn molten_tide_bonus_makes_a_real_taxed_cast_fundable() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        grant_molten_tide(&mut g, player);
        let mountain = put(&mut g, player, "Mountain", Zone::Battlefield);
        put(&mut g, player.other(), "Thalia, the Survivor", Zone::Battlefield);
        let spell = put(&mut g, player, "Shock", Zone::Hand);
        g.state.players[player.index()].mana = [0; 6];
        let saved = g.snapshot();
        for _ in 0..2 {
            act(&mut g, Action::AnnounceCast { card: spell });
            act(&mut g, Action::ChooseTarget { target: Target::Player(player.other()) });
            act(&mut g, Action::TapMana { card: mountain });
            assert_eq!(g.state.players[player.index()].mana, [0, 0, 0, 2, 0, 0]);
            pay(&mut g);
            assert_eq!(g.state.players[player.index()].mana, [0; 6]);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn molten_tide_bonus_is_unrestricted_when_original_mana_is_restricted() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        grant_molten_tide(&mut g, player);
        let land = put(&mut g, player, "Castle Doom", Zone::Battlefield);
        let reference = g.object_ref(land);
        let item = g.cast_preview(player, land, Zone::Battlefield, reference, 0, [0; 6]);
        g.add_basic_land_type(&item, Subject::Source, crate::cards::BasicLandType::Mountain);
        g.state.players[player.index()].mana = [0; 6];
        let activation = g.legal_actions().into_iter().find(|a| matches!(a,
            Action::ActivateMana { card, ability: 1, production: [0,0,0,1,0,0], .. } if *card == land)).unwrap();
        act(&mut g, activation);
        assert_eq!(g.state.players[player.index()].mana[3], 2);
        let shock = put(&mut g, player, "Shock", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: shock });
        act(&mut g, Action::ChooseTarget { target: Target::Player(player.other()) });
        pay(&mut g);
        assert_eq!(g.state.players[player.index()].mana[3], 1);
        assert_eq!(g.mana_lots(player).iter().find(|lot|
            lot.quality.restriction == ManaRestrictionView::ArtifactSpell).unwrap().amounts[3], 1);
    }
}

#[test]
fn molten_tide_counts_mountain_sacrificed_for_mana() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        grant_molten_tide(&mut g, player);
        let land = put(&mut g, player, "Great Hall of the Biblioplex", Zone::Battlefield);
        g.state.players[player.index()].mana = [0, 0, 0, 0, 0, 5];
        act(&mut g, Action::AnnounceActivate { card: land, ability: 2, counter_payment: Counters::default() });
        pay(&mut g);
        settle(&mut g);
        let land_ref = g.object_ref(land);
        let item = g.cast_preview(player, land, Zone::Battlefield, land_ref, 0, [0; 6]);
        g.add_basic_land_type(&item, Subject::Source, crate::cards::BasicLandType::Mountain);
        let ring = put(&mut g, player, "Lotus Ring", Zone::Battlefield);
        let ring_ref = g.object_ref(ring);
        assert!(g.attach_equipment(ring_ref, land_ref));
        g.state.players[player.index()].mana = [0; 6];
        let saved = g.snapshot();
        for _ in 0..2 {
            act(&mut g, Action::TapGrantedMana { card: land, grantor: ring_ref, production: [0, 0, 0, 0, 3, 0] });
            assert_eq!(g.object_view(land).zone, Zone::Graveyard);
            assert_eq!(g.state.players[player.index()].mana, [0, 0, 0, 1, 3, 0]);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn copied_molten_tide_creates_a_separate_trigger_for_its_controller() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = pyre_fixture(&mut g, player, Zone::Battlefield);
        let reference = g.object_ref(source);
        g.prepare_permanent(reference);
        let copy = g.state.preparations.as_ref().unwrap().iter().find(|row| row.permanent == reference).unwrap().spell_copy;
        g.state.players[player.index()].mana = [0, 0, 0, 1, 0, 0];
        act(&mut g, Action::AnnounceExile { card: copy.card });
        pay(&mut g);
        let original = g.state.stack.iter().find(|item| item.kind == StackKind::Spell
            && item.source_card == Card::test_index(5461)).unwrap().clone();
        let copy = g.allocate_spell_copy(&original, player.other());
        g.state.stack.push(copy);
        g.move_object(source, Zone::Graveyard);
        let saved = g.snapshot();
        for _ in 0..2 {
            g.resolve_top();
            settle(&mut g);
            assert_eq!(g.observe(player).players[player.index()].mountain_mana_bonus, Some(1));
            assert_eq!(g.observe(player).players[player.other().index()].mountain_mana_bonus, Some(1));
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn molten_tide_extra_mana_follows_kept_or_reversed_mana_event() {
    for player in [Player::P0, Player::P1] {
        for keep in [false, true] {
            let mut g = scenario(player);
            grant_molten_tide(&mut g, player);
            let silver = put(&mut g, player, "Silverquill, the Disputant", Zone::Battlefield);
            g.state.objects[silver as usize].growth.power = -1;
            let enemy = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
            let aura = put(&mut g, player.other(), "Witness Protection", Zone::Battlefield);
            g.state.objects[aura as usize].attached_to = Some(g.object_ref(enemy));
            let prison = put(&mut g, player, "Banishing Light", Zone::Hand);
            g.state.players[player.index()].mana = [8; 6];
            super::super::big_helpers::cast(&mut g, prison, None, &[]);
            for _ in 0..40 {
                if g.state.pending != Pending::Priority { break; }
                act(&mut g, Action::Pass);
            }
            act(&mut g, Action::ChooseTarget { target: Target::Card(aura) });
            super::super::big_helpers::settle(&mut g);
            let mountain = put(&mut g, player, "Mountain", Zone::Battlefield);
            let spell = put(&mut g, player, "Candy Grapple", Zone::Hand);
            g.state.players[player.index()].mana = [0, 0, 1, 0, 0, 0];
            act(&mut g, Action::AnnounceCast { card: spell });
            let sacrifice = g.object_ref(prison);
            act(&mut g, Action::ChooseBargain { sacrifice: Some(sacrifice) });
            act(&mut g, Action::ChooseCasualty { pay: true });
            act(&mut g, Action::ChooseTarget { target: Target::Card(enemy) });
            act(&mut g, Action::TapMana { card: mountain });
            pay(&mut g);
            let clause = g.observe(player).cast_costs.unwrap().clauses.iter()
                .position(|row| row.clause == CastCostClause::Additional).unwrap() as u16;
            act(&mut g, Action::ChooseCastCost { clause });
            let silver = g.object_ref(silver);
            act(&mut g, Action::ChooseAuraAttachment { object: silver });
            assert!(matches!(g.state.pending, Pending::ManaRollback { .. }));
            assert_eq!(g.observe(player).mana_rollback.unwrap().mountain_bonus, Some(1));
            act(&mut g, Action::ChooseManaRollback { keep });
            assert_eq!(g.object_view(spell).zone, Zone::Hand);
            assert_eq!(g.object_view(mountain).tapped, keep);
            assert_eq!(g.state.players[player.index()].mana, [0, 0, 1, if keep { 2 } else { 0 }, 0, 0]);
        }
    }
}

#[test]
fn molten_tide_announced_mana_pauses_before_the_mountain_is_tapped() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        grant_molten_tide(&mut g, player);
        let bolg = put(&mut g, player, "Bolg's Company", Zone::Battlefield);
        let goblin = put(&mut g, player, "Goblin Piker", Zone::Battlefield);
        let bolg_ref = g.object_ref(bolg);
        let timestamp = g.next_role_timestamp();
        g.state.roles.get_or_insert_default().animations.push(AnimationView {
            object: bolg_ref, timestamp, artifact: false, land: true,
            preserve_card_types: true, food_only: false, artifact_noncreature: false,
            colors: None, creature_types: None, all_creature_types: false,
            base: None, keywords: Keywords::NONE, expires_turn: None,
        });
        let item = g.cast_preview(player, bolg, Zone::Battlefield, bolg_ref, 0, [0; 6]);
        g.add_basic_land_type(&item, Subject::Source, crate::cards::BasicLandType::Mountain);
        assert!(g.has_subtype(bolg, "Mountain"));
        g.state.players[player.index()].mana = [0; 6];
        act(&mut g, Action::AnnounceMana { card: bolg, ability: 0,
            counter_payment: Counters::default(), production: [0, 0, 1, 1, 0, 0] });
        assert!(!g.object_view(bolg).tapped);
        assert_eq!(g.state.players[player.index()].mana, [0; 6]);
        let saved = g.snapshot();
        for _ in 0..2 {
            let invalid = g.object_ref(bolg);
            reject(&mut g, Action::ChooseAdditionalCost { object: invalid });
            let selected = g.object_ref(goblin);
            act(&mut g, Action::ChooseAdditionalCost { object: selected });
            pay(&mut g);
            assert!(g.object_view(bolg).tapped);
            assert_eq!(g.object_view(goblin).zone, Zone::Graveyard);
            assert_eq!(g.state.players[player.index()].mana, [0, 0, 1, 2, 0, 0]);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn molten_tide_borrowed_mana_uses_the_mountain_source_not_the_exiled_donor() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        grant_molten_tide(&mut g, player);
        let donor = put(&mut g, player.other(), "Bolg's Company", Zone::Battlefield);
        let forge = put(&mut g, player, "Territory Forge", Zone::Hand);
        g.state.players[player.index()].mana = [8; 6];
        cast(&mut g, forge, None);
        for _ in 0..96 {
            if g.state.pending == Pending::Priority && g.state.stack.is_empty()
                && g.state.pending_triggers.is_empty() && g.state.resolving.is_none() { break; }
            let target = Action::ChooseTarget { target: Target::Card(donor) };
            if g.legal_actions().contains(&target) { act(&mut g, target); }
            else { choose(&mut g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. } | Action::FinishTargets)); }
        }
        assert_eq!(g.object_view(donor).zone, Zone::Exile);
        let forge_ref = g.object_ref(forge);
        let item = g.cast_preview(player, forge, Zone::Battlefield, forge_ref, 0, [0; 6]);
        g.add_basic_land_type(&item, Subject::Source, crate::cards::BasicLandType::Mountain);
        let goblin = put(&mut g, player, "Goblin Piker", Zone::Battlefield);
        let descriptor = g.borrowed_ability_views().into_iter()
            .find(|row| row.source.card == forge && row.ability == 0).unwrap();
        g.state.players[player.index()].mana = [0; 6];
        let action = Action::AnnounceBorrowedMana { card: forge, exiled: descriptor.exiled,
            ability: 0, counter_payment: Counters::default(), production: [0, 0, 1, 1, 0, 0] };
        act(&mut g, action);
        assert_eq!(g.state.players[player.index()].mana, [0; 6]);
        let saved = g.snapshot();
        for _ in 0..2 {
            let selected = g.object_ref(goblin);
            act(&mut g, Action::ChooseAdditionalCost { object: selected });
            pay(&mut g);
            assert!(g.object_view(forge).tapped);
            assert_eq!(g.object_view(donor).zone, Zone::Exile);
            assert_eq!(g.object_view(goblin).zone, Zone::Graveyard);
            assert_eq!(g.state.players[player.index()].mana, [0, 0, 1, 2, 0, 0]);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn molten_tide_uses_current_tapper_not_owner_for_each_seat() {
    for player in [Player::P0, Player::P1] {
        for tapper_is_effect_controller in [false, true] {
            let mut g = scenario(player);
            grant_molten_tide(&mut g, player);
            let tapper = if tapper_is_effect_controller { player } else { player.other() };
            let mountain = put(&mut g, tapper.other(), "Mountain", Zone::Battlefield);
            let reference = g.object_ref(mountain);
            g.change_controller(reference, tapper);
            g.state.active = tapper;
            g.state.priority = tapper;
            g.state.players[tapper.index()].mana = [0; 6];
            let saved = g.snapshot();
            for _ in 0..2 {
                act(&mut g, Action::TapMana { card: mountain });
                assert_eq!(g.state.players[tapper.index()].mana[3], if tapper_is_effect_controller { 2 } else { 1 });
                assert_eq!(g.observe(player).players[player.index()].mountain_mana_bonus, Some(1));
                g.restore(&saved).unwrap();
            }
        }
    }
}
