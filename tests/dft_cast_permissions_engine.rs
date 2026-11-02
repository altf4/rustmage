//! Independent hidden exile entitlements and explicit graveyard casting routes.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const ALL_CREATURES: CreatureFilter = CreatureFilter {
    relation: -1,
    other: false,
    subtype: None,
    keyword: Keywords::NONE,
    attacking: false,
    tokens: None,
    artifact: false,
};
const GONTI: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayerDamaged {
            subject: DamageSubject::Creatures {
                filter: ALL_CREATURES,
                qualification: DamageQualification::Any,
            },
            combat_only: true,
            opponent_only: true,
            one_or_more: false,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ExileLibraryTopFaceDownPlay {
            library: PlayerSet::DamagedPlayer,
            beneficiary: PlayerSet::EventPlayer,
            amount: 1,
            spend_any_mana: true,
        }],
    }],
    ..CardRules::EMPTY
};
const SCRY: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, &[Instruction::Scry(1)]),
    ..CardRules::EMPTY
};
const COST: AdditionalCost = AdditionalCost {
    kind: EffectChoiceKind::Sacrifice,
    filter: SacrificeFilter {
        keywords: Keywords::NONE,
        source_only: false,
        kinds: 17,
        nonland: false,
        other: false,
        tokens: None,
        artifact_enchantment_or_token: false,
        subtype: None,
    },
};
const ROUTE_GROUP: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[
        StaticAbilityFragment::SelfGraveyardCastWithAdditionalCosts {
            life: 2,
            additional: COST,
        },
    ],
}];
const WICKER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: ROUTE_GROUP,
    extras: &CardExtras {
        static_ability_groups: ROUTE_GROUP,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const ARTIFACT_MANA: CardRules = CardRules {
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
        program: &[Instruction::AddMana(ManaProduction::Fixed([
            0, 0, 0, 0, 1, 0,
        ]))],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const PREVENT_SAC_GROUP: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::ControlledPermanentsCannotBeSacrificed],
}];
const PREVENT_SAC: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    static_groups: PREVENT_SAC_GROUP,
    extras: &CardExtras {
        static_ability_groups: PREVENT_SAC_GROUP,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}
fn route(card: CardId) -> Action {
    Action::AnnounceGraveyardAdditionalCosts { card }
}
fn idle(g: &Game) -> bool {
    g.state.stack.is_empty()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
        && g.state.resolving.is_none()
        && g.state.pending == Pending::Priority
}
fn drain(g: &mut Game) {
    for _ in 0..64 {
        if idle(g) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::FinishTargets
                    | Action::FinishStackTargets
            )
        });
    }
    panic!("64-action permission resolution ceiling");
}
fn main_phase(g: &mut Game, p: Player) {
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Main2;
    g.state.pending = Pending::Priority;
    g.state.passes = 0;
}
fn attack_until_trigger(g: &mut Game, observer: CardId, attackers: &[CardId]) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    g.state.priority = g.state.active;
    g.state.passes = 0;
    for &card in attackers {
        act(g, Action::Attack { card });
    }
    act(g, Action::FinishAttackers);
    for _ in 0..48 {
        if g.state
            .stack
            .iter()
            .chain(&g.state.pending_triggers)
            .chain(&g.state.ordering_triggers)
            .any(|item| {
                item.source.is_some_and(|s| s.id == observer)
                    && matches!(item.kind, StackKind::TriggeredAbility { .. })
            })
        {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass
                    | Action::FinishBlockers
                    | Action::AssignDamage {
                        target: Target::Player(_),
                        ..
                    }
                    | Action::PutTrigger { .. }
            )
        });
    }
    panic!("48-action Gonti combat ceiling");
}
fn exile_cast(g: &mut Game, card: CardId, target: Target) {
    act(g, Action::AnnounceExile { card });
    act(g, Action::ChooseTarget { target });
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
}
fn cost_index(g: &Game, clause: CastCostClause) -> u16 {
    g.observe(g.actor().unwrap())
        .cast_costs
        .unwrap()
        .clauses
        .iter()
        .position(|r| r.clause == clause && !r.paid)
        .unwrap() as u16
}
fn pay_route(g: &mut Game, material: CardId) {
    for _ in 0..8 {
        if g.state.announcing.is_none() {
            return;
        }
        if matches!(g.state.pending, Pending::PayCosts { .. }) {
            pay(g);
        } else if matches!(g.state.pending, Pending::CastCosts { .. }) {
            let costs = g.observe(g.actor().unwrap()).cast_costs.unwrap();
            let clause = costs
                .clauses
                .iter()
                .enumerate()
                .find(|(_, r)| !r.paid)
                .unwrap()
                .0 as u16;
            act(g, Action::ChooseCastCost { clause });
        } else if matches!(g.state.pending, Pending::AdditionalCost { .. }) {
            let object = g.object_ref(material);
            act(g, Action::ChooseAdditionalCost { object });
        } else {
            panic!("unexpected route payment phase {:?}", g.state.pending);
        }
    }
    panic!("8-action graveyard route payment ceiling");
}

