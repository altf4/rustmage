//! Owned manifest choices and the exact permanents produced by a resolution.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const ENCHANT_CREATURE: AuraSpec = AuraSpec {
    kinds: 1,
    relation: -1,
    bonus: EquipmentSpec {
        stats: Stats {
            power: 0,
            toughness: 0,
        },
        keywords: Keywords::NONE,
    },
};

const REPEATED: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::ManifestDreadRepeated(AmountValue::ChosenX),
            Instruction::PutCountersOnManifested {
                kind: CounterKind::PlusOne,
                amount: AmountValue::ChosenX,
            },
        ],
    ),
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

const ATTACHMENT: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ManifestDread,
            Instruction::AttachSourceToLastManifested,
        ],
    }],
    ..CardRules::EMPTY
};

const OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::YouManifestDread,
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCounters {
            subject: Subject::Source,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};

fn with_library(player: Player, names: &[&str]) -> (Game, Vec<CardId>) {
    let mut game = scenario(player);
    for card in game.state.players[player.index()].library.clone() {
        game.move_object(card, Zone::Exile);
    }
    let cards = names
        .iter()
        .map(|name| put(&mut game, player, name, Zone::Library))
        .collect();
    (game, cards)
}

fn start_repeated_manifest(game: &mut Game, player: Player, count: u32) {
    let spell = put(game, player, "Syncopate", Zone::Hand);
    act(game, Action::AnnounceCast { card: spell });
    act(game, Action::ChooseX { value: count });
    pay(game);
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn selection(game: &Game, card: CardId) -> Action {
    Action::ChooseManifest {
        object: game.object_ref(card),
    }
}

#[test]
fn repeated_manifests_own_private_choices_and_apply_counters_after_all_entries() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Syncopate").unwrap(), &REPEATED)]);
    for player in [Player::P0, Player::P1] {
        let (mut root, cards) =
            with_library(player, &["Forest", "Llanowar Elves", "Shock", "Bear Cub"]);
        let [bottom, second, third, top] = cards.try_into().unwrap();
        start_repeated_manifest(&mut root, player, 2);
        assert_eq!(root.actor(), Some(player));
        assert_eq!(
            root.observe(player)
                .manifest_choice
                .unwrap()
                .candidates
                .len(),
            2
        );
        assert!(root.observe(player.other()).manifest_choice.is_none());
        let old_choice = selection(&root, top);
        act(&mut root, old_choice.clone());

        let manifested = root.object_ref(top);
        assert_eq!(root.object_view(third).zone, Zone::Graveyard);
        assert_eq!(root.counter_count(top, CounterKind::PlusOne), 0);
        assert_eq!(
            root.observe(player).last_manifest_dread.unwrap().manifested,
            Some(manifested)
        );
        let hidden = root.observe(player.other());
        assert!(hidden.manifest_choice.is_none());
        assert_eq!(hidden.face_down_permanents[0].underlying, None);
        reject(&mut root, old_choice);

        let saved = root.snapshot();
        let raw_action = selection(&root, second);
        let alternate = selection(&root, bottom);
        let index = root
            .legal_actions()
            .iter()
            .position(|action| action == &raw_action)
            .unwrap();
        let revision = root.revision();
        let mut raw = root.clone();
        let mut indexed = root.clone();
        drop(root);

        act(&mut raw, raw_action);
        indexed.apply_index(player, revision, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.object_ref(top), manifested);
        assert_eq!(raw.counter_count(top, CounterKind::PlusOne), 2);
        assert_eq!(raw.counter_count(second, CounterKind::PlusOne), 2);
        assert_eq!(raw.object_view(bottom).zone, Zone::Graveyard);

        indexed.restore(&saved).unwrap();
        act(&mut indexed, alternate);
        assert_eq!(indexed.counter_count(top, CounterKind::PlusOne), 2);
        assert_eq!(indexed.counter_count(bottom, CounterKind::PlusOne), 2);
        assert_eq!(indexed.object_view(second).zone, Zone::Graveyard);
        assert_eq!(raw.object_view(second).zone, Zone::Battlefield);
    }
}

