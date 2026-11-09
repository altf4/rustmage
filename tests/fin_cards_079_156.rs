//! Typed Final Fantasy card outcomes, rows 79 through 156.
mod support;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn announce_ability(game: &mut Game, card: CardId, ability: u16) {
    act(
        game,
        Action::AnnounceActivate {
            card,
            ability,
            counter_payment: Counters::default(),
        },
    );
}

fn resolve_entries(game: &mut Game) {
    for _ in 0..32 {
        if game.state.stack.is_empty()
            && game.state.resolving.is_none()
            && game.state.pending == Pending::Priority
        {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::ChooseEntryTimestamp { .. }
                    | Action::ChooseCreatedToken { .. }
            )
        });
    }
    panic!("entry triggers exceeded the scenario budget");
}

fn declare(game: &mut Game, attackers: &[CardId]) {
    game.begin_step(Step::BeginCombat);
    pass_pair(game);
    for &card in attackers {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

#[test]
fn firion_copies_equipment_once_and_sacrifices_its_exact_token_next_upkeep() {
    let p = Player::P0;
    let mut game = scenario(p);
    let firion = put(&mut game, p, "Firion, Wild Rose Warrior", Zone::Battlefield);
    let bearer = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    game.state.objects[bearer as usize].summoning_sick = true;
    assert!(!game.keywords(bearer).contains(Keywords::HASTE));
    let original = put(&mut game, p, "Monk's Fist", Zone::Hand);
    cast(&mut game, original, None);
    resolve_entries(&mut game);
    let observed = game.observe(p.other());
    let [delayed] = observed.delayed_sacrifices.as_slice() else {
        panic!("only the one nontoken Equipment entry may trigger");
    };
    let [copied] = delayed.tokens.as_slice() else {
        panic!("the pending sacrifice records one exact token");
    };
    assert!(delayed.next_upkeep);
    assert_eq!(
        game.object_view(copied.card).card,
        game.object_view(original).card
    );
    assert!(game.object_view(copied.card).is_token());
    assert_ne!(
        game.object_view(original).attached_to,
        game.object_view(copied.card).attached_to
    );
    game.state.players[p.index()].mana = [0; 6];
    choose(&mut game, |action| {
        matches!(action,
        Action::Activate { card, target: Some(Target::Card(id)), payment, .. }
            if *card == copied.card && *id == bearer && *payment == [0; 6])
    });
    settle(&mut game);
    assert_eq!(
        game.object_view(copied.card).attached_to,
        Some(game.object_ref(bearer))
    );
    assert!(game.keywords(bearer).contains(Keywords::HASTE));
    game.move_object(firion, Zone::Graveyard);
    assert!(!game.keywords(bearer).contains(Keywords::HASTE));
    game.state.step = Step::Main2;
    pass_pair(&mut game);
    assert_eq!(game.state.step, Step::End);
    assert_eq!(game.object_view(copied.card).zone, Zone::Battlefield);
    pass_pair(&mut game);
    assert_eq!(game.state.active, p.other());
    assert_eq!(game.state.step, Step::Upkeep);
    settle(&mut game);
    assert_ne!(game.object_view(copied.card).zone, Zone::Battlefield);
    assert_eq!(game.object_view(original).zone, Zone::Battlefield);
}

#[test]
fn freya_flies_on_her_controllers_turn_and_pays_equipment_or_equip_only() {
    let p = Player::P1;
    let mut game = scenario(p);
    let freya = put(&mut game, p, "Freya Crescent", Zone::Battlefield);
    let equip = put(&mut game, p, "Swiftfoot Boots", Zone::Battlefield);
    let equipment = put(&mut game, p, "Lavaspur Boots", Zone::Hand);
    let artifact = put(&mut game, p, "Candy Trail", Zone::Hand);
    let bolt = put(&mut game, p, "Lightning Bolt", Zone::Hand);
    assert!(game.keywords(freya).contains(Keywords::FLYING));
    let mut other_turn = game.clone();
    other_turn.state.active = p.other();
    assert!(!other_turn.keywords(freya).contains(Keywords::FLYING));
    game.state.players[p.index()].mana = [0; 6];
    let red = [0, 0, 0, 1, 0, 0];
    let tap = Action::ActivateMana {
        card: freya,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production: red,
    };
    game.state.objects[freya as usize].summoning_sick = true;
    reject(&mut game, tap.clone());
    game.state.objects[freya as usize].summoning_sick = false;
    act(&mut game, tap);
    assert_eq!(game.state.players[p.index()].mana, red);
    for card in [artifact, bolt] {
        reject(&mut game, Action::AnnounceCast { card });
    }
    let mut paid_equip = game.clone();
    announce_ability(&mut paid_equip, equip, 0);
    act(
        &mut paid_equip,
        Action::ChooseTarget {
            target: Target::Card(freya),
        },
    );
    pay(&mut paid_equip);
    settle(&mut paid_equip);
    assert_eq!(
        paid_equip.object_view(equip).attached_to,
        Some(paid_equip.object_ref(freya))
    );
    cast(&mut game, equipment, None);
    settle(&mut game);
    assert_eq!(game.object_view(equipment).zone, Zone::Battlefield);
    assert_eq!(game.state.players[p.index()].mana, [0; 6]);
}

#[test]
fn graha_draws_once_from_the_first_controlled_creature_or_artifact_batch() {
    let p = Player::P1;
    let mut game = scenario(p);
    let source = put(&mut game, p, "G'raha Tia", Zone::Battlefield);
    let creature = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let artifact = put(&mut game, p, "Gold Pan", Zone::Battlefield);
    let later = put(&mut game, p, "Llanowar Elves", Zone::Battlefield);
    let enemy = put(&mut game, p.other(), "Bear Cub", Zone::Battlefield);
    assert!(game.keywords(source).contains(Keywords::REACH));
    game.sacrifice_batch(&[(p.other(), enemy)]);
    assert!(game.state.pending_triggers.is_empty());
    game.sacrifice_batch(&[(p, creature), (p, artifact)]);
    game.prepare_triggers();
    assert_eq!(game.state.stack.len(), 1);
    settle(&mut game);
    game.sacrifice_batch(&[(p, later)]);
    game.prepare_triggers();
    assert!(game.state.stack.is_empty());
    assert_eq!(game.hand_count(p), 1);
}

#[test]
fn giott_can_decline_its_entry_and_discard_to_draw_for_an_equipment() {
    let p = Player::P0;
    let mut game = scenario(p);
    let giott = put(&mut game, p, "Giott, King of the Dwarves", Zone::Hand);
    let discard = put(&mut game, p, "Forest", Zone::Hand);
    cast(&mut game, giott, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert!(game.keywords(giott).contains(Keywords::DOUBLE_STRIKE));
    act(&mut game, Action::DeclineEffectPayment);
    assert_eq!(game.object_view(discard).zone, Zone::Hand);
    let equipment = put(&mut game, p, "Swiftfoot Boots", Zone::Hand);
    cast(&mut game, equipment, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    act(&mut game, Action::Discard { card: discard });
    settle(&mut game);
    assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
    assert_eq!(game.hand_count(p), 1);
}

#[test]
fn ochu_locks_the_resolution_count_of_only_own_permanent_cards() {
    let p = Player::P0;
    let mut game = scenario(p);
    let ochu = put(&mut game, p, "Gran Pulse Ochu", Zone::Battlefield);
    let removed = put(&mut game, p, "Forest", Zone::Graveyard);
    put(&mut game, p, "Bear Cub", Zone::Graveyard);
    put(&mut game, p, "Gold Pan", Zone::Graveyard);
    put(&mut game, p, "Opt", Zone::Graveyard);
    put(&mut game, p.other(), "Bear Cub", Zone::Graveyard);
    assert!(game.keywords(ochu).contains(Keywords::DEATHTOUCH));
    announce_ability(&mut game, ochu, 0);
    pay(&mut game);
    game.move_object(removed, Zone::Exile);
    settle(&mut game);
    assert_eq!(stats(&game, ochu), (3, 3));
    put(&mut game, p, "Forest", Zone::Graveyard);
    assert_eq!(stats(&game, ochu), (3, 3));
}

#[test]
fn haste_magic_buffs_a_legal_target_and_grants_paid_play_through_the_next_own_end_step() {
    let p = Player::P0;
    let mut game = scenario(p.other());
    let target = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let spell = put(&mut game, p, "Haste Magic", Zone::Hand);
    let top = game.top_library_card(p).unwrap();
    act(&mut game, Action::Pass);
    cast(&mut game, spell, Some(Target::Card(target)));
    let mut fizzled = game.clone();
    fizzled.move_object(target, Zone::Exile);
    settle(&mut fizzled);
    assert_eq!(fizzled.object_view(top).zone, Zone::Library);
    settle(&mut game);
    assert_eq!(stats(&game, target), (5, 3));
    assert!(game.keywords(target).contains(Keywords::HASTE));
    assert_eq!(game.object_view(top).zone, Zone::Exile);
    game.begin_step(Step::End);
    assert!(
        game.observe(p)
            .exile_permissions
            .iter()
            .any(|row| row.object.card == top)
    );
    game.state.active = p;
    game.begin_step(Step::End);
    assert!(
        game.observe(p)
            .exile_permissions
            .iter()
            .all(|row| row.object.card != top)
    );
}

#[test]
fn hope_uses_life_gained_at_resolution_even_after_life_loss() {
    let p = Player::P1;
    let mut game = scenario(p);
    let hope = put(&mut game, p, "Hope Estheim", Zone::Battlefield);
    assert!(game.keywords(hope).contains(Keywords::LIFELINK));
    let count = game.state.players[p.other().index()].library.len();
    game.gain_life(p, 2);
    game.lose_life(p, 3);
    game.begin_step(Step::End);
    game.prepare_triggers();
    game.gain_life(p, 1);
    settle(&mut game);
    assert_eq!(game.state.players[p.index()].life, 20);
    assert_eq!(
        game.state.players[p.other().index()].library.len(),
        count - 3
    );
}

#[test]
fn shopkeep_targets_only_attacking_creatures_with_a_live_equipment_link() {
    let p = Player::P1;
    let mut game = scenario(p);
    put(&mut game, p, "Item Shopkeep", Zone::Battlefield);
    let equipped = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let plain = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let equipment = put(&mut game, p.other(), "Short Bow", Zone::Battlefield);
    assert!(game.attach_equipment(game.object_ref(equipment), game.object_ref(equipped)));
    declare(&mut game, &[equipped, plain]);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(plain),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(equipped),
        },
    );
    let mut detached = game.clone();
    detached.move_object(equipment, Zone::Exile);
    settle(&mut detached);
    assert!(!detached.keywords(equipped).contains(Keywords::MENACE));
    settle(&mut game);
    assert!(game.keywords(equipped).contains(Keywords::MENACE));
}

#[test]
fn jumbo_cactuar_gains_exactly_9999_power_on_declaration() {
    let p = Player::P0;
    let mut game = scenario(p);
    let jumbo = put(&mut game, p, "Jumbo Cactuar", Zone::Battlefield);
    assert_eq!(stats(&game, jumbo), (1, 7));
    declare(&mut game, &[jumbo]);
    settle(&mut game);
    assert_eq!(stats(&game, jumbo), (10000, 7));
}

fn unblocked_damage(game: &mut Game, attackers: &[CardId]) {
    declare(game, attackers);
    assert!(game.state.stack.is_empty());
    pass_pair(game);
    act(game, Action::FinishBlockers);
    pass_pair(game);
}

#[test]
fn garnet_removes_one_lore_from_each_chosen_controlled_saga() {
    let p = Player::P0;
    let mut game = scenario(p);
    let garnet = put(
        &mut game,
        p,
        "Garnet, Princess of Alexandria",
        Zone::Battlefield,
    );
    let name = "Awaken the Honored Dead";
    let own = put(&mut game, p, name, Zone::Battlefield);
    let rival = put(&mut game, p.other(), name, Zone::Battlefield);
    for id in [own, rival] {
        game.set_counters(id, Counters::of(CounterKind::Lore, 2));
    }
    assert!(game.keywords(garnet).contains(Keywords::LIFELINK));
    declare(&mut game, &[garnet]);
    pass_pair(&mut game);
    let choice = |game: &Game, card| Action::ChooseEffectObject {
        object: game.object_ref(card),
    };
    let invalid = choice(&game, rival);
    reject(&mut game, invalid);
    let valid = choice(&game, own);
    act(&mut game, valid.clone());
    reject(&mut game, valid);
    act(&mut game, Action::FinishEffectChoice);
    assert_eq!(game.counter_count(own, CounterKind::Lore), 1);
    assert_eq!(game.counter_count(rival, CounterKind::Lore), 2);
    assert_eq!(game.counter_count(garnet, CounterKind::PlusOne), 1);
}

#[test]
fn genji_untaps_its_original_attacker_and_adds_one_combat() {
    let p = Player::P0;
    let mut game = scenario(p);
    let glove = put(&mut game, p, "Genji Glove", Zone::Battlefield);
    let attacker = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let alternate = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    game.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
    announce_ability(&mut game, glove, 0);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(attacker),
        },
    );
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 0, 3],
        },
    );
    settle(&mut game);
    assert!(game.keywords(attacker).contains(Keywords::DOUBLE_STRIKE));
    declare(&mut game, &[attacker]);
    assert!(game.object_view(attacker).tapped);
    assert!(game.attach_equipment(game.object_ref(glove), game.object_ref(alternate)));
    game.state.objects[alternate as usize].tapped = true;
    settle(&mut game);
    assert!(!game.object_view(attacker).tapped);
    assert!(game.object_view(alternate).tapped);
    assert!(game.keywords(alternate).contains(Keywords::DOUBLE_STRIKE));
    assert_eq!(game.observe(p).additional_combats, 1);
}

