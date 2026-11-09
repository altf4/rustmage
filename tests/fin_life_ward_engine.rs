//! A dynamic Ward cost samples live power and exact pre-departure power.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const WARD: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Ward,
        targets: TargetPlan::NONE,
        program: &[Instruction::CounterUnlessPayCosts {
            subject: CounterSubject::Related,
            costs: &[ResolutionCost {
                life: Some(AmountValue::SourcePower),
                ..ResolutionCost::FREE
            }],
            exile: false,
        }],
    }],
    ..CardRules::EMPTY
};

fn target_source(game: &mut Game, card: Card, caster: Player) -> (CardId, CardId) {
    let source = spawn(game, caster.other(), card, Zone::Battlefield);
    let shock = put(game, caster, "Shock", Zone::Hand);
    cast(game, shock, Some(Target::Card(source)));
    assert_eq!(game.state.stack.len(), 2);
    (source, shock)
}

fn resolve_ward(game: &mut Game, caster: Player, amount: i128) {
    act(game, Action::Pass);
    act(game, Action::Pass);
    assert_eq!(game.actor(), Some(caster));
    let request = game.observe(caster).resolution_payment.unwrap();
    assert_eq!(request.life, Some(amount));
    assert_eq!(request.payer, caster);
}

#[test]
fn wide_life_payment_is_frozen_and_replays_after_source_departure() {
    let raubahn = Card::from_name("Raubahn, Bull of Ala Mhigo").unwrap();
    let _rules = fixture_rules::activate(&[(raubahn, &WARD)]);
    let caster = Player::P1;
    let mut parent = scenario(caster);
    let (source, shock) = target_source(&mut parent, raubahn, caster);
    let power = 5_000_000_000i128;
    parent.state.players[caster.index()].life = power + 4;
    parent.state.objects[source as usize].growth.power = power - 2;
    assert_eq!(parent.stats(source).power, power);
    let mut live = parent.clone();
    resolve_ward(&mut live, caster, power);
    act(&mut live, Action::PayResolutionLife);
    assert_eq!(live.state.players[caster.index()].life, 4);
    assert_eq!(live.object_view(shock).zone, Zone::Stack);

    let original = parent.object_ref(source);
    parent.move_object(source, Zone::Exile);
    parent.move_object(source, Zone::Battlefield);
    assert_ne!(parent.object_ref(source), original);
    assert_eq!(parent.stats(source).power, 2);
    resolve_ward(&mut parent, caster, power);
    let saved = parent.snapshot();
    let revision = parent.revision();
    let index = parent
        .legal_actions()
        .iter()
        .position(|action| *action == Action::PayResolutionLife)
        .unwrap();
    let mut indexed = parent.clone();
    act(&mut parent, Action::PayResolutionLife);
    indexed.apply_index(caster, revision, index).unwrap();
    assert_eq!(parent.snapshot(), indexed.snapshot());
    assert_eq!(parent.state.players[caster.index()].life, 4);
    assert_eq!(parent.object_view(shock).zone, Zone::Stack);
    parent.restore(&saved).unwrap();
    act(&mut parent, Action::DeclineResolutionLife);
    assert_eq!(parent.state.players[caster.index()].life, power + 4);
    assert_eq!(parent.object_view(shock).zone, Zone::Graveyard);
}

#[test]
fn negative_power_allows_zero_and_an_unaffordable_positive_cost_is_rejected() {
    let raubahn = Card::from_name("Raubahn, Bull of Ala Mhigo").unwrap();
    let _rules = fixture_rules::activate(&[(raubahn, &WARD)]);
    let caster = Player::P0;
    let mut game = scenario(caster);
    let (source, shock) = target_source(&mut game, raubahn, caster);
    let mut cannot_pay = game.clone();
    cannot_pay.state.objects[source as usize].growth.power = 19;
    resolve_ward(&mut cannot_pay, caster, 21);
    reject(&mut cannot_pay, Action::PayResolutionLife);
    act(&mut cannot_pay, Action::DeclineResolutionLife);
    assert_eq!(cannot_pay.object_view(shock).zone, Zone::Graveyard);
    assert_eq!(cannot_pay.state.players[caster.index()].life, 20);

    game.state.objects[source as usize].growth.power = -5;
    resolve_ward(&mut game, caster, 0);
    act(&mut game, Action::PayResolutionLife);
    assert_eq!(game.state.players[caster.index()].life, 20);
    assert_eq!(game.object_view(shock).zone, Zone::Stack);
}

