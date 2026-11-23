//! Affordability at proposal time must preserve affordable routes and manual mana.
use crate::support::*;
use rustmage as _;
use rustmage_core::cards::Counters;
use rustmage_core::engine::*;

fn reject_at_priority(game: &mut Game, player: Player, action: Action) {
    let snapshot = game.snapshot();
    let revision = game.revision();
    assert!(!game.legal_actions().contains(&action));
    assert!(game.apply(player, action).is_err());
    assert_eq!(game.revision(), revision);
    assert_eq!(game.snapshot(), snapshot);
}

#[test]
fn bargain_requires_a_possible_sacrifice_without_reserving_mana_sources() {
    let player = Player::P0;
    for enabled in [false, true] {
        let mut game = scenario(player).with_proposal_affordability(enabled);
        game.state.players[player.index()].mana = [0, 0, 0, 1, 0, 0];
        let spell = put(&mut game, player, "Torch the Tower", Zone::Hand);
        put(
            &mut game,
            player.other(),
            "Grizzly Bears",
            Zone::Battlefield,
        );
        put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        put(&mut game, player.other(), "Candy Trail", Zone::Battlefield);
        let mut with_treasure = game.clone();
        with_treasure.state.players[player.index()].mana = [0; 6];
        let treasure =
            rustmage_core::cards::Card::from_name("@token/b60f6cd3b44b22bd289fe4e2").unwrap();
        with_treasure.create_tokens(player, treasure, 1, false);
        let cast = Action::AnnounceCast { card: spell };
        act(&mut game, cast.clone());
        let pay = Action::DeclareBargain { pay: true };
        assert_eq!(game.legal_actions().contains(&pay), !enabled);
        if enabled {
            reject_at_priority(&mut game, player, pay.clone());
        }
        act(&mut game, Action::DeclareBargain { pay: false });

        // The same Treasure might be needed for both costs. Preserve the
        // choice when there is material and the costs would need joint search.
        act(&mut with_treasure, cast);
        act(&mut with_treasure, pay);
    }
}

#[test]
fn optional_evidence_requires_enough_graveyard_mana_value() {
    let player = Player::P0;
    let mut game = scenario(player).with_proposal_affordability(true);
    game.state.players[player.index()].mana = [0, 0, 2, 0, 0, 3];
    let spell = put(&mut game, player, "Deadly Cover-Up", Zone::Hand);
    put(&mut game, player, "Hill Giant", Zone::Graveyard);
    put(&mut game, player, "Opt", Zone::Graveyard);
    let mut funded = game.clone();
    put(&mut funded, player, "Opt", Zone::Graveyard);
    let cast = Action::AnnounceCast { card: spell };
    act(&mut game, cast.clone());
    let collect = Action::ChooseCollectEvidence { collect: true };
    reject_at_priority(&mut game, player, collect.clone());
    act(&mut game, Action::ChooseCollectEvidence { collect: false });
    pay(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Stack);

    act(&mut funded, cast);
    act(&mut funded, collect);
}

#[test]
fn linked_returns_only_disable_the_bound_when_funding_can_release_them() {
    let player = Player::P0;
    let mut game = scenario(player).with_proposal_affordability(true);
    game.state.players[player.index()].mana = [0; 6];
    let host = put(&mut game, player, "Perilous Snare", Zone::Battlefield);
    let producer = put(&mut game, player, "The Mind Stone", Zone::Exile);
    game.state.exile_links.push(ExileLink {
        ability_origin: None,
        source: game.object_ref(host),
        exiled: game.object_ref(producer),
        return_zone: Zone::Battlefield,
    });
    let opt = put(&mut game, player, "Opt", Zone::Hand);
    reject_at_priority(&mut game, player, Action::AnnounceCast { card: opt });

    let treasure = put(&mut game, player, "Buried Treasure", Zone::Battlefield);
    game.state.exile_links[0].source = game.object_ref(treasure);
    let drowner = put(&mut game, player, "Floodpits Drowner", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: drowner });
    act(
        &mut game,
        Action::ActivateMana {
            card: treasure,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: [0, 1, 0, 0, 0, 0],
        },
    );
    assert_eq!(game.object_view(producer).zone, Zone::Battlefield);
    choose(
        &mut game,
        |action| matches!(action, Action::TapMana { card } | Action::ActivateMana { card, .. } if *card == producer),
    );
    pay(&mut game);
    assert_eq!(game.object_view(drowner).zone, Zone::Stack);
}

