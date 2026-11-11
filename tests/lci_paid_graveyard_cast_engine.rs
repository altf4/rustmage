//! Owned one-use graveyard permissions and spell-to-permanent type effects.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

const GRANTOR: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GrantCreatureGraveyardCastThisTurn {
            entry_counter: Some(CounterKind::Finality),
            additional_types: &["Vampire"],
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

fn grant(g: &mut Game, source: CardId) {
    if let Some(action) = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::Activate { card, ability: 0, .. } if *card == source))
    {
        act(g, action);
    } else {
        act(
            g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        if g.state.announcing.is_some() {
            pay(g);
        }
    }
    settle(g);
}

fn route(g: &Game, spell: CardId, permission: u64) -> Action {
    Action::AnnounceGraveyardPaid {
        object: g.object_ref(spell),
        permission,
        face: CastFace::Normal,
        method: CastMethod::Normal,
        flash_surcharge: false,
    }
}

fn ready(g: &mut Game, goal: impl Fn(&Game) -> bool) {
    for _ in 0..48 {
        if goal(g) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::PayCosts { .. }
                    | Action::FinishTargets
                    | Action::BeginOrderedCosts
                    | Action::ChooseCastCost { .. }
            )
        });
    }
    panic!("paid graveyard scenario did not reach its next decision");
}

fn quiet(g: &mut Game) {
    ready(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
    });
}

#[test]
fn each_resolution_grants_one_cast_and_an_independent_permission_does_not_spend_it() {
    let _rules = fixture_rules::activate(&[(card("Short Bow"), &GRANTOR)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Short Bow", Zone::Battlefield);
    let first = put(&mut g, p, "Healer's Hawk", Zone::Graveyard);
    let second = put(&mut g, p, "Healer's Hawk", Zone::Graveyard);
    let separate = put(&mut g, p, "Healer's Hawk", Zone::Graveyard);
    let foreign = put(&mut g, p.other(), "Healer's Hawk", Zone::Graveyard);
    let noncreature = put(&mut g, p, "Disenchant", Zone::Graveyard);
    grant(&mut g, source);
    g.untap_objects(&[source], untap_rules::UntapCause::Effect);
    grant(&mut g, source);
    g.move_object(source, Zone::Graveyard);
    let permissions = g.observe(p).paid_graveyard_cast_permissions;
    assert_eq!(permissions.len(), 2);
    assert_ne!(permissions[0].id, permissions[1].id);
    let first_id = permissions[0].id;
    for id in [foreign, noncreature] {
        let action = route(&g, id, first_id);
        reject(&mut g, action);
    }
    g.grant_graveyard_permission(p, g.object_ref(separate), GraveyardCastKind::Cast);
    act(&mut g, Action::AnnounceCast { card: separate });
    pay(&mut g);
    quiet(&mut g);
    assert!(!g.has_subtype(separate, "Vampire"));
    assert_eq!(g.counter_count(separate, CounterKind::Finality), 0);
    assert_eq!(g.observe(p).paid_graveyard_cast_permissions, permissions);

    let action = route(&g, first, first_id);
    let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
    let saved = g.snapshot();
    let revision = g.revision();
    let mut indexed = g.clone();
    act(&mut g, action.clone());
    indexed.apply_index(p, revision, index).unwrap();
    assert_eq!(g.snapshot(), indexed.snapshot());
    assert_eq!(g.observe(p).paid_graveyard_cast_permissions.len(), 2);
    for branch in [&mut g, &mut indexed] {
        pay(branch);
        quiet(branch);
    }
    assert_eq!(g.snapshot(), indexed.snapshot());
    assert_eq!(
        g.observe(p).paid_graveyard_cast_permissions,
        permissions[1..]
    );
    let stale = route(&g, second, first_id);
    reject(&mut g, stale);
    let second_cast = route(&g, second, permissions[1].id);
    act(&mut g, second_cast);
    pay(&mut g);
    quiet(&mut g);
    assert!(g.observe(p).paid_graveyard_cast_permissions.is_empty());
    for id in [first, second] {
        assert!(g.has_subtype(id, "Vampire"));
        assert!(g.has_subtype(id, "Bird"));
        assert_eq!(g.counter_count(id, CounterKind::Finality), 1);
    }
    indexed.restore(&saved).unwrap();
    act(&mut indexed, action);
    pay(&mut indexed);
    quiet(&mut indexed);
    assert_eq!(
        indexed.observe(p).paid_graveyard_cast_permissions,
        permissions[1..]
    );
}

const WATCHER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::ControlledSubtypeEntryCounters {
            subtype: "Vampire",
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::CreatureSubtype("Vampire"),
                ordinal: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::SubtypeEntersYou("Vampire"),
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 2)],
        },
    ],
    ..CardRules::EMPTY
};

