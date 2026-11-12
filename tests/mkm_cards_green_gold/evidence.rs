use super::*;

const CLUE: u16 = 5296;
const SPIDER: u16 = 5685;

fn select_evidence(game: &mut Game, cards: &[CardId]) {
    for &card in cards {
        let object = game.object_ref(card);
        act(game, Action::ChooseEvidence { object });
    }
    act(game, Action::FinishEvidence);
}

fn cast_with_evidence(
    game: &mut Game,
    card: CardId,
    targets: &[CardId],
    evidence: Option<&[CardId]>,
) {
    act(game, Action::AnnounceCast { card });
    let mut targets = targets.iter().copied();
    for _ in 0..16 {
        match game.state.pending {
            Pending::EvidenceIntent { .. } => act(
                game,
                Action::ChooseCollectEvidence {
                    collect: evidence.is_some(),
                },
            ),
            Pending::Targets { .. } => {
                if let Some(card) = targets.next() {
                    act(game, target(card));
                } else {
                    act(game, Action::FinishTargets);
                }
            }
            Pending::Evidence { .. } => select_evidence(game, evidence.unwrap()),
            Pending::PayCosts { .. } => pay(game),
            Pending::Priority => {
                assert!(targets.next().is_none());
                assert!(game.state.stack.iter().any(|item| {
                    item.kind == StackKind::Spell
                        && item.source.is_some_and(|source| source.id == card)
                }));
                return;
            }
            ref pending => panic!("unexpected evidence cast choice {pending:?}"),
        }
    }
    panic!("evidence cast did not finish");
}

#[test]
fn analyze_the_pollen_changes_qualified_search_only_after_evidence() {
    let player = Player::P1;
    let mut game = scenario(player);
    let basic = put(&mut game, player, "Island", Zone::Library);
    let nonbasic = put(&mut game, player, "Cavern of Souls", Zone::Library);
    let creature = put(&mut game, player, "Bear Cub", Zone::Library);
    let first = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let second = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let spell = put(&mut game, player, "Analyze the Pollen", Zone::Hand);

    let mut basic_only = game.clone();
    cast_with_evidence(&mut basic_only, spell, &[], None);
    pass_pair(&mut basic_only);
    for card in [nonbasic, creature] {
        reject(&mut basic_only, Action::ChooseSearch { card });
    }
    let before_shuffle = basic_only.state.rng;
    let mut miss = basic_only.clone();
    act(&mut miss, Action::FinishSearch);
    assert_ne!(miss.state.rng, before_shuffle);
    act(&mut basic_only, Action::ChooseSearch { card: basic });
    act(&mut basic_only, Action::FinishSearch);
    assert_eq!(basic_only.object_view(basic).zone, Zone::Hand);
    assert_eq!(basic_only.object_view(first).zone, Zone::Graveyard);

    cast_with_evidence(&mut game, spell, &[], Some(&[first, second]));
    pass_pair(&mut game);
    assert!(
        game.legal_actions()
            .contains(&Action::ChooseSearch { card: nonbasic })
    );
    act(&mut game, Action::ChooseSearch { card: creature });
    act(&mut game, Action::FinishSearch);
    assert!(
        game.observe(player.other())
            .objects
            .iter()
            .any(|object| { object.id == creature && object.zone == Zone::Hand })
    );
    assert_eq!(game.object_view(first).zone, Zone::Exile);
    assert_eq!(game.object_view(second).zone, Zone::Exile);
}

