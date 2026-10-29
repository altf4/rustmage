//! Cross-check behold candidate and funding paths against changing witnesses.
use super::*;

fn elf_cost() -> AdditionalCost {
    AdditionalCost {
        kind: EffectChoiceKind::Behold,
        filter: SacrificeFilter {
            keywords: Keywords::NONE,
            source_only: false,
            kinds: 0,
            nonland: false,
            other: false,
            tokens: None,
            artifact_enchantment_or_token: false,
            subtype: Some("Elf"),
        },
    }
}

#[test]
fn behold_changeling_tracks_layered_type_replacement_and_expiry() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let witness = put(&mut game, player, "Changeling Wayfinder", Zone::Battlefield);
        let source = put(&mut game, player, "Shock", Zone::Stack);
        let reference = game.object_ref(witness);
        let source_ref = game.object_ref(source);
        let check = |game: &Game| {
            let choices = hand_choices::behold_candidates(game, player, "Elf", &[]);
            let direct = game.additional_candidate(elf_cost(), player, source_ref, reference);
            let funding = game.can_fund(FundingGoal {
                actor: player,
                required: [0; 6],
                generic: 0,
                reserve: Some((reference, elf_cost())),
                source: None,
            });
            (choices.contains(&reference), direct, funding)
        };
        assert_eq!(check(&game), (true, true, true));
        let secret = put(&mut game, player, "Secret Identity", Zone::Hand);
        big_helpers::cast(&mut game, secret, Some(1), &[Target::Card(witness)]);
        big_helpers::settle(&mut game);
        assert!(game.keywords(witness).contains(Keywords::CHANGELING));
        assert_eq!(check(&game), (false, false, false));
        game.finish_cleanup();
        assert_eq!(check(&game), (true, true, true));
    }
}

#[test]
fn behold_rejects_stale_references_and_changed_control_in_funding() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let elf = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
        let source = put(&mut game, player, "Shock", Zone::Stack);
        let original = game.object_ref(elf);
        let source_ref = game.object_ref(source);
        let valid = |game: &Game, reference: ObjectRef| {
            game.additional_candidate(elf_cost(), player, source_ref, reference)
                && game.can_fund(FundingGoal {
                    actor: player,
                    required: [0; 6],
                    generic: 0,
                    reserve: Some((reference, elf_cost())),
                    source: None,
                })
        };
        assert!(valid(&game, original));
        game.change_controller(original, player.other());
        assert!(!valid(&game, original));
        game.change_controller(original, player);
        assert!(valid(&game, original));
        game.move_object(elf, Zone::Hand);
        game.move_object(elf, Zone::Battlefield);
        let returned = game.object_ref(elf);
        assert_ne!(original, returned);
        assert!(!valid(&game, original));
        assert!(valid(&game, returned));
        super::super::contracts::branch_contract(game);
    }
}

#[test]
fn countersculpt_witness_is_hidden_until_revealed_and_rollback_remembers_it() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player.other());
        let target = put(&mut game, player.other(), "Bear Cub", Zone::Hand);
        let spell = put(&mut game, player, "Countersculpt", Zone::Hand);
        let witness = put(&mut game, player, "Jace, Reality Sculptor", Zone::Hand);
        let secret = put(&mut game, player, "Island", Zone::Hand);
        let foreign = put(&mut game, player.other(), "Jace, Reality Sculptor", Zone::Hand);
        cast(&mut game, target, None);
        game.state.priority = player;
        game.state.players[player.index()].mana = [0, 2, 0, 0, 0, 1];
        act(&mut game, Action::AnnounceCast { card: spell });
        act(&mut game, Action::ChooseSpellCost { cost: 0 });
        act(&mut game, Action::ChooseTarget { target: Target::Spell(target) });
        assert!(game.observe(player.other()).additional_cost.is_none());
        assert!(!game.observe(player.other()).known_hand.contains(&game.object_ref(witness)));
        let invalid = game.object_ref(foreign);
        reject(&mut game, Action::ChooseAdditionalCost { object: invalid });
        let invalid = game.object_ref(secret);
        reject(&mut game, Action::ChooseAdditionalCost { object: invalid });
        let selected = game.object_ref(witness);
        act(&mut game, Action::ChooseAdditionalCost { object: selected });
        assert!(!game.observe(player.other()).known_hand.contains(&selected));

        // Exercise the cost's actual commit at a rollback checkpoint, then
        // restore the pending action as an unpayable proposal would do.
        game.begin_attack_rollback(player);
        let mut announcement = game.state.announcing.as_ref().unwrap().clone();
        game.pay_additional_cost(&mut announcement);
        assert!(game.observe(player.other()).known_hand.contains(&selected));
        game.finish_cast_rollback(false);
        assert!(game.observe(player.other()).known_hand.contains(&selected));
        assert!(!game.observe(player.other()).objects.iter().any(|o| o.id == secret));
        assert_eq!(game.object_view(witness).zone, Zone::Hand);
        super::super::contracts::branch_contract(game);
    }
}

