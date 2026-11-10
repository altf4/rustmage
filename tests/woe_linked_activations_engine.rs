//! Outcome-gated linked exile and layer-ordered activation grants.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const ZERO: Mana = [0; 6];
const BLUE: Mana = [0, 1, 0, 0, 0, 0];
const COLORLESS: Mana = [0, 0, 0, 0, 0, 1];
const OWN_CREATURE: TargetPlan = TargetPlan::Single(TargetSpec::one(TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 1,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
}));
const ANY_GRAVE_CARD: TargetPlan = TargetPlan::Single(TargetSpec::one(TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 0,
    relation: -1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
}));

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

const PROVIDER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::SpendAnyColorForCreatureActivations],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[
                StaticAbilityFragment::GrantLinkedExileCreatureActivationsWithCounter(
                    CounterKind::PlusOne,
                ),
            ],
        },
    ],
    activated: &[ActivatedAbility {
        targets: ANY_GRAVE_CARD,
        ..ability(
            ActivationCost {
                tap: true,
                ..ActivationCost::ZERO
            },
            &[Instruction::ExileGraveTargetsLinkedGrant {
                creature_reflexive: 0,
            }],
        )
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: OWN_CREATURE,
        program: &[Instruction::PutCounters {
            subject: Subject::Targets,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};
const DONOR: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[
        ActivatedAbility {
            once_per_turn: true,
            ..ability(
                ActivationCost {
                    colored: [1, 0, 0, 0, 0, 0],
                    mana_component: true,
                    ..ActivationCost::ZERO
                },
                &[Instruction::PutCounters {
                    subject: Subject::Source,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                }],
            )
        },
        ability(
            ActivationCost {
                remove: Some((Some(CounterKind::PlusOne), 1)),
                ..ActivationCost::ZERO
            },
            &[Instruction::AddMana(ManaProduction::Fixed(BLUE))],
        ),
    ],
    ..CardRules::EMPTY
};

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn exile(game: &mut Game, provider: CardId, donor: CardId) {
    act(
        game,
        Action::AnnounceActivate {
            card: provider,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(game, target(donor));
    pay(game);
}

fn resolve_top(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn grow(card: CardId, exiled: ObjectRef) -> Action {
    Action::AnnounceBorrowedAbility {
        card,
        exiled,
        ability: 0,
        counter_payment: Counters::default(),
    }
}

fn count(game: &Game, card: CardId) -> u64 {
    game.counter_count(card, CounterKind::PlusOne)
}

struct ResetPrevention;
impl Drop for ResetPrevention {
    fn drop(&mut self) {
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(None));
    }
}

#[test]
fn only_an_actual_creature_exile_queues_a_separate_reflexive_target() {
    let _rules =
        fixture_rules::activate(&[(card("Short Bow"), &PROVIDER), (card("Bear Cub"), &DONOR)]);
    let player = Player::P0;
    let mut root = scenario(player);
    let provider = put(&mut root, player, "Short Bow", Zone::Battlefield);
    let recipient = put(&mut root, player, "Grizzly Bears", Zone::Battlefield);
    let foreign = put(
        &mut root,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let donor = put(&mut root, player.other(), "Bear Cub", Zone::Graveyard);
    let noncreature = put(&mut root, player, "Opt", Zone::Graveyard);
    let original_provider = root.object_ref(provider);

    let mut spell = root.clone();
    exile(&mut spell, provider, noncreature);
    resolve_top(&mut spell);
    assert_eq!(spell.object_view(noncreature).zone, Zone::Exile);
    assert_eq!(spell.state.pending, Pending::Priority);
    assert_eq!(count(&spell, recipient), 0);
    assert_eq!(
        spell.linked_exile_for_origin(original_provider, None),
        [spell.object_ref(noncreature)]
    );
    assert!(spell.observe(player).borrowed_abilities.is_empty());

    exile(&mut root, provider, donor);
    let original_donor = root.object_ref(donor);
    let mut prevented = root.clone();
    {
        let _reset = ResetPrevention;
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(Some(original_donor)));
        prevented.state.zone_rules.get_or_insert_default();
        resolve_top(&mut prevented);
        assert_eq!(prevented.object_ref(donor), original_donor);
        assert!(
            prevented
                .linked_exile_for_origin(original_provider, None)
                .is_empty()
        );
        assert_eq!(prevented.state.pending, Pending::Priority);
    }
    let mut stale = root.clone();
    stale.move_object(donor, Zone::Hand);
    stale.move_object(donor, Zone::Graveyard);
    resolve_top(&mut stale);
    assert_eq!(stale.object_view(donor).zone, Zone::Graveyard);
    assert_eq!(stale.state.pending, Pending::Priority);

    // The exact activated ability still performs its exile and reflexive
    // trigger after the granting permanent has left the battlefield.
    root.move_object(provider, Zone::Graveyard);
    resolve_top(&mut root);
    assert_eq!(root.object_view(donor).zone, Zone::Exile);
    assert_ne!(root.object_ref(donor), original_donor);
    assert_eq!(count(&root, recipient), 0);
    reject(&mut root, target(foreign));
    act(&mut root, target(recipient));
    resolve_top(&mut root);
    assert_eq!(count(&root, recipient), 1);
    assert!(root.observe(player).borrowed_abilities.is_empty());
}

#[test]
fn exact_donors_grant_independent_uses_in_layer_order_to_the_recipient() {
    let _rules =
        fixture_rules::activate(&[(card("Short Bow"), &PROVIDER), (card("Bear Cub"), &DONOR)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let first = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let receiver = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let other = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let first_donor = put(&mut game, player.other(), "Bear Cub", Zone::Graveyard);
    let second_donor = put(&mut game, player.other(), "Bear Cub", Zone::Graveyard);
    let receiver_ref = game.object_ref(receiver);
    let first_ref = game.object_ref(first);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
    game.add_counter(other, CounterKind::PlusOne, 1);

    exile(&mut game, first, first_donor);
    resolve_top(&mut game);
    act(&mut game, target(receiver));
    resolve_top(&mut game);
    let first_exile = game.object_ref(first_donor);
    let first_use = grow(receiver, first_exile);
    reject(&mut game, grow(other, first_exile));
    assert!(game.legal_actions().contains(&first_use));

    let mut no_counter = game.clone();
    no_counter.set_counters(receiver, Counters::default());
    reject(&mut no_counter, first_use.clone());
    let mut transferred = game.clone();
    transferred.change_controller(first_ref, player.other());
    reject(&mut transferred, first_use.clone());
    let mut blinked = game.clone();
    blinked.move_object(first, Zone::Exile);
    blinked.move_object(first, Zone::Battlefield);
    reject(&mut blinked, first_use.clone());
    let mut lost_donor = game.clone();
    lost_donor.move_object(first_donor, Zone::Graveyard);
    lost_donor.move_object(first_donor, Zone::Exile);
    reject(&mut lost_donor, first_use.clone());

    let mut layered = game.clone();
    layered.remove_abilities_exact(
        &[receiver_ref],
        layered.object_ref(other),
        player,
        AnimationDuration::EndOfTurn,
    );
    reject(&mut layered, first_use.clone());
    let late = put(&mut layered, player, "Short Bow", Zone::Battlefield);
    exile(&mut layered, late, second_donor);
    resolve_top(&mut layered);
    act(&mut layered, target(receiver));
    resolve_top(&mut layered);
    let late_use = grow(receiver, layered.object_ref(second_donor));
    assert!(layered.legal_actions().contains(&late_use));
    reject(&mut layered, first_use.clone());

    let second = put(&mut game, player, "Short Bow", Zone::Battlefield);
    exile(&mut game, second, second_donor);
    resolve_top(&mut game);
    act(&mut game, target(receiver));
    resolve_top(&mut game);
    let second_exile = game.object_ref(second_donor);
    let second_use = grow(receiver, second_exile);
    let rows = game.observe(player).borrowed_abilities;
    let first_row = rows
        .iter()
        .find(|row| row.exiled == first_exile && row.ability == 0)
        .unwrap();
    let second_row = rows
        .iter()
        .find(|row| row.exiled == second_exile && row.ability == 0)
        .unwrap();
    assert_eq!(first_row.source, receiver_ref);
    assert_eq!(second_row.source, receiver_ref);
    assert_eq!(first_row.grantor, Some(first_ref));
    assert_eq!(second_row.grantor, Some(game.object_ref(second)));
    assert_ne!(first_row.slot, second_row.slot);

    act(&mut game, first_use.clone());
    let domain = game.mana_payment_domain().unwrap();
    assert_eq!(domain.required, ZERO);
    assert_eq!(domain.generic, 1);
    pay(&mut game);
    let item = game.state.stack.last().unwrap();
    assert_eq!(item.source.unwrap().object, receiver_ref);
    assert_eq!(item.source.unwrap().card, card("Bear Cub"));
    let mut detached = game.clone();
    detached.move_object(first, Zone::Graveyard);
    detached.move_object(first_donor, Zone::Graveyard);
    resolve_top(&mut detached);
    assert_eq!(count(&detached, receiver), 3);
    assert_eq!(count(&detached, first_donor), 0);

    resolve_top(&mut game);
    reject(&mut game, first_use);
    act(&mut game, second_use);
    pay(&mut game);
    resolve_top(&mut game);
    assert_eq!(count(&game, receiver), 4);
    assert_eq!(count(&game, first_donor), 0);
    assert_eq!(count(&game, second_donor), 0);
}

#[test]
fn projected_borrowed_mana_can_spend_the_counter_that_granted_the_ability() {
    let _rules =
        fixture_rules::activate(&[(card("Short Bow"), &PROVIDER), (card("Bear Cub"), &DONOR)]);
    let player = Player::P1;
    let mut parent = scenario(player);
    let provider = put(&mut parent, player, "Short Bow", Zone::Battlefield);
    let recipient = put(&mut parent, player, "Grizzly Bears", Zone::Battlefield);
    let victim = put(
        &mut parent,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let donor = put(&mut parent, player.other(), "Bear Cub", Zone::Graveyard);
    let spell = put(&mut parent, player, "Unsummon", Zone::Hand);
    parent.state.players[player.index()].mana = ZERO;
    let cast = Action::AnnounceCast { card: spell };
    reject(&mut parent, cast.clone());
    exile(&mut parent, provider, donor);
    resolve_top(&mut parent);
    act(&mut parent, target(recipient));
    resolve_top(&mut parent);
    assert_eq!(count(&parent, recipient), 1);
    let exact = parent.object_ref(donor);
    act(&mut parent, cast);
    act(&mut parent, target(victim));

    let mana = Action::ActivateBorrowedMana {
        card: recipient,
        exiled: exact,
        ability: 1,
        payment: ZERO,
        counter_payment: Counters::of(CounterKind::PlusOne, 1),
        production: BLUE,
    };
    let index = parent
        .legal_actions()
        .iter()
        .position(|action| *action == mana)
        .unwrap();
    let saved = parent.snapshot();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    act(&mut raw, mana.clone());
    indexed
        .apply_index(player, parent.revision(), index)
        .unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    raw.restore(&saved).unwrap();
    act(&mut raw, mana.clone());
    for game in [&mut raw, &mut indexed] {
        assert_eq!(count(game, recipient), 0);
        assert_eq!(game.state.players[player.index()].mana, BLUE);
        assert_eq!(game.object_ref(donor), exact);
        reject(game, mana.clone());
        pay(game);
        settle(game);
        assert_eq!(game.object_view(victim).zone, Zone::Hand);
        assert!(game.observe(player).borrowed_abilities.is_empty());
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());
}