const ANY_OBJECT: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 0,
    nonland: false,
    other: false,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};

const COMPOUND_COSTS: &[ResolutionCost] = &[
    ResolutionCost {
        mana: ManaCost {
            generic: 1,
            ..ManaCost::ZERO
        },
        life: Some(AmountValue::Fixed(2)),
        objects: Some(ResolutionObjectCost {
            cost: AdditionalCost {
                kind: EffectChoiceKind::TapPermanents(1),
                filter: SacrificeFilter {
                    kinds: 2,
                    ..ANY_OBJECT
                },
            },
            count: 1,
        }),
        ..ResolutionCost::FREE
    },
    ResolutionCost {
        objects: Some(ResolutionObjectCost {
            cost: AdditionalCost {
                kind: EffectChoiceKind::Discard,
                filter: ANY_OBJECT,
            },
            count: 1,
        }),
        ..ResolutionCost::FREE
    },
];

const COMPOUND_WARD: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Ward,
        targets: TargetPlan::NONE,
        program: &[Instruction::CounterUnlessPayCosts {
            subject: CounterSubject::Related,
            costs: COMPOUND_COSTS,
            exile: false,
        }],
    }],
    ..CardRules::EMPTY
};

const TARGET_ABILITY: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost::ZERO,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        program: &[Instruction::PutCounters {
            subject: Subject::Targets,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

fn cost_object(game: &Game, card: CardId) -> Action {
    Action::ChooseAdditionalCost {
        object: game.object_ref(card),
    }
}

#[test]
fn the_same_ward_offers_atomic_private_payments_to_a_targeting_ability_controller() {
    let bearer = Card::GrizzlyBears;
    let activator = Card::HillGiant;
    let _rules = fixture_rules::activate(&[(bearer, &COMPOUND_WARD), (activator, &TARGET_ABILITY)]);
    let payer = Player::P0;
    let opponent = payer.other();
    let mut game = scenario(payer);
    let warded = spawn(&mut game, opponent, bearer, Zone::Battlefield);
    let ability = spawn(&mut game, payer, activator, Zone::Battlefield);
    let first = put(&mut game, payer, "Forest", Zone::Battlefield);
    let second = put(&mut game, payer, "Forest", Zone::Battlefield);
    let own_hand = put(&mut game, payer, "Mountain", Zone::Hand);
    let other_hand = put(&mut game, opponent, "Mountain", Zone::Hand);
    game.state.players[payer.index()].mana = [0; 6];
    act(
        &mut game,
        Action::AnnounceActivate {
            card: ability,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(warded),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    pay(&mut game);
    let victim = game.state.stack[0].id;
    assert!(matches!(
        game.state.stack[0].kind,
        StackKind::ActivatedAbility { .. }
    ));
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    let request = game.observe(opponent).resolution_payment.unwrap();
    assert_eq!(request.victim, Some(victim));
    assert_eq!(request.payer, payer);
    assert_eq!(request.costs.len(), 2);
    let choose_route = game.snapshot();

    act(&mut game, Action::ChooseResolutionCost { index: 0 });
    let select = cost_object(&game, first);
    act(&mut game, select);
    act(&mut game, Action::TapMana { card: first });
    let payment = Action::PayResolutionMana {
        payment: [0, 0, 0, 0, 1, 0],
    };
    reject(&mut game, payment.clone());
    let select = cost_object(&game, second);
    act(&mut game, select);
    assert_eq!(game.state.players[payer.index()].life, 20);
    assert!(!game.object_view(second).tapped);
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == payment)
        .unwrap();
    let revision = game.revision();
    let mut indexed = game.clone();
    act(&mut game, payment);
    indexed.apply_index(payer, revision, index).unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.state.players[payer.index()].life, 18);
    assert_eq!(game.state.players[payer.index()].mana, [0; 6]);
    assert!(game.object_view(first).tapped && game.object_view(second).tapped);
    assert_eq!(game.state.stack.last().unwrap().id, victim);

    game.restore(&choose_route).unwrap();
    act(&mut game, Action::ChooseResolutionCost { index: 1 });
    let illegal = cost_object(&game, other_hand);
    reject(&mut game, illegal);
    let selected = cost_object(&game, own_hand);
    act(&mut game, selected);
    assert_eq!(game.object_view(own_hand).zone, Zone::Hand);
    assert_eq!(
        game.observe(payer)
            .resolution_payment
            .unwrap()
            .selected_objects,
        [game.object_ref(own_hand)]
    );
    assert!(
        game.observe(opponent)
            .resolution_payment
            .unwrap()
            .selected_objects
            .is_empty()
    );
    let mut decline = game.clone();
    act(&mut decline, Action::DeclineResolutionMana);
    assert!(decline.state.stack.is_empty());
    assert_eq!(decline.object_view(own_hand).zone, Zone::Hand);
    assert_eq!(decline.state.players[payer.index()].life, 20);
    act(&mut game, Action::PayResolutionMana { payment: [0; 6] });
    assert_eq!(game.object_view(own_hand).zone, Zone::Graveyard);
    assert_eq!(game.object_view(other_hand).zone, Zone::Hand);
    assert_eq!(game.state.stack.last().unwrap().id, victim);
}

const OPTIONAL_SPELL: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::MayPayCosts {
                costs: &[ResolutionCost {
                    extra_generic: Some(AmountValue::Fixed(1)),
                    objects: Some(ResolutionObjectCost {
                        cost: AdditionalCost {
                            kind: EffectChoiceKind::Sacrifice,
                            filter: SacrificeFilter {
                                kinds: 1,
                                ..ANY_OBJECT
                            },
                        },
                        count: 1,
                    }),
                    ..ResolutionCost::FREE
                }],
                skip: 1,
            },
            Instruction::GainLife(PlayerSet::Controller, 3),
            Instruction::GainLife(PlayerSet::Controller, 1),
        ],
    ),
    ..CardRules::EMPTY
};

