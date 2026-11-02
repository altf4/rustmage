//! Actual provenance allocations and fixed-output guided payment continuations.
//! Immutable overlays preserve every fixture's physical identity, types and costs.
mod support;
use rustmage as _;
use rustmage_core::CardName;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const GREEN: Mana = [0, 0, 0, 0, 1, 0];
const RED: Mana = [0, 0, 0, 1, 0, 0];
const RG: Mana = [0, 0, 0, 1, 1, 0];
const TWO_RED: Mana = [0, 0, 0, 2, 0, 0];
const fn ability(cost: ActivationCost, program: &'static [Instruction]) -> ActivatedAbility {
    ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost,
        targets: TargetPlan::NONE,
        program,
        sorcery: false,
        once_per_turn: false,
    }
}
const CHILD_ABILITY: ActivatedAbility = ability(
    ActivationCost {
        colored: GREEN,
        mana_component: true,
        tap: true,
        ..ActivationCost::ZERO
    },
    &[Instruction::AddMana(ManaProduction::Fixed(RED))],
);
const PLAIN: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    ..CardRules::EMPTY
};
const NATIVE: CardRules = CardRules {
    activated: &[CHILD_ABILITY],
    ..PLAIN
};
const DONOR: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    activated: &[CHILD_ABILITY],
    ..CardRules::EMPTY
};
const TEMPLATE: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    activated: &[CHILD_ABILITY],
    extras: &CardExtras {
        self_granted_activation_templates: &[0],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const MAX_GROUP: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::SourceMaxSpeedActivation(0)],
}];
const INDEXED: CardRules = CardRules {
    static_groups: MAX_GROUP,
    ..NATIVE
};
const GY_GROUP: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::BorrowGraveyardActivations("Elf")],
}];
const GY_BORROWER: CardRules = CardRules {
    static_groups: GY_GROUP,
    ..PLAIN
};
const EXILE_BORROWER: CardRules = CardRules {
    activated: &[ActivatedAbility {
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        ..ability(
            ActivationCost::ZERO,
            &[Instruction::ExileAndBorrowActivatedAbilities],
        )
    }],
    extras: &CardExtras {
        borrows_exiled_activations: true,
        ..CardExtras::EMPTY
    },
    ..PLAIN
};
const RESTRICTED: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    activated: &[ability(
        ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        &[Instruction::AddMana(ManaProduction::Tagged {
            production: &ManaProduction::Fixed(GREEN),
            quality: &ManaQuality {
                restriction: ManaRestriction::CreatureActivation,
                spell_bonus: ManaSpellBonus::None,
                retention: ManaRetention::Step,
            },
        })],
    )],
    ..CardRules::EMPTY
};
const PARENT: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Draw(PlayerSet::Controller, 1)],
    ),
    ..CardRules::EMPTY
};
const OPTIONAL: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::MayPayMana {
                cost: ManaCost {
                    colored: RG,
                    generic: 0,
                },
                skip: 1,
            },
            Instruction::GainLife(PlayerSet::Controller, 4),
        ],
    ),
    ..CardRules::EMPTY
};
const GUIDED_OPTIONAL: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::MayPayMana {
                cost: ManaCost {
                    colored: RED,
                    generic: 1,
                },
                skip: 1,
            },
            Instruction::GainLife(PlayerSet::Controller, 4),
        ],
    ),
    ..CardRules::EMPTY
};
const AURA_GROUP: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::AttachmentMana(
        ManaProduction::Fixed(RED),
    )],
}];
const GRANTOR: CardRules = CardRules {
    effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(TargetRule::Creature))),
    aura: Some(AuraSpec {
        kinds: 1,
        relation: 0,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        },
    }),
    static_groups: AURA_GROUP,
    extras: &CardExtras {
        static_ability_groups: AURA_GROUP,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const TAX_GROUP: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::NamedActivationGenericIncrease(1)],
}];
const TAX: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: TAX_GROUP,
    extras: &CardExtras {
        static_ability_groups: TAX_GROUP,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
#[derive(Clone, Copy, Debug)]
enum Route {
    Native,
    Indexed,
    Granted,
    Acquired,
    ExileBorrowed,
    GraveyardBorrowed,
}
const ROUTES: &[Route] = &[
    Route::Native,
    Route::Indexed,
    Route::Granted,
    Route::Acquired,
    Route::ExileBorrowed,
    Route::GraveyardBorrowed,
];
fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}
fn rules(route: Route) -> &'static CardRules {
    match route {
        Route::Native => &NATIVE,
        Route::Indexed => &INDEXED,
        Route::Granted | Route::Acquired => &PLAIN,
        Route::ExileBorrowed => &EXILE_BORROWER,
        Route::GraveyardBorrowed => &GY_BORROWER,
    }
}
fn scope(route: Route) -> fixture_rules::Guard {
    fixture_rules::activate(&[
        (Card::GrizzlyBears, rules(route)),
        (
            Card::LlanowarElves,
            if matches!(route, Route::Acquired) {
                &TEMPLATE
            } else {
                &DONOR
            },
        ),
        (named("Druid of the Cowl"), &RESTRICTED),
        (named("Konstrari Charm"), &PARENT),
        (named("Opt"), &OPTIONAL),
        (named("Stop Cold"), &GRANTOR),
        (named("Ghost Vacuum"), &TAX),
    ])
}
fn idle(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.resolving.is_none()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
}
fn drain(g: &mut Game) {
    for _ in 0..80 {
        if idle(g) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass | Action::PutTrigger { .. } | Action::FinishTargets
            )
        });
    }
    panic!(
        "payment continuation resolution ceiling {:?}",
        g.state.pending
    );
}
fn ready(g: &mut Game, predicate: impl Fn(&Action) -> bool) {
    for _ in 0..80 {
        if g.legal_actions().iter().any(&predicate) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("payment continuation choice ceiling {:?}", g.state.pending);
}
fn indexed(g: &mut Game, action: Action) {
    let at = g
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .expect("indexed legal action");
    g.apply_index(g.actor().unwrap(), g.revision(), at).unwrap();
}
fn announce(card: CardId, production: Mana) -> Action {
    Action::AnnounceMana {
        card,
        ability: 0,
        counter_payment: Counters::default(),
        production,
    }
}
fn direct(card: CardId, production: Mana) -> Action {
    Action::ActivateMana {
        card,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production,
    }
}
#[derive(Clone, Copy)]
struct Child {
    source: CardId,
    donor: Option<ObjectRef>,
    grantor: Option<ObjectRef>,
    timestamp: Option<u64>,
    route: Route,
}
impl Child {
    fn announced(self, g: &Game) -> Action {
        match self.route {
            Route::Native | Route::Indexed => announce(self.source, RED),
            Route::Granted => Action::AnnounceGrantedMana {
                card: self.source,
                grantor: self.grantor.unwrap(),
                production: RED,
            },
            Route::Acquired => Action::AnnounceSelfGrantedMana {
                source: g.object_ref(self.source),
                grant_timestamp: self.timestamp.unwrap(),
                counter_payment: Counters::default(),
                production: RED,
            },
            Route::ExileBorrowed => Action::AnnounceBorrowedMana {
                card: self.source,
                exiled: self.donor.unwrap(),
                ability: 0,
                counter_payment: Counters::default(),
                production: RED,
            },
            Route::GraveyardBorrowed => Action::AnnounceGraveyardBorrowedMana {
                card: self.source,
                graveyard: self.donor.unwrap(),
                ability: 0,
                counter_payment: Counters::default(),
                production: RED,
            },
        }
    }
    fn borrowed_direct(self) -> Action {
        match self.route {
            Route::ExileBorrowed => Action::ActivateBorrowedMana {
                card: self.source,
                exiled: self.donor.unwrap(),
                ability: 0,
                counter_payment: Counters::default(),
                payment: GREEN,
                production: RED,
            },
            Route::GraveyardBorrowed => Action::ActivateGraveyardBorrowedMana {
                card: self.source,
                graveyard: self.donor.unwrap(),
                ability: 0,
                counter_payment: Counters::default(),
                payment: GREEN,
                production: RED,
            },
            _ => panic!("not a direct borrowed route"),
        }
    }
}
fn setup(route: Route, p: Player, spare: u32) -> (Game, Child, CardId) {
    let mut g = scenario(p);
    let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let mut child = Child {
        source,
        donor: None,
        grantor: None,
        timestamp: None,
        route,
    };
    match route {
        Route::Native => (),
        Route::Indexed => g.state.players[p.index()].speed = 4,
        Route::Granted => {
            let grantor = put(&mut g, p, "Stop Cold", Zone::Battlefield);
            g.state.objects[grantor as usize].attached_to = Some(g.object_ref(source));
            child.grantor = Some(g.object_ref(grantor));
            let tax = put(&mut g, p.other(), "Ghost Vacuum", Zone::Battlefield);
            let object = g.object_ref(tax);
            g.set_entry_value(
                object,
                EntryChoiceValue::CardName(CardName::from_name("Grizzly Bears").unwrap()),
            );
        }
        Route::Acquired => {
            let timestamp = g.next_role_timestamp();
            let object = g.object_ref(source);
            g.grant_recipient_ability(
                object,
                Card::LlanowarElves,
                SelfGrantedAbilityKind::Activated { ability: 0 },
                timestamp,
                None,
            );
            child.timestamp = Some(timestamp);
        }
        Route::ExileBorrowed => {
            let donor = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(donor),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            pay(&mut g);
            drain(&mut g);
            assert_eq!(g.object_view(donor).zone, Zone::Exile);
            child.donor = Some(g.object_ref(donor));
        }
        Route::GraveyardBorrowed => {
            let donor = put(&mut g, p, "Llanowar Elves", Zone::Graveyard);
            g.refresh_graveyard_borrowing();
            child.donor = Some(g.object_ref(donor));
        }
    }
    let restricted = put(&mut g, p, "Druid of the Cowl", Zone::Battlefield);
    g.state.players[p.index()].mana = [0, 0, 0, 0, 1 + spare, 0];
    act(&mut g, direct(restricted, GREEN));
    assert_eq!(g.state.players[p.index()].mana[4], 2 + spare);
    assert_eq!(g.available_mana(p, mana_lots::Purpose::Other)[4], 1 + spare);
    let parent = put(&mut g, p, "Konstrari Charm", Zone::Hand);
    (g, child, parent)
}
fn start_parent(g: &mut Game, parent: CardId) {
    act(g, Action::AnnounceCast { card: parent });
    assert!(matches!(g.state.pending, Pending::PayCosts { .. }));
    let cost = g.state.announcing.as_ref().unwrap().total_cost.unwrap();
    assert_eq!(cost.colored, RG);
    assert_eq!(cost.generic, 0);
}
fn complete_forced_child(g: &mut Game, child: Child, p: Player) {
    // Singleton guided bounds may commit automatically. No legal branch may
    // spend the only ordinary G while preserving an unusable restricted G.
    assert!(
        !g.legal_actions()
            .contains(&Action::ChooseManaLotSpend { amount: 0 })
    );
    reject(g, Action::ChooseManaLotSpend { amount: 0 });
    if matches!(g.state.pending, Pending::ManaLotSpend { .. }) {
        act(g, Action::ChooseManaLotSpend { amount: 1 });
    }
    assert!(g.object_view(child.source).tapped);
    assert_eq!(g.state.players[p.index()].mana, RG);
    assert_eq!(g.available_mana(p, mana_lots::Purpose::Other), RG);
    assert!(g.observe(p).mana_lots.iter().all(|lot| lot.player != p));
    assert!(matches!(g.state.pending, Pending::PayCosts { .. }));
}
fn complete_parent(g: &mut Game, parent: CardId, p: Player, spare: u32, spent_lot: u32) {
    pay(g);
    drain(g);
    assert_eq!(g.object_view(parent).zone, Zone::Graveyard);
    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
    assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, spare, 0]);
    assert_eq!(
        g.available_mana(p, mana_lots::Purpose::Other)[4],
        if spare == 0 { 0 } else { spent_lot }
    );
}

