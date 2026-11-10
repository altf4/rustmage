//! Token replacement, exact Saga lore choices, and one-shot own-end-step cleanup.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const COPY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
            base: &TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 32,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            predicate: &TargetPredicate::Nonlegendary,
        })),
        &[
            Instruction::CreateHastyCopyWithSagaLore { maximum: 3 },
            Instruction::GainLife(PlayerSet::Controller, 1),
        ],
    ),
    ..CardRules::EMPTY
};

const SAGA: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Chapter { number: 3 },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 4)],
    }],
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::Color),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn tokens(game: &Game) -> Vec<ObjectRef> {
    game.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, object)| object.zone == Zone::Battlefield && object.is_token())
        .map(|(id, _)| game.object_ref(id as CardId))
        .collect()
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn lore(object: ObjectRef, amount: u8) -> Action {
    Action::ChooseCopiedSagaLore { object, amount }
}

fn color(color: ChoiceColor) -> Action {
    Action::ChooseEntryValue {
        value: EntryChoiceValue::Color(color),
    }
}

#[test]
fn replaced_saga_copies_enter_before_independent_lore_choices_and_chapters() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Opt").unwrap(), &COPY),
        (Card::from_name("Summon: Anima").unwrap(), &SAGA),
    ]);
    let player = Player::P0;
    let mut parent = scenario(player);
    let original = put(&mut parent, player, "Summon: Anima", Zone::Battlefield);
    parent.state.objects[original as usize].tapped = true;
    parent.set_counters(original, Counters::of(CounterKind::Lore, 1));
    put(&mut parent, player, "Doubling Season", Zone::Battlefield);
    let spell = put(&mut parent, player, "Opt", Zone::Hand);
    cast(&mut parent, spell, Some(Target::Card(original)));
    pass_pair(&mut parent);
    assert!(matches!(parent.state.pending, Pending::EntryValue { .. }));
    assert!(tokens(&parent).is_empty());
    let before_entry = serde_json::to_value(parent.observe(player)).unwrap();
    assert!(before_entry.get("copied_saga_lore").is_none());
    act(&mut parent, color(ChoiceColor::Blue));
    assert!(matches!(parent.state.pending, Pending::EntryValue { .. }));
    assert!(tokens(&parent).is_empty());
    act(&mut parent, color(ChoiceColor::Green));

    let created = tokens(&parent);
    let [first, second] = created.as_slice() else {
        panic!("expected two Saga copies");
    };
    let first = *first;
    let second = *second;
    for (object, value) in [(first, ChoiceColor::Blue), (second, ChoiceColor::Green)] {
        let view = parent.object_view(object.card);
        assert_eq!(view.entry_choice, Some(EntryChoiceValue::Color(value)));
        assert_eq!(view.counters.get(CounterKind::Lore), 2);
        assert!(!view.tapped && view.summoning_sick);
        assert!(view.keywords.contains(Keywords::HASTE));
        assert!(
            !parent
                .copiable_recipe(object)
                .intrinsic_keywords()
                .contains(Keywords::HASTE)
        );
        assert!(parent.can_declare_attacker(player, object.card));
    }
    assert_eq!(parent.counter_count(original, CounterKind::Lore), 1);
    assert!(parent.observe(player).delayed_sacrifices.is_empty());
    for viewer in [player, player.other()] {
        let choice = parent.observe(viewer).copied_saga_lore.unwrap();
        assert_eq!((choice.object, choice.maximum), (first, 3));
    }
    for amount in 0..=3 {
        assert!(parent.legal_actions().contains(&lore(first, amount)));
    }
    for invalid in [
        lore(first, 4),
        lore(second, 1),
        lore(parent.object_ref(original), 1),
    ] {
        reject(&mut parent, invalid);
    }
    let action = lore(first, 1);
    let wire = serde_json::to_value(&action).unwrap();
    assert_eq!(wire["kind"], "ChooseCopiedSagaLore");
    assert_eq!(serde_json::from_value::<Action>(wire).unwrap(), action);
    let saved = parent.snapshot();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    drop(parent);

    for action in [action, lore(second, 0)] {
        let revision = indexed.revision();
        let index = indexed
            .legal_actions()
            .iter()
            .position(|legal| *legal == action)
            .unwrap();
        let first_choice = action == lore(first, 1);
        act(&mut raw, action);
        indexed.apply_index(player, revision, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        if first_choice {
            assert_eq!(raw.observe(player).copied_saga_lore.unwrap().object, second);
            assert_eq!(raw.counter_count(first.card, CounterKind::Lore), 4);
            assert_eq!(raw.counter_count(second.card, CounterKind::Lore), 2);
            assert_eq!(raw.state.players[player.index()].life, 20);
            assert!(raw.observe(player).delayed_sacrifices.is_empty());
            reject(&mut raw, lore(first, 1));
        }
    }
    assert_eq!(raw.state.players[player.index()].life, 21);
    assert_eq!(raw.observe(player).delayed_sacrifices[0].tokens, created);
    settle(&mut raw);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(indexed.state.players[player.index()].life, 25);
    assert_eq!(tokens(&indexed), [second]);

    raw.restore(&saved).unwrap();
    act(&mut raw, lore(first, 0));
    act(&mut raw, lore(second, 0));
    settle(&mut raw);
    assert_eq!(tokens(&raw), [first, second]);
    assert_eq!(raw.state.players[player.index()].life, 21);
    assert_eq!(tokens(&indexed), [second]);
}