#[test]
fn floating_spell_mana_respects_all_restrictions_and_the_cast_origin() {
    let player = Player::P0;
    let mut game = scenario(player).with_proposal_affordability(true);
    game.state.players[player.index()].mana = [0, 0, 0, 1, 0, 2];
    for restriction in [
        ManaRestrictionView::CastFromExile,
        ManaRestrictionView::AbilityActivation,
    ] {
        game.add_quality_mana(
            player,
            [0, 0, 0, 0, 0, 1],
            Some(mana_lots::Quality {
                restriction,
                ..mana_lots::Quality::ORDINARY
            }),
        );
    }

    let from_hand = put(&mut game, player, "Hill Giant", Zone::Hand);
    reject_at_priority(&mut game, player, Action::AnnounceCast { card: from_hand });

    let from_exile = put(&mut game, player, "Hill Giant", Zone::Exile);
    game.grant_exile_play_permission(player, game.object_ref(from_exile), false);
    act(&mut game, Action::AnnounceExile { card: from_exile });
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 0, 1, 0, 3],
        },
    );
    assert_eq!(game.state.players[player.index()].spells_cast_this_turn, 1);
}

#[test]
fn floating_creature_mana_requires_the_spells_creature_type() {
    use rustmage_core::cards::CreatureType;

    let player = Player::P0;
    for (creature_type, eligible) in [("Cat", false), ("Elf", true)] {
        let mut game = scenario(player).with_proposal_affordability(true);
        game.state.players[player.index()].mana = [0; 6];
        let creature_type = CreatureType::from_name(creature_type).unwrap();
        game.add_quality_mana(
            player,
            [0, 0, 0, 0, 1, 0],
            Some(mana_lots::Quality {
                restriction: ManaRestrictionView::CreatureSpell(creature_type),
                ..mana_lots::Quality::ORDINARY
            }),
        );

        let spell = put(&mut game, player, "Llanowar Elves", Zone::Hand);
        let cast = Action::AnnounceCast { card: spell };
        if eligible {
            act(&mut game, cast);
            pay(&mut game);
            assert_eq!(game.state.players[player.index()].spells_cast_this_turn, 1);
        } else {
            reject_at_priority(&mut game, player, cast);
        }
    }
}

#[test]
fn unpaid_spell_is_hidden_and_funded_spell_casts_from_either_seat() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player).with_proposal_affordability(true);
        game.state.players[player.index()].mana = [0; 6];
        let spell = put(&mut game, player, "Hill Giant", Zone::Hand);
        let cast = Action::AnnounceCast { card: spell };
        Game::reset_payment_planning_work();
        reject_at_priority(&mut game, player, cast.clone());

        let mountain = put(&mut game, player, "Mountain", Zone::Battlefield);
        let tap = Action::TapMana { card: mountain };
        assert!(game.legal_actions().contains(&tap));
        assert!(game.legal_actions().contains(&Action::Pass));
        reject_at_priority(&mut game, player, cast.clone());

        game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 3];
        let index = game
            .legal_actions()
            .iter()
            .position(|a| a == &cast)
            .unwrap();
        let mut indexed = game.clone();
        indexed
            .apply_index(player, indexed.revision(), index)
            .unwrap();
        act(&mut game, cast);
        assert_eq!(game.snapshot(), indexed.snapshot());
        act(&mut game, tap);
        act(
            &mut game,
            Action::PayCosts {
                payment: [0, 0, 0, 1, 0, 3],
            },
        );
        assert_eq!(game.state.players[player.index()].spells_cast_this_turn, 1);
        let work = Game::payment_planning_work();
        assert_eq!(
            (
                work.projected_moves,
                work.projected_steps,
                work.continuation_plans
            ),
            (0, 0, 0)
        );
    }
}