#[test]
fn direct_borrowed_mana_battlefield_and_graveyard_routes_preserve_the_only_viable_lot_plan() {
    for route in [Route::ExileBorrowed, Route::GraveyardBorrowed] {
        let _rules = scope(route);
        for p in [Player::P0, Player::P1] {
            let (mut g, child, parent) = setup(route, p, 0);
            start_parent(&mut g, parent);
            act(&mut g, child.borrowed_direct());
            complete_forced_child(&mut g, child, p);
            complete_parent(&mut g, parent, p, 0, 1);
        }
    }
}

#[test]
fn nested_native_indexed_granted_acquired_and_borrowed_payments_keep_only_complete_lot_plans() {
    for &route in ROUTES {
        let _rules = scope(route);
        for p in [Player::P0, Player::P1] {
            let (mut g, child, parent) = setup(route, p, 0);
            start_parent(&mut g, parent);
            let start = child.announced(&g);
            act(&mut g, start);
            assert!(matches!(g.state.pending, Pending::PayCosts { .. }));
            pay(&mut g);
            complete_forced_child(&mut g, child, p);
            complete_parent(&mut g, parent, p, 0, 1);
        }
    }
}

#[test]
fn spare_ordinary_mana_keeps_both_provenance_allocations_legal_for_direct_and_nested_routes() {
    for &route in ROUTES {
        let _rules = scope(route);
        for direct_borrow in [false, true] {
            if direct_borrow && !matches!(route, Route::ExileBorrowed | Route::GraveyardBorrowed) {
                continue;
            }
            for amount in [0, 1] {
                let p = Player::P0;
                let (mut g, child, parent) = setup(route, p, 1);
                start_parent(&mut g, parent);
                let action = if direct_borrow {
                    child.borrowed_direct()
                } else {
                    child.announced(&g)
                };
                act(&mut g, action);
                if !direct_borrow {
                    pay(&mut g);
                }
                assert!(matches!(g.state.pending, Pending::ManaLotSpend { .. }));
                for available in [0, 1] {
                    assert!(
                        g.legal_actions()
                            .contains(&Action::ChooseManaLotSpend { amount: available }),
                        "{route:?}"
                    );
                }
                act(&mut g, Action::ChooseManaLotSpend { amount });
                complete_parent(&mut g, parent, p, 1, amount);
            }
        }
    }
}

