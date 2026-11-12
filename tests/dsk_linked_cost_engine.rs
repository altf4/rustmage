//! Paid exile links, live exiled-card counts, and incarnation-bound in-place copies.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules, one};
use rustmage_core::{cards::*, engine::*};
use support::*;

const P: Player = Player::P0;
const OPPOSING_CREATURE: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 1,
    relation: 1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const LINKED_CAST: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: one(OPPOSING_CREATURE),
            program: &[Instruction::ExileAllTargetsLinkedToSource],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfLeaves,
            targets: TargetPlan::NONE,
            program: &[Instruction::ReturnLinkedExileToHand],
        },
    ],
    extras: &CardExtras {
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::ExileControlledForLinkedReturn,
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 1,
                nonland: false,
                other: false,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: None,
            },
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn begin_cast(game: &mut Game, spell: CardId) {
    act(game, Action::AnnounceCast { card: spell });
    if game.legal_actions().contains(&Action::BeginOrderedCosts) {
        act(game, Action::BeginOrderedCosts);
    }
    if matches!(game.state.pending, Pending::CastCosts { .. }) {
        let additional = game
            .state
            .announcing
            .as_ref()
            .unwrap()
            .cast_costs
            .as_ref()
            .unwrap()
            .clauses
            .iter()
            .position(|row| row.clause == CastCostClause::Additional)
            .unwrap();
        act(
            game,
            Action::ChooseCastCost {
                clause: additional as u16,
            },
        );
    }
    assert!(matches!(game.state.pending, Pending::AdditionalCost { .. }));
}

fn finish_cast(game: &mut Game) {
    for _ in 0..8 {
        match game.state.pending {
            Pending::Priority => return,
            Pending::CastCosts { .. } => choose(game, |action| {
                matches!(action, Action::ChooseCastCost { .. })
            }),
            Pending::PayCosts { .. } => pay(game),
            ref pending => panic!("unexpected cast choice {pending:?}"),
        }
    }
    panic!("cast did not finish");
}

fn enter_with_target(game: &mut Game, target: CardId) {
    pass_pair(game);
    act(
        game,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
}

fn links(game: &Game, source: ObjectRef) -> Vec<ObjectRef> {
    game.observe(P)
        .linked_exile
        .iter()
        .filter_map(|link| (link.source == source).then_some(link.exiled))
        .collect()
}

#[test]
fn cost_and_entry_links_follow_the_same_permanent_and_return_to_both_owners() {
    let fixture = card("Serra Angel");
    let _rules = fixture_rules::activate(&[(fixture, &LINKED_CAST)]);
    let mut game = scenario(P);
    let spell = spawn(&mut game, P, fixture, Zone::Hand);
    let cost = put(&mut game, P.other(), "Grizzly Bears", Zone::Battlefield);
    let victim = put(&mut game, P, "Hill Giant", Zone::Battlefield);
    game.change_controller(game.object_ref(cost), P);
    game.change_controller(game.object_ref(victim), P.other());

    begin_cast(&mut game, spell);
    let illegal = Action::ChooseAdditionalCost {
        object: game.object_ref(victim),
    };
    reject(&mut game, illegal);
    let pay_cost = Action::ChooseAdditionalCost {
        object: game.object_ref(cost),
    };
    let checkpoint = game.snapshot();
    let mut indexed = game.clone();
    let index = indexed
        .legal_actions()
        .iter()
        .position(|a| a == &pay_cost)
        .unwrap();
    indexed.apply_index(P, indexed.revision(), index).unwrap();
    act(&mut game, pay_cost.clone());
    assert_eq!(game.snapshot(), indexed.snapshot());
    game.restore(&checkpoint).unwrap();
    act(&mut game, pay_cost);
    assert_eq!(game.snapshot(), indexed.snapshot());
    finish_cast(&mut game);
    assert_eq!(game.object_view(cost).zone, Zone::Exile);
    assert!(links(&game, game.object_ref(spell)).is_empty());
    let cost_exiled = game.object_ref(cost);

    let mut early_departure = game.clone();
    enter_with_target(&mut early_departure, victim);
    early_departure.move_object(spell, Zone::Graveyard);
    early_departure.prepare_triggers();
    settle(&mut early_departure);
    assert_eq!(early_departure.object_view(cost).zone, Zone::Hand);
    assert_eq!(early_departure.object_view(victim).zone, Zone::Exile);

    enter_with_target(&mut game, victim);
    let permanent = game.object_ref(spell);
    assert_eq!(links(&game, permanent), [cost_exiled]);
    settle(&mut game);
    assert_eq!(
        links(&game, permanent),
        [cost_exiled, game.object_ref(victim)]
    );
    game.move_object(spell, Zone::Hand);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.object_view(cost).zone, Zone::Hand);
    assert_eq!(game.object_view(cost).owner, P.other());
    assert_eq!(game.object_view(victim).zone, Zone::Hand);
    assert_eq!(game.object_view(victim).owner, P);
}

