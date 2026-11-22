//! Cross-card TRK acceptance: printed mechanics share live, owned game state.
use crate::support::*;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};

fn drain(game: &mut Game) {
    for _ in 0..64 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.resolving.is_none()
        {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!(
        "cross-mechanic resolution did not finish: {:?}",
        game.state.pending
    );
}

fn station(game: &mut Game, ship: CardId, donor: CardId) {
    act(
        game,
        Action::AnnounceActivate {
            card: ship,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    let object = game.object_ref(donor);
    act(game, Action::ChooseAdditionalCost { object });
    pay(game);
    drain(game);
}

#[test]
fn sarcophagus_power_changes_enterprise_station_charges_and_live_threshold() {
    let mut game = scenario(Player::P0);
    put(
        &mut game,
        Player::P0,
        "Sarcophagus, Ship of the Dead",
        Zone::Battlefield,
    );
    let ship = put(
        &mut game,
        Player::P0,
        "U.S.S. Enterprise-D, Galaxy-Class",
        Zone::Battlefield,
    );
    let donors: Vec<_> = (0..4)
        .map(|_| put(&mut game, Player::P0, "Bear Cub", Zone::Battlefield))
        .collect();
    assert_eq!(stats(&game, donors[0]), (3, 2));
    assert_eq!(game.object_view(ship).kinds & 1, 0);
    station(&mut game, ship, donors[0]);
    assert_eq!(game.counter_count(ship, CounterKind::Charge), 3);
    assert_eq!(game.state.exile_permissions.len(), 1);
    station(&mut game, ship, donors[1]);
    assert_eq!(game.counter_count(ship, CounterKind::Charge), 6);
    assert_eq!(game.object_view(ship).kinds & 1, 0);
    station(&mut game, ship, donors[2]);
    assert_eq!(game.counter_count(ship, CounterKind::Charge), 9);
    assert_eq!(game.object_view(ship).kinds & 1, 1);
    assert_eq!(stats(&game, ship), (5, 5));
    assert!(
        game.keywords(ship)
            .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
    );
    assert_eq!(game.state.exile_permissions.len(), 1);

    // Counter removal is fixture setup; both printed continuous rules still
    // recompute when Station later adds counters again in the same turn.
    game.set_counters(ship, Counters::of(CounterKind::Charge, 6));
    assert_eq!(game.object_view(ship).kinds & 1, 0);
    station(&mut game, ship, donors[3]);
    assert_eq!(game.counter_count(ship, CounterKind::Charge), 9);
    assert_eq!(stats(&game, ship), (5, 5));
    assert_eq!(game.state.exile_permissions.len(), 1);
}

#[test]
fn federation_excludes_both_changeling_and_assimilated_borg_creatures() {
    let player = Player::P0;
    let mut base = scenario(player);
    let archer = put(&mut base, player, "Captain Jonathan Archer", Zone::Hand);
    cast(&mut base, archer, None);
    drain(&mut base);
    let bear = put(&mut base, player, "Bear Cub", Zone::Battlefield);
    let opposing = put(
        &mut base,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let grave = put(&mut base, player.other(), "Hill Giant", Zone::Graveyard);
    put(&mut base, player, "Death Is Irrelevant", Zone::Battlefield);
    let count = |game: &Game| {
        game.stat_value(
            StatValue::DistinctControlledCreatureTypesExcept("Borg"),
            game.object_ref(archer),
            player,
        )
    };
    assert_eq!(count(&base), 4); // Human, Officer, Dog, Bear.

    let mut changeling = base.clone();
    let infiltration = put(&mut changeling, player, "Dominion Infiltration", Zone::Hand);
    act(&mut changeling, Action::AnnounceCast { card: infiltration });
    act(&mut changeling, Action::ChooseX { value: 1 });
    act(
        &mut changeling,
        Action::ChooseTarget {
            target: Target::Card(opposing),
        },
    );
    if changeling.legal_actions().contains(&Action::FinishTargets) {
        act(&mut changeling, Action::FinishTargets);
    }
    pay(&mut changeling);
    drain(&mut changeling);
    let token = changeling
        .controlled_token_ids(player)
        .find(|&id| changeling.object_view(id).card == Card::GrizzlyBears)
        .unwrap();
    assert!(changeling.keywords(token).contains(Keywords::CHANGELING));
    assert!(changeling.has_subtype(token, "Borg"));
    assert_eq!(count(&changeling), 4);

    let mut assimilated = base.clone();
    assimilated.state.step = Step::Main2;
    assimilated.give_priority();
    act(&mut assimilated, Action::Pass);
    act(&mut assimilated, Action::Pass);
    for _ in 0..32 {
        if assimilated.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(grave),
        }) {
            break;
        }
        choose(&mut assimilated, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    act(
        &mut assimilated,
        Action::ChooseTarget {
            target: Target::Card(grave),
        },
    );
    if assimilated.legal_actions().contains(&Action::FinishTargets) {
        act(&mut assimilated, Action::FinishTargets);
    }
    drain(&mut assimilated);
    assert_eq!(assimilated.object_view(grave).controller, player);
    assert!(assimilated.has_subtype(grave, "Borg"));
    assert!(!assimilated.has_subtype(grave, "Giant"));
    assert_eq!(count(&assimilated), 4);
    assert_eq!(stats(&base, bear), (2, 2));
}

#[test]
fn vger_dilemma_fires_observers_once_before_its_private_hand_look() {
    let player = Player::P0;
    let mut game = scenario(player);
    let seven = put(&mut game, player, "Seven of Nine", Zone::Battlefield);
    let adjudicator = put(
        &mut game,
        player,
        "Melkotian Adjudicator",
        Zone::Battlefield,
    );
    let opposing_observer = put(
        &mut game,
        player.other(),
        "Obsidian Order Operative",
        Zone::Battlefield,
    );
    let secret = put(&mut game, player.other(), "Shock", Zone::Hand);
    let vger = put(&mut game, player, "V'Ger, the Intruder", Zone::Hand);
    cast(&mut game, vger, None);
    for _ in 0..24 {
        if game
            .legal_actions()
            .contains(&Action::ChooseMode { mode: 0 })
        {
            break;
        }
        choose(&mut game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    let checkpoint = game.snapshot();
    let mode = Action::ChooseMode { mode: 0 };
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == mode)
        .unwrap();
    let mut indexed = game.clone();
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    act(&mut game, mode.clone());
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&checkpoint).unwrap();
    act(&mut indexed, mode);
    assert_eq!(game.snapshot(), indexed.snapshot());
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(player.other()),
        },
    );
    if game.legal_actions().contains(&Action::FinishStackTargets) {
        act(&mut game, Action::FinishStackTargets);
    }
    for _ in 0..32 {
        if matches!(game.state.pending, Pending::HandCardChoice { .. }) {
            break;
        }
        choose(&mut game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    assert_eq!(
        game.state.players[player.index()].dilemmas_faced_this_turn,
        1
    );
    assert_eq!(game.counter_count(adjudicator, CounterKind::PlusOne), 1);
    assert_eq!(
        game.counter_count(opposing_observer, CounterKind::PlusOne),
        0
    );
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    assert_eq!(game.state.players[player.other().index()].life, 20);
    let own_view = game.observe(player).hand_choice.unwrap();
    assert_eq!(own_view.kind, HandChoiceKind::LookOnly);
    assert!(
        own_view
            .inspected_cards
            .unwrap()
            .iter()
            .any(|card| card.id == secret)
    );
    assert!(game.observe(player.other()).hand_choice.is_none());
    act(&mut game, Action::ChooseHandCard { object: None });
    drain(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
    assert_eq!(game.counter_count(adjudicator, CounterKind::PlusOne), 1);
    assert_eq!(game.object_view(seven).zone, Zone::Battlefield);
}

fn choose_cost_clause(game: &mut Game, predicate: impl Fn(CastCostClause) -> bool) {
    let clause = game
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|row| !row.paid && predicate(row.clause))
        .unwrap() as u16;
    act(game, Action::ChooseCastCost { clause });
}

#[test]
fn scotty_beams_private_mind_meld_past_marooned_and_exiles_on_stack_exit() {
    let player = Player::P0;
    let mut game = scenario(player);
    let scotty = put(
        &mut game,
        player,
        "Scotty, Miracle Worker",
        Zone::Battlefield,
    );
    let enchanted = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let returner = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let marooned = put(&mut game, player.other(), "Marooned", Zone::Battlefield);
    game.state.objects[marooned as usize].attached_to = Some(game.object_ref(enchanted));
    let own_top = put(&mut game, player, "Shock", Zone::Library);
    let their_first = put(&mut game, player.other(), "Forest", Zone::Library);
    let their_second = put(&mut game, player.other(), "Bear Cub", Zone::Library);
    let spell = put(&mut game, player, "Vulcan Mind-Meld", Zone::Graveyard);
    let action = Action::AnnounceBeamMeUp {
        card: spell,
        face: CastFace::Normal,
        cost: ManaCost {
            generic: 0,
            colored: [0, 1, 0, 0, 0, 0],
        },
        flash_surcharge: false,
        provider: Some(game.object_ref(scotty)),
    };
    assert!(game.legal_actions().contains(&action));
    act(&mut game, action);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(player.other()),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    act(&mut game, Action::BeginOrderedCosts);
    choose_cost_clause(&mut game, |row| row == CastCostClause::Mana);
    pay(&mut game);
    choose_cost_clause(&mut game, |row| {
        matches!(row, CastCostClause::BeamMeUpReturn(_, _))
    });
    let blocked = game.object_ref(enchanted);
    reject(&mut game, Action::PayBeamMeUpReturn { object: blocked });
    let source = game.object_ref(scotty);
    reject(&mut game, Action::PayBeamMeUpReturn { object: source });
    let chosen = game.object_ref(returner);
    let payment = Action::PayBeamMeUpReturn { object: chosen };
    let checkpoint = game.snapshot();
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == payment)
        .unwrap();
    let mut indexed = game.clone();
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    act(&mut game, payment.clone());
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&checkpoint).unwrap();
    act(&mut indexed, payment);
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.object_view(returner).zone, Zone::Hand);
    assert_eq!(game.object_view(enchanted).zone, Zone::Battlefield);
    assert_eq!(game.object_view(spell).zone, Zone::Stack);

    let mut countered = game.clone();
    countered.move_object(spell, Zone::Graveyard);
    assert_eq!(countered.object_view(spell).zone, Zone::Exile);
    drain(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Exile);
    for (card, beneficiary) in [
        (own_top, player.other()),
        (their_first, player),
        (their_second, player),
    ] {
        assert_eq!(game.object_view(card).zone, Zone::Exile);
        assert!(
            game.observe(beneficiary)
                .objects
                .iter()
                .any(|o| o.id == card)
        );
        assert!(
            !game
                .observe(beneficiary.other())
                .objects
                .iter()
                .any(|o| o.id == card)
        );
    }
}

fn cast_measure(game: &mut Game, target: CardId) -> CardId {
    let saga = put(game, Player::P0, "The Measure of a Man", Zone::Hand);
    cast(game, saga, None);
    for _ in 0..24 {
        if game.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(target),
        }) {
            break;
        }
        choose(game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    act(
        game,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    if game.legal_actions().contains(&Action::FinishStackTargets) {
        act(game, Action::FinishStackTargets);
    }
    drain(game);
    assert_eq!(game.object_view(target).zone, Zone::Exile);
    saga
}

fn dathon_lore(game: &mut Game, dathon: CardId, saga: CardId) {
    act(
        game,
        Action::AnnounceActivate {
            card: dathon,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(
        game,
        Action::ChooseTarget {
            target: Target::Card(saga),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
    pay(game);
    for _ in 0..16 {
        if matches!(
            game.state.pending,
            Pending::TargetExistingCounterKindToPut { .. }
        ) {
            break;
        }
        choose(game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    act(
        game,
        Action::ChooseTargetExistingCounterKindToPut {
            counter: CounterKind::Lore,
        },
    );
    drain(game);
}

#[test]
fn dathon_advances_only_the_selected_measure_saga_and_its_exact_linked_exile() {
    let player = Player::P0;
    let mut game = scenario(player);
    let artifact = put(&mut game, player.other(), "Tricorder", Zone::Battlefield);
    let creature = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let first = cast_measure(&mut game, artifact);
    let second = cast_measure(&mut game, creature);
    let dathon = put(
        &mut game,
        player,
        "Dathon, His Heart Open",
        Zone::Battlefield,
    );
    dathon_lore(&mut game, dathon, first);
    dathon_lore(&mut game, dathon, second);
    assert_eq!(game.counter_count(first, CounterKind::Lore), 2);
    assert_eq!(game.counter_count(second, CounterKind::Lore), 2);
    assert_eq!(game.object_view(artifact).zone, Zone::Exile);
    assert_eq!(game.object_view(creature).zone, Zone::Exile);

    act(
        &mut game,
        Action::AnnounceActivate {
            card: dathon,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    let checkpoint = game.snapshot();
    let select_first = Action::ChooseTarget {
        target: Target::Card(first),
    };
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == select_first)
        .unwrap();
    let mut indexed = game.clone();
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    act(&mut game, select_first.clone());
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&checkpoint).unwrap();
    act(&mut indexed, select_first);
    assert_eq!(game.snapshot(), indexed.snapshot());
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    pay(&mut game);
    for _ in 0..16 {
        if matches!(
            game.state.pending,
            Pending::TargetExistingCounterKindToPut { .. }
        ) {
            break;
        }
        choose(&mut game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    act(
        &mut game,
        Action::ChooseTargetExistingCounterKindToPut {
            counter: CounterKind::Lore,
        },
    );
    drain(&mut game);
    assert_eq!(game.object_view(artifact).zone, Zone::Battlefield);
    assert_eq!(game.object_view(artifact).controller, player.other());
    assert_eq!(game.object_view(creature).zone, Zone::Exile);
    assert_eq!(game.state.players[player.index()].life, 22);

    indexed.restore(&checkpoint).unwrap();
    act(
        &mut indexed,
        Action::ChooseTarget {
            target: Target::Card(second),
        },
    );
    if indexed.legal_actions().contains(&Action::FinishTargets) {
        act(&mut indexed, Action::FinishTargets);
    }
    pay(&mut indexed);
    for _ in 0..16 {
        if matches!(
            indexed.state.pending,
            Pending::TargetExistingCounterKindToPut { .. }
        ) {
            break;
        }
        choose(&mut indexed, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    act(
        &mut indexed,
        Action::ChooseTargetExistingCounterKindToPut {
            counter: CounterKind::Lore,
        },
    );
    drain(&mut indexed);
    assert_eq!(indexed.object_view(creature).zone, Zone::Battlefield);
    assert_eq!(indexed.object_view(creature).controller, player.other());
    assert_eq!(indexed.object_view(artifact).zone, Zone::Exile);
}