#[test]
fn provenance_choice_owned_raw_index_and_restore_do_not_leak_or_reuse_an_allocation_plan() {
    let route = Route::GraveyardBorrowed;
    let _rules = scope(route);
    let p = Player::P1;
    let (mut parent, child, spell) = setup(route, p, 1);
    start_parent(&mut parent, spell);
    act(&mut parent, child.borrowed_direct());
    assert!(matches!(parent.state.pending, Pending::ManaLotSpend { .. }));
    let saved = parent.snapshot();
    let action = Action::ChooseManaLotSpend { amount: 1 };
    let mut raw = parent.clone();
    let mut idx = parent.clone();
    act(&mut raw, action.clone());
    indexed(&mut idx, action);
    assert_eq!(raw.snapshot(), idx.snapshot());
    drop(parent);
    complete_parent(&mut raw, spell, p, 1, 1);
    complete_parent(&mut idx, spell, p, 1, 1);
    assert_eq!(raw.snapshot(), idx.snapshot());
    idx.restore(&saved).unwrap();
    reject(&mut idx, Action::ChooseManaLotSpend { amount: 2 });
    act(&mut idx, Action::ChooseManaLotSpend { amount: 0 });
    complete_parent(&mut idx, spell, p, 1, 0);
    assert_eq!(
        idx.state.players[p.index()].mana,
        raw.state.players[p.index()].mana
    );
    assert_ne!(
        idx.available_mana(p, mana_lots::Purpose::Other),
        raw.available_mana(p, mana_lots::Purpose::Other)
    );
}

