use crate::support::*;
use rustmage_core::{cards::*, engine::*};

const CLUE: Card = Card::from_catalogue_id(5296);

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn clues(game: &Game, player: Player) -> Vec<CardId> {
    game.controlled_token_ids(player)
        .filter(|&id| game.object_view(id).card == CLUE)
        .collect()
}

fn pass_to(game: &mut Game, pending: Pending) {
    for _ in 0..12 {
        if game.state.pending == pending {
            return;
        }
        act(game, Action::Pass);
    }
    panic!("expected a rules choice");
}

fn declare(game: &mut Game, cards: &[CardId]) {
    game.state.priority = game.state.active;
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    game.state.attack_declared = false;
    game.state.attackers.clear();
    game.state.selected_attackers.clear();
    for &card in cards {
        game.state.objects[card as usize].tapped = false;
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

fn activation(card: CardId, ability: u16, target: Option<Target>, payment: Mana) -> Action {
    Action::Activate {
        card,
        ability,
        target,
        payment,
        counter_payment: Counters::default(),
    }
}

#[test]
fn lammasu_clears_both_sides_then_its_death_can_suspect_one_opposing_creature() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let ours = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let theirs = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    for id in [ours, theirs] {
        game.set_suspected(game.object_ref(id), true);
    }
    let lammasu = put(&mut game, player, "Absolving Lammasu", Zone::Hand);
    cast(&mut game, lammasu, None);
    settle(&mut game);
    assert!(!game.is_suspected(ours) && !game.is_suspected(theirs));
    assert!(
        game.object_view(lammasu)
            .keywords
            .contains(Keywords::FLYING)
    );

    let murder = put(&mut game, player, "Murder", Zone::Hand);
    cast(&mut game, murder, Some(Target::Card(lammasu)));
    pass_pair(&mut game);
    reject(&mut game, target(ours));
    let mut none = game.clone();
    act(&mut none, Action::FinishTargets);
    settle(&mut none);
    assert_eq!(none.state.players[player.index()].life, 23);
    assert!(!none.is_suspected(theirs));
    act(&mut game, target(theirs));
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 23);
    assert!(game.is_suspected(theirs));
}

#[test]
fn arrival_and_on_the_job_grow_the_intended_creatures_and_each_investigate() {
    let player = Player::P0;
    let mut game = scenario(player);
    let first = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let other = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let arrival = put(&mut game, player, "Auspicious Arrival", Zone::Hand);
    cast(&mut game, arrival, Some(Target::Card(first)));
    let mut stale = game.clone();
    stale.move_object(first, Zone::Exile);
    settle(&mut stale);
    assert!(clues(&stale, player).is_empty());
    settle(&mut game);
    assert_eq!(stats(&game, first), (4, 4));
    assert_eq!(clues(&game, player).len(), 1);

    let job = put(&mut game, player, "On the Job", Zone::Hand);
    cast(&mut game, job, None);
    settle(&mut game);
    assert_eq!(stats(&game, first), (6, 5));
    assert_eq!(stats(&game, other), (4, 3));
    assert_eq!(stats(&game, enemy), (2, 2));
    assert_eq!(clues(&game, player).len(), 2);
    let newcomer = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    assert_eq!(stats(&game, newcomer), (2, 2));

    let deduce = put(&mut game, player, "Deduce", Zone::Hand);
    let drawn = *game.state.players[player.index()].library.last().unwrap();
    cast(&mut game, deduce, None);
    settle(&mut game);
    assert_eq!(game.object_view(drawn).zone, Zone::Hand);
    assert_eq!(clues(&game, player).len(), 3);
}