#[test]
fn an_off_turn_caster_orders_both_cards_exiled_to_pay_the_meld_cost() {
    let fixture = card("Serra Angel");
    let _rules = fixture_rules::activate(&[(fixture, &LINKED_CAST)]);
    let mut game = scenario(P);
    let vanille = put(&mut game, P, "Vanille, Cheerful l'Cie", Zone::Battlefield);
    let fang = put(&mut game, P, "Fang, Fearless l'Cie", Zone::Battlefield);
    game.begin_step(Step::Main1);
    game.prepare_triggers();
    pass_pair(&mut game);
    choose(&mut game, |action| {
        matches!(action, Action::PayResolutionMana { .. })
    });
    let companion = game.object_ref(fang);
    act(&mut game, Action::ChooseEffectObject { object: companion });
    settle(&mut game);
    let melded = game.object_ref(vanille);
    assert_eq!(game.observe(P).melded_permanents[0].object, melded);

    put(&mut game, P, "High Fae Trickster", Zone::Battlefield);
    let victim = put(&mut game, P.other(), "Hill Giant", Zone::Battlefield);
    let spell = spawn(&mut game, P, fixture, Zone::Hand);
    game.state.active = P.other();
    game.state.priority = P.other();
    act(&mut game, Action::Pass);
    assert_eq!(game.actor(), Some(P));
    begin_cast(&mut game, spell);
    act(&mut game, Action::ChooseAdditionalCost { object: melded });

    assert_eq!(game.actor(), Some(P));
    assert_eq!(game.state.pending, Pending::MeldOrdering { player: P });
    assert!(game.observe(P.other()).meld_ordering_choice.is_none());
    let choice = game.observe(P).meld_ordering_choice.unwrap();
    assert_eq!(
        (choice.permanent, choice.destination),
        (melded, Zone::Exile)
    );
    let order = Action::ChooseMeldOrder {
        first: choice.components[1],
    };
    let checkpoint = game.snapshot();
    assert_eq!(game.apply(P.other(), order.clone()), Err(Error::WrongActor));
    assert_eq!(game.snapshot(), checkpoint);
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == order)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(P, indexed.revision(), index).unwrap();
    act(&mut game, order);
    assert_eq!(game.snapshot(), indexed.snapshot());
    let paid = [game.object_ref(fang), game.object_ref(vanille)];
    assert_eq!(game.observe(P).meld_departure_orders[0].cards, paid);
    assert!(
        paid.iter()
            .all(|object| game.object_view(object.card).zone == Zone::Exile)
    );
    finish_cast(&mut game);
    enter_with_target(&mut game, victim);
    assert_eq!(links(&game, game.object_ref(spell)), paid);
    settle(&mut game);
    game.move_object(spell, Zone::Graveyard);
    game.prepare_triggers();
    settle(&mut game);
    assert!(
        paid.iter()
            .all(|object| game.object_view(object.card).zone == Zone::Hand)
    );
    assert_eq!(game.object_view(victim).zone, Zone::Hand);
}

#[test]
fn a_spell_copy_and_stale_exile_cannot_create_links_to_the_original_cost() {
    let fixture = card("Serra Angel");
    let _rules = fixture_rules::activate(&[(fixture, &LINKED_CAST)]);
    let mut game = scenario(P);
    let spell = spawn(&mut game, P, fixture, Zone::Hand);
    let cost = put(&mut game, P, "Grizzly Bears", Zone::Battlefield);
    let victim = put(&mut game, P.other(), "Hill Giant", Zone::Battlefield);
    begin_cast(&mut game, spell);
    let action = Action::ChooseAdditionalCost {
        object: game.object_ref(cost),
    };
    act(&mut game, action);
    finish_cast(&mut game);

    let mut stale = game.clone();
    stale.move_object(cost, Zone::Graveyard);
    stale.move_object(cost, Zone::Exile);
    enter_with_target(&mut stale, victim);
    assert!(links(&stale, stale.object_ref(spell)).is_empty());

    let template = game.state.stack.last().unwrap().clone();
    let copy = game.allocate_spell_copy(&template, P);
    let copy_id = copy.source.unwrap().id;
    game.state.stack.push(copy);
    enter_with_target(&mut game, victim);
    let copy_ref = game.object_ref(copy_id);
    assert!(links(&game, copy_ref).is_empty());
    pass_pair(&mut game);
    assert_eq!(links(&game, copy_ref), [game.object_ref(victim)]);
    game.move_object(copy_id, Zone::Graveyard);
    game.prepare_triggers();
    pass_pair(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Hand);
    assert_eq!(game.object_view(cost).zone, Zone::Exile);
    pass_pair(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Battlefield);
    // The original's mandatory target has left, so its entry trigger has no legal target.
    assert!(game.state.stack.is_empty());
    assert_eq!(
        links(&game, game.object_ref(spell)),
        [game.object_ref(cost)]
    );
}

