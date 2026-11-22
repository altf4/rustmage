use super::*;

fn cast_mode(game: &mut Game, card: CardId, mode: u16, recipient: Option<Target>) {
    act(game, Action::AnnounceCast { card });
    act(game, Action::ChooseMode { mode });
    if let Some(recipient) = recipient {
        target(game, recipient);
    }
    pay(game);
}

fn arrange(game: &mut Game, card: CardId, destination: LibraryDestination) {
    act(game, Action::ArrangeLibrary { card, destination });
}

#[test]
fn risky_research_surveils_privately_before_two_draws_and_life_loss() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Risky Research", Zone::Hand);
    let next = put(&mut game, player, "Plains", Zone::Library);
    let kept = put(&mut game, player, "Island", Zone::Library);
    let milled = put(&mut game, player, "Mountain", Zone::Library);
    game.state.players[player.index()].mana = [0, 0, 1, 0, 0, 2];
    cast(&mut game, spell, None);
    pass_pair(&mut game);
    let look = game.observe(player).library_choice.unwrap();
    assert!(look.surveil);
    assert_eq!(look.remaining.len(), 2);
    assert!(game.observe(player.other()).library_choice.is_none());
    assert_eq!(game.state.players[player.index()].life, 20);
    arrange(&mut game, milled, LibraryDestination::Graveyard);
    arrange(&mut game, kept, LibraryDestination::Top);
    settle(&mut game);
    assert_eq!(game.object_view(milled).zone, Zone::Graveyard);
    for id in [kept, next] {
        assert_eq!(game.object_view(id).zone, Zone::Hand);
    }
    assert_eq!(game.state.players[player.index()].life, 18);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
}

#[test]
fn robotics_mastery_flashes_in_but_needs_its_creature_to_enter_and_make_robots() {
    let player = Player::P0;
    let mut game = scenario(player.other());
    let recipient = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let aura = put(&mut game, player, "Robotics Mastery", Zone::Hand);
    act(&mut game, Action::Pass);
    cast(&mut game, aura, Some(Target::Card(recipient)));
    let mut invalid = game.clone();
    if invalid.actor() != Some(player) {
        act(&mut invalid, Action::Pass);
    }
    let bounce = put(&mut invalid, player, "Unsummon", Zone::Hand);
    cast(&mut invalid, bounce, Some(Target::Card(recipient)));
    settle(&mut invalid);
    assert_eq!(invalid.object_view(aura).zone, Zone::Graveyard);
    assert!(
        !invalid
            .state
            .objects
            .iter()
            .any(|o| o.zone == Zone::Battlefield && o.is_token())
    );

    settle(&mut game);
    assert_eq!(
        game.object_view(aura).attached_to,
        Some(game.object_ref(recipient))
    );
    assert_eq!(stats(&game, recipient), (4, 4));
    let robots: Vec<_> = game
        .state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| o.zone == Zone::Battlefield && o.is_token())
        .map(|(id, _)| game.object_view(id as CardId))
        .collect();
    assert_eq!(robots.len(), 2);
    for robot in robots {
        assert_eq!((robot.kinds, robot.colors), (17, 0));
        assert_eq!((robot.power, robot.toughness), (1, 1));
        assert!(robot.keywords.contains(Keywords::FLYING) && robot.has_subtype("Robot"));
    }
}

#[test]
fn romantic_rendezvous_draws_two_after_the_mandatory_discard_even_with_no_card_to_discard() {
    let player = Player::P0;
    for has_card in [false, true] {
        let mut game = scenario(player);
        let spell = put(&mut game, player, "Romantic Rendezvous", Zone::Hand);
        let discard = has_card.then(|| put(&mut game, player, "Island", Zone::Hand));
        cast(&mut game, spell, None);
        pass_pair(&mut game);
        if let Some(card) = discard {
            assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);
            assert!(
                game.observe(player.other())
                    .objects
                    .iter()
                    .all(|o| o.reference.card != card)
            );
            act(&mut game, Action::Discard { card });
            assert_eq!(game.object_view(card).zone, Zone::Graveyard);
        }
        settle(&mut game);
        assert_eq!(game.hand_count(player), 2);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
    }
}

