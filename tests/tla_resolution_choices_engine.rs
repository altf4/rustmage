//! Ordered public choices, private searches, and exact immediate cast offers.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardRules, one};
use rustmage_core::{cards::*, engine::*};
use support::*;

const KEEP: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SacrificeCreaturesExceptEachPlayersChoice(4)],
    ),
    ..CardRules::EMPTY
};
const EXILE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::ExileOpposingPermanentsChosenByEachPlayer(3)],
    ),
    ..CardRules::EMPTY
};
const PAIRED_SEARCH: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SearchTwoQualitiesToHand(&[
            SearchFilter::BasicLand,
            SearchFilter::Subtype("Shrine"),
        ])],
    ),
    ..CardRules::EMPTY
};
const SPLIT_SEARCH: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SearchDifferentNames(SearchSpec {
            filter: SearchFilter::Land,
            maximum: 4,
            destination: SearchDestination::BattlefieldAndOpponentChosenGraveyard { graveyard: 2 },
            tapped: true,
            reveal: true,
        })],
    ),
    ..CardRules::EMPTY
};
const SCRY_TWICE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Scry(1), Instruction::Scry(1)],
    ),
    ..CardRules::EMPTY
};
const TOP_OFFER: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ScryOrSurveil,
        targets: TargetPlan::NONE,
        program: &[Instruction::LookLibraryTopMayCastFreeOncePerTurn],
    }],
    ..CardRules::EMPTY
};
const HAND_OFFER: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CastFilteredHandFree(CastFilter::Noncreature)],
    ),
    ..CardRules::EMPTY
};
const LINKED_OFFER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost::ZERO,
        targets: one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 0,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        program: &[Instruction::ExileTargetsLinkedToSource],
        sorcery: false,
        once_per_turn: false,
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[Instruction::CastOneOwnedLinkedPaid(CastFilter::Subtype(
            "Ally",
        ))],
    }],
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn start(game: &mut Game, controller: Player) {
    if game.actor() != Some(controller) {
        act(game, Action::Pass);
    }
    let spell = put(game, controller, "Opt", Zone::Hand);
    cast(game, spell, None);
    pass_pair(game);
}

fn choose_object(game: &Game, id: CardId) -> Action {
    Action::ChooseEffectObject {
        object: game.object_ref(id),
    }
}

#[test]
fn both_players_keep_a_signed_power_subset_before_one_sacrifice_batch() {
    let _rules = fixture_rules::activate(&[(named("Opt"), &KEEP)]);
    let active = Player::P1;
    let controller = active.other();
    let mut game = scenario(active);
    let active_keep = put(&mut game, active, "Grizzly Bears", Zone::Battlefield);
    let active_lost = put(&mut game, active, "Bear Cub", Zone::Battlefield);
    game.state.objects[active_lost as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
    let positive = put(&mut game, controller, "Grizzly Bears", Zone::Battlefield);
    game.state.objects[positive as usize].growth.power = 3;
    let negative = put(&mut game, controller, "Bear Cub", Zone::Battlefield);
    game.state.objects[negative as usize].growth.power = -3;
    let too_large = put(&mut game, controller, "Hill Giant", Zone::Battlefield);
    game.state.objects[too_large as usize].growth.power = 4;
    let noncreature = put(&mut game, controller, "Short Bow", Zone::Battlefield);
    start(&mut game, controller);

    assert_eq!(game.actor(), Some(active));
    let choose = choose_object(&game, active_keep);
    act(&mut game, choose);
    act(&mut game, Action::FinishEffectChoice);
    assert_eq!(game.actor(), Some(controller));
    assert_eq!(
        game.observe(controller).effect_choice.unwrap().selected[active.index()],
        [game.object_ref(active_keep)]
    );
    assert_eq!(game.object_view(active_lost).zone, Zone::Battlefield);
    let choose = choose_object(&game, too_large);
    reject(&mut game, choose);
    let choose = choose_object(&game, positive);
    act(&mut game, choose);
    reject(&mut game, Action::FinishEffectChoice);

    let snapshot = game.snapshot();
    let choose = choose_object(&game, negative);
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == choose)
        .unwrap();
    let mut indexed = game.clone();
    indexed
        .apply_index(controller, game.revision(), index)
        .unwrap();
    act(&mut game, choose.clone());
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&snapshot).unwrap();
    act(&mut indexed, choose);
    act(&mut game, Action::FinishEffectChoice);
    act(&mut indexed, Action::FinishEffectChoice);
    assert_eq!(game.snapshot(), indexed.snapshot());
    for id in [positive, negative, active_keep, noncreature] {
        assert_eq!(game.object_view(id).zone, Zone::Battlefield);
    }
    for id in [active_lost, too_large] {
        assert_eq!(game.object_view(id).zone, Zone::Graveyard);
    }
}