#[test]
fn standalone_and_optional_declinable_windows_do_not_prune_legal_lot_allocations() {
    let _rules = scope(Route::Native);
    for optional in [false, true] {
        for amount in [0, 1] {
            let p = Player::P0;
            let (mut g, child, _) = setup(Route::Native, p, 0);
            if optional {
                let spell = put(&mut g, p, "Opt", Zone::Hand);
                g.state.players[0].mana[1] = 1;
                cast(&mut g, spell, None);
                ready(&mut g, |a| *a == Action::DeclineResolutionMana);
            }
            if optional {
                act(
                    &mut g,
                    Action::ActivateMana {
                        card: child.source,
                        ability: 0,
                        payment: GREEN,
                        counter_payment: Counters::default(),
                        production: RED,
                    },
                );
            } else {
                let action = child.announced(&g);
                act(&mut g, action);
                pay(&mut g);
            }
            assert!(matches!(g.state.pending, Pending::ManaLotSpend { .. }));
            for available in [0, 1] {
                assert!(
                    g.legal_actions()
                        .contains(&Action::ChooseManaLotSpend { amount: available })
                );
            }
            act(&mut g, Action::ChooseManaLotSpend { amount });
            assert_eq!(g.state.players[0].mana, RG);
            assert_eq!(
                g.available_mana(p, mana_lots::Purpose::Other),
                [0, 0, 0, 1, amount, 0]
            );
            if optional {
                assert!(matches!(g.state.pending, Pending::ResolutionMana { .. }));
                act(&mut g, Action::DeclineResolutionMana);
                drain(&mut g);
                assert_eq!(g.state.players[0].life, 20);
            } else {
                assert_eq!(g.state.pending, Pending::Priority);
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Window {
    Plot,
    FaceUp,
    Optional,
}
fn guided_setup(p: Player, window: Window) -> (Game, CardId, CardId, CardId, CardId) {
    let mut g = scenario(p);
    let rouser = put(&mut g, p, "Rubble Rouser", Zone::Battlefield);
    let fuel = put(&mut g, p, "Forest", Zone::Graveyard);
    let mountain = put(&mut g, p, "Mountain", Zone::Battlefield);
    let destination = match window {
        Window::Plot => put(&mut g, p, "Slickshot Show-Off", Zone::Hand),
        Window::FaceUp => {
            let id = put(&mut g, p, "Slickshot Show-Off", Zone::Library);
            let object = g.object_ref(id);
            let entry = g.manifest_entry(object, p).unwrap();
            g.stage_entries(vec![entry], None);
            assert!(!g.advance_entries());
            assert!(g.is_face_down(g.object_ref(id)));
            id
        }
        Window::Optional => put(&mut g, p, "Opt", Zone::Hand),
    };
    g.state.players[p.index()].mana = [0; 6];
    match window {
        Window::Plot => act(&mut g, Action::AnnouncePlot { card: destination }),
        Window::FaceUp => {
            let object = g.object_ref(destination);
            act(
                &mut g,
                Action::BeginFaceUp {
                    object,
                    cost: ManaCost {
                        colored: RED,
                        generic: 1,
                    },
                },
            );
        }
        Window::Optional => {
            g.state.players[p.index()].mana[1] = 1;
            cast(&mut g, destination, None);
            ready(&mut g, |a| *a == Action::DeclineResolutionMana);
        }
    }
    (g, rouser, fuel, mountain, destination)
}
fn pay_rouser(g: &mut Game, rouser: CardId, fuel: CardId) {
    act(g, announce(rouser, RED));
    let object = g.object_ref(fuel);
    act(g, Action::ChooseAdditionalCost { object });
    pay(g);
}
fn tap_mountain(g: &mut Game, mountain: CardId) {
    assert!(
        g.state.announcing.is_none(),
        "completed child header released"
    );
    act(g, Action::TapMana { card: mountain });
    assert!(
        g.state.announcing.is_none(),
        "ordinary fixed output keeps the direct fast path"
    );
}

#[test]
fn actual_fixed_rouser_output_resumes_plot_and_face_up_payment_with_its_reflexive_trigger_queued() {
    for p in [Player::P0, Player::P1] {
        for window in [Window::Plot, Window::FaceUp] {
            let (mut g, rouser, fuel, mountain, destination) = guided_setup(p, window);
            let before = g.state.pending.clone();
            pay_rouser(&mut g, rouser, fuel);
            assert_eq!(
                g.state.pending, before,
                "{window:?} must resume its original payment"
            );
            assert_eq!(g.state.players[p.index()].mana, RED);
            assert_eq!(g.object_view(fuel).zone, Zone::Exile);
            assert!(g.object_view(rouser).tapped);
            assert_eq!(g.state.players[p.other().index()].life, 20);
            assert!(!g.state.pending_triggers.is_empty());
            tap_mountain(&mut g, mountain);
            assert_eq!(g.state.pending, before);
            assert_eq!(g.state.players[p.index()].mana, TWO_RED);
            match window {
                Window::Plot => {
                    act(&mut g, Action::PayPlot { payment: TWO_RED });
                    assert_eq!(g.object_view(destination).zone, Zone::Exile);
                    assert_eq!(g.observe(p).plotted_cards.len(), 1);
                }
                Window::FaceUp => {
                    act(&mut g, Action::PayFaceUp { payment: TWO_RED });
                    assert!(!g.is_face_down(g.object_ref(destination)));
                    assert_eq!(stats(&g, destination), (1, 2));
                    assert!(
                        g.keywords(destination)
                            .contains(Keywords::FLYING.union(Keywords::HASTE))
                    );
                }
                Window::Optional => unreachable!(),
            }
            drain(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(g.state.players[p.other().index()].life, 19);
        }
    }
}

#[test]
fn actual_fixed_rouser_output_keeps_optional_resolution_pay_and_decline_available() {
    let _rules = fixture_rules::activate(&[(named("Opt"), &GUIDED_OPTIONAL)]);
    for p in [Player::P0, Player::P1] {
        for accept in [false, true] {
            let (mut g, rouser, fuel, mountain, _) = guided_setup(p, Window::Optional);
            let pending = g.state.pending.clone();
            pay_rouser(&mut g, rouser, fuel);
            assert_eq!(g.state.pending, pending);
            assert!(g.legal_actions().contains(&Action::DeclineResolutionMana));
            tap_mountain(&mut g, mountain);
            assert_eq!(g.state.pending, pending);
            assert!(
                g.legal_actions()
                    .contains(&Action::PayResolutionMana { payment: TWO_RED })
            );
            if accept {
                act(&mut g, Action::PayResolutionMana { payment: TWO_RED });
            } else {
                act(&mut g, Action::DeclineResolutionMana);
            }
            drain(&mut g);
            assert_eq!(
                g.state.players[p.index()].life,
                if accept { 24 } else { 20 }
            );
            assert_eq!(
                g.state.players[p.index()].mana,
                if accept { [0; 6] } else { TWO_RED }
            );
            assert_eq!(g.state.players[p.other().index()].life, 19);
        }
    }
}

#[test]
fn fixed_announced_output_owns_the_guided_resume_across_raw_index_parent_disposal_and_restore() {
    let p = Player::P1;
    let (mut parent, rouser, fuel, mountain, plotted) = guided_setup(p, Window::Plot);
    act(&mut parent, announce(rouser, RED));
    let object = parent.object_ref(fuel);
    act(&mut parent, Action::ChooseAdditionalCost { object });
    let saved = parent.snapshot();
    let payment = parent
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    let mut raw = parent.clone();
    let mut idx = parent.clone();
    act(&mut raw, payment.clone());
    indexed(&mut idx, payment);
    assert_eq!(raw.snapshot(), idx.snapshot());
    drop(parent);
    for g in [&mut raw, &mut idx] {
        assert_eq!(g.state.pending, Pending::PlotPayment { player: p });
        tap_mountain(g, mountain);
        act(g, Action::PayPlot { payment: TWO_RED });
        drain(g);
        assert_eq!(g.object_view(plotted).zone, Zone::Exile);
        assert_eq!(g.state.players[p.other().index()].life, 19);
    }
    assert_eq!(raw.snapshot(), idx.snapshot());
    idx.restore(&saved).unwrap();
    pay(&mut idx);
    assert_eq!(idx.state.pending, Pending::PlotPayment { player: p });
    assert_eq!(idx.state.players[p.index()].mana, RED);
    assert_eq!(idx.object_view(fuel).zone, Zone::Exile);
    reject(&mut idx, Action::PayPlot { payment: RED });
    tap_mountain(&mut idx, mountain);
    act(&mut idx, Action::PayPlot { payment: TWO_RED });
    drain(&mut idx);
    assert_eq!(raw.snapshot(), idx.snapshot());
}

#[test]
fn converging_suffixes_keep_both_valid_provenance_plans_for_a_double_red_parent() {
    const ABILITY_RESTRICTED: CardRules = CardRules {
        effect: Effect::Creature(1, 3),
        activated: &[ability(
            ActivationCost {
                tap: true,
                ..ActivationCost::ZERO
            },
            &[Instruction::AddMana(ManaProduction::Tagged {
                production: &ManaProduction::Fixed(GREEN),
                quality: &ManaQuality {
                    restriction: ManaRestriction::AbilityActivation,
                    spell_bonus: ManaSpellBonus::None,
                    retention: ManaRetention::Step,
                },
            })],
        )],
        ..CardRules::EMPTY
    };
    const SECOND: CardRules = CardRules {
        effect: Effect::Creature(3, 3),
        activated: &[CHILD_ABILITY],
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &NATIVE),
        (Card::HillGiant, &SECOND),
        (named("Druid of the Cowl"), &ABILITY_RESTRICTED),
        (named("Scorching Shot"), &PARENT),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let first = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let second = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let producer = put(&mut g, p, "Druid of the Cowl", Zone::Battlefield);
        g.state.players[p.index()].mana = GREEN;
        act(&mut g, direct(producer, GREEN));
        let spell = put(&mut g, p, "Scorching Shot", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: spell });
        assert_eq!(
            g.state
                .announcing
                .as_ref()
                .unwrap()
                .total_cost
                .unwrap()
                .colored,
            TWO_RED
        );
        act(&mut g, announce(first, RED));
        pay(&mut g);
        assert!(matches!(g.state.pending, Pending::ManaLotSpend { .. }));
        for amount in [0, 1] {
            assert!(
                g.legal_actions()
                    .contains(&Action::ChooseManaLotSpend { amount }),
                "both prefixes converge to an identical RR suffix, so one plan cannot prune the other"
            );
        }
        let saved = g.snapshot();
        let mut raw = g.clone();
        let mut idx = g.clone();
        let action = Action::ChooseManaLotSpend { amount: 1 };
        act(&mut raw, action.clone());
        indexed(&mut idx, action);
        assert_eq!(raw.snapshot(), idx.snapshot());
        drop(g);
        for branch in [&mut raw, &mut idx] {
            act(branch, announce(second, RED));
            pay(branch);
            assert!(branch.object_view(first).tapped && branch.object_view(second).tapped);
            assert_eq!(branch.state.players[p.index()].mana, TWO_RED);
            assert_eq!(branch.available_mana(p, mana_lots::Purpose::Other), TWO_RED);
            complete_parent(branch, spell, p, 0, 1);
        }
        assert_eq!(raw.snapshot(), idx.snapshot());
        idx.restore(&saved).unwrap();
        act(&mut idx, Action::ChooseManaLotSpend { amount: 0 });
        act(&mut idx, announce(second, RED));
        pay(&mut idx);
        assert!(
            !idx.legal_actions()
                .contains(&Action::ChooseManaLotSpend { amount: 0 })
        );
        if matches!(idx.state.pending, Pending::ManaLotSpend { .. }) {
            act(&mut idx, Action::ChooseManaLotSpend { amount: 1 });
        }
        assert_eq!(idx.available_mana(p, mana_lots::Purpose::Other), TWO_RED);
        complete_parent(&mut idx, spell, p, 0, 1);
        assert_eq!(idx.snapshot(), raw.snapshot());
    }
}

const REPEATABLE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ability(
        ActivationCost {
            colored: RED,
            mana_component: true,
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::ExileGraveyard,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 0,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
            }),
            ..ActivationCost::ZERO
        },
        &[Instruction::AddMana(ManaProduction::Fixed(TWO_RED))],
    )],
    ..CardRules::EMPTY
};
const THREE_RED: Mana = [0, 0, 0, 3, 0, 0];
fn repeat_converter(g: &mut Game, source: CardId, fuel: CardId, expected: Pending, total_red: u32) {
    act(g, announce(source, TWO_RED));
    let object = g.object_ref(fuel);
    act(g, Action::ChooseAdditionalCost { object });
    act(g, Action::PayCosts { payment: RED });
    assert_eq!(g.state.pending, expected);
    assert!(!g.object_view(source).tapped);
    assert_eq!(g.object_view(fuel).zone, Zone::Exile);
    assert!(g.state.announcing.is_none());
    assert_eq!(
        g.state.players[g.state.active.index()].mana,
        [0, 0, 0, total_red, 0, 0]
    );
}