#[test]
fn combat_exile_uses_frozen_dealer_controller_and_independent_permanent_private_permission() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &GONTI)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let observer = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let dealer = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        g.state.objects[dealer as usize].controller = p;
        let top = put(&mut g, p.other(), "Shock", Zone::Library);
        attack_until_trigger(&mut g, observer, &[dealer]);
        let packet = g
            .state
            .stack
            .iter()
            .chain(&g.state.pending_triggers)
            .chain(&g.state.ordering_triggers)
            .find(|s| s.source.is_some_and(|x| x.id == observer))
            .unwrap();
        assert_eq!(packet.event_player, Some(p));
        g.state.objects[dealer as usize].controller = p.other();
        g.move_object(dealer, Zone::Graveyard);
        g.move_object(observer, Zone::Graveyard);
        drain(&mut g);
        let exact = g.object_ref(top);
        assert_eq!(g.object_view(top).zone, Zone::Exile);
        assert!(
            g.observe(p)
                .objects
                .iter()
                .any(|o| o.id == top && o.card == named("Shock"))
        );
        assert!(
            !g.observe(p.other()).objects.iter().any(|o| o.id == top),
            "library owner has no right to look at face-down exile"
        );
        let permission = g
            .state
            .exile_permissions
            .iter()
            .find(|row| row.object == exact)
            .unwrap();
        assert_eq!(permission.player, p);
        assert_eq!(permission.expiry, ExilePlayExpiry::WhileExiled);
        assert!(permission.spend_any_mana);
        g.finish_cleanup();
        main_phase(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceExile { card: top })
        );
        exile_cast(&mut g, top, Target::Player(p.other()));
        drain(&mut g);
        assert_eq!(
            g.object_view(top).zone,
            Zone::Graveyard,
            "permission does not add an exile-on-death replacement"
        );
        assert!(g.observe(p.other()).objects.iter().any(|o| o.id == top));
    }
}

#[test]
fn hidden_land_permissions_use_normal_timing_and_one_shared_land_limit() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &GONTI)]);
    let p = Player::P1;
    let mut g = scenario(p);
    let observer = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let a = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let b = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
    let first = put(&mut g, p.other(), "Forest", Zone::Library);
    let second = put(&mut g, p.other(), "Forest", Zone::Library);
    attack_until_trigger(&mut g, observer, &[a, b]);
    drain(&mut g);
    assert_eq!(g.object_view(first).zone, Zone::Exile);
    assert_eq!(g.object_view(second).zone, Zone::Exile);
    assert!(
        !g.legal_actions()
            .contains(&Action::PlayLand { card: first }),
        "combat is not land-play timing"
    );
    main_phase(&mut g, p);
    act(&mut g, Action::PlayLand { card: first });
    assert_eq!(g.object_view(first).controller, p);
    assert_eq!(g.object_view(first).owner, p.other());
    reject(&mut g, Action::PlayLand { card: second });
    g.move_object(second, Zone::Graveyard);
    g.move_object(second, Zone::Exile);
    assert!(
        !g.can_play_exiled(p, second),
        "new exile incarnation never inherits the old permission"
    );
}

#[test]
fn combat_exile_rejects_noncombat_and_damage_to_observers_own_player() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &GONTI)]);
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let spell = put(&mut g, p, "Shock", Zone::Hand);
    cast(&mut g, spell, Some(Target::Player(p.other())));
    drain(&mut g);
    assert!(g.state.exile_permissions.is_empty());
    let observer = g
        .state
        .objects
        .iter()
        .position(|o| o.zone == Zone::Battlefield && o.card == Card::GrizzlyBears)
        .unwrap() as CardId;
    let dealer = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    g.state.active = p.other();
    g.state.priority = p.other();
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    act(&mut g, Action::Attack { card: dealer });
    act(&mut g, Action::FinishAttackers);
    for _ in 0..48 {
        if g.state.step == Step::EndCombat && idle(&g) {
            break;
        }
        choose(&mut g, |a| {
            matches!(
                a,
                Action::Pass
                    | Action::FinishBlockers
                    | Action::AssignDamage {
                        target: Target::Player(_),
                        ..
                    }
                    | Action::PutTrigger { .. }
            )
        });
    }
    assert_eq!(g.state.players[p.index()].life, 17);
    assert!(g.state.exile_permissions.is_empty());
    assert!(g.on_battlefield(observer));
}