#[test]
fn sanctum_is_public_while_its_private_behold_choice_is_pending() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let witness = put(&mut game, player, "Jace, Reality Sculptor", Zone::Hand);
        let secret = put(&mut game, player, "Island", Zone::Hand);
        let enemy = put(&mut game, player.other(), "Jace, Reality Sculptor", Zone::Battlefield);
        game.add_counter(enemy, CounterKind::Loyalty, 2);
        let land = put(&mut game, player, "Theorist's Sanctum", Zone::Hand);
        act(&mut game, Action::PlayLand { card: land });
        assert!(matches!(game.state.pending, Pending::EntryBehold { .. }));
        let opponent_view = game.observe(player.other());
        assert!(opponent_view.objects.iter().any(|o| o.id == land));
        assert!(!opponent_view.objects.iter().any(|o| o.id == witness || o.id == secret));
        let foreign = game.object_ref(enemy);
        reject(&mut game, Action::ChooseEntryBehold { object: Some(foreign) });
        let valid = game.object_ref(witness);
        act(&mut game, Action::ChooseEntryBehold { object: Some(valid) });
        assert!(game.observe(player.other()).known_hand.contains(&valid));
        assert!(!game.observe(player.other()).objects.iter().any(|o| o.id == secret));
    }
}

#[test]
fn sanctum_declined_behold_followed_by_spelunking_enters_untapped() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        put(&mut game, player, "Spelunking", Zone::Battlefield);
        put(&mut game, player, "Jace, Reality Sculptor", Zone::Hand);
        let land = put(&mut game, player, "Theorist's Sanctum", Zone::Hand);
        act(&mut game, Action::PlayLand { card: land });
        let intrinsic = game.object_ref(land);
        act(&mut game, Action::ChooseEntryReplacement { source: intrinsic, intrinsic: true });
        act(&mut game, Action::ChooseEntryBehold { object: None });
        assert_eq!(game.object_view(land).zone, Zone::Battlefield);
        assert!(!game.object_view(land).tapped);
    }
}

fn countersculpt_against(game: &mut Game, player: Player, target: CardId) -> CardId {
    let spell = put(game, player, "Countersculpt", Zone::Hand);
    game.state.priority = player;
    game.state.players[player.index()].mana = [0, 2, 0, 0, 0, 1];
    act(game, Action::AnnounceCast { card: spell });
    act(game, Action::ChooseSpellCost { cost: 1 });
    act(game, Action::ChooseTarget { target: Target::Spell(target) });
    pay(game);
    spell
}

#[test]
fn countersculpt_uncounterable_target_still_empowers_but_stale_target_fizzles() {
    for player in [Player::P0, Player::P1] {
        for stale in [false, true] {
            let mut game = scenario(player.other());
            let target = put(&mut game, player.other(), if stale { "Bear Cub" } else { "Sphinx of the Final Word" }, Zone::Hand);
            cast(&mut game, target, None);
            let target_item = game.state.stack.last().unwrap().id;
            let spell = countersculpt_against(&mut game, player, target);
            if stale {
                game.counter_stack_item(target_item, false);
            }
            settle(&mut game);
            assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
            assert_eq!(game.object_view(target).zone, if stale { Zone::Graveyard } else { Zone::Battlefield });
            let tokens = game.state.objects.iter().enumerate().filter(|(id, object)| {
                object.zone == Zone::Battlefield && object.controller == player && object.is_token()
                    && game.has_subtype(*id as CardId, "Jace")
            }).count();
            assert_eq!(tokens, usize::from(!stale));
        }
    }
}

#[test]
fn countersculpt_chooses_one_of_multiple_existing_jace_tokens() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player.other());
        game.create_tokens(player, Card::JacePlaneswalkerToken, 2, false);
        let tokens: Vec<_> = game.state.objects.iter().enumerate().filter_map(|(id, object)| {
            (object.zone == Zone::Battlefield && object.controller == player && object.is_token()
                && game.has_subtype(id as CardId, "Jace")).then_some(id as CardId)
        }).collect();
        assert_eq!(tokens.len(), 2);
        for &id in &tokens { game.state.objects[id as usize].counters.loyalty = 2; }
        let target = put(&mut game, player.other(), "Bear Cub", Zone::Hand);
        cast(&mut game, target, None);
        countersculpt_against(&mut game, player, target);
        for _ in 0..20 {
            if matches!(game.state.pending, Pending::EffectChoice { .. }) { break; }
            act(&mut game, Action::Pass);
        }
        let first = game.object_ref(tokens[0]);
        let second = game.object_ref(tokens[1]);
        assert!(game.legal_actions().contains(&Action::ChooseJace { object: first }));
        assert!(game.legal_actions().contains(&Action::ChooseJace { object: second }));
        let bad = game.object_ref(target);
        reject(&mut game, Action::ChooseJace { object: bad });
        act(&mut game, Action::ChooseJace { object: second });
        settle(&mut game);
        assert_eq!(game.counter_count(tokens[0], CounterKind::Loyalty), 2);
        assert_eq!(game.counter_count(tokens[1], CounterKind::Loyalty), 3);
    }
}

#[test]
fn sanctum_cannot_behold_a_jace_entering_in_the_same_batch() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let land = put(&mut game, player, "Theorist's Sanctum", Zone::Graveyard);
        let jace = put(&mut game, player, "Jace, Reality Sculptor", Zone::Graveyard);
        let land_move = game.entry_move(land, Zone::Battlefield, false);
        let jace_move = game.entry_move(jace, Zone::Battlefield, false);
        game.stage_entries(vec![land_move, jace_move], None);
        game.advance_entries();
        assert!(!matches!(game.state.pending, Pending::EntryBehold { .. }));
        assert_eq!(game.object_view(land).zone, Zone::Battlefield);
        assert_eq!(game.object_view(jace).zone, Zone::Battlefield);
        assert!(game.object_view(land).tapped);
    }
}