#[test]
fn controller_first_exile_choices_use_the_same_opposing_set_and_deduplicate() {
    let _rules = fixture_rules::activate(&[(named("Opt"), &EXILE)]);
    let opponent = Player::P0;
    let player = opponent.other();
    let mut game = scenario(opponent);
    let own = put(&mut game, player, "Kyoshi Warriors", Zone::Battlefield);
    let low = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let first = put(&mut game, opponent, "Kyoshi Warriors", Zone::Battlefield);
    let second = put(&mut game, opponent, "Master Piandao", Zone::Battlefield);
    game.state.objects[first as usize].gained_keywords = Keywords::HEXPROOF;

    start(&mut game, player);
    assert_eq!(game.actor(), Some(player));
    for illegal in [own, low] {
        let choose = choose_object(&game, illegal);
        reject(&mut game, choose);
    }
    let choose = choose_object(&game, first);
    act(&mut game, choose.clone());
    assert_eq!(game.actor(), Some(opponent));
    assert_eq!(game.object_view(first).zone, Zone::Battlefield);
    assert_eq!(
        game.observe(opponent).effect_choice.unwrap().selected[player.index()],
        [game.object_ref(first)]
    );
    let mut distinct = game.clone();
    let other = choose_object(&distinct, second);
    act(&mut distinct, other);
    act(&mut game, choose);
    assert_eq!(game.object_view(first).zone, Zone::Exile);
    assert_eq!(game.object_view(second).zone, Zone::Battlefield);
    assert_eq!(distinct.object_view(first).zone, Zone::Exile);
    assert_eq!(distinct.object_view(second).zone, Zone::Exile);
    assert_eq!(game.object_view(own).zone, Zone::Battlefield);
    assert_eq!(game.object_view(low).zone, Zone::Battlefield);
}

#[test]
fn one_private_paired_search_enforces_roles_and_reveals_only_the_results() {
    let _rules = fixture_rules::activate(&[(named("Opt"), &PAIRED_SEARCH)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let watcher = put(
        &mut game,
        opponent,
        "Wan Shi Tong, Librarian",
        Zone::Battlefield,
    );
    let basic = put(&mut game, player, "Plains", Zone::Library);
    let other_basic = put(&mut game, player, "Island", Zone::Library);
    let shrine = spawn(
        &mut game,
        player,
        named("Southern Air Temple"),
        Zone::Library,
    );
    let other_shrine = spawn(
        &mut game,
        player,
        named("Crescent Island Temple"),
        Zone::Library,
    );
    let nonbasic = put(&mut game, player, "Abandoned Air Temple", Zone::Library);
    start(&mut game, player);
    assert!(game.observe(opponent).search_choice.is_none());
    reject(&mut game, Action::ChooseSearch { card: nonbasic });

    let mut decline = game.clone();
    act(&mut decline, Action::FinishSearch);
    settle(&mut decline);
    assert!(decline.observe(opponent).zone_reveals.is_empty());
    assert_eq!(decline.counter_count(watcher, CounterKind::PlusOne), 1);

    act(&mut game, Action::ChooseSearch { card: shrine });
    reject(&mut game, Action::ChooseSearch { card: other_shrine });
    assert!(game.observe(opponent).zone_reveals.is_empty());
    let mut reverse = game.clone();
    act(&mut reverse, Action::FinishSearch);
    settle(&mut reverse);
    assert_eq!(reverse.object_view(shrine).zone, Zone::Hand);

    act(&mut game, Action::ChooseSearch { card: basic });
    reject(&mut game, Action::ChooseSearch { card: other_basic });
    act(&mut game, Action::FinishSearch);
    assert_eq!(game.observe(opponent).zone_reveals.len(), 2);
    assert_eq!(game.object_view(shrine).zone, Zone::Hand);
    assert_eq!(game.object_view(basic).zone, Zone::Hand);
    assert_eq!(game.object_view(other_shrine).zone, Zone::Library);
    settle(&mut game);
    assert_eq!(game.counter_count(watcher, CounterKind::PlusOne), 1);
}

#[test]
fn revealed_distinct_search_results_pass_to_the_opponent_before_movement() {
    let _rules = fixture_rules::activate(&[(named("Opt"), &SPLIT_SEARCH)]);
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let [first, second, third, fourth] = ["Plains", "Island", "Swamp", "Mountain"]
        .map(|name| put(&mut game, player, name, Zone::Library));
    let duplicate = put(&mut game, player, "Plains", Zone::Library);
    let hidden = put(&mut game, player, "Bear Cub", Zone::Library);
    start(&mut game, player);
    act(&mut game, Action::ChooseSearch { card: first });
    reject(&mut game, Action::ChooseSearch { card: duplicate });
    assert!(game.observe(opponent).search_choice.is_none());
    assert!(game.observe(opponent).search_distribution.is_none());

    let mut short = game.clone();
    act(&mut short, Action::FinishSearch);
    assert_eq!(short.object_view(first).zone, Zone::Graveyard);
    assert!(short.observe(opponent).search_distribution.is_none());

    for card in [second, third, fourth] {
        act(&mut game, Action::ChooseSearch { card });
    }
    let rng = game.state.rng;
    act(&mut game, Action::FinishSearch);
    assert_eq!(game.actor(), Some(opponent));
    for viewer in [player, opponent] {
        let observation = game.observe(viewer);
        assert!(observation.search_choice.is_none());
        let revealed = observation.search_distribution.unwrap();
        assert_eq!(revealed.player, opponent);
        assert_eq!(revealed.graveyard_count, Some(2));
        assert_eq!(
            revealed.candidates.iter().map(|o| o.id).collect::<Vec<_>>(),
            [first, second, third, fourth,]
        );
    }
    assert_eq!(game.object_view(first).zone, Zone::Library);
    assert_eq!(game.state.rng, rng);
    reject(&mut game, Action::ChooseSearchGraveyard { card: hidden });
    act(&mut game, Action::ChooseSearchGraveyard { card: first });
    reject(&mut game, Action::ChooseSearchGraveyard { card: first });
    let root = game.snapshot();
    let choose = Action::ChooseSearchGraveyard { card: second };
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == choose)
        .unwrap();
    let mut indexed = game.clone();
    indexed
        .apply_index(opponent, game.revision(), index)
        .unwrap();
    act(&mut game, choose.clone());
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&root).unwrap();
    act(&mut indexed, Action::ChooseSearchGraveyard { card: third });
    assert_eq!(indexed.object_view(third).zone, Zone::Graveyard);
    assert_eq!(indexed.object_view(second).zone, Zone::Battlefield);

    assert_eq!(game.object_view(first).zone, Zone::Graveyard);
    assert_eq!(game.object_view(second).zone, Zone::Graveyard);
    for card in [third, fourth] {
        assert_eq!(game.object_view(card).zone, Zone::Battlefield);
        assert!(game.object_view(card).tapped);
    }
    assert_ne!(game.state.rng, rng);
    assert_eq!(game.object_view(hidden).zone, Zone::Library);
}

