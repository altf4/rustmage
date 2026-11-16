//! Further disjoint Star Trek E–M card scenarios.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn owned(game: &mut Game, player: Player, name: &str, zone: Zone) -> CardId {
    spawn(game, player, Card::from_name(name).unwrap(), zone)
}

fn drain(game: &mut Game) {
    for _ in 0..48 {
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
    panic!("resolution budget");
}

fn target_when_available(game: &mut Game, target: Target) {
    for _ in 0..10 {
        if game
            .legal_actions()
            .contains(&Action::ChooseTarget { target })
        {
            act(game, Action::ChooseTarget { target });
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("target choice absent");
}

#[test]
fn la_an_needs_formidable_total_power_when_attackers_are_declared() {
    let player = Player::P0;
    for formidable in [false, true] {
        let mut game = scenario(player);
        owned(
            &mut game,
            player,
            "La'An Noonien-Singh, Security",
            Zone::Battlefield,
        );
        let attacker = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let opposing = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        if formidable {
            put(&mut game, player, "Hill Giant", Zone::Battlefield);
        }
        game.state.objects[attacker as usize].summoning_sick = false;
        game.state.step = Step::Attackers;
        game.state.pending = Pending::Attackers;
        act(&mut game, Action::Attack { card: attacker });
        act(&mut game, Action::FinishAttackers);
        if formidable {
            target_when_available(&mut game, Target::Card(opposing));
            drain(&mut game);
            assert!(game.keywords(opposing).contains(Keywords::CANT_BLOCK));
        } else {
            drain(&mut game);
            assert!(!game.keywords(opposing).contains(Keywords::CANT_BLOCK));
        }
    }
}

#[test]
fn l_rell_creates_hasty_klingon_until_next_end_and_surveils_after_paid_sacrifice() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = owned(
        &mut game,
        player,
        "L'Rell, Fierce Visionary",
        Zone::Battlefield,
    );
    game.begin_step(Step::BeginCombat);
    game.prepare_triggers();
    drain(&mut game);
    let token = (0..game.state.objects.len())
        .map(|id| id as CardId)
        .find(|&id| {
            game.object_view(id).zone == Zone::Battlefield
                && game.object_view(id).card == Card::from_catalogue_id(6010)
        })
        .unwrap();
    assert!(game.keywords(token).contains(Keywords::HASTE));
    game.begin_step(Step::End);
    game.prepare_triggers();
    drain(&mut game);
    assert_ne!(game.object_view(token).zone, Zone::Battlefield);
    assert_eq!(game.object_view(source).zone, Zone::Battlefield);

    let material = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let top = put(&mut game, player, "Island", Zone::Library);
    act(
        &mut game,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    let object = game.object_ref(material);
    act(&mut game, Action::ChooseAdditionalCost { object });
    pay(&mut game);
    for _ in 0..10 {
        if game.observe(player).library_choice.is_some() {
            break;
        }
        choose(&mut game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    assert_eq!(game.object_view(material).zone, Zone::Graveyard);
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: top,
            destination: LibraryDestination::Graveyard,
        },
    );
    drain(&mut game);
    assert_eq!(game.object_view(top).zone, Zone::Graveyard);
}

#[test]
fn locutus_boosts_only_own_artifact_creatures_and_discards_for_damage_and_life() {
    let player = Player::P0;
    let mut game = scenario(player);
    let own_artifact = put(&mut game, player, "Gingerbrute", Zone::Battlefield);
    let own_plain = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let opponent_artifact = put(&mut game, player.other(), "Gingerbrute", Zone::Battlefield);
    let locutus = owned(
        &mut game,
        player,
        "Locutus, the One Who Speaks",
        Zone::Battlefield,
    );
    assert_eq!(stats(&game, locutus), (2, 2));
    assert_eq!(stats(&game, own_artifact), (2, 2));
    assert_eq!(stats(&game, own_plain), (2, 2));
    assert_eq!(stats(&game, opponent_artifact), (1, 1));
    game.move_object(locutus, Zone::Graveyard);
    assert_eq!(stats(&game, own_artifact), (1, 1));

    let mut activated = scenario(player);
    let victim = put(
        &mut activated,
        player.other(),
        "Bear Cub",
        Zone::Battlefield,
    );
    let source = owned(
        &mut activated,
        player,
        "Locutus, the One Who Speaks",
        Zone::Hand,
    );
    activated.state.players[player.index()].mana = [1, 0, 1, 0, 0, 0];
    act(
        &mut activated,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(
        &mut activated,
        Action::ChooseTarget {
            target: Target::Card(victim),
        },
    );
    pay(&mut activated);
    drain(&mut activated);
    assert_eq!(activated.object_view(source).zone, Zone::Graveyard);
    assert_eq!(activated.object_view(victim).zone, Zone::Graveyard);
    assert_eq!(activated.state.players[player.index()].life, 22);
}

#[test]
fn make_it_so_untaps_and_draws_for_target_or_creates_officer_and_draws() {
    let player = Player::P0;
    let mut first = scenario(player);
    let land = put(&mut first, player, "Forest", Zone::Battlefield);
    first.tap_objects(&[land]);
    let spell = owned(&mut first, player, "Make It So", Zone::Hand);
    act(&mut first, Action::AnnounceCast { card: spell });
    act(&mut first, Action::ChooseMode { mode: 0 });
    act(
        &mut first,
        Action::ChooseTarget {
            target: Target::Card(land),
        },
    );
    act(
        &mut first,
        Action::ChooseTarget {
            target: Target::Player(player.other()),
        },
    );
    pay(&mut first);
    drain(&mut first);
    assert!(!first.object_view(land).tapped);
    assert_eq!(
        first.state.players[player.other().index()].cards_drawn_this_turn,
        2
    );
    assert_eq!(first.state.players[player.index()].cards_drawn_this_turn, 0);

    let mut second = scenario(player);
    let spell = owned(&mut second, player, "Make It So", Zone::Hand);
    act(&mut second, Action::AnnounceCast { card: spell });
    act(&mut second, Action::ChooseMode { mode: 1 });
    pay(&mut second);
    drain(&mut second);
    assert_eq!(
        second.state.players[player.index()].cards_drawn_this_turn,
        2
    );
    assert!(second.state.objects.iter().any(|object| {
        object.zone == Zone::Battlefield
            && object.controller == player
            && object.card == Card::from_catalogue_id(6008)
    }));
}

const NONARTIFACT_ACTIVATION: rustmage_core::cards::authored::CardRules =
    rustmage_core::cards::authored::CardRules {
        effect: Effect::Creature(2, 2),
        activated: &[ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                generic: 1,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
            sorcery: false,
            once_per_turn: false,
        }],
        ..rustmage_core::cards::authored::CardRules::EMPTY
    };

#[test]
fn isolinear_mana_funds_only_artifact_spells_and_artifact_source_abilities() {
    let _fixture = fixture_rules::activate(&[(Card::GrizzlyBears, &NONARTIFACT_ACTIVATION)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let processor = owned(&mut game, player, "Isolinear Processor", Zone::Battlefield);
    let bear = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let bow = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let artifact_spell = put(&mut game, player, "Lavaspur Boots", Zone::Hand);
    game.state.players[player.index()].mana = [0; 6];
    act(
        &mut game,
        Action::ActivateMana {
            card: processor,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: [0, 0, 0, 0, 0, 1],
        },
    );
    assert_eq!(
        game.observe(player).mana_lots[0].restriction,
        ManaRestrictionView::ArtifactSpellOrArtifactActivation
    );
    reject(
        &mut game,
        Action::AnnounceActivate {
            card: bear,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    let checkpoint = game.snapshot();
    let mut casting = game.clone();
    cast(&mut casting, artifact_spell, None);
    drain(&mut casting);
    assert_eq!(casting.object_view(artifact_spell).zone, Zone::Battlefield);
    casting.restore(&checkpoint).unwrap();
    assert_eq!(casting.snapshot(), checkpoint);
    act(
        &mut game,
        Action::AnnounceActivate {
            card: bow,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    pay(&mut game);
    drain(&mut game);
    assert_eq!(
        game.object_view(bow).attached_to,
        Some(game.object_ref(bear))
    );
}

#[test]
fn isolinear_paid_loot_draws_then_discards_exact_private_card() {
    let player = Player::P0;
    let mut game = scenario(player);
    let processor = owned(&mut game, player, "Isolinear Processor", Zone::Battlefield);
    let top = put(&mut game, player, "Island", Zone::Library);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
    act(
        &mut game,
        Action::AnnounceActivate {
            card: processor,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    pay(&mut game);
    for _ in 0..12 {
        if game
            .legal_actions()
            .contains(&Action::Discard { card: top })
        {
            break;
        }
        choose(&mut game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    assert_eq!(game.object_view(top).zone, Zone::Hand);
    act(&mut game, Action::Discard { card: top });
    drain(&mut game);
    assert_eq!(game.object_view(top).zone, Zone::Graveyard);
    assert!(game.state.objects[processor as usize].tapped);
}