#[test]
fn completed_repeatable_converter_can_run_again_to_finish_root_plot_and_face_up_payments() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &REPEATABLE)]);
    for p in [Player::P0, Player::P1] {
        for plot in [false, true] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let first_fuel = put(&mut g, p, "Forest", Zone::Graveyard);
            let second_fuel = put(&mut g, p, "Mountain", Zone::Graveyard);
            let target = put(
                &mut g,
                p,
                "Irascible Wolverine",
                if plot { Zone::Hand } else { Zone::Library },
            );
            if !plot {
                let object = g.object_ref(target);
                let entry = g.manifest_entry(object, p).unwrap();
                g.stage_entries(vec![entry], None);
                assert!(!g.advance_entries());
            }
            g.state.players[p.index()].mana = RED;
            if plot {
                act(&mut g, Action::AnnouncePlot { card: target });
            } else {
                let object = g.object_ref(target);
                act(
                    &mut g,
                    Action::BeginFaceUp {
                        object,
                        cost: ManaCost {
                            colored: RED,
                            generic: 2,
                        },
                    },
                );
            }
            let pending = g.state.pending.clone();
            repeat_converter(&mut g, source, first_fuel, pending.clone(), 2);
            assert!(
                g.legal_actions().contains(&announce(source, TWO_RED)),
                "a completed activation instance is no longer an active recursion ancestor"
            );
            repeat_converter(&mut g, source, second_fuel, pending, 3);
            if plot {
                act(&mut g, Action::PayPlot { payment: THREE_RED });
                assert_eq!(g.object_view(target).zone, Zone::Exile);
                assert_eq!(g.observe(p).plotted_cards.len(), 1);
            } else {
                act(&mut g, Action::PayFaceUp { payment: THREE_RED });
                assert!(!g.is_face_down(g.object_ref(target)));
                assert_eq!(stats(&g, target), (3, 2));
            }
            drain(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert!(!g.object_view(source).tapped);
        }
    }
}

