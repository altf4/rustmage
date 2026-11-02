//! Exact self-granted max-speed templates in actual and speculative payment worlds.
//! Fixtures retain existing physical metadata and admit no ordinary DFT card.
mod support;
use rustmage::Card;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::*;
use support::*;

const MAX_GROUP: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::SourceMaxSpeedActivation(0)],
}];
const BLUE: Mana = [0, 3, 0, 0, 0, 0];
const MAX_MANA_ABILITY: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost {
        tap: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::AddMana(ManaProduction::Fixed(BLUE))],
    sorcery: false,
    once_per_turn: false,
};
const MAX_MANA: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: MAX_GROUP,
    activated: &[MAX_MANA_ABILITY],
    ..CardRules::EMPTY
};
const PAID_MAX_MANA: CardRules = CardRules {
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        ..MAX_MANA_ABILITY
    }],
    ..MAX_MANA
};
const GY_ABILITY: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Graveyard,
    cycling: false,
    cost: ActivationCost {
        generic: 3,
        mana_component: true,
        exile_self: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    sorcery: false,
    once_per_turn: false,
};
const MAX_GY: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: MAX_GROUP,
    activated: &[GY_ABILITY],
    ..CardRules::EMPTY
};
const MAX_HAND: CardRules = CardRules {
    activated: &[ActivatedAbility {
        origin: ActivationOrigin::Hand,
        cost: ActivationCost {
            discard: true,
            ..ActivationCost::ZERO
        },
        ..GY_ABILITY
    }],
    ..MAX_GY
};
const ABILITY_COUNTER: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Ability)),
        &[Instruction::Counter],
    ),
    ..CardRules::EMPTY
};
const BORROWER: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::BorrowGraveyardActivations("Bear")],
    }],
    ..CardRules::EMPTY
};
const EXILE_BORROWER: CardRules = CardRules {
    activated: &[ActivatedAbility {
        cost: ActivationCost::ZERO,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        program: &[Instruction::ExileAndBorrowActivatedAbilities],
        ..MAX_MANA_ABILITY
    }],
    extras: &CardExtras {
        borrows_exiled_activations: true,
        ..CardExtras::EMPTY
    },
    ..BORROWER
};
const EXILE_PROHIBITOR: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::GraveyardCardsCannotBeExiled],
    }],
    ..CardRules::EMPTY
};
const REMOVABLE_PROHIBITOR: CardRules = CardRules {
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        program: &[Instruction::AddMana(ManaProduction::Fixed([
            0, 0, 0, 0, 0, 3,
        ]))],
        ..MAX_MANA_ABILITY
    }],
    ..EXILE_PROHIBITOR
};

fn pay(g: &mut Game) {
    if !g
        .legal_actions()
        .iter()
        .any(|a| matches!(a, Action::PayCosts { .. }))
    {
        let a = g.state.announcing.as_ref().expect("payment announcement");
        let goal = g.funding_goal(a);
        let world = FundingState::from_game(g, &goal);
        eprintln!(
            "grant payment diagnostic: goal={goal:?}, obligations={}, continuation={}, source={:?}, grant={:?}",
            world.obligations(&goal),
            world.complete_funding_continuation(&goal),
            a.item.source,
            a.item.attachment_trigger
        );
    }
    support::pay(g);
}

fn source_action(g: &Game, card: CardId) -> Action {
    Action::AnnounceSourceGrant {
        source: g.object_ref(card),
        ability: 0,
    }
}
fn has_mana(g: &Game, id: CardId) -> bool {
    g.legal_actions().iter().any(|a| {
        matches!(a, Action::ActivateMana { card, ability: 0, .. }
            | Action::AnnounceMana { card, ability: 0, .. } if *card == id)
    })
}
fn can_cast(g: &Game, id: CardId) -> bool {
    g.legal_actions().iter().any(
        |a| matches!(a, Action::AnnounceCast { card } | Action::Cast { card, .. } if *card == id),
    )
}
fn mana(g: &mut Game, id: CardId) {
    choose(
        g,
        |a| matches!(a, Action::ActivateMana { card, ability: 0, .. } if *card == id),
    );
}
fn grant(g: &mut Game, id: CardId) {
    let action = source_action(g, id);
    act(g, action);
    pay(g);
}

