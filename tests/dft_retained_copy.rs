//! Retained copied activations keep immutable templates and exact linked origins.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;
const MIME: CardRules = CardRules {
    effect: Effect::Creature(0, 0),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::LinkedExileCreature {
            ability_origin: None,
        })),
        program: &[Instruction::CopySourceFromLinkedExile { base: (0, 0) }],
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        x_symbols: 1,
        entry_exile_counters: Some(EntryExileCountersSpec {
            filter: SearchFilter::Creature,
            kind: CounterKind::PlusOne,
            per_card: 3,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
fn setup() -> (Game, CardId, CardId, CardId) {
    let mut g = scenario(Player::P0);
    let a = put(&mut g, Player::P0, "Grizzly Bears", Zone::Graveyard);
    let b = put(&mut g, Player::P0, "Savannah Lions", Zone::Graveyard);
    let source = put(&mut g, Player::P0, "Pterafractyl", Zone::Exile);
    let mut movement = g.entry_move(source, Zone::Battlefield, false);
    movement.x = 2;
    g.stage_entries(vec![movement], None);
    assert!(g.advance_entries());
    for id in [a, b] {
        let object = g.object_ref(id);
        act(&mut g, Action::ChooseEntryExile { object });
    }
    settle(&mut g);
    assert_eq!(stats(&g, source), (6, 6));
    (g, source, a, b)
}
fn target_and_pay(g: &mut Game, target: CardId) {
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets)
    }
    pay(g);
}
fn original(g: &mut Game, source: CardId, target: CardId) {
    act(
        g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    target_and_pay(g, target);
    settle(g);
}
fn retained(g: &mut Game, source: CardId, target: CardId) {
    let source = g.object_ref(source);
    act(g, Action::AnnounceRetainedCopyAbility { source });
    target_and_pay(g, target);
    settle(g);
}
#[test]
fn repeated_permanent_copies_keep_counters_and_the_original_immutable_linked_activation() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Pterafractyl").unwrap(), &MIME)]);
    eprintln!(
        "retained-copy layout: Object={} ObjectView={} CopyRecipe={} CopyExceptions={} CopyLayerView={}",
        std::mem::size_of::<Object>(),
        std::mem::size_of::<ObjectView>(),
        std::mem::size_of::<CopyRecipe>(),
        std::mem::size_of::<CopyExceptions>(),
        std::mem::size_of::<CopyLayerView>()
    );
    let (mut g, source, a, b) = setup();
    original(&mut g, source, a);
    assert_eq!(
        g.state.objects[source as usize].card,
        Card::from_name("Grizzly Bears").unwrap()
    );
    assert_eq!(stats(&g, source), (6, 6));
    assert_eq!(g.retained_link_origin(g.object_ref(source)), None);
    let recipe = g.copiable_recipe(g.object_ref(source));
    assert_eq!(recipe.exceptions.base, Some((0, 0)));
    assert_eq!(
        recipe.exceptions.retained_activation,
        Some(RetainedCopyActivation {
            definition: Card::from_name("Pterafractyl").unwrap(),
            ability: 0
        })
    );
    retained(&mut g, source, b);
    assert_eq!(
        g.state.objects[source as usize].card,
        Card::from_name("Savannah Lions").unwrap()
    );
    assert_eq!(stats(&g, source), (6, 6));
    retained(&mut g, source, a);
    g.expire_temporary_copies(true);
    assert_eq!(
        g.state.objects[source as usize].card,
        Card::from_name("Grizzly Bears").unwrap()
    );
    assert!(
        g.legal_actions()
            .contains(&Action::AnnounceRetainedCopyAbility {
                source: g.object_ref(source)
            })
    );
}
#[test]
fn linked_copy_targets_reject_unlinked_cards_and_stale_exile_incarnations() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Pterafractyl").unwrap(), &MIME)]);
    let (mut g, source, a, b) = setup();
    let unlinked = put(&mut g, Player::P0, "Hill Giant", Zone::Exile);
    act(
        &mut g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(unlinked),
        },
    );
    target_and_pay(&mut g, a);
    settle(&mut g);
    let old = g.copiable_recipe(g.object_ref(source));
    let object = g.object_ref(source);
    act(
        &mut g,
        Action::AnnounceRetainedCopyAbility { source: object },
    );
    target_and_pay(&mut g, b);
    g.move_object(b, Zone::Hand);
    g.move_object(b, Zone::Exile);
    settle(&mut g);
    assert_eq!(g.copiable_recipe(g.object_ref(source)), old);
    assert!(
        !g.linked_exile_for_origin(g.object_ref(source), None)
            .contains(&g.object_ref(b))
    );
}
#[test]
fn another_copier_and_a_later_overwriting_copy_get_fresh_unlinked_abilities() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Pterafractyl").unwrap(), &MIME)]);
    let (mut g, source, a, _) = setup();
    original(&mut g, source, a);
    let recipe = g.copiable_recipe(g.object_ref(source));
    let another = put(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
    g.state.objects[another as usize].counters.plus_one = 2;
    let reference = g.object_ref(another);
    g.apply_temporary_copy(
        reference,
        recipe,
        CopyDeadline::WhileSource { source: reference },
    );
    assert!(
        !g.legal_actions()
            .contains(&Action::AnnounceRetainedCopyAbility { source: reference })
    );
    let reference = g.object_ref(source);
    g.apply_temporary_copy(reference, recipe, CopyDeadline::EndOfTurn);
    assert!(
        !g.legal_actions()
            .contains(&Action::AnnounceRetainedCopyAbility { source: reference })
    );
    g.expire_temporary_copies(true);
    assert!(
        g.legal_actions()
            .contains(&Action::AnnounceRetainedCopyAbility { source: reference })
    );
}
#[test]
fn retained_activation_is_a_real_ability_and_layer_six_removal_suppresses_it() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Pterafractyl").unwrap(), &MIME)]);
    let (mut g, source, a, _) = setup();
    original(&mut g, source, a);
    assert!(g.object_has_rules_abilities(source));
    assert!(additional_costs::has_craft_activated_ability(&g, source));
    let aura = put(&mut g, Player::P1, "Witness Protection", Zone::Battlefield);
    g.state.objects[aura as usize].attached_to = Some(g.object_ref(source));
    let action = Action::AnnounceRetainedCopyAbility {
        source: g.object_ref(source),
    };
    assert!(!g.legal_actions().contains(&action));
    g.move_object(aura, Zone::Hand);
    assert!(g.legal_actions().contains(&action));
}
#[test]
fn retained_activation_target_binding_survives_a_later_copy_but_never_rebinds_a_departed_source() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Pterafractyl").unwrap(), &MIME)]);
    let (mut g, source, a, b) = setup();
    original(&mut g, source, a);
    let reference = g.object_ref(source);
    act(
        &mut g,
        Action::AnnounceRetainedCopyAbility { source: reference },
    );
    target_and_pay(&mut g, b);
    g.apply_temporary_copy(
        reference,
        CopyRecipe::new(Card::from_name("Hill Giant").unwrap()),
        CopyDeadline::EndOfTurn,
    );
    settle(&mut g);
    assert_eq!(
        g.state.objects[source as usize].card,
        Card::from_name("Savannah Lions").unwrap()
    );
    act(
        &mut g,
        Action::AnnounceRetainedCopyAbility { source: reference },
    );
    target_and_pay(&mut g, a);
    g.move_object(source, Zone::Hand);
    settle(&mut g);
    assert_eq!(
        g.state.objects[source as usize].card,
        Card::from_name("Pterafractyl").unwrap()
    );
    assert_eq!(g.state.objects[source as usize].zone, Zone::Hand);
    assert!(g.temporary_copy_view(g.object_ref(source)).is_none());
}
#[test]
fn retained_activation_announcement_and_targeting_are_owned_across_raw_index_and_restore() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Pterafractyl").unwrap(), &MIME)]);
    let (mut parent, source, a, b) = setup();
    original(&mut parent, source, a);
    let saved = parent.snapshot();
    let action = Action::AnnounceRetainedCopyAbility {
        source: parent.object_ref(source),
    };
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let index = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let actor = indexed.actor().unwrap();
    let revision = indexed.revision();
    act(&mut raw, action);
    indexed.apply_index(actor, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    drop(parent);
    target_and_pay(&mut raw, b);
    settle(&mut raw);
    target_and_pay(&mut indexed, b);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    retained(&mut indexed, source, a);
    assert_eq!(
        indexed.state.objects[source as usize].card,
        Card::from_name("Grizzly Bears").unwrap()
    );
}
#[test]
fn a_new_controller_can_use_the_same_retained_linked_ability() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Pterafractyl").unwrap(), &MIME)]);
    let (mut g, source, a, b) = setup();
    original(&mut g, source, a);
    g.change_controller(g.object_ref(source), Player::P1);
    g.state.priority = Player::P1;
    retained(&mut g, source, b);
    assert_eq!(g.state.objects[source as usize].controller, Player::P1);
    assert_eq!(
        g.state.objects[source as usize].card,
        Card::from_name("Savannah Lions").unwrap()
    );
}