#[test]
fn root_attack_payment_keeps_its_owned_receipt_and_retires_each_completed_converter() {
    const ATTACK_TAX: CardRules = CardRules {
        effect: Effect::Permanent,
        static_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::AttackControllerTax {
                amount: 3,
                condition: StatValue::Fixed(1),
            }],
        }],
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &REPEATABLE),
        (named("Short Bow"), &ATTACK_TAX),
    ]);
    let p = Player::P1;
    let mut parent = scenario(p);
    let source = put(&mut parent, p, "Grizzly Bears", Zone::Battlefield);
    let first_fuel = put(&mut parent, p, "Forest", Zone::Graveyard);
    let second_fuel = put(&mut parent, p, "Mountain", Zone::Graveyard);
    let attacker = put(&mut parent, p, "Goblin Piker", Zone::Battlefield);
    put(&mut parent, p.other(), "Short Bow", Zone::Battlefield);
    parent.state.players[p.index()].mana = RED;
    parent.state.step = Step::Attackers;
    parent.state.pending = Pending::Attackers;
    act(&mut parent, Action::Attack { card: attacker });
    act(&mut parent, Action::FinishAttackers);
    let pending = Pending::AttackPayment { player: p };
    assert_eq!(parent.state.pending, pending);
    repeat_converter(&mut parent, source, first_fuel, pending.clone(), 2);
    let saved = parent.snapshot();
    let mut raw = parent.clone();
    let mut idx = parent.clone();
    let start = announce(source, TWO_RED);
    act(&mut raw, start.clone());
    indexed(&mut idx, start);
    assert_eq!(raw.snapshot(), idx.snapshot());
    drop(parent);
    for g in [&mut raw, &mut idx] {
        let object = g.object_ref(second_fuel);
        act(g, Action::ChooseAdditionalCost { object });
        act(g, Action::PayCosts { payment: RED });
        assert_eq!(g.object_view(second_fuel).zone, Zone::Exile);
        assert_eq!(g.state.pending, pending);
        reject(g, Action::PayAttackCost { payment: TWO_RED });
        act(g, Action::PayAttackCost { payment: THREE_RED });
        assert_eq!(g.state.attackers.len(), 1);
        assert_eq!(g.state.attackers[0].card, attacker);
        assert!(g.object_view(attacker).tapped);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
    assert_eq!(raw.snapshot(), idx.snapshot());
    idx.restore(&saved).unwrap();
    repeat_converter(&mut idx, source, second_fuel, pending, 3);
    act(&mut idx, Action::PayAttackCost { payment: THREE_RED });
    assert_eq!(raw.snapshot(), idx.snapshot());
}
