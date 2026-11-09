mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const SOURCE: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEntersOrAttacks,
            targets: TargetPlan::NONE,
            program: &[Instruction::SelectLibraryThenReflexiveEquipment {
                effect: LibrarySelectionEffect {
                    spec: LibrarySelectionSpec {
                        look: 6,
                        count: LibrarySelectionCount::All,
                        filter: LibrarySelectionFilter::Subtype("Equipment"),
                        optional: true,
                        reveal: false,
                        remainder: LibraryRemainder::BottomRandom,
                    },
                    look: LibraryLook::Fixed(6),
                    destination: LibraryPickDestination::Battlefield,
                    public: false,
                    maximum_mana_value: None,
                },
                ability: 1,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::NONE,
            program: &[Instruction::MayAttachReflexiveEquipment {
                recipient_subtype: "Samurai",
            }],
        },
    ],
    ..CardRules::EMPTY
};
const COUNTER: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Ability)),
        &[Instruction::Counter],
    ),
    ..CardRules::EMPTY
};
const BLINK: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 16,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[Instruction::BlinkImmediately {
            counter: None,
            tapped: false,
        }],
    ),
    ..CardRules::EMPTY
};
const PROTECTED_SAMURAI: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::PROTECTION_EVERYTHING,
    ..CardRules::EMPTY
};
const PROVIDER: &str = "Colleen Wing, Street Samurai";
const REFLEXIVE: StackKind = StackKind::TriggeredAbility { ability: 1 };

fn fixture() -> fixture_rules::Guard {
    fixture_rules::activate(&[
        (Card::from_name(PROVIDER).unwrap(), &SOURCE),
        (Card::from_name("Negate").unwrap(), &COUNTER),
        (Card::from_name("Opt").unwrap(), &BLINK),
        (
            Card::from_name("Leonardo, Sewer Samurai").unwrap(),
            &PROTECTED_SAMURAI,
        ),
    ])
}

