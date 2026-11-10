//! Live library-top play and private visibility granted by a resolved attack trigger.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const ATTACK_GRANT: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[Instruction::GrantLibraryTopLifePlayUntilEndTurn],
    }],
    ..CardRules::EMPTY
};

const ORDINARY_PERMISSION: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::CastFromLibraryTop(CastFilter::Any)],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::CastSubtypesFromLibraryTopWithCounter(&["Elf"])],
        },
    ],
    ..CardRules::EMPTY
};

const TAX: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::OpponentNoncreatureSpellTax(2)],
    }],
    ..CardRules::EMPTY
};

fn definition(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn attack(game: &mut Game) -> CardId {
    let attacker = put(game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(game, Action::Attack { card: attacker });
    act(game, Action::FinishAttackers);
    assert!(game.state.stack.last().is_some_and(|item| {
        item.source.is_some_and(|source| source.id == attacker)
            && matches!(item.kind, StackKind::TriggeredAbility { .. })
    }));
    attacker
}

fn advance_to(game: &mut Game, step: Step) {
    for _ in 0..40 {
        if game.state.step == step && game.state.pending == Pending::Priority {
            return;
        }
        choose(game, |a| {
            matches!(
                a,
                Action::Pass | Action::FinishAttackers | Action::FinishBlockers
            )
        });
    }
    panic!("expected {step:?} within the turn");
}

fn announce(card: CardId, face: CastFace) -> Action {
    Action::AnnounceLibraryLife {
        card,
        face,
        flash_surcharge: false,
    }
}

fn finish_costs(game: &mut Game) {
    for _ in 0..8 {
        match game.state.pending {
            Pending::Priority => return,
            Pending::CastCosts { .. } => choose(game, |action| {
                matches!(action, Action::ChooseCastCost { .. })
            }),
            Pending::PayCosts { .. } => pay(game),
            Pending::Targets { .. } => act(game, Action::FinishTargets),
            ref pending => panic!("unexpected cast choice {pending:?}"),
        }
    }
    panic!("cast did not finish");
}

fn visible_top(game: &Game, viewer: Player) -> Option<ObjectRef> {
    game.observe(viewer).known_library_top[Player::P0.index()].map(|top| top.object)
}

#[test]
fn the_resolved_attack_grants_only_its_controller_live_visibility_through_payment() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &ATTACK_GRANT)]);
    let mut game = scenario(Player::P0);
    let next = put(&mut game, Player::P0, "Opt", Zone::Library);
    let top = put(&mut game, Player::P0, "Shock", Zone::Library);
    let removal = put(&mut game, Player::P1, "Shock", Zone::Hand);
    let attacker = attack(&mut game);
    let top_ref = game.object_ref(top);
    assert_eq!(visible_top(&game, Player::P0), None);
    reject(&mut game, announce(top, CastFace::Normal));
    act(&mut game, Action::Pass);
    game.state.players[Player::P1.index()].mana = [0, 0, 0, 1, 0, 0];
    cast(&mut game, removal, Some(Target::Card(attacker)));
    settle(&mut game);
    assert_eq!(game.object_view(attacker).zone, Zone::Graveyard);
    assert_eq!(visible_top(&game, Player::P0), Some(top_ref));
    assert_eq!(visible_top(&game, Player::P1), None);
    game.state.players[0].mana = [0; 6];
    reject(&mut game, announce(next, CastFace::Normal));
    reject(&mut game, Action::AnnounceCast { card: top });

    let before = game.snapshot();
    let action = announce(top, CastFace::Normal);
    let mut indexed = game.clone();
    let index = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    indexed
        .apply_index(Player::P0, indexed.revision(), index)
        .unwrap();
    act(&mut game, action.clone());
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.object_view(top).zone, Zone::Stack);
    assert_eq!(visible_top(&game, Player::P0), None);
    assert_eq!(visible_top(&game, Player::P1), None);
    assert_eq!(game.state.players[0].life, 20);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(next),
        },
    );
    let during = game.snapshot();

    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(Player::P1),
        },
    );
    finish_costs(&mut game);
    let cast = game.state.stack.last().unwrap().cast.unwrap();
    assert_eq!(cast.origin, Zone::Library);
    assert_eq!(cast.method, CastMethod::LibraryLife);
    assert_eq!(game.state.players[0].life, 19);
    assert_eq!(visible_top(&game, Player::P0), Some(game.object_ref(next)));
    assert_eq!(visible_top(&game, Player::P1), None);

    game.restore(&during).unwrap();
    assert_eq!(visible_top(&game, Player::P0), None);
    game.restore(&before).unwrap();
    assert_eq!(visible_top(&game, Player::P0), Some(top_ref));
    assert!(game.legal_actions().contains(&action));
}