#[test]
fn max_speed_mana_is_one_indexed_activation_and_funding_agrees_at_three_and_four() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &MAX_MANA)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let spell = put(&mut g, p, "Divination", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        g.state.players[p.index()].speed = 3;
        assert!(!g.source_activation_template_available(source, 0));
        assert!(!has_mana(&g, source));
        assert!(
            !can_cast(&g, spell),
            "funding must not invent the absent grant"
        );
        g.state.players[p.index()].speed = 4;
        assert!(g.source_activation_template_available(source, 0));
        assert!(has_mana(&g, source));
        assert!(can_cast(&g, spell));
        assert!(!g.legal_actions().contains(&source_action(&g, source)));
        assert!(
            g.granted_mana(source).is_empty(),
            "indexed costs are not flattened tap grants"
        );
        mana(&mut g, source);
        assert_eq!(g.state.players[p.index()].mana, BLUE);
        assert!(g.object_view(source).tapped);
        assert!(g.state.stack.is_empty(), "mana resolves immediately");
        cast(&mut g, spell, None);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
    }
}

#[test]
fn max_speed_mana_checks_current_controller_and_layer_six_suppression() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &MAX_MANA)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    g.state.players[0].speed = 4;
    let aura = put(&mut g, Player::P0, "Stop Cold", Zone::Hand);
    cast(&mut g, aura, Some(Target::Card(source)));
    settle(&mut g);
    g.untap_objects(&[source], untap_rules::UntapCause::Effect);
    g.state.players[0].mana = [0; 6];
    let spell = put(&mut g, Player::P0, "Divination", Zone::Hand);
    assert!(!g.source_activation_template_available(source, 0));
    assert!(!has_mana(&g, source));
    assert!(!can_cast(&g, spell));
    g.move_object(aura, Zone::Graveyard);
    assert!(g.source_activation_template_available(source, 0));
    assert!(has_mana(&g, source));
    g.record_control(g.object_ref(source), Player::P1, None);
    g.state.objects[source as usize].summoning_sick = false;
    assert!(
        !g.source_activation_template_available(source, 0),
        "owner speed no longer applies"
    );
    g.state.players[1].speed = 4;
    act(&mut g, Action::Pass);
    assert_eq!(g.actor(), Some(Player::P1));
    assert!(g.source_activation_template_available(source, 0));
    assert!(has_mana(&g, source));
    mana(&mut g, source);
    assert_eq!(g.state.players[1].mana[1], 11);
}

#[test]
fn nested_payment_keeps_indexed_grant_cost_provenance_and_owned_branches() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &PAID_MAX_MANA)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let land = put(&mut g, Player::P0, "Forest", Zone::Battlefield);
    let spell = put(&mut g, Player::P0, "Divination", Zone::Hand);
    g.state.players[0].mana = [0; 6];
    g.state.players[0].speed = 3;
    assert!(!can_cast(&g, spell));
    g.state.players[0].speed = 4;
    act(&mut g, Action::AnnounceCast { card: spell });
    choose(
        &mut g,
        |a| matches!(a, Action::AnnounceMana { card, ability: 0, .. } if *card == source),
    );
    assert_eq!(g.state.mana_announcement_stack.as_ref().unwrap().len(), 1);
    let item = &g.state.announcing.as_ref().unwrap().item;
    assert_eq!(item.kind, StackKind::ActivatedAbility { ability: 0 });
    let origin = item.attachment_trigger.as_ref().unwrap();
    assert_eq!(origin.grantor, g.object_ref(source));
    assert_eq!(origin.recipient, g.object_ref(source));
    assert!(origin.granted);
    assert!(
        !g.legal_actions().iter().any(|a| matches!(a,
        Action::AnnounceMana { card, ability: 0, .. } if *card == source)),
        "a mana ability cannot recursively pay its own activation"
    );
    let snapshot = g.snapshot();
    let mut raw = g.clone();
    let mut indexed = g.clone();
    let mut restored = g.clone();
    restored.restore(&snapshot).unwrap();
    drop(g);
    for (which, branch) in [&mut raw, &mut indexed, &mut restored]
        .into_iter()
        .enumerate()
    {
        let actions = branch.legal_actions();
        let index = actions
            .iter()
            .position(|a| *a == Action::TapMana { card: land })
            .unwrap();
        // Both APIs consume the same exact action at the same revision.
        if which == 1 {
            branch
                .apply_index(branch.actor().unwrap(), branch.revision(), index)
                .unwrap();
        } else {
            act(branch, actions[index].clone());
        }
        pay(branch);
        assert!(matches!(branch.state.pending, Pending::PayCosts { .. }));
        assert!(branch.state.mana_announcement_stack.is_none());
        pay(branch);
        settle(branch);
        assert_eq!(branch.state.players[0].cards_drawn_this_turn, 2);
        assert!(branch.object_view(source).tapped);
        assert!(branch.object_view(land).tapped);
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.snapshot(), restored.snapshot());
}