fn keep_scried_top(game: &mut Game, card: CardId) {
    act(
        game,
        Action::ArrangeLibrary {
            card,
            destination: LibraryDestination::Top,
        },
    );
}

fn order_triggers(game: &mut Game) {
    while matches!(game.state.pending, Pending::OrderTriggers { .. }) {
        choose(game, |a| matches!(a, Action::PutTrigger { .. }));
    }
}

fn finish_offer(game: &mut Game) {
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
    act(game, Action::FinishResolutionCasts);
}

#[test]
fn top_free_cast_allowance_requires_commit_and_queued_triggers_still_look() {
    let _rules = fixture_rules::activate(&[
        (named("Opt"), &SCRY_TWICE),
        (named("Short Bow"), &TOP_OFFER),
    ]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    put(&mut game, player, "Short Bow", Zone::Battlefield);
    let next = put(&mut game, player, "Bear Cub", Zone::Library);
    let top = put(&mut game, player, "Grizzly Bears", Zone::Library);
    start(&mut game, player);
    keep_scried_top(&mut game, top);
    keep_scried_top(&mut game, top);
    order_triggers(&mut game);
    pass_pair(&mut game);
    assert_eq!(
        game.observe(player).known_library_top[player.index()]
            .as_ref()
            .unwrap()
            .object
            .card,
        top
    );
    assert!(game.observe(opponent).known_library_top[player.index()].is_none());
    assert!(game.observe(opponent).resolution_free_casts.is_none());

    let mut decline = game.clone();
    act(&mut decline, Action::FinishResolutionCasts);
    pass_pair(&mut decline);
    assert!(decline.observe(player).resolution_free_casts.is_some());

    let object = game.object_ref(top);
    act(
        &mut game,
        Action::CastExiledDuringResolution {
            object,
            face: CastFace::Normal,
        },
    );
    finish_offer(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(top).zone, Zone::Battlefield);
    assert_eq!(
        game.observe(player).known_library_top[player.index()]
            .as_ref()
            .unwrap()
            .object
            .card,
        next
    );
    assert!(game.observe(opponent).known_library_top[player.index()].is_none());

    start(&mut game, player);
    keep_scried_top(&mut game, next);
    keep_scried_top(&mut game, next);
    assert!(game.state.stack.is_empty());
    assert!(game.state.pending_triggers.is_empty());
}

#[test]
fn a_filtered_hand_offer_uses_the_cast_face_and_keeps_choices_private() {
    let _rules = fixture_rules::activate(&[(named("Opt"), &HAND_OFFER)]);
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let bear = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    let artifact_creature = put(&mut game, player, "Steel Hellkite", Zone::Hand);
    let adventure = put(&mut game, player, "Questing Druid", Zone::Hand);
    let artifact = put(&mut game, player, "Fire Nation Warship", Zone::Hand);
    start(&mut game, player);
    assert!(game.observe(opponent).resolution_free_casts.is_none());

    let offer = |g: &Game, card, face| Action::CastFreeDuringResolution {
        object: g.object_ref(card),
        face,
    };
    for illegal in [bear, artifact_creature, adventure] {
        let action = offer(&game, illegal, CastFace::Normal);
        reject(&mut game, action);
    }
    assert!(
        game.legal_actions()
            .contains(&offer(&game, adventure, CastFace::Adventure))
    );
    let mana = game.state.players[player.index()].mana;
    let action = offer(&game, artifact, CastFace::Normal);
    act(&mut game, action);
    finish_offer(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(artifact).zone, Zone::Battlefield);
    assert_eq!(game.state.players[player.index()].mana, mana);
}

#[test]
fn linked_paid_cast_requires_exact_owned_ally_and_charges_its_mana_cost() {
    let _rules = fixture_rules::activate(&[(named("Grizzly Bears"), &LINKED_OFFER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let eligible = put(&mut game, player, "Water Tribe Captain", Zone::Graveyard);
    let opponent_card = put(
        &mut game,
        player.other(),
        "Water Tribe Captain",
        Zone::Graveyard,
    );
    let other_type = put(&mut game, player, "Bear Cub", Zone::Graveyard);
    let unlinked = put(&mut game, player, "Water Tribe Captain", Zone::Exile);
    for target in [eligible, opponent_card, other_type] {
        choose(&mut game, |action| {
            matches!(action, Action::Activate {
            card, ability: 0, target: Some(Target::Card(chosen)), ..
        } if *card == source && *chosen == target)
        });
        settle(&mut game);
        assert_eq!(game.object_view(target).zone, Zone::Exile);
    }
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: source });
    act(&mut game, Action::FinishAttackers);
    pass_pair(&mut game);

    let offer = |g: &Game, card| Action::CastExiledDuringResolution {
        object: g.object_ref(card),
        face: CastFace::Normal,
    };
    for illegal in [unlinked, opponent_card, other_type] {
        let action = offer(&game, illegal);
        reject(&mut game, action);
    }
    let action = offer(&game, eligible);
    let eligible_ref = game.object_ref(eligible);
    let mut wrong_color = game.clone();
    wrong_color
        .grant_exile_play_permission_with_expiry(
            player,
            eligible_ref,
            None,
            ExilePlayExpiry::WhileExiled,
        )
        .unwrap()
        .spend_any_mana = true;
    wrong_color.state.players[player.index()].mana = [0, 0, 0, 8, 0, 0];
    reject(&mut wrong_color, action.clone());

    let mana = game.state.players[player.index()].mana.iter().sum::<u32>();
    act(&mut game, action);
    finish_offer(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(eligible).zone, Zone::Battlefield);
    assert_eq!(
        game.state.players[player.index()].mana.iter().sum::<u32>(),
        mana - 3
    );
}

#[test]
fn a_linked_paid_disguise_uses_its_own_origin_rules() {
    let _rules = fixture_rules::activate(&[(named("Grizzly Bears"), &LINKED_OFFER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let leyline = put(&mut game, player, "Leyline of Transformation", Zone::Hand);
    cast(&mut game, leyline, None);
    pass_pair(&mut game);
    act(
        &mut game,
        Action::ChooseEntryValue {
            value: EntryChoiceValue::CreatureType(CreatureType::from_name("Ally").unwrap()),
        },
    );

    let source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let concealed = put(&mut game, player, "Bubble Smuggler", Zone::Graveyard);
    choose(&mut game, |action| {
        matches!(action, Action::Activate {
            card, ability: 0, target: Some(Target::Card(chosen)), ..
        } if *card == source && *chosen == concealed)
    });
    settle(&mut game);
    let object = game.object_ref(concealed);
    game.grant_exile_play_permission_with_expiry(
        player,
        object,
        None,
        ExilePlayExpiry::WhileExiled,
    )
    .unwrap()
    .exile_on_graveyard = true;
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: source });
    act(&mut game, Action::FinishAttackers);
    pass_pair(&mut game);

    act(&mut game, Action::AnnounceDisguise { card: concealed });
    finish_offer(&mut game);
    assert!(game.has_subtype(concealed, "Ally"));
    act(&mut game, Action::Pass);
    let scatter = put(&mut game, player.other(), "Essence Scatter", Zone::Hand);
    cast(&mut game, scatter, Some(Target::Spell(concealed)));
    settle(&mut game);
    assert_eq!(game.object_view(concealed).zone, Zone::Graveyard);
}