const THRESHOLD: StatValue = StatValue::AtLeast {
    value: &StatValue::LinkedExileCardCount,
    minimum: 3,
};
const BOOST: ScalingModifier = ScalingModifier {
    subject: Subject::Source,
    value: THRESHOLD,
    stats: Stats {
        power: 3,
        toughness: 3,
    },
};
const EXILE_COUNT: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::SourceConditionalKeywords {
                condition: THRESHOLD,
                keywords: Keywords::HEXPROOF,
            },
            StaticAbilityFragment::ScalingModifier(0, BOOST),
        ],
    }],
    extras: &CardExtras {
        scaling_modifiers: &[BOOST],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

#[test]
fn linked_count_tracks_active_ability_origins_and_exact_still_exiled_cards() {
    let definition = card("Bear Cub");
    let _rules = fixture_rules::activate(&[(definition, &EXILE_COUNT)]);
    let mut game = scenario(P);
    let source = spawn(&mut game, P, definition, Zone::Battlefield);
    let reference = game.object_ref(source);
    let mut last = 0;
    for (index, owner) in [P, P.other(), P].into_iter().enumerate() {
        last = put(&mut game, owner, "Forest", Zone::Exile);
        game.link_exile_to_ability(reference, None, game.object_ref(last));
        let boosted = index == 2;
        assert_eq!(
            game.object_view(source)
                .keywords
                .contains(Keywords::HEXPROOF),
            boosted
        );
        assert_eq!(stats(&game, source), if boosted { (5, 4) } else { (2, 1) });
    }
    let mut copied = game.clone();
    copied.apply_temporary_copy(
        reference,
        CopyRecipe::new(definition),
        CopyDeadline::EndOfTurn,
    );
    assert_eq!(stats(&copied, source), (2, 1));
    copied.expire_temporary_copies(true);
    assert_eq!(stats(&copied, source), (5, 4));

    game.move_object(last, Zone::Graveyard);
    game.move_object(last, Zone::Exile);
    assert_eq!(stats(&game, source), (2, 1));
    assert!(
        !game
            .object_view(source)
            .keywords
            .contains(Keywords::HEXPROOF)
    );
    copied.move_object(source, Zone::Exile);
    copied.move_object(source, Zone::Battlefield);
    assert_ne!(copied.object_ref(source), reference);
    assert_eq!(stats(&copied, source), (2, 1));
}

const COPIER: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    keywords: Keywords::CANT_BE_BLOCKED,
    activated: &[ActivatedAbility {
        origin: ActivationOrigin::Battlefield,
        guard: ActivationGuard::Always,
        cost: ActivationCost::ZERO,
        targets: one(TargetRule::creature_you_control()),
        program: &[Instruction::CopyTargetForSourceIncarnation],
        sorcery: false,
        once_per_turn: false,
        cycling: false,
        exhaust: false,
    }],
    ..CardRules::EMPTY
};

#[test]
fn in_place_copy_survives_cleanup_and_blueprint_departure_but_never_blink() {
    let definition = card("Bear Cub");
    let hawk = card("Healer's Hawk");
    let _rules = fixture_rules::activate(&[(definition, &COPIER)]);
    let mut game = scenario(P);
    let source = spawn(&mut game, P, definition, Zone::Battlefield);
    let blueprint = put(&mut game, P, "Hill Giant", Zone::Battlefield);
    let original = game.object_ref(source);
    game.apply_temporary_copy(
        game.object_ref(blueprint),
        CopyRecipe::new(hawk),
        CopyDeadline::EndOfTurn,
    );
    game.add_counter(blueprint, CounterKind::PlusOne, 6);
    game.add_counter(source, CounterKind::PlusOne, 2);
    game.state.objects[source as usize].tapped = true;

    act(
        &mut game,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(blueprint),
        },
    );
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    let mut stale = game.clone();
    stale.move_object(source, Zone::Exile);
    stale.move_object(source, Zone::Battlefield);
    settle(&mut stale);
    assert_eq!(stale.object_view(source).card, definition);

    settle(&mut game);
    assert_eq!(game.object_ref(source), original);
    assert_eq!(game.object_view(source).card, hawk);
    assert!(game.object_view(source).tapped);
    assert_eq!(stats(&game, source), (3, 3));
    assert!(game.object_view(source).keywords.contains(Keywords::FLYING));
    assert!(
        !game
            .object_view(source)
            .keywords
            .contains(Keywords::CANT_BE_BLOCKED)
    );
    game.expire_temporary_copies(true);
    game.move_object(blueprint, Zone::Graveyard);
    assert_eq!(game.object_view(source).card, hawk);
    game.move_object(source, Zone::Exile);
    game.move_object(source, Zone::Battlefield);
    assert_eq!(game.object_view(source).card, definition);
    assert_eq!(stats(&game, source), (1, 1));
}