#[test]
fn thrull_can_flash_in_to_stop_novice_inspectors_entry_trigger() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let novice = put(&mut game, player, "Novice Inspector", Zone::Hand);
    let thrull = put(&mut game, opponent, "Doorkeeper Thrull", Zone::Hand);
    cast(&mut game, novice, None);
    let mut suppressed = game.clone();
    act(&mut suppressed, Action::Pass);
    cast(&mut suppressed, thrull, None);
    settle(&mut suppressed);
    assert_eq!(suppressed.object_view(novice).zone, Zone::Battlefield);
    assert!(
        suppressed
            .object_view(thrull)
            .keywords
            .contains(Keywords::FLYING)
    );
    assert!(clues(&suppressed, player).is_empty());

    settle(&mut game);
    let clue = clues(&game, player)[0];
    let token = game.object_view(clue);
    assert_eq!(token.kinds, 16);
    assert_eq!(token.colors, 0);
    assert!(token.has_subtype("Clue"));
    let hand = game.hand_count(player);
    act(&mut game, activation(clue, 0, None, [0, 0, 0, 0, 0, 2]));
    settle(&mut game);
    assert_eq!(game.hand_count(player), hand + 1);
}

#[test]
fn griffnaut_tracker_targets_two_cards_only_if_they_share_a_graveyard() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let tracker = put(&mut game, player, "Griffnaut Tracker", Zone::Hand);
    let other_owner = put(&mut game, player, "Bear Cub", Zone::Graveyard);
    let first = put(&mut game, opponent, "Forest", Zone::Graveyard);
    let second = put(&mut game, opponent, "Shock", Zone::Graveyard);
    cast(&mut game, tracker, None);
    pass_pair(&mut game);
    act(&mut game, target(first));
    reject(&mut game, target(other_owner));
    act(&mut game, target(second));
    game.move_object(first, Zone::Hand);
    settle(&mut game);
    assert_eq!(game.object_view(first).zone, Zone::Hand);
    assert_eq!(game.object_view(second).zone, Zone::Exile);
    assert_eq!(game.object_view(other_owner).zone, Zone::Graveyard);
    assert!(
        game.object_view(tracker)
            .keywords
            .contains(Keywords::FLYING)
    );
}

#[test]
fn make_your_move_accepts_artifacts_enchantments_and_large_creatures() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let removal = put(&mut game, player, "Make Your Move", Zone::Hand);
    let artifact = put(&mut game, opponent, "Candlestick", Zone::Battlefield);
    let enchantment = put(
        &mut game,
        opponent,
        "Case of the Shattered Pact",
        Zone::Battlefield,
    );
    let small = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let big = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    game.add_counter(big, CounterKind::PlusOne, 1);
    act(&mut game, Action::AnnounceCast { card: removal });
    reject(&mut game, target(small));
    for recipient in [artifact, enchantment, big] {
        let mut branch = game.clone();
        act(&mut branch, target(recipient));
        pay(&mut branch);
        settle(&mut branch);
        assert_eq!(branch.object_view(recipient).zone, Zone::Graveyard);
    }
}

#[test]
fn makeshift_binding_returns_its_exact_exile_and_life_does_not_require_source_to_stay() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let binding = put(&mut game, player, "Makeshift Binding", Zone::Hand);
    let victim = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let friend = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    cast(&mut game, binding, None);
    pass_pair(&mut game);
    reject(&mut game, target(friend));
    act(&mut game, target(victim));
    let old = game.object_ref(victim);
    let mut early = game.clone();
    early.move_object(binding, Zone::Graveyard);
    settle(&mut early);
    assert_eq!(early.object_ref(victim), old);
    assert_eq!(early.object_view(victim).zone, Zone::Battlefield);
    assert_eq!(early.state.players[player.index()].life, 22);

    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 22);
    assert_eq!(game.object_view(victim).zone, Zone::Exile);
    game.move_object(binding, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Battlefield);
    assert_ne!(game.object_ref(victim), old);
}