#[test]
fn axebane_ferox_attacks_immediately_and_only_opponents_owe_its_ward() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let ferox = put(&mut game, player, "Axebane Ferox", Zone::Hand);
    cast(&mut game, ferox, None);
    settle(&mut game);
    assert!(game.object_view(ferox).summoning_sick);
    let keywords = game.object_view(ferox).keywords;
    assert!(keywords.contains(Keywords::HASTE.union(Keywords::DEATHTOUCH)));

    let mut own = game.clone();
    let shock = put(&mut own, player, "Shock", Zone::Hand);
    cast(&mut own, shock, Some(Target::Card(ferox)));
    assert_eq!(own.state.stack.len(), 1);
    settle(&mut own);
    assert_eq!(own.object_view(ferox).damage, 2);

    let mut combat = game.clone();
    let blocker = put(
        &mut combat,
        opponent,
        "Earthshaker Dreadmaw",
        Zone::Battlefield,
    );
    attack(&mut combat, &[ferox]);
    pass_pair(&mut combat);
    act(
        &mut combat,
        Action::Block {
            blocker,
            attacker: ferox,
        },
    );
    act(&mut combat, Action::FinishBlockers);
    pass_pair(&mut combat);
    assert_eq!(combat.object_view(blocker).zone, Zone::Graveyard);

    let murder = put(&mut game, opponent, "Murder", Zone::Hand);
    let fuel = put(&mut game, opponent, "Hill Giant", Zone::Graveyard);
    act(&mut game, Action::Pass);
    cast(&mut game, murder, Some(Target::Card(ferox)));
    assert_eq!(game.state.stack.len(), 2);
    pass_pair(&mut game);
    assert_eq!(game.observe(opponent).evidence.unwrap().threshold, 4);
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineEvidence);
    settle(&mut declined);
    assert!(declined.on_battlefield(ferox));
    select_evidence(&mut game, &[fuel]);
    settle(&mut game);
    assert_eq!(game.object_view(ferox).zone, Zone::Graveyard);
    assert_eq!(game.object_view(fuel).zone, Zone::Exile);
}

#[test]
fn bite_down_locks_the_reduction_and_keeps_independent_target_effects() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Bite Down on Crime", Zone::Hand);
    let own = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let foe = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let fuel = put(&mut game, player, "Earthshaker Dreadmaw", Zone::Graveyard);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 3];

    let mut ordinary = game.clone();
    cast_with_evidence(&mut ordinary, spell, &[own, foe], None);
    assert_eq!(ordinary.state.players[player.index()].mana, [0; 6]);
    settle(&mut ordinary);
    assert_eq!(ordinary.object_view(foe).zone, Zone::Graveyard);
    assert_eq!(stats(&ordinary, own), (4, 2));
    ordinary.finish_cleanup();
    assert_eq!(stats(&ordinary, own), (2, 2));

    cast_with_evidence(&mut game, spell, &[own, foe], Some(&[fuel]));
    assert_eq!(game.state.players[player.index()].mana, [0, 0, 0, 0, 0, 2]);
    let mut lost_own = game.clone();
    lost_own.move_object(own, Zone::Hand);
    settle(&mut lost_own);
    assert_eq!(lost_own.object_view(foe).damage, 0);

    let mut lost_foe = game.clone();
    lost_foe.move_object(foe, Zone::Hand);
    settle(&mut lost_foe);
    assert_eq!(stats(&lost_foe, own), (4, 2));

    settle(&mut game);
    assert_eq!(game.object_view(foe).zone, Zone::Graveyard);
    assert_eq!(game.object_view(fuel).zone, Zone::Exile);
}

#[test]
fn sample_collector_collects_before_choosing_the_reflexive_target() {
    let player = Player::P1;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Sample Collector", Zone::Battlefield);
    let own = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let foe = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let fuel = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    attack(&mut game, &[source]);
    pass_pair(&mut game);
    assert_eq!(game.observe(player).evidence.unwrap().threshold, 3);
    assert!(game.observe(player.other()).evidence.is_none());
    reject(&mut game, target(own));

    let mut declined = game.clone();
    act(&mut declined, Action::DeclineEvidence);
    assert!(declined.state.stack.is_empty());
    assert_eq!(declined.object_view(fuel).zone, Zone::Graveyard);

    select_evidence(&mut game, &[fuel]);
    reject(&mut game, target(foe));
    act(&mut game, target(own));
    assert_eq!(game.counter_count(own, CounterKind::PlusOne), 0);
    let mut stale = game.clone();
    stale.move_object(own, Zone::Hand);
    stale.move_object(own, Zone::Battlefield);
    settle(&mut stale);
    assert_eq!(stale.counter_count(own, CounterKind::PlusOne), 0);
    settle(&mut game);
    assert_eq!(game.counter_count(own, CounterKind::PlusOne), 1);
}