#[test]
fn prevented_exile_retains_only_beneficiary_top_knowledge_with_owned_restore_and_invalidation() {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|v| v.set(None));
        }
    }
    let _reset = Reset;
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &GONTI), (named("Opt"), &SCRY)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let owner = p.other();
        let observer = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let dealer = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let top = put(&mut g, owner, "Shock", Zone::Library);
        let exact = g.object_ref(top);
        attack_until_trigger(&mut g, observer, &[dealer]);
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|v| v.set(Some(exact)));
        g.state.zone_rules.get_or_insert_default();
        // The final pass actually resolves the queued instruction in both branches.
        g.state.passes = 1;
        g.state.priority = owner;
        let action = Action::Pass;
        let rev = g.revision();
        let index = g.legal_actions().iter().position(|a| a == &action).unwrap();
        let snapshot = g.snapshot();
        let mut raw = g.clone();
        let mut indexed = g.clone();
        drop(g);
        raw.apply(owner, action.clone()).unwrap();
        indexed.apply_index(owner, rev, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(
            raw.observe(p).known_library_top[owner.index()]
                .unwrap()
                .object,
            exact
        );
        assert!(raw.observe(owner).known_library_top[owner.index()].is_none());
        assert!(raw.public_library_top(owner).is_none());
        assert!(raw.state.exile_permissions.is_empty());
        assert_eq!(raw.object_view(top).zone, Zone::Library);
        indexed.restore(&snapshot).unwrap();
        assert!(indexed.observe(p).known_library_top[owner.index()].is_none());
        assert_eq!(
            indexed.apply_index(owner, rev, index),
            Err(Error::StaleRevision)
        );
        act(&mut indexed, action);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|v| v.set(None));
        let mut shuffled = raw.clone();
        shuffled.shuffle_library_cards(owner);
        assert!(shuffled.observe(p).known_library_top[owner.index()].is_none());
        let mut reordered = raw.clone();
        reordered.state.priority = owner;
        reordered.state.passes = 0;
        // Advancing through combat emptied the fixture's initial mana pools.
        reordered.state.players[owner.index()].mana = [0, 1, 0, 0, 0, 0];
        let scry = put(&mut reordered, owner, "Opt", Zone::Hand);
        cast(&mut reordered, scry, None);
        for _ in 0..32 {
            if matches!(reordered.state.pending, Pending::EffectLibrary { .. }) {
                break;
            }
            choose(&mut reordered, |action| {
                matches!(action, Action::Pass | Action::PutTrigger { .. })
            });
        }
        act(
            &mut reordered,
            Action::ArrangeLibrary {
                card: top,
                destination: LibraryDestination::Top,
            },
        );
        drain(&mut reordered);
        assert_eq!(reordered.top_library_card(owner), Some(top));
        assert!(
            reordered.observe(p).known_library_top[owner.index()].is_none(),
            "a private reorder decision invalidates prior correlation even if it happens to retain the same top"
        );
        let mut moved = raw.clone();
        moved.move_object(top, Zone::Graveyard);
        moved.move_object(top, Zone::Library);
        assert_ne!(moved.object_ref(top), exact);
        assert!(moved.observe(p).known_library_top[owner.index()].is_none());
        raw.draw(owner);
        assert!(raw.observe(p).known_library_top[owner.index()].is_none());
    }
}

#[test]
fn self_graveyard_route_charges_life_and_sacrifice_in_addition_to_printed_mana() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &WICKER)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Grizzly Bears", Zone::Graveyard);
        let material = put(&mut g, p, "Clamorous Ironclad", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
        act(&mut g, route(source));
        pay_route(&mut g, material);
        assert_eq!(g.state.players[p.index()].life, 18);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.object_view(material).zone, Zone::Graveyard);
        assert_eq!(g.object_view(source).zone, Zone::Stack);
        assert_eq!(
            g.state.stack.last().unwrap().cast.unwrap().method,
            CastMethod::GraveyardAdditionalCosts
        );
        drain(&mut g);
        g.move_object(source, Zone::Graveyard);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    }
}