#[test]
fn marketwatch_and_neighborhood_guardian_use_the_other_entrants_power() {
    let player = Player::P0;
    let mut game = scenario(player);
    let phantom = put(&mut game, player, "Marketwatch Phantom", Zone::Battlefield);
    let guardian = put(
        &mut game,
        player,
        "Neighborhood Guardian",
        Zone::Battlefield,
    );
    let giant = put(&mut game, player, "Hill Giant", Zone::Hand);
    cast(&mut game, giant, None);
    settle(&mut game);
    assert!(
        !game
            .object_view(phantom)
            .keywords
            .contains(Keywords::FLYING)
    );
    assert_eq!(stats(&game, guardian), (2, 2));

    let entrant = put(&mut game, player, "Bear Cub", Zone::Hand);
    cast(&mut game, entrant, None);
    pass_pair(&mut game);
    for _ in 0..6 {
        match game.state.pending {
            Pending::OrderTriggers { .. } => choose(&mut game, |action| {
                matches!(action, Action::PutTrigger { .. })
            }),
            Pending::Targets { .. } => act(&mut game, target(guardian)),
            Pending::Priority => break,
            ref pending => panic!("unexpected trigger choice {pending:?}"),
        }
    }
    settle(&mut game);
    assert!(
        game.object_view(phantom)
            .keywords
            .contains(Keywords::FLYING)
    );
    assert_eq!(stats(&game, guardian), (3, 3));
    assert_eq!(stats(&game, entrant), (2, 2));
}

#[test]
fn seasoned_consultant_needs_one_three_attacker_declaration_and_watch_exiles_an_attacker() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let consultant = put(&mut game, player, "Seasoned Consultant", Zone::Battlefield);
    let first = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let third = put(&mut game, player, "Savannah Lions", Zone::Battlefield);
    let watch = put(&mut game, opponent, "Not on My Watch", Zone::Hand);
    for attackers in [[first, second], [first, third]] {
        declare(&mut game, &attackers);
        settle(&mut game);
        assert_eq!(stats(&game, consultant), (1, 3));
    }
    declare(&mut game, &[first, second, third]);
    settle(&mut game);
    assert_eq!(stats(&game, consultant), (3, 3));
    act(&mut game, Action::Pass);
    act(&mut game, Action::AnnounceCast { card: watch });
    reject(&mut game, target(consultant));
    act(&mut game, target(third));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(third).zone, Zone::Exile);
}

#[test]
fn benthic_criminologists_offers_artifact_sacrifice_independently_on_entry_and_attack() {
    let player = Player::P0;
    let mut game = scenario(player);
    let benthic = put(&mut game, player, "Benthic Criminologists", Zone::Hand);
    let mine = put(&mut game, player, "Candlestick", Zone::Battlefield);
    let theirs = put(&mut game, player.other(), "Candlestick", Zone::Battlefield);
    cast(&mut game, benthic, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    let other = game.object_ref(theirs);
    reject(&mut game, Action::Sacrifice { object: other });
    act(&mut game, Action::DeclineEffectPayment);
    assert_eq!(game.hand_count(player), 0);
    assert_eq!(game.object_view(mine).zone, Zone::Battlefield);
    game.state.objects[benthic as usize].summoning_sick = false;
    pass_to(&mut game, Pending::Attackers);
    act(&mut game, Action::Attack { card: benthic });
    act(&mut game, Action::FinishAttackers);
    pass_pair(&mut game);
    let object = game.object_ref(mine);
    act(&mut game, Action::Sacrifice { object });
    settle(&mut game);
    assert_eq!(game.object_view(mine).zone, Zone::Graveyard);
    assert_eq!(game.hand_count(player), 1);
}

#[test]
fn burden_of_proof_switches_between_its_bonus_and_base_stats_with_control() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let burden = put(&mut game, player, "Burden of Proof", Zone::Hand);
    let enchanted = put(&mut game, player, "Seasoned Consultant", Zone::Battlefield);
    let detective = put(&mut game, player, "Seasoned Consultant", Zone::Battlefield);
    let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    game.add_counter(enchanted, CounterKind::PlusOne, 1);
    cast(&mut game, burden, Some(Target::Card(enchanted)));
    settle(&mut game);
    assert_eq!(stats(&game, enchanted), (4, 6));

    game.record_control(game.object_ref(enchanted), opponent, None);
    assert_eq!(stats(&game, enchanted), (2, 2));
    declare(&mut game, &[detective, bear]);
    pass_to(&mut game, Pending::Blockers);
    reject(
        &mut game,
        Action::Block {
            blocker: enchanted,
            attacker: detective,
        },
    );
    assert!(game.legal_actions().contains(&Action::Block {
        blocker: enchanted,
        attacker: bear
    }));
}