#[test]
fn sole_cards_and_empty_library_complete_each_manifest_independently() {
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Syncopate").unwrap(), &REPEATED),
        (Card::from_name("Bear Cub").unwrap(), &OBSERVER),
    ]);
    let player = Player::P0;
    let (mut game, cards) = with_library(player, &["Llanowar Elves"]);
    let observer = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    start_repeated_manifest(&mut game, player, 0);
    settle(&mut game);
    assert!(game.observe(player).last_manifest_dread.is_none());
    assert_eq!(game.counter_count(observer, CounterKind::PlusOne), 0);
    assert_eq!(game.object_view(cards[0]).zone, Zone::Library);

    start_repeated_manifest(&mut game, player, 3);
    assert!(game.observe(player).manifest_choice.is_none());
    for _ in 0..3 {
        if !matches!(game.state.pending, Pending::OrderTriggers { .. }) {
            break;
        }
        choose(&mut game, |action| {
            matches!(action, Action::PutTrigger { .. })
        });
    }
    assert_eq!(game.state.stack.len(), 3);
    settle(&mut game);
    assert_eq!(game.object_view(cards[0]).zone, Zone::Battlefield);
    assert_eq!(game.counter_count(cards[0], CounterKind::PlusOne), 3);
    assert_eq!(game.counter_count(observer, CounterKind::PlusOne), 3);
    assert_eq!(
        game.observe(player).last_manifest_dread.unwrap().manifested,
        None
    );
}

struct ResetMovePrevention;
impl Drop for ResetMovePrevention {
    fn drop(&mut self) {
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|slot| slot.set(None));
    }
}