#[test]
fn sick_mana_creature_ignores_unrelated_keywords_and_counters_but_can_tap_with_haste() {
    use rustmage_core::cards::CounterKind;

    let player = Player::P0;
    let mut game = scenario(player).with_proposal_affordability(true);
    game.state.players[player.index()].mana = [0; 6];
    let source = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
    game.state.objects[source as usize].summoning_sick = true;
    let flyer = put(
        &mut game,
        player.other(),
        "Spyglass Siren",
        Zone::Battlefield,
    );
    game.add_counter(flyer, CounterKind::Quest, 1);
    put(&mut game, player, "Slickshot Show-Off", Zone::Battlefield);
    let spell = put(&mut game, player, "Llanowar Elves", Zone::Hand);
    let cast = Action::AnnounceCast { card: spell };
    reject_at_priority(&mut game, player, cast.clone());

    game.add_counter(source, CounterKind::Haste, 1);
    act(&mut game, cast);
    act(&mut game, Action::TapMana { card: source });
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 1, 0],
        },
    );
    assert_eq!(game.state.players[player.index()].spells_cast_this_turn, 1);
}

#[test]
fn craft_materials_only_supply_mana_when_the_host_can_borrow_their_abilities() {
    let player = Player::P0;
    for (host, borrows) in [("Braided Quipu", false), ("Locus of Enlightenment", true)] {
        let mut game = scenario(player).with_proposal_affordability(true);
        game.state.players[player.index()].mana = [0; 6];
        let host = put(&mut game, player, host, Zone::Battlefield);
        let donor = put(&mut game, player, "Llanowar Elves", Zone::Exile);
        let spell = put(&mut game, player, "Candy Trail", Zone::Hand);
        let donor = game.object_ref(donor);
        game.register_craft_materials(game.object_ref(host), &[donor]);
        game.state.borrowed_slots = borrowed_activations::refreshed_slots(&game);
        let cast = Action::AnnounceCast { card: spell };
        if borrows {
            assert!(game.legal_actions().contains(&cast));
            choose(&mut game, |action| {
                matches!(action,
                Action::AnnounceBorrowedMana { card, exiled, .. } if *card == host && *exiled == donor)
            });
            if matches!(game.state.pending, Pending::PayCosts { .. }) {
                pay(&mut game);
            }
            assert_eq!(game.state.players[player.index()].mana[4], 1);
            act(&mut game, cast);
            pay(&mut game);
            assert_eq!(game.state.players[player.index()].spells_cast_this_turn, 1);
        } else {
            reject_at_priority(&mut game, player, cast);
        }
    }
}

#[test]
fn spell_cast_trigger_granted_by_animation_cannot_fund_the_cast() {
    let player = Player::P0;
    let mut game = scenario(player).with_proposal_affordability(true);
    let hall = put(
        &mut game,
        player,
        "Great Hall of the Biblioplex",
        Zone::Battlefield,
    );
    act(
        &mut game,
        Action::AnnounceActivate {
            card: hall,
            ability: 2,
            counter_payment: Counters::default(),
        },
    );
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.temporary_abilities.as_ref().unwrap().len(), 1);
    game.state.players[player.index()].mana = [0; 6];
    let giant = put(&mut game, player, "Hill Giant", Zone::Hand);
    reject_at_priority(&mut game, player, Action::AnnounceCast { card: giant });
    game.state.players[player.index()].mana = [0, 0, 0, 1, 0, 3];
    announce(&mut game, giant, &[]);
}

#[test]
fn offspring_base_override_keeps_the_token_definition_for_funding() {
    use rustmage_core::cards::{Card, RetainedCopyActivation};
    let player = Player::P0;
    let mut game = scenario(player).with_proposal_affordability(true);
    game.state.players[player.index()].mana = [0; 6];
    let mut recipe = CopyRecipe::new(Card::from_name("Starscape Cleric").unwrap());
    recipe.exceptions.base = Some((1, 1));
    game.stage_copy_creation(player, recipe, 1);
    game.advance_entries();
    let token = game
        .observe(player)
        .objects
        .into_iter()
        .find(|object| object.zone == Zone::Battlefield && object.is_token())
        .unwrap()
        .id;
    let spell = put(&mut game, player, "Llanowar Elves", Zone::Hand);
    let cast = Action::AnnounceCast { card: spell };
    reject_at_priority(&mut game, player, cast.clone());

    // A retained slot is an extra ability absent from the definition scanned above.
    recipe.exceptions.retained_activation = Some(RetainedCopyActivation {
        definition: Card::from_name("Candy Trail").unwrap(),
        ability: 0,
    });
    game.set_copy_recipe(game.object_ref(token), recipe);
    assert!(game.legal_actions().contains(&cast));
}