#[test]
fn added_spell_subtype_guides_cavern_mana_and_projects_into_entry_replacements() {
    let _rules =
        fixture_rules::activate(&[(card("Short Bow"), &GRANTOR), (card("Bear Cub"), &WATCHER)]);
    let p = Player::P1;
    let mut g = scenario(p);
    g.state.players[p.index()].mana = [0; 6];
    let source = put(&mut g, p, "Short Bow", Zone::Battlefield);
    put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let cavern = put(&mut g, p, "Cavern of Souls", Zone::Battlefield);
    g.set_entry_value(
        g.object_ref(cavern),
        EntryChoiceValue::CreatureType(CreatureType::from_name("Vampire").unwrap()),
    );
    let spell = put(&mut g, p, "Healer's Hawk", Zone::Graveyard);
    grant(&mut g, source);
    let permission = g.observe(p).paid_graveyard_cast_permissions[0].id;
    let action = route(&g, spell, permission);
    act(&mut g, action);
    assert!(g.has_subtype(spell, "Vampire"));
    assert_eq!(
        g.observe(p.other()).spell_characteristics[0].additional_creature_types,
        Some(&["Vampire"][..])
    );
    choose(&mut g, |a| {
        matches!(a, Action::ActivateMana { card, ability: 1, production, .. }
        if *card == cavern && *production == [1, 0, 0, 0, 0, 0])
    });
    pay(&mut g);
    let stack = g
        .state
        .stack
        .iter()
        .find(|item| item.kind == StackKind::Spell)
        .unwrap();
    assert!(g.observe(p).uncounterable_spell_ids.contains(&stack.id));
    quiet(&mut g);
    assert_eq!(g.state.players[p.index()].life, 23);
    assert_eq!(g.counter_count(spell, CounterKind::PlusOne), 1);
    assert_eq!(g.counter_count(spell, CounterKind::Finality), 1);
    g.move_object(spell, Zone::Hand);
    g.move_object(spell, Zone::Battlefield);
    assert!(!g.has_subtype(spell, "Vampire"));
    assert_eq!(g.counter_count(spell, CounterKind::Finality), 0);
}

const COPY: CardRules = CardRules {
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

#[test]
fn copies_omit_the_cast_modifier_and_a_countered_spell_still_consumes_permission() {
    let _rules = fixture_rules::activate(&[(card("Short Bow"), &GRANTOR), (card("Opt"), &COPY)]);
    let p = Player::P0;
    for counter in [false, true] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Short Bow", Zone::Battlefield);
        let spell = put(&mut g, p, "Healer's Hawk", Zone::Graveyard);
        let response = put(
            &mut g,
            p.other(),
            if counter { "Essence Scatter" } else { "Opt" },
            Zone::Hand,
        );
        grant(&mut g, source);
        let permission = g.observe(p).paid_graveyard_cast_permissions[0].id;
        let action = route(&g, spell, permission);
        act(&mut g, action);
        pay(&mut g);
        assert!(g.observe(p).paid_graveyard_cast_permissions.is_empty());
        act(&mut g, Action::Pass);
        cast(&mut g, response, Some(Target::Spell(spell)));
        if counter {
            quiet(&mut g);
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            assert!(!g.has_subtype(spell, "Vampire"));
            assert!(g.observe(p).cast_entry_counters.is_empty());
            let consumed = route(&g, spell, permission);
            reject(&mut g, consumed);
        } else {
            ready(&mut g, |g| {
                g.state
                    .stack
                    .iter()
                    .any(|item| item.kind == StackKind::Spell && item.is_copy)
            });
            let copy = g
                .state
                .stack
                .iter()
                .find(|item| item.kind == StackKind::Spell && item.is_copy)
                .unwrap()
                .source
                .unwrap();
            assert!(!copy.facts.has_subtype("Vampire"));
            assert!(!g.has_subtype(copy.id, "Vampire"));
            quiet(&mut g);
            assert_eq!(g.object_view(copy.id).zone, Zone::Battlefield);
            assert!(!g.has_subtype(copy.id, "Vampire"));
            assert_eq!(g.counter_count(copy.id, CounterKind::Finality), 0);
            assert!(g.has_subtype(spell, "Vampire"));
        }
    }
}

#[test]
fn normal_timing_and_cleanup_apply_to_an_unspent_graveyard_permission() {
    let _rules = fixture_rules::activate(&[(card("Short Bow"), &GRANTOR)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Short Bow", Zone::Battlefield);
    let flash = put(&mut g, p, "Flock Impostor", Zone::Graveyard);
    let sorcery = put(&mut g, p, "Healer's Hawk", Zone::Graveyard);
    grant(&mut g, source);
    let permission = g.observe(p).paid_graveyard_cast_permissions[0].id;
    g.state.step = Step::End;
    assert!(g.legal_actions().contains(&route(&g, flash, permission)));
    let wrong_timing = route(&g, sorcery, permission);
    reject(&mut g, wrong_timing);
    act(&mut g, Action::Pass);
    act(&mut g, Action::Pass);
    assert!(g.observe(p).paid_graveyard_cast_permissions.is_empty());
}