#[test]
fn graveyard_grant_uses_owner_speed_origin_and_exact_incarnation() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &MAX_GY)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let grave = put(&mut g, p, "Grizzly Bears", Zone::Graveyard);
        let battlefield = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let hand = put(&mut g, p, "Grizzly Bears", Zone::Hand);
        let exile = put(&mut g, p, "Grizzly Bears", Zone::Exile);
        g.state.objects[grave as usize].controller = p.other();
        g.state.players[p.other().index()].speed = 4;
        g.state.players[p.index()].speed = 3;
        let old = source_action(&g, grave);
        assert!(!g.legal_actions().contains(&old));
        g.state.players[p.index()].speed = 4;
        assert!(g.source_activation_template_available(grave, 0));
        assert!(g.legal_actions().contains(&old));
        for id in [battlefield, hand, exile] {
            assert!(!g.source_activation_template_available(id, 0));
            assert!(!g.legal_actions().contains(&source_action(&g, id)));
        }
        act(&mut g, old.clone());
        assert_eq!(g.state.announcing.as_ref().unwrap().item.controller, p);
        assert_eq!(g.object_view(grave).zone, Zone::Graveyard);
        pay(&mut g);
        assert_eq!(
            g.object_view(grave).zone,
            Zone::Exile,
            "exile is an activation cost"
        );
        assert_eq!(
            g.state
                .stack
                .last()
                .unwrap()
                .source
                .as_ref()
                .unwrap()
                .object
                .generation,
            0
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        g.move_object(grave, Zone::Graveyard);
        reject(&mut g, old);
        assert!(g.legal_actions().contains(&source_action(&g, grave)));
    }
}

#[test]
fn graveyard_exile_payment_survives_countering_and_source_departure() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &MAX_GY),
        (Card::Counterspell, &ABILITY_COUNTER),
    ]);
    let mut g = scenario(Player::P0);
    g.state.players[0].speed = 4;
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Graveyard);
    grant(&mut g, source);
    let ability = g.state.stack.last().unwrap().id;
    assert_eq!(g.object_view(source).zone, Zone::Exile);
    act(&mut g, Action::Pass);
    let counter = put(&mut g, Player::P1, "Counterspell", Zone::Hand);
    cast(&mut g, counter, Some(Target::Stack(ability)));
    settle(&mut g);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 0);
    assert_eq!(g.object_view(source).zone, Zone::Exile);
    g.move_object(source, Zone::Graveyard);
    grant(&mut g, source);
    g.move_object(source, Zone::Hand);
    settle(&mut g);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
    assert_eq!(g.object_view(source).zone, Zone::Hand);
}

#[test]
fn max_speed_hand_template_uses_existing_source_grant_replay() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &MAX_HAND)]);
    let mut g = scenario(Player::P1);
    let source = put(&mut g, Player::P1, "Grizzly Bears", Zone::Hand);
    g.state.players[1].speed = 3;
    assert!(!g.legal_actions().contains(&source_action(&g, source)));
    g.state.players[1].speed = 4;
    grant(&mut g, source);
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    settle(&mut g);
    assert_eq!(g.state.players[1].cards_drawn_this_turn, 1);
}