#[test]
fn manifest_attachment_uses_only_its_own_successful_entry() {
    let _scope =
        fixture_rules::activate(&[(Card::from_name("Glimmerlight").unwrap(), &ATTACHMENT)]);
    let player = Player::P1;
    let (mut root, cards) = with_library(player, &["Bear Cub"]);
    let first = put(&mut root, player, "Glimmerlight", Zone::Hand);
    cast(&mut root, first, None);
    settle(&mut root);
    assert_eq!(
        root.object_view(first).attached_to,
        Some(root.object_ref(cards[0]))
    );

    for prevent in [false, true] {
        let mut game = root.clone();
        let next = put(&mut game, player, "Glimmerlight", Zone::Hand);
        let _reset = ResetMovePrevention;
        if prevent {
            let blocked = put(&mut game, player, "Llanowar Elves", Zone::Library);
            zone_replacements::TEST_PREVENT_ZONE_MOVE
                .with(|slot| slot.set(Some(game.object_ref(blocked))));
        }
        cast(&mut game, next, None);
        settle(&mut game);
        assert_eq!(game.object_view(next).attached_to, None);
        assert_eq!(
            game.object_view(first).attached_to,
            Some(game.object_ref(cards[0]))
        );
        assert_eq!(
            game.observe(player).last_manifest_dread.unwrap().manifested,
            None
        );
    }
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn linked_exile_owners_choose_once_each_before_either_manifest_commits() {
    const DEPARTURE: CardRules = CardRules {
        effect: Effect::Permanent,
        triggers: &[TriggeredAbility {
            condition: TriggerCondition::SelfLeaves,
            targets: TargetPlan::NONE,
            program: &[Instruction::ManifestDreadForLinkedExileOwners],
        }],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Short Bow").unwrap(), &DEPARTURE),
        (Card::from_name("Bear Cub").unwrap(), &OBSERVER),
    ]);
    let active = Player::P1;
    let other = active.other();
    let (mut game, cards) = with_library(active, &["Shock", "Grizzly Bears"]);
    for id in game.state.players[other.index()].library.clone() {
        game.move_object(id, Zone::Exile);
    }
    let other_manifest = put(&mut game, other, "Llanowar Elves", Zone::Library);
    let other_mill = put(&mut game, other, "Plains", Zone::Library);
    let source = put(&mut game, other, "Short Bow", Zone::Battlefield);
    let source_ref = game.object_ref(source);
    let watchers =
        [other, active].map(|owner| put(&mut game, owner, "Bear Cub", Zone::Battlefield));
    let mut exiled = Vec::new();
    for owner in [other, active, active] {
        let id = put(&mut game, owner, "Grizzly Bears", Zone::Exile);
        game.link_exile_to_ability(source_ref, None, game.object_ref(id));
        exiled.push(id);
    }
    let disenchant = put(&mut game, active, "Disenchant", Zone::Hand);
    let mut stale = game.clone();
    stale.move_object(exiled[0], Zone::Graveyard);
    stale.move_object(exiled[0], Zone::Exile);
    for branch in [&mut game, &mut stale] {
        cast(branch, disenchant, Some(Target::Card(source)));
        pass_pair(branch);
        assert_eq!(branch.object_view(source).zone, Zone::Graveyard);
        pass_pair(branch);
        assert_eq!(branch.actor(), Some(active));
        let mine = branch.observe(active).manifest_choice.unwrap();
        assert_eq!(mine.player, active);
        assert_eq!(
            mine.candidates
                .iter()
                .map(|&(_, card)| card)
                .collect::<Vec<_>>(),
            [Card::GrizzlyBears, Card::from_name("Shock").unwrap()]
        );
    }
    assert!(stale.observe(other).manifest_choice.is_none());
    let first = selection(&game, cards[1]);
    let second = selection(&game, other_manifest);
    reject(&mut game, second.clone());
    act(&mut stale, first.clone());
    settle(&mut stale);
    assert_eq!(stale.object_view(other_manifest).zone, Zone::Library);
    assert_eq!(stale.counter_count(watchers[0], CounterKind::PlusOne), 0);
    assert_eq!(stale.counter_count(watchers[1], CounterKind::PlusOne), 1);

    act(&mut game, first.clone());
    assert_eq!(game.actor(), Some(other));
    assert_eq!(game.object_view(cards[1]).zone, Zone::Library);
    assert_eq!(game.object_view(other_manifest).zone, Zone::Library);
    assert!(game.observe(active).face_down_permanents.is_empty());
    assert_eq!(game.observe(other).manifest_choice.unwrap().player, other);
    assert!(game.observe(active).actions.is_empty());
    reject(&mut game, first);

    let checkpoint = game.snapshot();
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == second)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(other, revision, index).unwrap();
    act(&mut game, second);
    assert_eq!(indexed.snapshot(), game.snapshot());
    indexed.restore(&checkpoint).unwrap();
    assert_eq!(indexed.actor(), Some(other));
    assert!(indexed.observe(other).face_down_permanents.is_empty());
    assert_eq!(game.object_view(cards[0]).zone, Zone::Graveyard);
    assert_eq!(game.object_view(other_mill).zone, Zone::Graveyard);
    for (owner, id) in [(active, cards[1]), (other, other_manifest)] {
        assert_eq!(game.object_view(id).controller, owner);
        let object = game.object_ref(id);
        assert!(game.is_face_down(object));
        let view = game.observe(owner.other()).face_down_permanents;
        assert!(
            view.iter()
                .any(|row| row.object == object && row.underlying.is_none())
        );
    }
    settle(&mut game);
    for watcher in watchers {
        assert_eq!(game.counter_count(watcher, CounterKind::PlusOne), 1);
    }
}

#[test]
fn destruction_sends_the_private_choice_to_the_targets_controller_even_if_it_survives() {
    const REMAKE: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            &[
                Instruction::Destroy,
                Instruction::ManifestDreadForTargetController,
            ],
        ),
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(Card::from_name("Murder").unwrap(), &REMAKE)]);
    for indestructible in [false, true] {
        let player = Player::P0;
        let opponent = player.other();
        let mut game = scenario(player);
        let victim = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let object = &mut game.state.objects[victim as usize];
        object.controller = opponent;
        if indestructible {
            object.gained_keywords = Keywords::INDESTRUCTIBLE;
        }
        let card = put(&mut game, opponent, "Llanowar Elves", Zone::Library);
        let spell = put(&mut game, player, "Murder", Zone::Hand);
        cast(&mut game, spell, Some(Target::Card(victim)));
        pass_pair(&mut game);
        assert_eq!(
            game.object_view(victim).zone,
            if indestructible {
                Zone::Battlefield
            } else {
                Zone::Graveyard
            }
        );
        assert_eq!(game.actor(), Some(opponent));
        assert!(game.observe(player).manifest_choice.is_none());
        assert!(game.observe(opponent).manifest_choice.is_some());
        let action = selection(&game, card);
        let saved = game.snapshot();
        assert!(game.apply(player, action.clone()).is_err());
        assert_eq!(game.snapshot(), saved);
        act(&mut game, action);
        assert_eq!(game.object_view(card).controller, opponent);
        assert_eq!(game.object_view(card).zone, Zone::Battlefield);
        assert_eq!(
            game.observe(player).face_down_permanents[0].underlying,
            None
        );
    }
}

