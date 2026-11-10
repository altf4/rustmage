//! Paid Web-slinging and exact tapped-creature return on the split engine.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules, one};
use rustmage_core::{cards::*, engine::*};
use support::*;

const GREEN: Mana = [0, 0, 0, 0, 1, 0];
const WEB_COST: ManaCost = ManaCost {
    colored: GREEN,
    generic: 0,
};
const CREATURE_COST: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 1,
    nonland: false,
    other: false,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};
const WEB_CREATURE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    extras: &CardExtras {
        web_slinging_cost: Some(WEB_COST),
        entry_counter_rules: &[EntryCounterRule {
            guard: StepGuard::Always,
            amount: EntryCounterAmount::WebSlingingReturnedManaValue,
        }],
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersWithWebSlinging,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 3)],
    }],
    ..CardRules::EMPTY
};
const FUNDER: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Fixed(GREEN))],
        exhaust: false,
        cycling: false,
        sorcery: false,
        once_per_turn: false,
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfLeaves,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 2)],
    }],
    ..CardRules::EMPTY
};
const HAND_LAND: CardRules = CardRules {
    effect: Effect::Land,
    entry: EntryTapped::Always,
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::ReturnTappedCreature,
                filter: CREATURE_COST,
            }),
            ..ActivationCost::ZERO
        },
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Hand,
        targets: TargetPlan::NONE,
        program: &[Instruction::PutHandSourceOntoBattlefield],
        exhaust: false,
        cycling: false,
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const WEB_GRANT: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::ColoredLegendaryWebSlinging(WEB_COST)],
    }],
    ..CardRules::EMPTY
};
const SACRIFICE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    extras: &CardExtras {
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::Sacrifice,
            filter: CREATURE_COST,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const COPY_SPELL: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Spell),
        &[Instruction::CopyTargetSpell {
            group: 0,
            retarget: false,
            exceptions: CopyExceptions::NONE,
        }],
    ),
    ..CardRules::EMPTY
};

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn web(card: CardId, face: CastFace) -> Action {
    Action::AnnounceWebSlinging {
        card,
        face,
        cost: WEB_COST,
        flash_surcharge: false,
        origin: WebSlingingOrigin::Ordinary,
    }
}

fn clause(game: &mut Game, want: impl Fn(CastCostClause) -> bool) {
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
        .position(|row| !row.paid && want(row.clause))
        .unwrap()
        .try_into()
        .unwrap();
    act(game, Action::ChooseCastCost { clause });
}

fn pay_mana(game: &mut Game) {
    clause(game, |kind| kind == CastCostClause::Mana);
    pay(game);
}

fn return_creature(game: &mut Game, returned: ObjectRef) {
    clause(game, |kind| {
        matches!(kind, CastCostClause::WebSlingingReturn(_))
    });
    act(game, Action::PayWebSlingingReturn { object: returned });
}

fn tap_funder(game: &mut Game, funder: CardId) {
    choose(game, |action| {
        matches!(action, Action::ActivateMana { card, production, .. }
            if *card == funder && *production == GREEN)
    });
}