fn passes(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn look(game: &mut Game, player: Player) -> ObjectRef {
    let source = put(game, player, PROVIDER, Zone::Hand);
    cast(game, source, None);
    for _ in 0..10 {
        if matches!(game.state.pending, Pending::EffectLibrarySelection { .. }) {
            return game.object_ref(source);
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("the entry trigger did not reach private selection");
}

fn order_entries(game: &mut Game) {
    while matches!(game.state.pending, Pending::EntryTimestamp { .. }) {
        choose(game, |action| {
            matches!(action, Action::ChooseEntryTimestamp { .. })
        });
    }
}

struct ResetPrevention;
impl Drop for ResetPrevention {
    fn drop(&mut self) {
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|slot| slot.set(None));
    }
}

#[test]
fn private_selection_queues_once_for_successful_exact_entries_and_can_be_countered() {
    let _rules = fixture();
    let _reset = ResetPrevention;
    let player = Player::P0;
    let mut game = scenario(player);
    let unrelated = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    let foreign = put(&mut game, player.other(), PROVIDER, Zone::Battlefield);
    let protected = put(
        &mut game,
        player,
        "Leonardo, Sewer Samurai",
        Zone::Battlefield,
    );
    let nonsamurai = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let buried = put(&mut game, player, "Short Bow", Zone::Library);
    let first = put(&mut game, player, "Short Bow", Zone::Library);
    let second = put(&mut game, player, "Lavaspur Boots", Zone::Library);
    let prevented = put(&mut game, player, "Gold Pan", Zone::Library);
    let other_cards =
        ["Mountain", "Island", "Forest"].map(|name| put(&mut game, player, name, Zone::Library));
    let source = look(&mut game, player);
    let view = game.observe(player).library_selection.unwrap();
    assert_eq!(view.remaining.len(), 6);
    assert_eq!(view.candidates.len(), 3);
    assert!(game.observe(player.other()).library_selection.is_none());
    for card in [buried, other_cards[0]] {
        reject(&mut game, Action::SelectLibrary { card });
    }
    let old_prevented = game.object_ref(prevented);
    for card in [first, second, prevented] {
        act(&mut game, Action::SelectLibrary { card });
    }
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|slot| slot.set(Some(old_prevented)));
    game.state.zone_rules.get_or_insert_default();
    let saved = game.snapshot();
    let action = Action::FinishLibrarySelection;
    let index = game
        .legal_actions()
        .iter()
        .position(|candidate| *candidate == action)
        .unwrap();
    let revision = game.revision();
    let mut indexed = game.clone();
    act(&mut game, action);
    indexed.apply_index(player, revision, index).unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    order_entries(&mut game);
    order_entries(&mut indexed);
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    assert_eq!(indexed.snapshot(), saved);
    assert_eq!(
        game.state.players[player.index()].library.last(),
        Some(&buried)
    );
    let mut bottom = game.state.players[player.index()].library[..4].to_vec();
    let mut expected = other_cards
        .into_iter()
        .chain([prevented])
        .collect::<Vec<_>>();
    bottom.sort_unstable();
    expected.sort_unstable();
    assert_eq!(bottom, expected);
    assert_eq!(game.object_ref(prevented), old_prevented);
    assert_eq!(game.object_view(prevented).zone, Zone::Library);

    assert_eq!(game.state.stack.len(), 1);
    let reflexive = &game.state.stack[0];
    assert_eq!(reflexive.kind, REFLEXIVE);
    assert!(reflexive.targets.is_empty());
    let subject = reflexive.reflexive_subject.as_ref().unwrap();
    let first_ref = game.object_ref(first);
    let second_ref = game.object_ref(second);
    let mut entrants = [subject.object, subject.additional_objects[0]];
    entrants.sort_unstable();
    assert_eq!(entrants, [first_ref, second_ref]);
    assert_eq!(subject.additional_objects.len(), 1);

    let mut countered = game.clone();
    let counter = put(&mut countered, player, "Negate", Zone::Hand);
    cast(&mut countered, counter, Some(Target::Stack(reflexive.id)));
    settle(&mut countered);
    assert!(countered.observe(player).effect_choice.is_none());
    assert_eq!(countered.object_view(first).attached_to, None);

    let mut blinking = game.clone();
    let blink = put(&mut blinking, player, "Opt", Zone::Hand);
    cast(&mut blinking, blink, Some(Target::Card(first)));
    passes(&mut blinking);
    assert_ne!(blinking.object_ref(first), first_ref);
    passes(&mut blinking);
    act(
        &mut blinking,
        Action::ChooseEquipmentRecipient { recipient: source },
    );
    assert!(blinking.observe(player).equipment_attachment.is_none());
    assert_eq!(blinking.object_view(first).attached_to, None);
    assert_eq!(blinking.object_view(second).attached_to, Some(source));

    passes(&mut game);
    let view = game.observe(player.other()).effect_choice.unwrap();
    assert!(view.optional);
    assert_eq!(view.candidates, vec![source]);
    for id in [foreign, protected, nonsamurai] {
        let recipient = game.object_ref(id);
        reject(&mut game, Action::ChooseEquipmentRecipient { recipient });
    }
    let mut declined = game.clone();
    act(&mut declined, Action::ChooseOptional { accept: false });
    assert_eq!(declined.object_view(first).attached_to, None);
    act(
        &mut game,
        Action::ChooseEquipmentRecipient { recipient: source },
    );
    let view = game.observe(player.other()).equipment_attachment.unwrap();
    assert_eq!(view.recipient.reference, source);
    let mut offered = view.candidates;
    offered.sort_unstable();
    assert_eq!(offered, entrants);
    assert!(!view.required);
    let unrelated = game.object_ref(unrelated);
    reject(
        &mut game,
        Action::ChooseEquipmentAttachment {
            equipment: Some(unrelated),
        },
    );
    act(
        &mut game,
        Action::ChooseEquipmentAttachment {
            equipment: Some(second_ref),
        },
    );
    assert_eq!(game.object_view(first).attached_to, None);
    assert_eq!(game.object_view(second).attached_to, Some(source));
}

#[test]
fn an_empty_or_wholly_prevented_selection_queues_no_attachment_ability() {
    let _rules = fixture();
    let _reset = ResetPrevention;
    let player = Player::P0;
    for select in [false, true] {
        let mut game = scenario(player);
        let card = put(&mut game, player, "Short Bow", Zone::Library);
        look(&mut game, player);
        if select {
            act(&mut game, Action::SelectLibrary { card });
            zone_replacements::TEST_PREVENT_ZONE_MOVE
                .with(|slot| slot.set(Some(game.object_ref(card))));
            game.state.zone_rules.get_or_insert_default();
        }
        act(&mut game, Action::FinishLibrarySelection);
        assert_eq!(game.state.pending, Pending::Priority);
        assert!(game.state.stack.is_empty());
        assert!(game.state.pending_triggers.is_empty());
        assert_eq!(game.object_view(card).zone, Zone::Library);
    }
}

#[test]
fn job_select_and_the_reflexive_attachment_resolve_in_either_stack_order() {
    let _rules = fixture();
    let player = Player::P0;
    for reflexive_first in [false, true] {
        let mut game = scenario(player);
        let katana = put(&mut game, player, "Samurai's Katana", Zone::Library);
        let source = look(&mut game, player);
        act(&mut game, Action::SelectLibrary { card: katana });
        act(&mut game, Action::FinishLibrarySelection);
        assert!(matches!(game.state.pending, Pending::OrderTriggers { .. }));
        assert_eq!(game.state.ordering_triggers.len(), 2);
        let first_placed = game
            .state
            .ordering_triggers
            .iter()
            .find(|item| (item.kind != REFLEXIVE) == reflexive_first)
            .unwrap()
            .id;
        act(&mut game, Action::PutTrigger { id: first_placed });
        assert_eq!(
            game.state.stack.last().unwrap().kind == REFLEXIVE,
            reflexive_first
        );
        if !reflexive_first {
            passes(&mut game);
            assert!(game.object_view(katana).attached_to.is_some());
        }
        passes(&mut game);
        act(
            &mut game,
            Action::ChooseEquipmentRecipient { recipient: source },
        );
        assert_eq!(game.object_view(katana).attached_to, Some(source));
        if reflexive_first {
            passes(&mut game);
            let hero = game.object_view(katana).attached_to.unwrap();
            assert_eq!(game.object_view(hero.card).name, "Hero Token");
        }
    }
}