#[test]
fn the_spell_controller_manifests_after_a_counter_attempt() {
    const COUNTER: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::Single(TargetSpec::one(TargetRule::Spell)),
            &[Instruction::CounterAndManifestForController],
        ),
        ..CardRules::EMPTY
    };
    const EMPTY_SPELL: CardRules = CardRules {
        effect: Effect::Spell(TargetPlan::NONE, &[]),
        ..CardRules::EMPTY
    };
    const UNCOUNTERABLE: CardRules = CardRules {
        extras: &CardExtras {
            uncounterable: true,
            ..CardExtras::EMPTY
        },
        ..EMPTY_SPELL
    };
    for uncounterable in [false, true] {
        let _scope = fixture_rules::activate(&[
            (Card::from_name("Negate").unwrap(), &COUNTER),
            (
                Card::from_name("Shock").unwrap(),
                if uncounterable {
                    &UNCOUNTERABLE
                } else {
                    &EMPTY_SPELL
                },
            ),
        ]);
        let player = Player::P1;
        let mut game = scenario(player);
        let card = put(&mut game, player, "Bear Cub", Zone::Library);
        let spell = put(&mut game, player, "Shock", Zone::Hand);
        cast(&mut game, spell, None);
        act(&mut game, Action::Pass);
        let counter = put(&mut game, player.other(), "Negate", Zone::Hand);
        cast(&mut game, counter, Some(Target::Spell(spell)));
        pass_pair(&mut game);
        assert_eq!(
            game.object_view(spell).zone,
            if uncounterable {
                Zone::Stack
            } else {
                Zone::Graveyard
            }
        );
        assert_eq!(game.actor(), Some(player));
        assert!(game.observe(player.other()).manifest_choice.is_none());
        let action = selection(&game, card);
        act(&mut game, action);
        assert_eq!(game.object_view(card).zone, Zone::Battlefield);
        settle(&mut game);
    }
}

#[test]
fn manifest_observer_returns_only_a_card_from_its_own_graveyard_receipt() {
    const RECOVER: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        triggers: &[TriggeredAbility {
            condition: TriggerCondition::YouManifestDread,
            targets: TargetPlan::NONE,
            program: &[Instruction::ChooseObjects {
                kind: EffectChoiceKind::ReturnManifestGraveyardToHand,
                players: PlayerSet::Controller,
                filter: SacrificeFilter {
                    kinds: 0,
                    keywords: Keywords::NONE,
                    source_only: false,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
                amount: 1,
                optional_skip: None,
            }],
        }],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Syncopate").unwrap(), &REPEATED),
        (Card::from_name("Bear Cub").unwrap(), &RECOVER),
    ]);
    let player = Player::P0;
    let (mut game, cards) = with_library(player, &["Forest", "Llanowar Elves"]);
    let [forest, creature] = cards.try_into().unwrap();
    put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let unrelated = put(&mut game, player, "Forest", Zone::Graveyard);
    start_repeated_manifest(&mut game, player, 1);
    let action = selection(&game, creature);
    act(&mut game, action);
    let receipt = game.object_ref(forest);
    let mut stale = game.clone();
    stale.move_object(forest, Zone::Exile);
    stale.move_object(forest, Zone::Graveyard);
    settle(&mut stale);
    assert_eq!(stale.object_view(forest).zone, Zone::Graveyard);

    pass_pair(&mut game);
    let other = Action::SelectGraveyardReturn {
        object: game.object_ref(unrelated),
    };
    reject(&mut game, other);
    act(&mut game, Action::SelectGraveyardReturn { object: receipt });
    assert_eq!(game.object_view(forest).zone, Zone::Hand);
    assert_eq!(game.object_view(unrelated).zone, Zone::Graveyard);
}