#[test]
fn normal_hand_and_external_graveyard_permission_never_inherit_intrinsic_route_costs() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &WICKER)]);
    for p in [Player::P0, Player::P1] {
        for grave in [false, true] {
            let mut g = scenario(p);
            g.state.players[p.index()].life = 1;
            let source = put(
                &mut g,
                p,
                "Grizzly Bears",
                if grave { Zone::Graveyard } else { Zone::Hand },
            );
            if grave {
                let object = g.object_ref(source);
                g.grant_graveyard_permission(p, object, GraveyardCastKind::Cast);
            }
            g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
            assert!(!g.legal_actions().contains(&route(source)));
            cast(&mut g, source, None);
            drain(&mut g);
            assert_eq!(g.state.players[p.index()].life, 1);
            assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        }
    }
}

#[test]
fn intrinsic_graveyard_route_rejects_wrong_owner_timing_life_and_sacrifice_legality() {
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &WICKER),
        (Card::HillGiant, &PREVENT_SAC),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Grizzly Bears", Zone::Graveyard);
    assert!(
        !g.legal_actions().contains(&route(source)),
        "graveyard card cannot sacrifice itself"
    );
    let material = put(&mut g, p, "Clamorous Ironclad", Zone::Battlefield);
    assert!(g.legal_actions().contains(&route(source)));
    g.state.players[p.index()].life = 1;
    reject(&mut g, route(source));
    g.state.players[p.index()].life = 2;
    assert!(g.legal_actions().contains(&route(source)));
    g.state.players[p.index()].life = 20;
    g.state.step = Step::BeginCombat;
    reject(&mut g, route(source));
    g.state.step = Step::Main1;
    g.state.objects[material as usize].controller = p.other();
    reject(&mut g, route(source));
    g.state.objects[material as usize].controller = p;
    let blocker = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    reject(&mut g, route(source));
    g.move_object(blocker, Zone::Graveyard);
    g.state.objects[source as usize].owner = p.other();
    reject(&mut g, route(source));
}

#[test]
fn route_funding_allows_tap_then_sacrifice_but_rejects_double_consumed_treasure() {
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &WICKER),
        (named("Clamorous Ironclad"), &ARTIFACT_MANA),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Grizzly Bears", Zone::Graveyard);
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
    let treasure = g.state.objects.len() as CardId;
    g.create_tokens(p, named("@token/b60f6cd3b44b22bd289fe4e2"), 1, false);
    assert!(
        !g.legal_actions().contains(&route(source)),
        "the only artifact cannot both sacrifice for mana and pay the separate sacrifice"
    );
    g.move_object(treasure, Zone::Exile);
    let producer = put(&mut g, p, "Clamorous Ironclad", Zone::Battlefield);
    assert!(g.legal_actions().contains(&route(source)));
    act(&mut g, route(source));
    choose(
        &mut g,
        |a| matches!(a,Action::ActivateMana {card,..}if *card==producer),
    );
    pay_route(&mut g, producer);
    assert_eq!(g.object_view(producer).zone, Zone::Graveyard);
    assert_eq!(g.state.players[p.index()].life, 18);
    drain(&mut g);
    assert_eq!(g.object_view(source).zone, Zone::Battlefield);
}

#[test]
fn route_nonmana_costs_can_precede_mana_after_the_mana_window_is_closed() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &WICKER)]);
    let p = Player::P1;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Grizzly Bears", Zone::Graveyard);
    let material = put(&mut g, p, "Soul-Guide Lantern", Zone::Battlefield);
    g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
    act(&mut g, route(source));
    act(&mut g, Action::BeginOrderedCosts);
    let clause = cost_index(&g, CastCostClause::Life(2));
    act(&mut g, Action::ChooseCastCost { clause });
    assert_eq!(g.state.players[p.index()].life, 18);
    assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 1, 1]);
    let clause = cost_index(&g, CastCostClause::Additional);
    act(&mut g, Action::ChooseCastCost { clause });
    let object = g.object_ref(material);
    act(&mut g, Action::ChooseAdditionalCost { object });
    assert_eq!(g.object_view(material).zone, Zone::Graveyard);
    let clause = cost_index(&g, CastCostClause::Mana);
    act(&mut g, Action::ChooseCastCost { clause });
    pay(&mut g);
    drain(&mut g);
    assert_eq!(g.object_view(source).zone, Zone::Battlefield);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}
