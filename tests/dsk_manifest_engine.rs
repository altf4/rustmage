//! Owned manifest choices and the exact permanents produced by a resolution.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

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