#[test]
fn judgment_bolt_counts_controlled_equipment_at_resolution() {
    let p = Player::P1;
    let mut game = scenario(p);
    let target = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    game.state.objects[target as usize].growth.toughness = 3;
    put(&mut game, p, "Short Bow", Zone::Battlefield);
    put(&mut game, p, "Swiftfoot Boots", Zone::Battlefield);
    put(&mut game, p.other(), "Short Bow", Zone::Battlefield);
    let removed = put(&mut game, p, "Gold Pan", Zone::Battlefield);
    let spell = put(&mut game, p, "Judgment Bolt", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(target)));
    game.move_object(removed, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.object_view(target).damage, 5);
    assert_eq!(game.state.players[p.other().index()].life, 18);
}

#[test]
fn kain_gives_the_exact_permanent_away_before_the_damage_sized_rewards() {
    let p = Player::P0;
    let mut game = scenario(p);
    let kain = put(&mut game, p, "Kain, Traitorous Dragoon", Zone::Battlefield);
    assert!(game.keywords(kain).contains(Keywords::FLYING));
    unblocked_damage(&mut game, &[kain]);
    let mut gone = game.clone();
    gone.move_object(kain, Zone::Exile);
    settle(&mut gone);
    assert_eq!(gone.state.players[p.index()].life, 20);
    assert_eq!(gone.hand_count(p), 0);

    settle(&mut game);
    assert_eq!(game.object_view(kain).controller, p.other());
    assert!(!game.keywords(kain).contains(Keywords::FLYING));
    assert_eq!(game.state.players[p.index()].life, 18);
    assert_eq!(game.hand_count(p), 2);
    let treasures: Vec<_> = game
        .observe(p)
        .objects
        .into_iter()
        .filter(|view| {
            view.controller == p && view.zone == Zone::Battlefield && view.has_subtype("Treasure")
        })
        .collect();
    assert_eq!(treasures.len(), 2);
    assert!(treasures.iter().all(|view| view.tapped && view.is_token()));
}