#[test]
fn vitu_ghazi_inspector_checks_its_own_cast_payment_and_exact_target() {
    let player = Player::P0;
    let mut game = scenario(player);
    let inspector = put(&mut game, player, "Vitu-Ghazi Inspector", Zone::Hand);
    let other = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let fuel = put(&mut game, player, "Earthshaker Dreadmaw", Zone::Graveyard);

    let mut unpaid = game.clone();
    cast_with_evidence(&mut unpaid, inspector, &[], None);
    settle(&mut unpaid);
    assert!(
        unpaid
            .object_view(inspector)
            .keywords
            .contains(Keywords::REACH)
    );
    assert_eq!(unpaid.state.players[player.index()].life, 20);

    let mut put_directly = game.clone();
    put_directly.move_object(inspector, Zone::Battlefield);
    put_directly.prepare_triggers();
    assert!(put_directly.state.stack.is_empty());

    cast_with_evidence(&mut game, inspector, &[], Some(&[fuel]));
    pass_pair(&mut game);
    act(&mut game, target(other));
    let mut stale = game.clone();
    stale.move_object(other, Zone::Hand);
    settle(&mut stale);
    assert_eq!(stale.state.players[player.index()].life, 20);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 22);
    assert_eq!(game.counter_count(other, CounterKind::PlusOne), 1);
}

#[test]
fn evidence_examiner_investigates_each_own_collection_and_offers_one_on_own_combat() {
    let player = Player::P0;
    let mut game = scenario(player);
    let examiner = put(&mut game, player, "Evidence Examiner", Zone::Hand);
    let ordinary_exile = put(&mut game, player, "Opt", Zone::Graveyard);
    let first = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let second = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let combat_fuel = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let pollen = put(&mut game, player, "Analyze the Pollen", Zone::Hand);
    cast(&mut game, examiner, None);
    settle(&mut game);
    game.move_object(ordinary_exile, Zone::Exile);
    game.prepare_triggers();
    assert!(game.state.stack.is_empty());

    cast_with_evidence(&mut game, pollen, &[], Some(&[first, second]));
    pass_pair(&mut game);
    assert_eq!(tokens(&game, player, CLUE).len(), 1);
    pass_pair(&mut game);
    act(&mut game, Action::FinishSearch);

    pass_pair(&mut game);
    assert_eq!(game.state.step, Step::BeginCombat);
    pass_pair(&mut game);
    assert_eq!(game.observe(player).evidence.unwrap().threshold, 4);
    let mut decline = game.clone();
    act(&mut decline, Action::DeclineEvidence);
    assert_eq!(tokens(&decline, player, CLUE).len(), 1);
    select_evidence(&mut game, &[combat_fuel]);
    settle(&mut game);
    assert_eq!(tokens(&game, player, CLUE).len(), 2);
}