#[test]
fn optional_blight_requires_a_controlled_creature_even_when_the_target_is_legal() {
    let player = Player::P0;
    for enabled in [false, true] {
        let mut game = scenario(player).with_proposal_affordability(enabled);
        let spell = put(&mut game, player, "Requiting Hex", Zone::Hand);
        put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);

        let mut with_enchantment = game.clone();
        let object = put(
            &mut with_enchantment,
            player,
            "Enduring Curiosity",
            Zone::Battlefield,
        );
        let form = characteristic_layers::enchantment_noncreature_form(
            with_enchantment.object_ref(object),
            with_enchantment.next_role_timestamp(),
        );
        with_enchantment
            .state
            .roles
            .get_or_insert_default()
            .animations
            .push(form);
        assert!(!with_enchantment.is_creature(object));
        act(&mut with_enchantment, Action::AnnounceCast { card: spell });
        assert_eq!(
            with_enchantment
                .legal_actions()
                .contains(&Action::ChooseBlight { pay: true }),
            !enabled
        );

        let mut with_creature = game.clone();
        let own = put(&mut with_creature, player, "Bear Cub", Zone::Battlefield);
        with_creature.state.objects[own as usize].tapped = true;
        act(&mut with_creature, Action::AnnounceCast { card: spell });
        act(&mut with_creature, Action::ChooseBlight { pay: true });

        act(&mut game, Action::AnnounceCast { card: spell });
        let paid = Action::ChooseBlight { pay: true };
        assert_eq!(game.legal_actions().contains(&paid), !enabled);
        if enabled {
            reject_at_priority(&mut game, player, paid);
        }
        act(&mut game, Action::ChooseBlight { pay: false });
    }
}

#[test]
fn optional_teamwork_counts_untapped_power_even_on_a_single_summoning_sick_creature() {
    let player = Player::P0;
    let mut game = scenario(player).with_proposal_affordability(true);
    let spell = put(&mut game, player, "Repulsor Blast", Zone::Hand);
    put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    put(
        &mut game,
        player.other(),
        "Elusive Otter",
        Zone::Battlefield,
    );
    let elf = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
    game.state.objects[elf as usize].summoning_sick = true;
    let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    game.state.objects[bear as usize].tapped = true;
    for (bonus, enabled, offered) in [(0, true, false), (0, false, true), (1, true, true)] {
        let mut branch = game.clone().with_proposal_affordability(enabled);
        branch.state.objects[elf as usize].counters.plus_one = bonus;
        act(&mut branch, Action::AnnounceCast { card: spell });
        let paid = Action::ChooseTeamwork { pay: true };
        let unpaid = Action::ChooseTeamwork { pay: false };
        let actions = branch.legal_actions();
        assert!(actions.contains(&unpaid));
        assert_eq!(actions.contains(&paid), offered);
        let choice = if offered { paid } else { unpaid };
        let index = actions.iter().position(|action| action == &choice).unwrap();
        let mut indexed = branch.clone();
        indexed
            .apply_index(player, indexed.revision(), index)
            .unwrap();
        act(&mut branch, choice);
        assert_eq!(branch.snapshot(), indexed.snapshot());
    }
}

#[test]
fn max_speed_enables_its_nonmana_slot_without_funding_a_spell() {
    let player = Player::P0;
    let mut game = scenario(player).with_proposal_affordability(true);
    let snare = put(&mut game, player, "Perilous Snare", Zone::Battlefield);
    put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let spell = put(&mut game, player, "Llanowar Elves", Zone::Hand);
    game.state.players[player.index()].mana = [0; 6];
    game.state.players[player.index()].speed = 4;
    assert!(game.legal_actions().contains(&Action::AnnounceSourceGrant {
        source: game.object_ref(snare),
        ability: 0,
    }));
    reject_at_priority(&mut game, player, Action::AnnounceCast { card: spell });
    game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 0];
    announce(&mut game, spell, &[]);
}

