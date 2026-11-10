mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const CALLBACKS: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
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
        program: &[
            Instruction::DelayNextCreatureSpell { ability: 0 },
            Instruction::DelayNextCreatureSpell { ability: 1 },
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::DelayedSpellCastCreature,
            targets: TargetPlan::NONE,
            program: &[Instruction::AddCastEntryCounters {
                kind: CounterKind::PlusOne,
                amount: AmountValue::Fixed(1),
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::DelayedSpellCastCreature,
            targets: TargetPlan::NONE,
            program: &[Instruction::GrantCastSpellHaste],
        },
    ],
    ..CardRules::EMPTY
};
const FORK: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Spell)),
        &[Instruction::CopyTargetSpell {
            group: 0,
            retarget: false,
            exceptions: CopyExceptions::NONE,
        }],
    ),
    ..CardRules::EMPTY
};

fn install(game: &mut Game, player: Player) -> CardId {
    let provider = put(game, player, "Hill Giant", Zone::Battlefield);
    act(
        game,
        Action::AnnounceActivate {
            card: provider,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(game);
    settle(game);
    assert_eq!(game.observe(player).delayed_spell_copies.len(), 2);
    provider
}

fn pass_priority(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn order_callbacks(game: &mut Game) {
    choose(game, |action| matches!(action, Action::PutTrigger { .. }));
}

fn resolve_callbacks(game: &mut Game) {
    pass_priority(game);
    pass_priority(game);
}

fn haste(game: &Game, creature: CardId) -> bool {
    game.keywords(creature).contains(Keywords::HASTE)
}

#[test]
fn next_creature_cast_survives_source_departure_and_skips_nonmatching_spells() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Hill Giant").unwrap(), &CALLBACKS),
        (Card::Shock, &FORK),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let provider = install(&mut game, player);

    let bounce = put(&mut game, player, "Unsummon", Zone::Hand);
    cast(&mut game, bounce, Some(Target::Card(provider)));
    settle(&mut game);
    assert_eq!(game.object_view(provider).zone, Zone::Hand);

    act(&mut game, Action::Pass);
    let opponent = put(&mut game, player.other(), "Cathar Commando", Zone::Hand);
    cast(&mut game, opponent, None);
    if game.actor() != Some(player) {
        act(&mut game, Action::Pass);
    }
    let fork = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, fork, Some(Target::Spell(opponent)));
    pass_priority(&mut game);
    let copy = game.state.stack.last().unwrap();
    assert!(copy.is_copy);
    assert_eq!(copy.controller, player);
    let copy = copy.source.unwrap().id;
    settle(&mut game);
    assert_eq!(game.observe(player).delayed_spell_copies.len(), 2);
    assert!(!haste(&game, copy));
    assert!(!haste(&game, opponent));
    assert_eq!(game.counter_count(opponent, CounterKind::PlusOne), 0);

    put(&mut game, player, "Doubling Season", Zone::Battlefield);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    cast(&mut game, creature, None);
    let spell = &game.state.stack[0];
    assert_eq!(game.state.ordering_triggers.len(), 2);
    for callback in &game.state.ordering_triggers {
        let event = callback.cast_event.as_ref().unwrap();
        assert_eq!(event.stack_id, spell.id);
        assert_eq!(event.object, game.object_ref(creature));
        assert_eq!(event.payment, spell.cast.unwrap().payment);
        assert_eq!(event.controller, player);
        assert_eq!(callback.source.unwrap().id, provider);
        assert!(callback.copy_event.is_none());
    }
    order_callbacks(&mut game);
    settle(&mut game);
    assert!(game.observe(player).delayed_spell_copies.is_empty());
    assert!(haste(&game, creature));
    assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 2);

    let second = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    cast(&mut game, second, None);
    settle(&mut game);
    assert!(!haste(&game, second));
    assert_eq!(game.counter_count(second, CounterKind::PlusOne), 0);
    pass_priority(&mut game);
    pass_priority(&mut game);
    let actions = game.legal_actions();
    assert!(actions.contains(&Action::Attack { card: creature }));
    assert!(!actions.contains(&Action::Attack { card: second }));
}