#[test]
fn light_of_judgment_destroys_one_attached_equipment_before_lethal_sbas() {
    let p = Player::P1;
    let mut game = scenario(p);
    let target = put(&mut game, p.other(), "Grizzly Bears", Zone::Battlefield);
    let attached = put(&mut game, p.other(), "Short Bow", Zone::Battlefield);
    let loose = put(&mut game, p, "Gold Pan", Zone::Battlefield);
    let attached_ref = game.object_ref(attached);
    assert!(game.attach_equipment(attached_ref, game.object_ref(target)));
    let spell = put(&mut game, p, "Light of Judgment", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(target)));
    pass_pair(&mut game);
    assert_eq!(game.object_view(target).damage, 6);
    assert_eq!(game.object_view(target).zone, Zone::Battlefield);
    let invalid = Action::ChooseEffectObject {
        object: game.object_ref(loose),
    };
    reject(&mut game, invalid);
    act(
        &mut game,
        Action::ChooseEffectObject {
            object: attached_ref,
        },
    );
    assert_eq!(game.object_view(target).zone, Zone::Graveyard);
    assert_eq!(game.object_view(attached).zone, Zone::Graveyard);
    assert_eq!(game.object_view(loose).zone, Zone::Battlefield);
}

#[test]
fn lightning_stagger_doubles_the_following_normal_strike_after_first_strike() {
    let p = Player::P0;
    let mut game = scenario(p);
    let lightning = put(&mut game, p, "Lightning, Army of One", Zone::Battlefield);
    let follower = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    assert!(
        game.keywords(lightning).contains(
            Keywords::FIRST_STRIKE
                .union(Keywords::TRAMPLE)
                .union(Keywords::LIFELINK)
        )
    );
    unblocked_damage(&mut game, &[lightning, follower]);
    assert_eq!(game.state.step, Step::FirstStrikeDamage);
    assert_eq!(game.state.players[p.index()].life, 23);
    assert_eq!(game.state.players[p.other().index()].life, 17);
    game.move_object(lightning, Zone::Graveyard);
    settle(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.state.players[p.other().index()].life, 13);
}