#[test]
fn tap_cost_reserves_the_activating_permanent_for_fomori_vault() {
    let player = Player::P0;
    let mut game = scenario(player).with_proposal_affordability(true);
    game.state.players[player.index()].mana = [0; 6];
    let source = put(&mut game, player, "Fomori Vault", Zone::Battlefield);
    let discard = put(&mut game, player, "Hill Giant", Zone::Hand);
    let discard = game.object_ref(discard);
    let activate = Action::AnnounceActivate {
        card: source,
        ability: 1,
        counter_payment: Counters::default(),
    };
    for _ in 0..2 {
        put(&mut game, player, "Mountain", Zone::Battlefield);
    }
    reject_at_priority(&mut game, player, activate.clone());
    put(&mut game, player, "Mountain", Zone::Battlefield);
    act(&mut game, activate);
    act(&mut game, Action::ChooseAdditionalCost { object: discard });
    let attempt = game.attempt_view(player).unwrap();
    let result = game
        .auto_pay_mana(
            player,
            game.revision(),
            attempt.attempt_id,
            &Default::default(),
        )
        .unwrap();
    assert_eq!(result.status, AutoTapStatus::Completed);
    assert!(game.object_view(source).tapped);
}

#[test]
fn conditional_mana_requires_its_land_type_but_a_tapped_land_supplies_that_type() {
    let player = Player::P0;
    let mut game = scenario(player).with_proposal_affordability(true);
    game.state.players[player.index()].mana = [0; 6];
    let verge = put(&mut game, player, "Riverpyre Verge", Zone::Battlefield);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    let cast = Action::AnnounceCast { card: spell };
    reject_at_priority(&mut game, player, cast.clone());
    let mountain = put(&mut game, player, "Mountain", Zone::Battlefield);
    game.state.objects[mountain as usize].tapped = true;
    act(&mut game, cast);
    choose(&mut game, |action| {
        matches!(action,
        Action::ActivateMana { card, production, .. }
            if *card == verge && *production == [0, 1, 0, 0, 0, 0])
    });
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 1, 0, 0, 0, 0],
        },
    );
    assert_eq!(game.state.players[player.index()].spells_cast_this_turn, 1);
}

#[test]
fn a_single_dual_land_cannot_pay_both_colored_symbols() {
    let player = Player::P0;
    let mut game = scenario(player).with_proposal_affordability(true);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];
    put(&mut game, player, "Concealed Courtyard", Zone::Battlefield);
    let card = put(&mut game, player, "Amalia Benavides Aguirre", Zone::Hand);
    let cast = Action::AnnounceCast { card };
    reject_at_priority(&mut game, player, cast.clone());
    put(&mut game, player, "Plains", Zone::Battlefield);
    act(&mut game, cast);
    let attempt = game.attempt_view(player).unwrap();
    let payment = game
        .auto_pay_mana(
            player,
            game.revision(),
            attempt.attempt_id,
            &Default::default(),
        )
        .unwrap();
    assert_eq!(payment.status, AutoTapStatus::Completed);
}

#[test]
fn a_graveyard_discount_keeps_an_affordable_spell_available() {
    let player = Player::P0;
    let mut game = scenario(player).with_proposal_affordability(true);
    game.state.players[player.index()].mana = [0, 2, 0, 0, 0, 0];
    let card = put(&mut game, player, "Eddymurk Crab", Zone::Hand);
    for _ in 0..5 {
        put(&mut game, player, "Opt", Zone::Graveyard);
    }
    act(&mut game, Action::AnnounceCast { card });
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 2, 0, 0, 0, 0],
        },
    );
    assert_eq!(game.state.players[player.index()].spells_cast_this_turn, 1);
}

#[test]
fn a_turn_discount_keeps_a_face_down_cast_available() {
    let player = Player::P0;
    let mut game = scenario(player).with_proposal_affordability(true);
    let disguised = put(&mut game, player, "Fugitive Codebreaker", Zone::Hand);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];
    // The native continuous rule added by two Goblin Maskmaker attack triggers.
    game.grant_face_down_spell_reduction(player, 2);
    let cast = Action::AnnounceDisguise { card: disguised };
    assert!(game.legal_actions().contains(&cast));
    act(&mut game, cast);
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 0, 1],
        },
    );
    assert_eq!(game.state.players[player.index()].spells_cast_this_turn, 1);
}