#[test]
fn restricted_mana_funds_the_face_up_special_action() {
    const TWO_GREEN: Mana = [0, 0, 0, 0, 2, 0];
    const PRODUCER: CardRules = CardRules {
        effect: Effect::Creature(1, 1),
        activated: &[ActivatedAbility {
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            exhaust: false,
            cycling: false,
            sorcery: false,
            once_per_turn: false,
            cost: ActivationCost {
                tap: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Tagged {
                production: &ManaProduction::Choice {
                    mask: 31,
                    amount: 2,
                },
                quality: &ManaQuality {
                    restriction: ManaRestriction::TurnFaceUp,
                    spell_bonus: ManaSpellBonus::None,
                    retention: ManaRetention::Step,
                },
            })],
        }],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Syncopate").unwrap(), &REPEATED),
        (Card::from_name("Llanowar Elves").unwrap(), &PRODUCER),
    ]);
    for player in [Player::P0, Player::P1] {
        let (mut root, cards) = with_library(player, &["Grizzly Bears"]);
        let producer = put(&mut root, player, "Llanowar Elves", Zone::Battlefield);
        let in_hand = put(&mut root, player, "Bear Cub", Zone::Hand);
        start_repeated_manifest(&mut root, player, 1);
        settle(&mut root);
        let object = root.object_ref(cards[0]);
        root.state.players[player.index()].mana = [0; 6];
        let begin = Action::BeginFaceUp {
            object,
            cost: ManaCost {
                colored: [0, 0, 0, 0, 1, 0],
                generic: 1,
            },
        };
        let tap = |game: &mut Game| {
            choose(game, |action| {
                matches!(action,
                    Action::ActivateMana { card, production, .. }
                        if *card == producer && *production == TWO_GREEN
                )
            });
        };
        let mut prefunded = root.clone();
        tap(&mut prefunded);
        reject(&mut prefunded, Action::AnnounceCast { card: in_hand });
        act(&mut prefunded, begin.clone());
        act(&mut prefunded, Action::PayFaceUp { payment: TWO_GREEN });

        act(&mut root, begin);
        tap(&mut root);
        reject(
            &mut root,
            Action::PayFaceUp {
                payment: [0, 0, 0, 0, 1, 0],
            },
        );
        let revision = root.revision();
        let paid = Action::PayFaceUp { payment: TWO_GREEN };
        let index = root
            .legal_actions()
            .iter()
            .position(|action| *action == paid)
            .unwrap();
        root.apply_index(player, revision, index).unwrap();
        for game in [&root, &prefunded] {
            assert_eq!(game.object_ref(cards[0]), object);
            assert_eq!(game.object_view(cards[0]).card, Card::GrizzlyBears);
            assert!(game.observe(player).face_down_permanents.is_empty());
            assert_eq!(game.counter_count(cards[0], CounterKind::PlusOne), 1);
            assert_eq!(game.state.players[player.index()].mana, [0; 6]);
        }
    }
}