#[test]
fn candlestick_grants_surveil_to_its_bearer_and_sacrifices_to_draw() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let candle = put(&mut game, player, "Candlestick", Zone::Battlefield);
    let bearer = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    act(
        &mut game,
        activation(candle, 1, Some(Target::Card(bearer)), [0, 0, 0, 0, 0, 2]),
    );
    settle(&mut game);
    assert_eq!(stats(&game, bearer), (3, 3));
    game.record_control(game.object_ref(bearer), opponent, None);
    game.state.objects[bearer as usize].summoning_sick = false;
    game.state.active = opponent;
    let kept = put(&mut game, opponent, "Island", Zone::Library);
    let dumped = put(&mut game, opponent, "Shock", Zone::Library);
    declare(&mut game, &[bearer]);
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(opponent));
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: dumped,
            destination: LibraryDestination::Graveyard,
        },
    );
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: kept,
            destination: LibraryDestination::Top,
        },
    );
    assert_eq!(game.object_view(dumped).zone, Zone::Graveyard);
    assert_eq!(
        *game.state.players[opponent.index()].library.last().unwrap(),
        kept
    );

    game.state.priority = player;
    act(&mut game, activation(candle, 0, None, [0, 0, 0, 0, 0, 2]));
    settle(&mut game);
    assert_eq!(game.hand_count(player), 1);
    assert_eq!(stats(&game, bearer), (2, 2));
}

#[test]
fn cold_case_cracker_investigates_on_death_but_not_exile() {
    let player = Player::P0;
    let mut game = scenario(player);
    let cracker = put(&mut game, player, "Cold Case Cracker", Zone::Battlefield);
    let exiled = put(&mut game, player, "Cold Case Cracker", Zone::Battlefield);
    assert!(
        game.object_view(cracker)
            .keywords
            .contains(Keywords::FLYING)
    );
    game.move_object(exiled, Zone::Exile);
    game.prepare_triggers();
    assert!(game.state.stack.is_empty());
    let murder = put(&mut game, player, "Murder", Zone::Hand);
    cast(&mut game, murder, Some(Target::Card(cracker)));
    settle(&mut game);
    assert_eq!(clues(&game, player).len(), 1);
}

#[test]
fn crimestopper_sprite_applies_stun_only_when_its_additional_evidence_was_paid() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let sprite = put(&mut game, player, "Crimestopper Sprite", Zone::Hand);
    let enemy = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    game.state.objects[enemy as usize].tapped = true;
    let fuel = [
        put(&mut game, player, "Hill Giant", Zone::Graveyard),
        put(&mut game, player, "Hill Giant", Zone::Graveyard),
    ];
    act(&mut game, Action::AnnounceCast { card: sprite });
    let mut declined = game.clone();
    for (branch, collect) in [(&mut declined, false), (&mut game, true)] {
        act(branch, Action::ChooseCollectEvidence { collect });
        if collect {
            for card in fuel {
                let object = branch.object_ref(card);
                act(branch, Action::ChooseEvidence { object });
            }
            act(branch, Action::FinishEvidence);
        }
        pay(branch);
        pass_pair(branch);
        act(branch, target(enemy));
        settle(branch);
        assert!(branch.object_view(enemy).tapped);
        assert!(
            branch
                .object_view(sprite)
                .keywords
                .contains(Keywords::FLYING)
        );
        assert_eq!(
            branch.counter_count(enemy, CounterKind::Stun),
            u64::from(collect)
        );
    }
}