#[test]
fn security_sergeant_permission_ends_at_the_first_control_loss() {
    let p = Player::P1;
    let mut game = scenario(p);
    let lightning = put(
        &mut game,
        p,
        "Lightning, Security Sergeant",
        Zone::Battlefield,
    );
    let top = game.top_library_card(p).unwrap();
    assert!(game.keywords(lightning).contains(Keywords::MENACE));
    unblocked_damage(&mut game, &[lightning]);
    settle(&mut game);
    let exiled = game.object_ref(top);
    assert_eq!(game.object_view(top).zone, Zone::Exile);
    assert!(
        game.observe(p)
            .exile_permissions
            .iter()
            .any(|row| row.object == exiled)
    );
    let exact = game.object_ref(lightning);
    game.record_control(exact, p.other(), None);
    game.record_control(exact, p, None);
    assert!(
        game.observe(p)
            .exile_permissions
            .iter()
            .all(|row| row.object != exiled)
    );
}

#[test]
fn louisoix_counters_with_legendary_sacrifice_or_the_two_mana_fee() {
    let p = Player::P0;
    let mut original = scenario(p);
    let legend = put(
        &mut original,
        p,
        "Giott, King of the Dwarves",
        Zone::Battlefield,
    );
    let plain = put(&mut original, p, "Bear Cub", Zone::Battlefield);
    let spell = put(&mut original, p, "Louisoix's Sacrifice", Zone::Hand);
    let victim = put(&mut original, p.other(), "Opt", Zone::Hand);
    act(&mut original, Action::Pass);
    cast(&mut original, victim, None);
    act(&mut original, Action::Pass);
    for fee in [false, true] {
        let mut game = original.clone();
        if fee {
            game.move_object(legend, Zone::Hand);
        }
        let payment = [0, 1, 0, 0, 0, if fee { 2 } else { 0 }];
        game.state.players[p.index()].mana = payment;
        act(&mut game, Action::AnnounceCast { card: spell });
        act(
            &mut game,
            Action::ChooseSpellCost {
                cost: u16::from(fee),
            },
        );
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Spell(victim),
            },
        );
        if !fee {
            let invalid = Action::ChooseAdditionalCost {
                object: game.object_ref(plain),
            };
            reject(&mut game, invalid);
            let exact = game.object_ref(legend);
            act(&mut game, Action::ChooseAdditionalCost { object: exact });
        }
        act(&mut game, Action::PayCosts { payment });
        settle(&mut game);
        assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
        assert_eq!(
            game.object_view(legend).zone,
            if fee { Zone::Hand } else { Zone::Graveyard }
        );
        assert_eq!(game.object_view(plain).zone, Zone::Battlefield);
    }
}

#[test]
fn scythe_requires_a_block_and_grants_first_strike_only_after_optional_attachment() {
    let p = Player::P0;
    let mut game = scenario(p);
    let target = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let scythe = put(&mut game, p, "Magitek Scythe", Zone::Hand);
    cast(&mut game, scythe, None);
    pass_pair(&mut game);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    pass_pair(&mut game);
    let mut declined = game.clone();
    act(&mut declined, Action::ChooseOptional { accept: false });
    assert_eq!(declined.object_view(scythe).attached_to, None);
    assert_eq!(declined.block_requirement_weight(target), 0);
    act(&mut game, Action::ChooseOptional { accept: true });
    assert_eq!(
        game.object_view(scythe).attached_to,
        Some(game.object_ref(target))
    );
    assert_eq!(stats(&game, target), (4, 3));
    assert!(game.keywords(target).contains(Keywords::FIRST_STRIKE));
    assert_eq!(game.block_requirement_weight(target), 1);
}