#[test]
fn an_attached_face_up_prohibition_follows_the_exact_aura() {
    const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::AttachedFaceUpProhibited],
    }];
    const AURA: CardRules = CardRules {
        effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(TargetRule::Creature))),
        aura: Some(ENCHANT_CREATURE),
        static_groups: GROUPS,
        extras: &CardExtras {
            aura: Some(ENCHANT_CREATURE),
            static_ability_groups: GROUPS,
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Syncopate").unwrap(), &REPEATED),
        (Card::from_name("Pacifism").unwrap(), &AURA),
    ]);
    let player = Player::P1;
    let (mut game, cards) = with_library(player, &["Grizzly Bears"]);
    start_repeated_manifest(&mut game, player, 1);
    settle(&mut game);
    let bear = game.object_ref(cards[0]);
    let aura = put(&mut game, player, "Pacifism", Zone::Hand);
    cast(&mut game, aura, Some(Target::Card(bear.card)));
    settle(&mut game);
    assert_eq!(game.object_view(aura).attached_to, Some(bear));
    let begin = Action::BeginFaceUp {
        object: bear,
        cost: ManaCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 1,
        },
    };
    reject(&mut game, begin.clone());
    assert!(game.is_face_down(bear));

    let disenchant = put(&mut game, player, "Disenchant", Zone::Hand);
    cast(&mut game, disenchant, Some(Target::Card(aura)));
    settle(&mut game);
    assert_eq!(game.object_view(aura).zone, Zone::Graveyard);
    act(&mut game, begin);
    choose(&mut game, |action| {
        matches!(action, Action::PayFaceUp { .. })
    });
    assert_eq!(game.object_ref(bear.card), bear);
    assert_eq!(game.object_view(bear.card).card, Card::GrizzlyBears);
    assert_eq!(game.counter_count(bear.card, CounterKind::PlusOne), 1);
}

#[test]
fn shuffling_uses_the_last_exact_attachment_then_manifests_even_if_it_left() {
    const AURA: CardRules = CardRules {
        effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(TargetRule::Creature))),
        aura: Some(ENCHANT_CREATURE),
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                colored: [0, 2, 0, 0, 0, 0],
                generic: 4,
                ..ActivationCost::ZERO
            },
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::ShuffleAttachedIntoOwnerLibrary,
                Instruction::ManifestDread,
            ],
            cycling: false,
            exhaust: false,
            sorcery: true,
            once_per_turn: false,
        }],
        extras: &CardExtras {
            aura: Some(ENCHANT_CREATURE),
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(Card::from_name("Pacifism").unwrap(), &AURA)]);
    let player = Player::P0;
    let owner = player.other();
    let (mut game, cards) = with_library(player, &["Forest", "Llanowar Elves"]);
    let victim = put(&mut game, owner, "Grizzly Bears", Zone::Battlefield);
    game.state.objects[victim as usize].controller = player;
    let attached = game.object_ref(victim);
    let aura = put(&mut game, player, "Pacifism", Zone::Hand);
    cast(&mut game, aura, Some(Target::Card(victim)));
    settle(&mut game);
    assert_eq!(game.object_view(aura).attached_to, Some(attached));
    act(
        &mut game,
        Action::AnnounceActivate {
            card: aura,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(&mut game);
    let destroy = put(&mut game, player, "Disenchant", Zone::Hand);
    cast(&mut game, destroy, Some(Target::Card(aura)));
    pass_pair(&mut game);
    assert_eq!(game.object_view(aura).zone, Zone::Graveyard);
    let old_library_size = game.state.players[owner.index()].library.len();
    let mut new_incarnation = game.clone();
    new_incarnation.move_object(victim, Zone::Hand);
    new_incarnation.move_object(victim, Zone::Battlefield);
    assert_ne!(new_incarnation.object_ref(victim), attached);

    for (branch, zone, arrivals) in [
        (&mut game, Zone::Library, 1),
        (&mut new_incarnation, Zone::Battlefield, 0),
    ] {
        pass_pair(branch);
        assert_eq!(branch.object_view(victim).zone, zone);
        assert_eq!(
            branch.state.players[owner.index()].library.len(),
            old_library_size + arrivals
        );
        assert!(branch.observe(owner).manifest_choice.is_none());
        let selected = selection(branch, cards[1]);
        act(branch, selected);
        assert!(branch.is_face_down(branch.object_ref(cards[1])));
    }
}

#[test]
fn a_look_permission_learns_face_down_creatures_through_a_live_provider() {
    use rustmage_core::cards::authored::object;

    const LOOK: &[StaticAbilityGroup] = &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::LookAtOpponentFaceDownCreatures],
    }];
    const PROVIDER: CardRules = CardRules {
        effect: Effect::Permanent,
        static_groups: LOOK,
        extras: &CardExtras {
            static_ability_groups: LOOK,
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    const REMOVE: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::Single(TargetSpec::one(object(0))),
            &[Instruction::RemoveTargetAbilitiesUntilEndOfTurn],
        ),
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Short Bow").unwrap(), &PROVIDER),
        (Card::from_name("Opt").unwrap(), &REMOVE),
    ]);
    let owner = Player::P1;
    let viewer = owner.other();
    let mut game = scenario(owner);
    let provider = put(&mut game, viewer, "Short Bow", Zone::Battlefield);
    let weapon = Card::from_name("Concealed Weapon").unwrap();
    let disguised = spawn(&mut game, owner, weapon, Zone::Hand);
    let mut suppressed = game.clone();
    let remove = put(&mut suppressed, owner, "Opt", Zone::Hand);
    cast(&mut suppressed, remove, Some(Target::Card(provider)));
    settle(&mut suppressed);

    for branch in [&mut game, &mut suppressed] {
        act(branch, Action::AnnounceDisguise { card: disguised });
        pay(branch);
        assert_eq!(branch.observe(viewer).face_down_spells[0].underlying, None);
        assert_eq!(
            branch.observe(owner).face_down_spells[0].underlying,
            Some(weapon)
        );
        act(branch, Action::Pass);
    }
    let saved = game.snapshot();
    let mut indexed = game.clone();
    let index = indexed
        .legal_actions()
        .iter()
        .position(|action| *action == Action::Pass)
        .unwrap();
    indexed
        .apply_index(viewer, indexed.revision(), index)
        .unwrap();
    act(&mut game, Action::Pass);
    act(&mut suppressed, Action::Pass);
    assert_eq!(indexed.snapshot(), game.snapshot());
    indexed.restore(&saved).unwrap();
    assert_eq!(indexed.observe(viewer).face_down_spells[0].underlying, None);
    assert_eq!(
        suppressed.observe(viewer).face_down_permanents[0].underlying,
        None
    );
    assert_eq!(
        game.observe(viewer).face_down_permanents[0].underlying,
        Some(weapon)
    );

    let disenchant = put(&mut game, owner, "Disenchant", Zone::Hand);
    cast(&mut game, disenchant, Some(Target::Card(provider)));
    settle(&mut game);
    assert_eq!(game.object_view(provider).zone, Zone::Graveyard);
    let smuggler = Card::from_name("Bubble Smuggler").unwrap();
    let another = spawn(&mut game, owner, smuggler, Zone::Hand);
    act(&mut game, Action::AnnounceDisguise { card: another });
    pay(&mut game);
    settle(&mut game);
    let view = game.observe(viewer).face_down_permanents;
    assert_eq!(view[0].underlying, Some(weapon));
    assert_eq!(view[1].underlying, None);
}

