//! Exact relational targets, subtype-count mana, spell history, and attachment grantors.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules, one};
use rustmage_core::{cards::*, engine::*};
use support::*;

const TAP: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost {
        tap: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[],
    sorcery: false,
    once_per_turn: false,
};
const TUNNEL: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 4,
            mana_component: true,
            ..TAP.cost
        },
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Grouped {
                base: &TargetRule::creature_you_control(),
                constraint: TargetGroupConstraint::SharedCreatureType,
            },
            min: 2,
            max: 2,
        }),
        program: &[Instruction::GrantKeywords {
            subject: Subject::Targets,
            keywords: Keywords::CANT_BE_BLOCKED,
        }],
        ..TAP
    }],
    ..CardRules::EMPTY
};
const TILE: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        program: &[Instruction::AddMana(ManaProduction::ChoiceStat {
            colors: 31,
            amount: &StatValue::LargestControlledCreatureTypeGroup,
        })],
        ..TAP
    }],
    ..CardRules::EMPTY
};
const REDUCTION: SpellReduction = SpellReduction {
    source_only: false,
    filter: CastFilter::FirstQualifiedCreature(CreatureCastQualifier::FlyingNonLemur),
    amount: SpellReductionAmount::Fixed(1),
    turn: CostTurn::Own,
    guard: StepGuard::Always,
};
const FLYER_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::CostReduction(0, REDUCTION)],
}];
const FLYER: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    static_groups: FLYER_GROUPS,
    extras: &CardExtras {
        static_ability_groups: FLYER_GROUPS,
        cost_reductions: &[REDUCTION],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const BOOMERANG: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::AttachmentActivation {
            ability: 0,
            tap_attachment: false,
        }],
    }],
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            ..TAP.cost
        },
        targets: one(TargetRule::Creature),
        program: &[Instruction::Tap, Instruction::ReturnAttachmentGrantorToHand],
        ..TAP
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

fn select_target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