#[test]
fn peni_counters_a_creature_and_draws_once_per_modified_combat_damage_dealer() {
    let player = Player::P0;
    let mut game = scenario(player);
    let peni = put(&mut game, player, "SP//dr, Piloted by Peni", Zone::Hand);
    let first = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let plain = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    game.add_counter(second, CounterKind::Stun, 1);
    cast(&mut game, peni, None);
    pass_pair(&mut game);
    target(&mut game, Target::Card(first));
    settle(&mut game);
    assert_eq!(game.counter_count(first, CounterKind::PlusOne), 1);
    assert_eq!(game.object_view(peni).kinds, 17);
    assert!(
        game.object_view(peni)
            .keywords
            .contains(Keywords::VIGILANCE)
    );
    declare(&mut game, &[first, second, plain]);
    pass_pair(&mut game);
    act(&mut game, Action::FinishBlockers);
    pass_pair(&mut game);
    for _ in 0..2 {
        if matches!(game.state.pending, Pending::OrderTriggers { .. }) {
            choose(&mut game, |a| matches!(a, Action::PutTrigger { .. }));
        }
    }
    settle(&mut game);
    assert_eq!(game.state.players[player.other().index()].life, 13);
    assert_eq!(game.hand_count(player), 2);
}

#[test]
fn sandman_returns_tapped_with_his_land_and_rejects_a_power_two_blocker() {
    let player = Player::P0;
    let mut game = scenario(player);
    for _ in 0..2 {
        put(&mut game, player, "Forest", Zone::Battlefield);
    }
    let sandman = put(
        &mut game,
        player,
        "Sandman, Shifting Scoundrel",
        Zone::Graveyard,
    );
    let land = put(&mut game, player, "Forest", Zone::Graveyard);
    assert_eq!(stats(&game, sandman), (2, 2));
    let small = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let large = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 2, 3];
    activate(&mut game, sandman, 0);
    target(&mut game, Target::Card(land));
    pay(&mut game);

    let mut invalid = game.clone();
    invalid.move_object(land, Zone::Exile);
    settle(&mut invalid);
    assert_eq!(invalid.object_view(sandman).zone, Zone::Graveyard);

    settle(&mut game);
    for id in [sandman, land] {
        assert_eq!(game.object_view(id).zone, Zone::Battlefield);
        assert!(game.object_view(id).tapped);
    }
    assert_eq!(stats(&game, sandman), (3, 3));
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    game.state.objects[sandman as usize].tapped = false;
    game.state.objects[sandman as usize].summoning_sick = false;
    declare(&mut game, &[sandman]);
    pass_pair(&mut game);
    reject(
        &mut game,
        Action::Block {
            blocker: small,
            attacker: sandman,
        },
    );
    act(
        &mut game,
        Action::Block {
            blocker: large,
            attacker: sandman,
        },
    );
}

#[test]
fn the_hideouts_enter_tapped_produce_each_color_and_surveil_for_four() {
    let player = Player::P0;
    for (name, colors) in [("Savage Mansion", [3, 4]), ("Sinister Hideout", [1, 2])] {
        let mut game = scenario(player);
        let land = put(&mut game, player, name, Zone::Hand);
        act(&mut game, Action::PlayLand { card: land });
        assert!(game.object_view(land).tapped);
        assert!(!game.legal_actions().iter().any(|a| matches!(
            a, Action::ActivateMana { card, .. } if *card == land
        )));
        game.state.objects[land as usize].tapped = false;
        for color in colors {
            let mut branch = game.clone();
            branch.state.players[player.index()].mana = [0; 6];
            let mut production = [0; 6];
            production[color] = 1;
            choose(&mut branch, |a| {
                matches!(
                    a, Action::ActivateMana { card, production: produced, .. }
                        if *card == land && *produced == production
                )
            });
            assert_eq!(branch.state.players[player.index()].mana, production);
            assert!(branch.object_view(land).tapped);
        }
        let top = put(&mut game, player, "Opt", Zone::Library);
        game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 4];
        activate(&mut game, land, 1);
        pay(&mut game);
        pass_pair(&mut game);
        assert!(game.observe(player).library_choice.unwrap().surveil);
        assert!(game.observe(player.other()).library_choice.is_none());
        arrange(&mut game, top, LibraryDestination::Graveyard);
        assert_eq!(game.object_view(top).zone, Zone::Graveyard);
        assert!(game.object_view(land).tapped);
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    }
}