#[test]
fn an_entry_only_observer_ignores_face_up_entries_and_turning_a_permanent_up() {
    const ENTRY_ONLY: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        triggers: &[TriggeredAbility {
            condition: TriggerCondition::FilteredEntersYou(EntryPredicate::FaceDown),
            targets: TargetPlan::NONE,
            program: &[Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            }],
        }],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Syncopate").unwrap(), &REPEATED),
        (Card::from_name("Bear Cub").unwrap(), &ENTRY_ONLY),
    ]);
    let player = Player::P0;
    let (mut game, cards) = with_library(player, &["Grizzly Bears"]);
    let yours = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let opposing = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);

    let ordinary = put(&mut game, player, "Hill Giant", Zone::Hand);
    cast(&mut game, ordinary, None);
    settle(&mut game);
    assert_eq!(game.counter_count(yours, CounterKind::PlusOne), 0);

    start_repeated_manifest(&mut game, player, 1);
    settle(&mut game);
    let face_down = game.object_ref(cards[0]);
    assert!(game.is_face_down(face_down));
    assert_eq!(game.counter_count(yours, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(opposing, CounterKind::PlusOne), 0);

    choose(
        &mut game,
        |action| matches!(action, Action::BeginFaceUp { object, .. } if *object == face_down),
    );
    choose(&mut game, |action| {
        matches!(action, Action::PayFaceUp { .. })
    });
    settle(&mut game);
    assert_eq!(game.object_ref(cards[0]), face_down);
    assert!(!game.is_face_down(face_down));
    assert_eq!(game.counter_count(yours, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(opposing, CounterKind::PlusOne), 0);
}

#[test]
fn the_effect_player_can_turn_an_opponents_permanent_face_up() {
    use rustmage_core::cards::authored::object;

    const REVEAL: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
                base: &object(0),
                predicate: &TargetPredicate::FaceDown,
            })),
            &[
                Instruction::RevealFaceDownTargetAndMayTurnUp,
                Instruction::TurnTargetFaceUp,
            ],
        ),
        ..CardRules::EMPTY
    };
    const FACE_OBSERVERS: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        triggers: &[
            TriggeredAbility {
                condition: TriggerCondition::YouTurnPermanentFaceUp,
                targets: TargetPlan::NONE,
                program: &[Instruction::PutCountersOnEventPermanent {
                    kind: CounterKind::PlusOne,
                    amount: 1,
                }],
            },
            TriggeredAbility {
                condition: TriggerCondition::FaceDownPermanentEntersOrTurnsUpYou,
                targets: TargetPlan::NONE,
                program: &[Instruction::PutCounters {
                    subject: Subject::Source,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                }],
            },
        ],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Syncopate").unwrap(), &REPEATED),
        (Card::from_name("Opt").unwrap(), &REVEAL),
        (Card::from_name("Bear Cub").unwrap(), &FACE_OBSERVERS),
    ]);
    let controller = Player::P1;
    let player = controller.other();
    let (mut game, cards) = with_library(controller, &["Grizzly Bears"]);
    let actor_observer = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let controller_observer = put(&mut game, controller, "Bear Cub", Zone::Battlefield);
    start_repeated_manifest(&mut game, controller, 1);
    settle(&mut game);
    let facedown = game.object_ref(cards[0]);
    act(&mut game, Action::Pass);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(facedown.card)));
    assert_eq!(
        game.observe(player).face_down_permanents[0].underlying,
        None
    );
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(player));
    for viewer in [player, controller] {
        assert_eq!(
            game.observe(viewer).face_down_permanents[0].underlying,
            Some(Card::GrizzlyBears)
        );
    }

    let mut decline = game.clone();
    act(&mut decline, Action::ChooseOptional { accept: false });
    assert!(decline.is_face_down(facedown));
    assert!(
        decline.state.players[controller.index()]
            .turn_facts
            .face_down_entry_or_turn_up
    );
    assert!(
        !decline.state.players[player.index()]
            .turn_facts
            .face_down_entry_or_turn_up
    );
    assert_eq!(
        decline.counter_count(controller_observer, CounterKind::PlusOne),
        1
    );
    assert_eq!(
        decline.counter_count(facedown.card, CounterKind::PlusOne),
        1
    );
    act(&mut game, Action::ChooseOptional { accept: true });
    settle(&mut game);
    assert_eq!(game.object_ref(facedown.card), facedown);
    assert_eq!(game.object_view(facedown.card).controller, controller);
    assert_eq!(game.object_view(facedown.card).card, Card::GrizzlyBears);
    assert!(
        game.state.players[player.index()]
            .turn_facts
            .face_down_entry_or_turn_up
    );
    assert_eq!(game.counter_count(facedown.card, CounterKind::PlusOne), 2);
    assert_eq!(game.counter_count(actor_observer, CounterKind::PlusOne), 0);
    assert_eq!(
        game.counter_count(controller_observer, CounterKind::PlusOne),
        2
    );

    let turn = game.state.turn;
    for _ in 0..32 {
        if game.state.turn != turn {
            break;
        }
        choose(&mut game, |action| {
            matches!(action, Action::Pass | Action::FinishAttackers)
        });
    }
    assert_ne!(game.state.turn, turn);
    assert!(
        game.state
            .players
            .iter()
            .all(|state| !state.turn_facts.face_down_entry_or_turn_up)
    );
}