#[test]
fn an_exile_discount_does_not_reduce_a_hand_cast() {
    let player = Player::P0;
    let mut game = scenario(player).with_proposal_affordability(true);
    put(
        &mut game,
        player,
        "Doc Aurlock, Grizzled Genius",
        Zone::Battlefield,
    );
    let hand = put(&mut game, player, "Bramble Familiar", Zone::Hand);
    let exiled = put(&mut game, player, "Bramble Familiar", Zone::Exile);
    game.grant_exile_play_permission(player, game.object_ref(exiled), false);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 0];

    reject_at_priority(&mut game, player, Action::AnnounceCast { card: hand });
    act(&mut game, Action::AnnounceExile { card: exiled });
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 1, 0],
        },
    );
    assert_eq!(game.state.players[player.index()].spells_cast_this_turn, 1);
    assert_eq!(game.object_view(exiled).zone, Zone::Stack);
}

#[test]
fn a_sacrifice_or_mana_spell_requires_the_mana_route_when_no_material_matches() {
    let player = Player::P0;
    let mut game = scenario(player).with_proposal_affordability(true);
    let spell = put(&mut game, player, "Stir Up Trouble", Zone::Hand);
    put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let cast = Action::AnnounceCast { card: spell };

    // The target is present. Only our permanents can pay the sacrifice.
    put(&mut game, player, "Swamp", Zone::Battlefield);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 3];
    reject_at_priority(&mut game, player, cast.clone());

    let mut mana_route = game.clone();
    mana_route.state.players[player.index()].mana[5] = 4;
    act(&mut mana_route, cast.clone());

    let mut sacrifice_route = game;
    sacrifice_route.state.players[player.index()].mana[5] = 0;
    put(&mut sacrifice_route, player, "Bear Cub", Zone::Battlefield);
    act(&mut sacrifice_route, cast);
}

#[test]
fn an_exile_cast_keeps_its_colors_unless_the_permission_allows_flexible_spending() {
    let player = Player::P0;
    let mut game = scenario(player).with_proposal_affordability(true);
    game.state.players[player.index()].mana = [0, 0, 0, 1, 0, 0];
    let spell = put(&mut game, player, "Opt", Zone::Exile);
    game.grant_exile_play_permission(player, game.object_ref(spell), false);
    let cast = Action::AnnounceExile { card: spell };
    reject_at_priority(&mut game, player, cast.clone());

    game.state
        .exile_permissions
        .last_mut()
        .unwrap()
        .spend_any_mana = true;
    act(&mut game, cast);
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 0, 1, 0, 0],
        },
    );
    assert_eq!(game.state.players[player.index()].spells_cast_this_turn, 1);
}

#[test]
fn harmonize_checks_the_mana_shortfall_after_the_largest_available_discount() {
    let player = Player::P0;
    let mut game = scenario(player).with_proposal_affordability(true);
    game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 2];
    let spell = put(&mut game, player, "Winternight Stories", Zone::Graveyard);
    let helper = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let cast = Action::AnnounceHarmonize { card: spell };
    game.state.objects[helper as usize].tapped = true;
    reject_at_priority(&mut game, player, cast.clone());

    game.state.objects[helper as usize].tapped = false;
    act(&mut game, cast);
    let creature = game.object_ref(helper);
    act(
        &mut game,
        Action::ChooseHarmonizer {
            object: Some(creature),
        },
    );
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 1, 0, 0, 0, 2],
        },
    );
    act(&mut game, Action::ChooseCastCost { clause: 0 });
    assert_eq!(game.state.players[player.index()].spells_cast_this_turn, 1);
}

#[test]
fn an_unrelated_blocking_rule_does_not_disable_affordability() {
    let player = Player::P0;
    let mut game = scenario(player).with_proposal_affordability(true);
    game.state.players[player.index()].mana = [0; 6];
    put(
        &mut game,
        player.other(),
        "Elusive Otter",
        Zone::Battlefield,
    );
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    let cast = Action::AnnounceCast { card: spell };
    reject_at_priority(&mut game, player, cast.clone());

    game.state.players[player.index()].mana[1] = 1;
    act(&mut game, cast);
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 1, 0, 0, 0, 0],
        },
    );
    assert_eq!(game.state.players[player.index()].spells_cast_this_turn, 1);
}

#[test]
fn allocating_the_zone_rules_cache_does_not_change_priority_offers() {
    let player = Player::P0;
    let mut game = scenario(player).with_proposal_affordability(true);
    game.state.players[player.index()].mana = [0; 6];
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    let cast = Action::AnnounceCast { card: spell };
    game.state.zone_rules = None;
    let visible = game.legal_actions();
    assert!(!visible.contains(&cast));

    // This storage may be allocated based on cards outside the player's view.
    game.state.zone_rules = Some(Box::default());
    assert_eq!(game.legal_actions(), visible);
}