#[test]
fn school_daze_draws_three_or_counters_and_draws_even_if_the_target_is_uncounterable() {
    let player = Player::P0;
    for mode in [0, 1] {
        for uncounterable in [false, true] {
            if mode == 0 && uncounterable {
                continue;
            }
            let mut game = scenario(player);
            let daze = put(&mut game, player, "School Daze", Zone::Hand);
            let victim = (mode == 1).then(|| {
                let shock = put(&mut game, player, "Shock", Zone::Hand);
                cast(&mut game, shock, Some(Target::Player(player.other())));
                Target::Spell(shock)
            });
            if uncounterable {
                put(&mut game, player.other(), "Spider-Punk", Zone::Battlefield);
            }
            cast_mode(&mut game, daze, mode, victim);
            settle(&mut game);
            assert_eq!(game.hand_count(player), if mode == 0 { 3 } else { 1 });
            assert_eq!(
                game.state.players[player.other().index()].life,
                if uncounterable { 18 } else { 20 }
            );
        }
    }
}

#[test]
fn scorpions_sting_reduces_toughness_without_damage_and_expires_in_cleanup() {
    let player = Player::P0;
    for (name, survives) in [("Hill Giant", false), ("Spider-Rex, Daring Dino", true)] {
        let mut game = scenario(player);
        let creature = put(&mut game, player, name, Zone::Battlefield);
        game.state.objects[creature as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
        let sting = put(&mut game, player, "Scorpion's Sting", Zone::Hand);
        cast(&mut game, sting, Some(Target::Card(creature)));
        settle(&mut game);
        if survives {
            assert_eq!(stats(&game, creature), (3, 3));
            assert_eq!(game.object_view(creature).damage, 0);
            cleanup(&mut game);
            assert_eq!(stats(&game, creature), (6, 6));
        } else {
            assert_eq!(game.object_view(creature).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn scorpion_connives_after_a_death_on_its_controllers_end_step() {
    let player = Player::P0;
    for died in [false, true] {
        let mut game = scenario(player);
        let scorpion = put(&mut game, player, "Scorpion, Seething Striker", Zone::Hand);
        let bearer = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let enemy = put(
            &mut game,
            player.other(),
            "Grizzly Bears",
            Zone::Battlefield,
        );
        let discarded = put(&mut game, player, "Opt", Zone::Hand);
        cast(&mut game, scorpion, None);
        settle(&mut game);
        assert!(
            game.object_view(scorpion)
                .keywords
                .contains(Keywords::DEATHTOUCH)
        );
        let removal = put(
            &mut game,
            player,
            if died { "Shock" } else { "Unsummon" },
            Zone::Hand,
        );
        cast(&mut game, removal, Some(Target::Card(enemy)));
        settle(&mut game);
        start_step(&mut game, Step::End);
        if died {
            target(&mut game, Target::Card(bearer));
            pass_pair(&mut game);
            act(&mut game, Action::Discard { card: discarded });
        }
        settle(&mut game);
        assert_eq!(
            game.counter_count(bearer, CounterKind::PlusOne),
            u64::from(died)
        );
        assert_eq!(
            game.object_view(discarded).zone,
            if died { Zone::Graveyard } else { Zone::Hand }
        );
        assert_eq!(
            game.state.players[player.index()].cards_drawn_this_turn,
            u64::from(died)
        );
    }
}

#[test]
fn scout_the_city_recovers_only_a_milled_permanent_or_destroys_a_flier() {
    let player = Player::P0;
    let mut game = scenario(player);
    let scout = put(&mut game, player, "Scout the City", Zone::Hand);
    let desired = put(&mut game, player, "Island", Zone::Library);
    let other = put(&mut game, player, "Forest", Zone::Library);
    let instant = put(&mut game, player, "Opt", Zone::Library);
    let old = put(&mut game, player, "Plains", Zone::Graveyard);
    let flying = put(
        &mut game,
        player.other(),
        "Healer's Hawk",
        Zone::Battlefield,
    );
    let ground = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );

    let mut destroy = game.clone();
    act(&mut destroy, Action::AnnounceCast { card: scout });
    act(&mut destroy, Action::ChooseMode { mode: 1 });
    reject(
        &mut destroy,
        Action::ChooseTarget {
            target: Target::Card(ground),
        },
    );
    target(&mut destroy, Target::Card(flying));
    pay(&mut destroy);
    settle(&mut destroy);
    assert_eq!(destroy.object_view(flying).zone, Zone::Graveyard);

    cast_mode(&mut game, scout, 0, None);
    pass_pair(&mut game);
    for card in [old, instant] {
        let object = Some(game.object_ref(card));
        reject(&mut game, Action::RecoverMilled { object });
    }
    let object = Some(game.object_ref(desired));
    act(&mut game, Action::RecoverMilled { object });
    assert_eq!(game.object_view(desired).zone, Zone::Hand);
    assert_eq!(game.object_view(other).zone, Zone::Graveyard);
    assert_eq!(game.object_view(instant).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].life, 23);
}

#[test]
fn secret_identity_preserves_card_types_and_counters_in_both_temporary_forms() {
    let player = Player::P0;
    for (mode, subtype, keywords, form) in [
        (0, "Citizen", Keywords::HEXPROOF, (2, 2)),
        (
            1,
            "Hero",
            Keywords::FLYING.union(Keywords::VIGILANCE),
            (4, 5),
        ),
    ] {
        let mut game = scenario(player);
        let recipient = put(&mut game, player, "Oasis Gardener", Zone::Battlefield);
        let original = game.object_view(recipient);
        game.add_counter(recipient, CounterKind::PlusOne, 1);
        let spell = put(&mut game, player, "Secret Identity", Zone::Hand);
        cast_mode(&mut game, spell, mode, Some(Target::Card(recipient)));
        settle(&mut game);
        let current = game.object_view(recipient);
        assert_eq!(
            (current.kinds, current.colors),
            (original.kinds, original.colors)
        );
        assert_eq!((current.power, current.toughness), form);
        assert!(current.has_subtype(subtype));
        assert!(!current.has_subtype("Scarecrow"));
        assert!(current.keywords.contains(keywords));
        assert!(game.legal_actions().iter().any(|a| matches!(
            a, Action::ActivateMana { card, .. } if *card == recipient
        )));
        cleanup(&mut game);
        assert_eq!(stats(&game, recipient), (3, 3));
        assert!(game.object_view(recipient).has_subtype("Scarecrow"));
        assert!(!game.object_view(recipient).keywords.contains(keywords));
    }
}

#[test]
fn shock_pays_one_red_and_deals_two_damage_to_a_planeswalker() {
    let player = Player::P0;
    let mut game = scenario(player);
    let jace = put(
        &mut game,
        player.other(),
        "Jace Reawakened",
        Zone::Battlefield,
    );
    game.add_counter(jace, CounterKind::Loyalty, 3);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];
    reject(&mut game, Action::AnnounceCast { card: shock });
    game.state.players[player.index()].mana = [0, 0, 0, 1, 0, 0];
    cast(&mut game, shock, Some(Target::Card(jace)));
    settle(&mut game);
    assert_eq!(game.counter_count(jace, CounterKind::Loyalty), 1);
    assert_eq!(game.state.players[player.other().index()].life, 20);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
}

#[test]
fn skyward_spiders_hybrid_cost_and_conditional_flying_respect_actual_modification() {
    let player = Player::P0;
    for payment in [[2, 0, 0, 0, 0, 0], [1, 1, 0, 0, 0, 0], [0, 2, 0, 0, 0, 0]] {
        let mut game = scenario(player);
        let spider = put(&mut game, player, "Skyward Spider", Zone::Hand);
        game.state.players[player.index()].mana = payment;
        act(&mut game, Action::AnnounceCast { card: spider });
        choose(&mut game, |a| matches!(a, Action::ChooseSpellCost { .. }));
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
        let aura = put(&mut game, player.other(), "Pacifism", Zone::Battlefield);
        game.state.objects[aura as usize].attached_to = Some(game.object_ref(spider));
        assert!(!game.object_view(spider).keywords.contains(Keywords::FLYING));
        game.change_controller(game.object_ref(aura), player);
        assert!(game.object_view(spider).keywords.contains(Keywords::FLYING));
        game.move_object(aura, Zone::Graveyard);
        assert!(!game.object_view(spider).keywords.contains(Keywords::FLYING));
    }
}

#[test]
fn skyward_spider_and_spider_rex_counter_an_opponents_target_unless_they_pay_two() {
    let opponent = Player::P1;
    for name in ["Skyward Spider", "Spider-Rex, Daring Dino"] {
        for pay_ward in [false, true] {
            let mut game = scenario(opponent);
            let recipient = put(&mut game, opponent.other(), name, Zone::Battlefield);
            let bounce = put(&mut game, opponent, "Unsummon", Zone::Hand);
            game.state.players[opponent.index()].mana = [0, 1, 0, 0, 0, 2];
            cast(&mut game, bounce, Some(Target::Card(recipient)));
            assert_eq!(game.state.stack.len(), 2);
            pass_pair(&mut game);
            if pay_ward {
                act(
                    &mut game,
                    Action::PayResolutionMana {
                        payment: [0, 0, 0, 0, 0, 2],
                    },
                );
            } else {
                act(&mut game, Action::DeclineResolutionMana);
            }
            settle(&mut game);
            assert_eq!(
                game.object_view(recipient).zone,
                if pay_ward {
                    Zone::Hand
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(
                game.state.players[opponent.index()].mana[5],
                if pay_ward { 0 } else { 2 }
            );
        }
    }
}

#[test]
fn spectacular_tactics_keeps_the_counter_but_rechecks_power_for_the_destroy_mode() {
    let player = Player::P0;
    let mut game = scenario(player);
    let bearer = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let spell = put(&mut game, player, "Spectacular Tactics", Zone::Hand);
    cast_mode(&mut game, spell, 0, Some(Target::Card(bearer)));
    settle(&mut game);
    assert_eq!(game.counter_count(bearer, CounterKind::PlusOne), 1);
    assert!(
        game.object_view(bearer)
            .keywords
            .contains(Keywords::HEXPROOF)
    );

    let enemy = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    game.add_counter(enemy, CounterKind::PlusOne, 1);
    let spell = put(&mut game, player, "Spectacular Tactics", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    act(&mut game, Action::ChooseMode { mode: 1 });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(bearer),
        },
    );
    target(&mut game, Target::Card(enemy));
    pay(&mut game);
    let mut shrunk = game.clone();
    let sting = put(&mut shrunk, player, "Scorpion's Sting", Zone::Hand);
    cast(&mut shrunk, sting, Some(Target::Card(enemy)));
    settle(&mut shrunk);
    assert_eq!(stats(&shrunk, enemy), (1, 1));
    assert_eq!(shrunk.object_view(enemy).zone, Zone::Battlefield);

    settle(&mut game);
    assert_eq!(game.object_view(enemy).zone, Zone::Graveyard);
    cleanup(&mut game);
    assert_eq!(game.counter_count(bearer, CounterKind::PlusOne), 1);
    assert!(
        !game
            .object_view(bearer)
            .keywords
            .contains(Keywords::HEXPROOF)
    );
}

#[test]
fn spider_manifestation_adds_both_printed_colors_and_untaps_only_for_the_cast_threshold() {
    let player = Player::P0;
    for (mana, name, qualifies) in [
        ([0, 0, 0, 1, 0, 0], "Grizzly Bears", false),
        ([0, 0, 0, 0, 1, 0], "Hill Giant", true),
    ] {
        let mut game = scenario(player);
        let source = put(&mut game, player, "Spider Manifestation", Zone::Battlefield);
        assert!(game.object_view(source).keywords.contains(Keywords::REACH));
        let prior = game.state.players[player.index()].mana;
        choose(&mut game, |a| {
            matches!(
                a, Action::ActivateMana { card, production, .. } if *card == source && *production == mana
            )
        });
        assert!(game.object_view(source).tapped);
        assert_eq!(
            game.state.players[player.index()].mana,
            std::array::from_fn(|index| prior[index] + mana[index])
        );
        let spell = put(&mut game, player, name, Zone::Hand);
        cast(&mut game, spell, None);
        settle(&mut game);
        assert_eq!(game.object_view(source).tapped, !qualifies);
    }
}

#[test]
fn spider_byte_can_decline_or_return_a_nonland_permanent_to_its_owner() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spider = put(&mut game, player, "Spider-Byte, Web Warden", Zone::Hand);
    let borrowed = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    game.change_controller(game.object_ref(borrowed), player);
    let land = put(&mut game, player.other(), "Forest", Zone::Battlefield);
    cast(&mut game, spider, None);
    pass_pair(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(land),
        },
    );
    let mut decline = game.clone();
    act(&mut decline, Action::FinishTargets);
    settle(&mut decline);
    assert_eq!(decline.object_view(borrowed).zone, Zone::Battlefield);
    target(&mut game, Target::Card(borrowed));
    settle(&mut game);
    assert_eq!(game.object_view(borrowed).zone, Zone::Hand);
    assert_eq!(game.hand_count(player.other()), 1);
    assert_eq!(game.hand_count(player), 0);
}

#[test]
fn spider_gwens_tap_trigger_draws_only_if_the_player_pays_its_private_discard() {
    let player = Player::P0;
    let mut game = scenario(player);
    let gwen = put(
        &mut game,
        player,
        "Spider-Gwen, Free Spirit",
        Zone::Battlefield,
    );
    let discard = put(&mut game, player, "Mountain", Zone::Hand);
    assert!(game.object_view(gwen).keywords.contains(Keywords::REACH));
    declare(&mut game, &[gwen]);
    pass_pair(&mut game);
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineEffectPayment);
    assert_eq!(declined.object_view(discard).zone, Zone::Hand);
    assert_eq!(
        declined.state.players[player.index()].cards_drawn_this_turn,
        0
    );

    assert!(
        game.observe(player.other())
            .objects
            .iter()
            .all(|o| o.reference.card != discard)
    );
    act(&mut game, Action::Discard { card: discard });
    assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    assert_eq!(game.hand_count(player), 1);
}