#[test]
fn fae_flight_grants_hexproof_in_response_and_leaves_a_continuous_flying_bonus() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(opponent);
    let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let murder = put(&mut game, opponent, "Murder", Zone::Hand);
    let flight = put(&mut game, player, "Fae Flight", Zone::Hand);
    cast(&mut game, murder, Some(Target::Card(creature)));
    act(&mut game, Action::Pass);
    cast(&mut game, flight, Some(Target::Card(creature)));
    settle(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Battlefield);
    assert_eq!(stats(&game, creature), (3, 2));
    let keywords = game.object_view(creature).keywords;
    assert!(keywords.contains(Keywords::FLYING) && keywords.contains(Keywords::HEXPROOF));
}

#[test]
fn forensic_researcher_pays_tap_and_evidence_for_its_two_different_targets() {
    let player = Player::P0;
    let mut game = scenario(player);
    let researcher = put(&mut game, player, "Forensic Researcher", Zone::Battlefield);
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    game.state.objects[land as usize].tapped = true;
    let self_untap = activation(researcher, 0, Some(Target::Card(researcher)), [0; 6]);
    reject(&mut game, self_untap);
    act(
        &mut game,
        activation(researcher, 0, Some(Target::Card(land)), [0; 6]),
    );
    settle(&mut game);
    assert!(!game.object_view(land).tapped);
    assert!(game.object_view(researcher).tapped);

    game.state.objects[researcher as usize].tapped = false;
    let fuel = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    act(
        &mut game,
        Action::AnnounceActivate {
            card: researcher,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    reject(&mut game, target(researcher));
    act(&mut game, target(enemy));
    let object = game.object_ref(fuel);
    act(&mut game, Action::ChooseEvidence { object });
    act(&mut game, Action::FinishEvidence);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(fuel).zone, Zone::Exile);
    assert!(game.object_view(enemy).tapped);
    assert!(game.object_view(researcher).tapped);
}

#[test]
fn out_cold_survives_unpaid_ward_and_applies_both_stuns_and_one_clue() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let cold = put(&mut game, player, "Out Cold", Zone::Hand);
    let ward = put(
        &mut game,
        opponent,
        "Aurelia's Vindicator",
        Zone::Battlefield,
    );
    let second = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    announce(&mut game, cold, &[Target::Card(ward), Target::Card(second)]);
    pass_pair(&mut game);
    act(&mut game, Action::DeclineResolutionMana);
    settle(&mut game);
    for id in [ward, second] {
        assert!(game.object_view(id).tapped);
        assert_eq!(game.counter_count(id, CounterKind::Stun), 1);
    }
    assert_eq!(clues(&game, player).len(), 1);
}

#[test]
fn reasonable_doubt_suspects_its_creature_even_when_the_spell_controller_pays() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(opponent);
    let shock = put(&mut game, opponent, "Shock", Zone::Hand);
    let bear = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let doubt = put(&mut game, player, "Reasonable Doubt", Zone::Hand);
    cast(&mut game, shock, Some(Target::Player(player)));
    act(&mut game, Action::Pass);
    announce(
        &mut game,
        doubt,
        &[Target::Spell(shock), Target::Card(bear)],
    );
    pass_pair(&mut game);
    let mut unpaid = game.clone();
    act(&mut unpaid, Action::DeclineResolutionMana);
    settle(&mut unpaid);
    assert_eq!(unpaid.state.players[player.index()].life, 20);
    assert!(unpaid.is_suspected(bear));

    act(
        &mut game,
        Action::PayResolutionMana {
            payment: [0, 0, 0, 0, 0, 2],
        },
    );
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 18);
    assert!(game.is_suspected(bear));
}