#[test]
fn two_battlefield_targets_share_an_effective_type_and_departure_uses_exact_facts() {
    let _rules = fixture_rules::activate(&[(named("Rumble Arena"), &TUNNEL)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Rumble Arena", Zone::Battlefield);
    let survivor = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let departing = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let alone = put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
    let foreign = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let announce = Action::AnnounceActivate {
        card: source,
        ability: 0,
        counter_payment: Counters::default(),
    };
    reject(&mut game, announce.clone());

    let old = game.object_ref(departing);
    game.add_characteristics_to(
        &[old],
        CharacteristicAddition {
            card_types: 0,
            creature_types: &["Bear"],
            colors: 0,
            duration: AnimationDuration::EndOfTurn,
        },
    );
    assert!(game.has_subtype(departing, "Bear"));
    act(&mut game, announce);
    reject(&mut game, select_target(alone));
    reject(&mut game, select_target(foreign));
    act(&mut game, select_target(survivor));
    reject(&mut game, select_target(survivor));
    act(&mut game, select_target(departing));
    pay(&mut game);
    assert!(game.object_view(source).tapped);

    let checkpoint = game.snapshot();
    game.move_object(departing, Zone::Hand);
    game.move_object(departing, Zone::Battlefield);
    assert_ne!(game.object_ref(departing), old);
    assert!(!game.has_subtype(departing, "Bear"));
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == Action::Pass)
        .unwrap();
    let mut indexed = game.clone();
    act(&mut game, Action::Pass);
    indexed.apply_index(player, revision, index).unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    settle(&mut game);
    assert!(
        game.object_view(survivor)
            .keywords
            .contains(Keywords::CANT_BE_BLOCKED)
    );
    assert!(
        !game
            .object_view(departing)
            .keywords
            .contains(Keywords::CANT_BE_BLOCKED)
    );

    game.restore(&checkpoint).unwrap();
    settle(&mut game);
    assert!(
        game.object_view(departing)
            .keywords
            .contains(Keywords::CANT_BE_BLOCKED)
    );
}

#[test]
fn largest_shared_subtype_supplies_one_color_in_live_and_projected_payments() {
    let _rules = fixture_rules::activate(&[(named("Short Bow"), &TILE)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let source_ref = game.object_ref(source);
    let amount = |game: &Game| {
        game.stat_value(
            StatValue::LargestControlledCreatureTypeGroup,
            source_ref,
            player,
        )
    };
    assert_eq!(amount(&game), 0);
    put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let cub = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let changeling = put(&mut game, player, "Mutable Explorer", Zone::Battlefield);
    put(&mut game, player, "Hill Giant", Zone::Battlefield);
    put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    assert!(game.has_subtype(changeling, "Bear"));
    assert_eq!(amount(&game), 3);

    let spell = put(&mut game, player, "Water Tribe Captain", Zone::Hand);
    game.state.players[player.index()].mana = [0; 6];
    let announce = Action::AnnounceCast { card: spell };
    let mut reduced = game.clone();
    reduced.move_object(cub, Zone::Graveyard);
    assert_eq!(amount(&reduced), 2);
    reject(&mut reduced, announce.clone());
    act(&mut game, announce);
    let white = [3, 0, 0, 0, 0, 0];
    let activate = |production| Action::ActivateMana {
        card: source,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production,
    };
    reject(&mut game, activate([2, 1, 0, 0, 0, 0]));
    let direct = activate(white);
    if game.legal_actions().contains(&direct) {
        act(&mut game, direct);
    } else {
        act(&mut game, activate([0; 6]));
        reject(
            &mut game,
            Action::ChooseMana {
                production: [2, 1, 0, 0, 0, 0],
            },
        );
        act(&mut game, Action::ChooseMana { production: white });
    }
    assert!(game.object_view(source).tapped);
    assert_eq!(game.state.players[player.index()].mana, white);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Battlefield);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
}

#[test]
fn only_a_committed_effective_non_lemur_flyer_consumes_the_own_turn_discount() {
    let _rules = fixture_rules::activate(&[(named("Wargling"), &FLYER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let provider = put(&mut game, player, "Wargling", Zone::Battlefield);
    let grounded = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    cast(&mut game, grounded, None);
    settle(&mut game);
    let leyline = put(&mut game, player, "Leyline of Transformation", Zone::Hand);
    cast(&mut game, leyline, None);
    pass_pair(&mut game);
    act(
        &mut game,
        Action::ChooseEntryValue {
            value: EntryChoiceValue::CreatureType(CreatureType::from_name("Lemur").unwrap()),
        },
    );
    let lemur = put(&mut game, player, "Glider Kids", Zone::Hand);
    assert!(game.has_subtype(lemur, "Lemur"));
    act(&mut game, Action::AnnounceCast { card: lemur });
    assert_eq!(game.observe(player).payment_request.unwrap().generic, 2);
    pay(&mut game);
    assert_eq!(
        game.state.players[player.index()]
            .turn_facts
            .flying_non_lemur_creature_casts,
        0
    );
    game.move_object(leyline, Zone::Graveyard);
    pass_pair(&mut game);
    pass_pair(&mut game);
    let top = *game.state.players[player.index()].library.last().unwrap();
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: top,
            destination: LibraryDestination::Top,
        },
    );

    let mut off_turn = game.clone();
    off_turn.state.active = player.other();
    off_turn.state.priority = player;
    off_turn.state.players[player.index()].mana = [1, 0, 0, 0, 0, 0];
    let flash = put(&mut off_turn, player, "Doorkeeper Thrull", Zone::Hand);
    reject(&mut off_turn, Action::AnnounceCast { card: flash });

    let angel = put(&mut game, player, "Serra Angel", Zone::Hand);
    game.state.players[player.index()].mana = [2, 0, 0, 0, 0, 2];
    let announce = Action::AnnounceCast { card: angel };
    let mut absent = game.clone();
    absent.move_object(provider, Zone::Graveyard);
    reject(&mut absent, announce.clone());
    act(&mut game, announce);
    let observation = game.observe(player);
    assert_eq!(observation.payment_request.unwrap().generic, 2);
    assert_eq!(
        observation.turn_facts.unwrap()[player.index()].flying_non_lemur_creature_casts,
        0
    );
    pay(&mut game);
    assert_eq!(
        game.state.players[player.index()]
            .turn_facts
            .flying_non_lemur_creature_casts,
        1
    );
    act(&mut game, Action::Pass);
    let scatter = put(&mut game, player.other(), "Essence Scatter", Zone::Hand);
    cast(&mut game, scatter, Some(Target::Spell(angel)));
    settle(&mut game);
    assert_eq!(game.object_view(angel).zone, Zone::Graveyard);

    let next = put(&mut game, player, "Glider Kids", Zone::Hand);
    game.state.players[player.index()].mana = [1, 0, 0, 0, 0, 1];
    let announce = Action::AnnounceCast { card: next };
    reject(&mut game, announce.clone());
    game.state.players[player.index()].mana[5] = 2;
    act(&mut game, announce);
    assert_eq!(game.observe(player).payment_request.unwrap().generic, 2);
}

#[test]
fn a_paid_attached_activation_returns_only_its_exact_grantor_after_a_legal_target() {
    let _rules = fixture_rules::activate(&[(named("Short Bow"), &BOOMERANG)]);
    let player = Player::P1;
    let owner = player.other();
    let mut game = scenario(player);
    let equipment = put(&mut game, owner, "Short Bow", Zone::Battlefield);
    let other_equipment = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let bearer = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let recipient = put(&mut game, owner, "Hill Giant", Zone::Battlefield);
    let target = put(&mut game, owner, "Grizzly Bears", Zone::Battlefield);
    let grantor = game.object_ref(equipment);
    let bearer_ref = game.object_ref(bearer);
    let recipient_ref = game.object_ref(recipient);
    assert!(game.attach_equipment(grantor, bearer_ref));
    assert!(game.attach_equipment(game.object_ref(other_equipment), bearer_ref));
    let activation = Action::AnnounceAttachedAbility {
        grantor,
        ability: 0,
    };
    game.state.objects[bearer as usize].summoning_sick = true;
    reject(&mut game, activation.clone());
    game.state.objects[bearer as usize].summoning_sick = false;
    act(&mut game, activation);
    act(&mut game, select_target(target));
    assert_eq!(game.observe(player).payment_request.unwrap().generic, 1);
    pay(&mut game);
    assert!(game.object_view(bearer).tapped);
    assert!(!game.object_view(equipment).tapped);

    let mut blinked = game.clone();
    blinked.move_object(equipment, Zone::Exile);
    blinked.move_object(equipment, Zone::Battlefield);
    settle(&mut blinked);
    assert!(blinked.object_view(target).tapped);
    assert_ne!(blinked.object_ref(equipment), grantor);
    assert_eq!(blinked.object_view(equipment).zone, Zone::Battlefield);

    let mut fizzled = game.clone();
    fizzled.move_object(target, Zone::Hand);
    settle(&mut fizzled);
    assert_eq!(fizzled.object_view(equipment).zone, Zone::Battlefield);

    assert!(game.attach_equipment(grantor, recipient_ref));
    settle(&mut game);
    assert!(game.object_view(target).tapped);
    assert_eq!(game.object_view(equipment).zone, Zone::Hand);
    assert_eq!(game.object_view(equipment).owner, owner);
    assert_eq!(game.object_view(other_equipment).zone, Zone::Battlefield);
}