#[test]
fn spider_rex_can_block_a_flier_and_tramples_over_a_two_toughness_blocker() {
    let player = Player::P0;
    let mut defender = scenario(player.other());
    let rex = put(
        &mut defender,
        player,
        "Spider-Rex, Daring Dino",
        Zone::Battlefield,
    );
    let hawk = put(
        &mut defender,
        player.other(),
        "Healer's Hawk",
        Zone::Battlefield,
    );
    declare(&mut defender, &[hawk]);
    pass_pair(&mut defender);
    act(
        &mut defender,
        Action::Block {
            blocker: rex,
            attacker: hawk,
        },
    );

    let mut game = scenario(player);
    let rex = put(
        &mut game,
        player,
        "Spider-Rex, Daring Dino",
        Zone::Battlefield,
    );
    let blocker = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    declare(&mut game, &[rex]);
    pass_pair(&mut game);
    act(
        &mut game,
        Action::Block {
            blocker,
            attacker: rex,
        },
    );
    act(&mut game, Action::FinishBlockers);
    pass_pair(&mut game);
    for (target, amount) in [
        (Target::Card(blocker), 2),
        (Target::Player(player.other()), 4),
    ] {
        let action = Action::AssignDamage { target, amount };
        if game.legal_actions().contains(&action) {
            act(&mut game, action);
        }
    }
    settle(&mut game);
    assert_eq!(game.state.players[player.other().index()].life, 16);
    assert_eq!(game.object_view(blocker).zone, Zone::Graveyard);
}