#[test]
fn exact_stack_grants_branch_restore_continue_on_entry_and_stay_off_copies() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Hill Giant").unwrap(), &CALLBACKS),
        (Card::Shock, &FORK),
    ]);
    let player = Player::P0;
    let mut raw = scenario(player);
    install(&mut raw, player);
    let creature = put(&mut raw, player, "Grizzly Bears", Zone::Hand);
    cast(&mut raw, creature, None);
    let original = raw.object_ref(creature);
    let action = raw
        .legal_actions()
        .into_iter()
        .find(|action| matches!(action, Action::PutTrigger { .. }))
        .unwrap();
    let index = raw
        .legal_actions()
        .iter()
        .position(|candidate| *candidate == action)
        .unwrap();
    let saved = raw.snapshot();
    let revision = raw.revision();
    let mut indexed = raw.clone();
    act(&mut raw, action.clone());
    indexed.apply_index(player, revision, index).unwrap();
    resolve_callbacks(&mut raw);
    resolve_callbacks(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    act(&mut indexed, action);
    resolve_callbacks(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.object_ref(creature), original);
    assert!(haste(&raw, creature));
    let stack_effect = raw
        .observe(player)
        .keyword_effects
        .into_iter()
        .find(|row| row.recipient == original)
        .unwrap();
    assert_eq!(stack_effect.expires_turn, Some(raw.state.turn));

    let entry = raw.entry_move(creature, Zone::Battlefield, false);
    let preview = raw.ghost_entry_preview(&entry).unwrap();
    assert!(preview.keywords.contains(Keywords::HASTE));
    assert_eq!(raw.snapshot(), indexed.snapshot());

    let fork = put(&mut raw, player, "Shock", Zone::Hand);
    cast(&mut raw, fork, Some(Target::Spell(creature)));
    pass_priority(&mut raw);
    let copy = raw.state.stack.last().unwrap();
    assert!(copy.is_copy);
    let copy = copy.source.unwrap().id;
    assert!(!haste(&raw, copy));
    settle(&mut raw);
    assert_eq!(raw.object_view(copy).zone, Zone::Battlefield);
    assert_eq!(raw.counter_count(copy, CounterKind::PlusOne), 0);
    assert!(!haste(&raw, copy));
    assert_eq!(raw.counter_count(creature, CounterKind::PlusOne), 1);
    let entered = raw.object_ref(creature);
    assert_ne!(entered, original);
    assert!(haste(&raw, creature));
    assert!(
        raw.observe(player)
            .keyword_effects
            .contains(&KeywordEffectView {
                recipient: entered,
                ..stack_effect
            })
    );

    raw.state.step = Step::End;
    pass_priority(&mut raw);
    assert!(!haste(&raw, creature));
    assert!(raw.observe(player).keyword_effects.is_empty());
    assert_eq!(raw.counter_count(creature, CounterKind::PlusOne), 1);
}

#[test]
fn a_countered_first_spell_consumes_the_callbacks_and_cannot_carry_them_to_replay() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Hill Giant").unwrap(), &CALLBACKS)]);
    let player = Player::P0;
    for resolve_first in [false, true] {
        let mut game = scenario(player);
        install(&mut game, player);
        let creature = put(&mut game, player, "Grizzly Bears", Zone::Hand);
        cast(&mut game, creature, None);
        let original = game.object_ref(creature);
        order_callbacks(&mut game);
        if resolve_first {
            resolve_callbacks(&mut game);
            assert!(haste(&game, creature));
            assert_eq!(game.observe(player).cast_entry_counters[0].spell, original);
        }
        if game.actor() != Some(player.other()) {
            act(&mut game, Action::Pass);
        }
        let cancel = put(&mut game, player.other(), "Cancel", Zone::Hand);
        cast(&mut game, cancel, Some(Target::Spell(creature)));
        settle(&mut game);
        assert_eq!(game.object_view(creature).zone, Zone::Graveyard);
        assert!(game.observe(player).delayed_spell_copies.is_empty());
        assert!(game.observe(player).cast_entry_counters.is_empty());
        assert!(game.observe(player).keyword_effects.is_empty());

        game.move_object(creature, Zone::Hand);
        cast(&mut game, creature, None);
        assert_ne!(game.object_ref(creature), original);
        assert!(!haste(&game, creature));
        settle(&mut game);
        assert!(!haste(&game, creature));
        assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 0);
    }
}

#[test]
fn unused_callbacks_expire_before_the_controllers_next_cast() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Hill Giant").unwrap(), &CALLBACKS)]);
    let player = Player::P0;
    let mut game = scenario(player);
    install(&mut game, player);
    let turn = game.state.turn;
    game.state.step = Step::End;
    pass_priority(&mut game);
    assert_eq!(game.state.turn, turn + 1);
    assert!(game.observe(player).delayed_spell_copies.is_empty());

    let first_land = put(&mut game, player, "Plains", Zone::Battlefield);
    let second_land = put(&mut game, player, "Plains", Zone::Battlefield);
    let creature = put(&mut game, player, "Cathar Commando", Zone::Hand);
    act(&mut game, Action::Pass);
    act(&mut game, Action::TapMana { card: first_land });
    act(&mut game, Action::TapMana { card: second_land });
    cast(&mut game, creature, None);
    settle(&mut game);
    assert!(!haste(&game, creature));
    assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 0);
}