#[test]
fn ordinary_library_casting_and_its_entry_counter_keep_separate_routes() {
    let provider = definition("Phyrexian Arena");
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &ATTACK_GRANT),
        (provider, &ORDINARY_PERMISSION),
    ]);
    let mut game = scenario(Player::P0);
    spawn(&mut game, Player::P0, provider, Zone::Battlefield);
    let elf = put(&mut game, Player::P0, "Llanowar Elves", Zone::Library);
    attack(&mut game);
    settle(&mut game);
    reject(&mut game, announce(elf, CastFace::Normal));
    advance_to(&mut game, Step::Main2);
    game.state.players[0].mana = [0, 0, 0, 0, 1, 0];
    let before = game.snapshot();
    let ordinary = Action::AnnounceCast { card: elf };
    let alternative = announce(elf, CastFace::Normal);
    assert!(game.legal_actions().contains(&ordinary));
    assert!(game.legal_actions().contains(&alternative));
    let mana = game.state.players[0].mana;
    act(&mut game, alternative);
    assert!(
        !game
            .legal_actions()
            .iter()
            .any(|action| { matches!(action, Action::ChooseLibraryCastPermission { .. }) })
    );
    finish_costs(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[0].life, 19);
    assert_eq!(game.state.players[0].mana, mana);
    assert_eq!(game.counter_count(elf, CounterKind::PlusOne), 0);

    game.restore(&before).unwrap();
    act(&mut game, ordinary);
    act(
        &mut game,
        Action::ChooseLibraryCastPermission { with_counter: true },
    );
    finish_costs(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[0].life, 20);
    assert_eq!(game.state.players[0].mana, [0; 6]);
    assert_eq!(game.counter_count(elf, CounterKind::PlusOne), 1);
}

#[test]
fn the_chosen_spell_face_determines_life_and_x_is_fixed_at_zero() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &ATTACK_GRANT)]);
    let mut game = scenario(Player::P0);
    let elemental = put(&mut game, Player::P0, "Hearth Elemental", Zone::Library);
    let front = definition("Hearth Elemental");
    let adventure = front.adventure_spell().unwrap();
    assert_ne!(front.fixed_mana_value(), adventure.fixed_mana_value());
    attack(&mut game);
    settle(&mut game);
    advance_to(&mut game, Step::Main2);
    game.state.players[0].mana = [0; 6];
    let saved = game.snapshot();
    for (face, expected) in [(CastFace::Normal, front), (CastFace::Adventure, adventure)] {
        game.restore(&saved).unwrap();
        act(&mut game, announce(elemental, face));
        finish_costs(&mut game);
        let item = game.state.stack.last().unwrap();
        assert_eq!(item.source.unwrap().card, expected);
        assert_eq!(item.cast.unwrap().face, face);
        assert_eq!(
            game.state.players[0].life,
            20 - i128::from(expected.fixed_mana_value())
        );
    }

    game.restore(&saved).unwrap();
    let scourge = put(&mut game, Player::P0, "Wildwood Scourge", Zone::Library);
    act(&mut game, announce(scourge, CastFace::Normal));
    assert!(game.x_choice().is_none());
    reject(&mut game, Action::ChooseX { value: 1 });
    finish_costs(&mut game);
    assert_eq!(game.state.stack.last().unwrap().x, Some(0));
    assert_eq!(game.state.players[0].life, 19);
}