#[test]
fn summoning_sickness_excludes_a_creatures_tap_mana_in_a_simple_board() {
    let player = Player::P0;
    let mut game = scenario(player).with_proposal_affordability(true);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];
    let elf = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
    let spell = put(&mut game, player, "Bear Cub", Zone::Hand);
    let cast = Action::AnnounceCast { card: spell };
    game.state.objects[elf as usize].summoning_sick = true;
    reject_at_priority(&mut game, player, cast.clone());

    game.state.objects[elf as usize].summoning_sick = false;
    act(&mut game, cast);
    act(&mut game, Action::TapMana { card: elf });
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 1, 1],
        },
    );
    assert_eq!(game.state.players[player.index()].spells_cast_this_turn, 1);
}

#[test]
fn mana_for_activations_does_not_count_toward_a_spell() {
    let player = Player::P0;
    let mut game = scenario(player).with_proposal_affordability(true);
    game.state.players[player.index()].mana = [0; 6];
    put(&mut game, player, "The Enigma Jewel", Zone::Battlefield);
    let spell = put(&mut game, player, "Torpor Orb", Zone::Hand);
    let cast = Action::AnnounceCast { card: spell };
    reject_at_priority(&mut game, player, cast.clone());

    game.state.players[player.index()].mana[5] = 2;
    act(&mut game, cast);
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 0, 2],
        },
    );
    assert_eq!(game.state.players[player.index()].spells_cast_this_turn, 1);
}

#[test]
fn kicker_and_offspring_check_the_entire_spell_cost() {
    let player = Player::P0;
    for (name, decline, pay, base, full) in [
        (
            "Consult the Star Charts",
            Action::ChooseKicker { pay: false },
            Action::ChooseKicker { pay: true },
            [0, 1, 0, 0, 0, 1],
            [0, 2, 0, 0, 0, 2],
        ),
        (
            "Starscape Cleric",
            Action::ChooseOffspring { pay: false },
            Action::ChooseOffspring { pay: true },
            [0, 0, 1, 0, 0, 1],
            [0, 0, 2, 0, 0, 3],
        ),
    ] {
        for enabled in [false, true] {
            let mut game = scenario(player).with_proposal_affordability(enabled);
            game.state.players[player.index()].mana = base;
            let spell = put(&mut game, player, name, Zone::Hand);
            act(&mut game, Action::AnnounceCast { card: spell });
            assert!(game.legal_actions().contains(&decline));
            assert_eq!(game.legal_actions().contains(&pay), !enabled);
            if enabled {
                reject_at_priority(&mut game, player, pay.clone());
            }
            let mut declined = game.clone();
            act(&mut declined, decline.clone());
            act(&mut declined, Action::PayCosts { payment: base });
            assert_eq!(
                declined.state.players[player.index()].spells_cast_this_turn,
                1
            );

            game.state.players[player.index()].mana = full;
            act(&mut game, pay.clone());
            act(&mut game, Action::PayCosts { payment: full });
            assert_eq!(game.state.players[player.index()].spells_cast_this_turn, 1);
        }
    }
}

#[test]
fn a_kicked_spell_keeps_its_exile_origin_when_pricing_mana() {
    let player = Player::P0;
    let mut game = scenario(player).with_proposal_affordability(true);
    let spell = put(&mut game, player, "Consult the Star Charts", Zone::Exile);
    game.grant_exile_play_permission(player, game.object_ref(spell), false);
    game.state.players[player.index()].mana = [0, 2, 0, 0, 0, 1];
    game.add_quality_mana(
        player,
        [0, 0, 0, 0, 0, 1],
        Some(mana_lots::Quality {
            restriction: ManaRestrictionView::CastFromExile,
            ..mana_lots::Quality::ORDINARY
        }),
    );
    act(&mut game, Action::AnnounceExile { card: spell });
    act(&mut game, Action::ChooseKicker { pay: true });
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 2, 0, 0, 0, 2],
        },
    );
    assert_eq!(game.state.players[player.index()].spells_cast_this_turn, 1);
}