#[test]
fn graveyard_borrower_sees_only_current_granted_graveyard_templates() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &MAX_GY),
        (Card::GoblinPiker, &BORROWER),
    ]);
    let mut g = scenario(Player::P0);
    let borrower = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
    let donor = put(&mut g, Player::P0, "Grizzly Bears", Zone::Graveyard);
    g.state.players[0].speed = 3;
    assert!(g.observe(Player::P0).graveyard_borrowed_abilities.is_none());
    g.state.players[0].speed = 4;
    let inventory = g.observe(Player::P0).graveyard_borrowed_abilities.unwrap();
    assert_eq!(inventory.len(), 1);
    assert_eq!(inventory[0].source, g.object_ref(borrower));
    assert_eq!(inventory[0].graveyard, g.object_ref(donor));
    assert_eq!(inventory[0].ability, 0);
    g.move_object(donor, Zone::Exile);
    assert!(g.observe(Player::P0).graveyard_borrowed_abilities.is_none());
    g.move_object(donor, Zone::Graveyard);
    g.state.players[0].speed = 3;
    assert!(g.observe(Player::P0).graveyard_borrowed_abilities.is_none());
}

#[test]
fn battlefield_only_max_mana_is_not_an_ability_of_a_graveyard_or_exile_donor() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &MAX_MANA),
        (Card::GoblinPiker, &EXILE_BORROWER),
    ]);
    let mut g = scenario(Player::P0);
    let borrower = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
    let donor = put(&mut g, Player::P0, "Grizzly Bears", Zone::Graveyard);
    g.state.players[0].speed = 4;
    assert!(!g.source_activation_template_available(donor, 0));
    assert!(g.observe(Player::P0).graveyard_borrowed_abilities.is_none());
    let live_donor = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    assert!(g.source_activation_template_available(live_donor, 0));
    act(
        &mut g,
        Action::AnnounceActivate {
            card: borrower,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(live_donor),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(&mut g, Action::FinishTargets);
    }
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.object_view(live_donor).zone, Zone::Exile);
    assert!(!g.source_activation_template_available(live_donor, 0));
    assert!(
        g.observe(Player::P0).borrowed_abilities.is_empty(),
        "an exiled donor no longer has its battlefield-only granted activation"
    );
    assert!(g.observe(Player::P0).graveyard_borrowed_abilities.is_none());
}

#[test]
fn source_exile_prohibition_is_checked_transactionally_in_the_payment_world() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &MAX_GY),
        (Card::GoblinPiker, &EXILE_PROHIBITOR),
    ]);
    let mut g = scenario(Player::P0);
    g.state.players[0].speed = 4;
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Graveyard);
    let prohibition = put(&mut g, Player::P1, "Goblin Piker", Zone::Battlefield);
    let action = source_action(&g, source);
    assert!(!g.legal_actions().contains(&action));
    reject(&mut g, action.clone());
    g.move_object(prohibition, Zone::Graveyard);
    act(&mut g, action);
    let prohibition = put(&mut g, Player::P1, "Goblin Piker", Zone::Battlefield);
    assert!(
        !g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::PayCosts { .. }))
    );
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(
            Player::P0,
            Action::PayCosts {
                payment: [0, 0, 0, 0, 0, 3]
            }
        ),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
    g.move_object(prohibition, Zone::Graveyard);
    pay(&mut g);
    assert_eq!(g.object_view(source).zone, Zone::Exile);
    settle(&mut g);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
}

#[test]
fn source_exile_can_be_funded_by_sacrificing_its_prohibition_provider() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &MAX_GY),
        (Card::GoblinPiker, &REMOVABLE_PROHIBITOR),
    ]);
    let mut g = scenario(Player::P0);
    g.state.players[0].speed = 4;
    g.state.players[0].mana = [0; 6];
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Graveyard);
    let prohibition = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
    assert!(
        g.legal_actions().contains(&source_action(&g, source)),
        "funding may first remove the prohibition provider"
    );
    let action = source_action(&g, source);
    act(&mut g, action);
    mana(&mut g, prohibition);
    assert_eq!(g.object_view(prohibition).zone, Zone::Graveyard);
    pay(&mut g);
    assert_eq!(g.object_view(source).zone, Zone::Exile);
    settle(&mut g);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
}