#[test]
fn mana_activation_taps_stolen_return_and_exact_lki_survives_branch_restore() {
    let _rules = fixture_rules::activate(&[
        (card("Grizzly Bears"), &WEB_CREATURE),
        (card("Hill Giant"), &FUNDER),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        game.state.players[player.index()].mana = [0; 6];
        let spell = put(&mut game, player, "Grizzly Bears", Zone::Hand);
        let funder = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
        let returned = game.object_ref(funder);
        let action = web(spell, CastFace::Normal);

        reject(&mut game, action.clone());
        game.change_controller(returned, player);
        game.state.objects[funder as usize].summoning_sick = false;
        act(&mut game, action);
        assert!(!game.legal_actions().contains(&Action::BeginOrderedCosts));
        tap_funder(&mut game, funder);
        assert!(game.state.objects[funder as usize].tapped);
        act(&mut game, Action::BeginOrderedCosts);
        pay_mana(&mut game);
        clause(&mut game, |kind| {
            matches!(kind, CastCostClause::WebSlingingReturn(_))
        });

        let action = Action::PayWebSlingingReturn { object: returned };
        reject(
            &mut game,
            Action::PayWebSlingingReturn {
                object: ObjectRef {
                    generation: returned.generation + 1,
                    ..returned
                },
            },
        );
        let before = game.snapshot();
        let revision = game.revision();
        let index = game
            .legal_actions()
            .iter()
            .position(|a| a == &action)
            .unwrap();
        let mut child = game.clone();
        child.apply_index(player, revision, index).unwrap();
        assert_eq!(game.snapshot(), before);
        assert_eq!(
            game.apply(player.other(), action.clone()),
            Err(Error::WrongActor)
        );
        act(&mut game, action.clone());
        assert_eq!(game.snapshot(), child.snapshot());
        game.restore(&before).unwrap();
        assert_eq!(
            game.apply_index(player, revision, index),
            Err(Error::StaleRevision)
        );
        act(&mut game, action);
        assert_eq!(game.snapshot(), child.snapshot());

        let object = &game.state.objects[funder as usize];
        assert_eq!(object.zone, Zone::Hand);
        assert_eq!(object.owner, player.other());
        assert_ne!(game.object_ref(funder), returned);
        assert_eq!(game.state.stack.len(), 2);
        let cast = game.state.stack[0].cast.unwrap();
        assert_eq!(cast.method, CastMethod::WebSlinging);
        assert_eq!(cast.web_slinging_return_mana_value, Some(4));
        assert_eq!(cast.payment, GREEN);
        assert_eq!(game.state.stack[1].controller, player);

        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        assert_eq!(game.state.players[player.index()].life, 22);
        assert_eq!(game.state.objects[spell as usize].zone, Zone::Stack);
        settle(&mut game);
        assert_eq!(game.counter_count(spell, CounterKind::PlusOne), 4);
        assert_eq!(game.state.players[player.index()].life, 25);
    }
}

#[test]
fn granted_web_slinging_uses_the_modal_face_and_keeps_printed_additional_costs() {
    let front = card("Bruce Banner // The Incredible Hulk");
    let back = front.back_face().expect("modal back");
    let _rules = fixture_rules::activate(&[
        (front, &SACRIFICE),
        (back, &SACRIFICE),
        (card("Hill Giant"), &WEB_GRANT),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let provider = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let spell = spawn(&mut game, player, front, Zone::Hand);
    let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    game.state.objects[creature as usize].tapped = true;
    let colorless = spawn(&mut game, player, card("The Endstone"), Zone::Hand);
    reject(&mut game, web(colorless, CastFace::Normal));
    assert!(game.legal_actions().contains(&web(spell, CastFace::Normal)));
    let back_action = web(spell, CastFace::Back);
    assert!(game.legal_actions().contains(&back_action));

    let mut short = game.clone();
    short.move_object(creature, Zone::Exile);
    short.state.objects[provider as usize].tapped = true;
    reject(&mut short, back_action.clone());

    act(&mut game, back_action);
    act(&mut game, Action::BeginOrderedCosts);
    let returned = game.object_ref(creature);
    return_creature(&mut game, returned);
    clause(&mut game, |kind| kind == CastCostClause::Additional);
    let sacrifice = game.object_ref(provider);
    act(
        &mut game,
        Action::ChooseAdditionalCost { object: sacrifice },
    );
    pay_mana(&mut game);
    let cast = game.state.stack.last().unwrap().cast.unwrap();
    assert_eq!(cast.face, CastFace::Back);
    assert_eq!(cast.method, CastMethod::WebSlinging);
    assert_eq!(cast.web_slinging_return_mana_value, Some(2));
    assert_eq!(game.state.objects[provider as usize].zone, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.state.objects[spell as usize].card, back);
}

#[test]
fn paid_exile_permission_allows_web_slinging_and_uncast_copy_has_no_entry_receipt() {
    let _rules = fixture_rules::activate(&[
        (card("Grizzly Bears"), &WEB_CREATURE),
        (card("Opt"), &COPY_SPELL),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Grizzly Bears", Zone::Exile);
    let duplicator = put(&mut game, player, "Opt", Zone::Hand);
    let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    game.state.objects[creature as usize].tapped = true;
    let action = web(spell, CastFace::Normal);
    let spell_ref = game.object_ref(spell);
    game.grant_exile_free_permission(player, spell_ref);
    reject(&mut game, action.clone());
    game.grant_exile_play_permission(player, spell_ref, false);
    act(&mut game, action);
    act(&mut game, Action::BeginOrderedCosts);
    pay_mana(&mut game);
    let returned = game.object_ref(creature);
    return_creature(&mut game, returned);
    let original = game.state.stack.last().unwrap();
    assert_eq!(original.cast.unwrap().origin, Zone::Exile);
    cast(&mut game, duplicator, Some(Target::Spell(spell)));
    settle(&mut game);
    let copy = game
        .state
        .objects
        .iter()
        .enumerate()
        .find(|(_, object)| object.zone == Zone::Battlefield && object.is_token())
        .expect("resolved copied permanent")
        .0 as CardId;
    assert_eq!(game.counter_count(copy, CounterKind::PlusOne), 0);
    assert_eq!(game.counter_count(spell, CounterKind::PlusOne), 2);
    assert_eq!(game.state.players[player.index()].life, 23);
}

#[test]
fn hand_activation_reuses_tapped_return_and_does_not_use_a_land_play() {
    let _rules = fixture_rules::activate(&[
        (card("Tranquil Cove"), &HAND_LAND),
        (card("Hill Giant"), &FUNDER),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    game.state.land_played = true;
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];
    let land = put(&mut game, player, "Tranquil Cove", Zone::Hand);
    let funder = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let announce = Action::AnnounceActivate {
        card: land,
        ability: 0,
        counter_payment: Counters::default(),
    };
    let mut wrong_step = game.clone();
    wrong_step.state.step = Step::End;
    reject(&mut wrong_step, announce.clone());
    act(&mut game, announce);
    tap_funder(&mut game, funder);
    act(&mut game, Action::BeginOrderedCosts);
    pay_mana(&mut game);
    clause(&mut game, |kind| kind == CastCostClause::Additional);
    let returned = game.object_ref(funder);
    act(&mut game, Action::ChooseAdditionalCost { object: returned });
    settle(&mut game);
    assert_eq!(game.state.objects[land as usize].zone, Zone::Battlefield);
    assert!(game.state.objects[land as usize].tapped);
    assert_eq!(game.state.objects[funder as usize].zone, Zone::Hand);
    assert!(game.state.land_played);
}

#[test]
fn ordinary_land_taps_do_not_create_a_web_return_witness() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &WEB_CREATURE)]);
    let player = Player::P0;
    let mut game = scenario(player);
    game.state.players[player.index()].mana = [0; 6];
    for _ in 0..10 {
        put(&mut game, player, "Forest", Zone::Battlefield);
    }
    put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let spell = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    let alternative = web(spell, CastFace::Normal);
    reject(&mut game, alternative.clone());

    // The same untapped creature can now contribute to the Parents' mana
    // ability, making the subsequent return possible.
    put(&mut game, player, "Supportive Parents", Zone::Battlefield);
    act(&mut game, alternative);
}