#[test]
fn non_saga_copies_wait_for_own_end_step_and_retained_haste_survives_cleanup() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &COPY)]);
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(opponent);
    let enchantment = put(&mut game, player, "Impact Tremors", Zone::Battlefield);
    put(&mut game, player, "Doubling Season", Zone::Battlefield);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    act(&mut game, Action::Pass);
    assert_eq!(game.actor(), Some(player));
    cast(&mut game, spell, Some(Target::Card(enchantment)));

    let mut stale = game.clone();
    stale.move_object(enchantment, Zone::Exile);
    stale.move_object(enchantment, Zone::Battlefield);
    settle(&mut stale);
    assert!(tokens(&stale).is_empty());
    assert!(stale.observe(player).delayed_sacrifices.is_empty());
    assert_eq!(stale.state.players[player.index()].life, 20);

    settle(&mut game);
    let created = tokens(&game);
    let [stolen, retained] = created.as_slice() else {
        panic!("expected two copies");
    };
    let stolen = *stolen;
    let retained = *retained;
    let obs = game.observe(opponent);
    assert!(obs.copied_saga_lore.is_none());
    assert_eq!(obs.delayed_sacrifices.len(), 1);
    let delay = &obs.delayed_sacrifices[0];
    assert_eq!(delay.controller, player);
    assert!(delay.own_end_step);
    assert_eq!(delay.tokens, created);
    for object in [stolen, retained] {
        assert!(game.keywords(object.card).contains(Keywords::HASTE));
    }
    game.record_control(stolen, opponent, None);
    game.state.step = Step::Main2;
    pass_pair(&mut game);
    assert_eq!(game.state.step, Step::End);
    assert_eq!(game.state.active, opponent);
    assert_eq!(tokens(&game), created);
    assert_eq!(game.observe(player).delayed_sacrifices.len(), 1);

    pass_pair(&mut game);
    assert_eq!(game.state.active, player);
    assert_eq!(game.state.step, Step::Upkeep);
    assert!(game.keywords(stolen.card).contains(Keywords::HASTE));
    assert!(game.keywords(retained.card).contains(Keywords::HASTE));
    game.state.step = Step::Main2;
    pass_pair(&mut game);
    assert_eq!(game.state.step, Step::End);
    settle(&mut game);
    assert_eq!(tokens(&game), [stolen]);
    assert_eq!(game.object_view(stolen.card).controller, opponent);
    assert!(game.keywords(stolen.card).contains(Keywords::HASTE));
    assert_eq!(game.object_view(enchantment).zone, Zone::Battlefield);
    assert!(game.observe(player).delayed_sacrifices.is_empty());
}