#[test]
fn an_ordinary_resolving_spell_can_use_the_same_mana_and_sacrifice_payment() {
    let spell_card = Card::from_name("Shock").unwrap();
    let _rules = fixture_rules::activate(&[(spell_card, &OPTIONAL_SPELL)]);
    let payer = Player::P0;
    let mut game = scenario(payer);
    let spell = spawn(&mut game, payer, spell_card, Zone::Hand);
    let creature = put(&mut game, payer, "Grizzly Bears", Zone::Battlefield);
    let land = put(&mut game, payer, "Forest", Zone::Battlefield);
    cast(&mut game, spell, None);
    game.state.players[payer.index()].mana = [0; 6];
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.observe(payer).resolution_payment.unwrap().victim, None);
    let mut decline = game.clone();
    act(&mut decline, Action::DeclineResolutionMana);
    assert_eq!(decline.state.players[payer.index()].life, 21);
    assert_eq!(decline.object_view(creature).zone, Zone::Battlefield);

    let wrong_type = cost_object(&game, land);
    reject(&mut game, wrong_type);
    let select = cost_object(&game, creature);
    act(&mut game, select);
    let payment = Action::PayResolutionMana {
        payment: [0, 0, 0, 0, 1, 0],
    };
    reject(&mut game, payment.clone());
    act(&mut game, Action::TapMana { card: land });
    act(&mut game, payment);
    assert_eq!(game.state.players[payer.index()].life, 24);
    assert_eq!(game.object_view(creature).zone, Zone::Graveyard);
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
}