#[test]
fn izoni_uses_both_evidence_triggers_and_four_tokens_pay_one_activation() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Izoni, Center of the Web", Zone::Hand);
    let first = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let second = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let card_drawn_second = put(&mut game, player, "Grizzly Bears", Zone::Library);
    let card_drawn_first = put(&mut game, player, "Shock", Zone::Library);
    let milled = put(&mut game, player, "Island", Zone::Library);
    let nontoken = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    cast(&mut game, source, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineEvidence);
    assert!(tokens(&declined, player, SPIDER).is_empty());

    select_evidence(&mut game, &[first]);
    assert!(game.object_view(source).keywords.contains(Keywords::MENACE));
    assert_eq!(tokens(&game, player, SPIDER).len(), 2);
    game.create_tokens(player, Card::from_catalogue_id(CLUE), 1, false);
    assert!(!game.advance_entries());
    reject(&mut game, activation(source, 0));

    game.state.objects[source as usize].summoning_sick = false;
    attack(&mut game, &[source]);
    pass_pair(&mut game);
    select_evidence(&mut game, &[second]);
    let spiders = tokens(&game, player, SPIDER);
    assert_eq!(spiders.len(), 4);
    for &id in &spiders {
        let spider = game.object_view(id);
        assert_eq!((spider.power, spider.toughness, spider.colors), (2, 1, 20));
        assert!(
            spider
                .keywords
                .contains(Keywords::MENACE.union(Keywords::REACH))
        );
    }
    let clue = tokens(&game, player, CLUE)[0];
    act(&mut game, activation(source, 0));
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    let not_a_token = game.object_ref(nontoken);
    reject(
        &mut game,
        Action::ChooseAdditionalCost {
            object: not_a_token,
        },
    );
    for card in [spiders[0], spiders[1], spiders[2], clue] {
        let object = game.object_ref(card);
        act(&mut game, Action::ChooseAdditionalCost { object });
    }
    if game
        .legal_actions()
        .contains(&Action::FinishAdditionalCostGroup)
    {
        act(&mut game, Action::FinishAdditionalCostGroup);
    }
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    pass_pair(&mut game);
    for (card, destination) in [
        (milled, LibraryDestination::Graveyard),
        (card_drawn_first, LibraryDestination::Top),
    ] {
        act(&mut game, Action::ArrangeLibrary { card, destination });
    }
    settle(&mut game);
    assert_eq!(tokens(&game, player, SPIDER), [spiders[3]]);
    assert!(tokens(&game, player, CLUE).is_empty());
    assert_eq!(game.object_view(milled).zone, Zone::Graveyard);
    for card in [card_drawn_first, card_drawn_second] {
        assert_eq!(game.object_view(card).zone, Zone::Hand);
    }
    assert_eq!(game.state.players[player.index()].life, 22);
}

#[test]
fn kyloxs_voltstrider_crews_or_collects_and_casts_only_linked_spells_for_mana() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let vehicle = put(&mut game, player, "Kylox's Voltstrider", Zone::Battlefield);
    let crewer = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let shock = put(&mut game, player, "Shock", Zone::Graveyard);
    let sorcery = put(&mut game, player, "Divination", Zone::Graveyard);
    let other = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    let unrelated = put(&mut game, player, "Opt", Zone::Exile);
    assert!(!game.is_creature(vehicle));

    let mut crewed = game.clone();
    act(&mut crewed, activation(vehicle, 1));
    if matches!(crewed.state.pending, Pending::PayCosts { .. }) {
        pay(&mut crewed);
    }
    reject(&mut crewed, Action::FinishCrew);
    let object = crewed.object_ref(crewer);
    act(&mut crewed, Action::ChooseCrewer { object });
    act(&mut crewed, Action::FinishCrew);
    settle(&mut crewed);
    assert!(crewed.object_view(crewer).tapped);
    assert_eq!(stats(&crewed, vehicle), (4, 4));
    crewed.finish_cleanup();
    assert!(!crewed.is_creature(vehicle));

    act(&mut game, activation(vehicle, 0));
    select_evidence(&mut game, &[shock, sorcery, other]);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, vehicle), (4, 4));
    assert_eq!(game.object_view(vehicle).kinds, 1 | 16);
    attack(&mut game, &[vehicle]);
    pass_pair(&mut game);
    let linked = Action::CastExiledDuringResolution {
        object: game.object_ref(shock),
        face: CastFace::Normal,
    };
    for card in [unrelated, other] {
        let object = game.object_ref(card);
        reject(
            &mut game,
            Action::CastExiledDuringResolution {
                object,
                face: CastFace::Normal,
            },
        );
    }
    let mut broke = game.clone();
    broke.state.players[player.index()].mana = [0; 6];
    reject(&mut broke, linked.clone());

    let mana_before = game.state.players[player.index()].mana;
    act(&mut game, linked);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(opponent),
        },
    );
    pay(&mut game);
    act(&mut game, Action::FinishResolutionCasts);
    assert_eq!(
        game.state.players[player.index()].mana[3],
        mana_before[3] - 1
    );

    let mut countered = game.clone();
    let id = countered.state.stack.last().unwrap().id;
    assert!(countered.counter_stack_item(id, false));
    assert_eq!(countered.state.players[player.index()].library[0], shock);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].library[0], shock);
    assert_eq!(game.state.players[opponent.index()].life, 18);
}