#[test]
fn lands_use_the_ordinary_allowance_and_top_play_expires_in_cleanup() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &ATTACK_GRANT)]);
    let mut game = scenario(Player::P0);
    let next = put(&mut game, Player::P0, "Island", Zone::Library);
    let top = put(&mut game, Player::P0, "Mountain", Zone::Library);
    attack(&mut game);
    settle(&mut game);
    reject(&mut game, Action::PlayLand { card: top });
    advance_to(&mut game, Step::Main2);
    act(&mut game, Action::PlayLand { card: top });
    assert_eq!(game.object_view(top).zone, Zone::Battlefield);
    assert_eq!(game.state.players[0].life, 20);
    assert_eq!(visible_top(&game, Player::P0), Some(game.object_ref(next)));
    reject(&mut game, Action::PlayLand { card: next });

    let turn = game.state.turn;
    for _ in 0..12 {
        if game.state.turn != turn {
            break;
        }
        act(&mut game, Action::Pass);
    }
    assert_ne!(game.state.turn, turn);
    assert_eq!(visible_top(&game, Player::P0), None);
    assert_eq!(visible_top(&game, Player::P1), None);
    assert!(!game.can_play_static_land(Player::P0, next));
}

#[test]
fn failed_cast_restores_the_exact_top_and_preserves_its_public_announcement() {
    let tax = definition("Short Bow");
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &ATTACK_GRANT), (tax, &TAX)]);
    let mut game = scenario(Player::P0);
    spawn(&mut game, Player::P1, tax, Zone::Battlefield);
    let next = put(&mut game, Player::P0, "Mountain", Zone::Library);
    let top = put(&mut game, Player::P0, "Think Twice", Zone::Library);
    attack(&mut game);
    settle(&mut game);
    let action = announce(top, CastFace::Normal);
    game.state.players[0].mana = [0; 6];
    reject(&mut game, action.clone());
    game.state.players[0].mana[5] = 2;
    game.state.players[0].life = 1;
    reject(&mut game, action.clone());
    game.state.players[0].life = 20;

    // A potential foreign entry choice makes the announcement own a rollback
    // checkpoint. Branch after announcing and choosing the taxed mana payment.
    let link = put(&mut game, Player::P0, "Llanowar Elves", Zone::Battlefield);
    let returnee = put(&mut game, Player::P1, "Hill Giant", Zone::Exile);
    game.state.exile_links.push(ExileLink {
        ability_origin: None,
        source: game.object_ref(link),
        exiled: game.object_ref(returnee),
        return_zone: Zone::Battlefield,
    });
    let original = game.object_ref(top);
    let mut expected = game.clone();
    expected.remember_library_top(Player::P0, original);
    act(&mut game, action.clone());
    assert!(game.state.cast_rollback.is_some());
    assert!(matches!(game.state.pending, Pending::PayCosts { .. }));
    pay(&mut game);
    assert!(matches!(game.state.pending, Pending::CastCosts { .. }));
    assert_eq!(game.state.players[0].life, 20);
    assert_eq!(visible_top(&game, Player::P0), None);
    let mut committed = game.clone();
    finish_costs(&mut committed);
    assert_eq!(committed.state.players[0].life, 18);
    assert_eq!(committed.state.players[0].mana, [0; 6]);

    game.finish_cast_rollback(false);
    assert_eq!(game.snapshot(), expected.snapshot());
    assert_eq!(visible_top(&game, Player::P0), Some(original));
    assert_eq!(visible_top(&game, Player::P1), Some(original));
    reject(&mut game, announce(next, CastFace::Normal));
    assert!(game.legal_actions().contains(&action));
}
