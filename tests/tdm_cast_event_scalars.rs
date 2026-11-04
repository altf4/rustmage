//! Opt-in frozen cast facts for scalar consumers; no authored card overlays.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

macro_rules! observer {
    ($program:expr) => {
        CardRules {
            effect: Effect::Creature(2, 2),
            triggers: &[TriggeredAbility {
                condition: TriggerCondition::SpellCast {
                    relation: CastRelation::You,
                    filter: CastFilter::Any,
                    ordinal: None,
                },
                targets: TargetPlan::NONE,
                program: $program,
            }],
            ..CardRules::EMPTY
        }
    };
}
const LOSS: CardRules = observer!(&[Instruction::LoseLifeScaled(
    PlayerSet::Controller,
    AmountValue::CastSpellManaValue,
)]);
const FIXED: CardRules = observer!(&[Instruction::LoseLife(PlayerSet::Controller, 1)]);
const MULTIPLE: CardRules = observer!(&[Instruction::LoseLifeScaled(
    PlayerSet::Controller,
    AmountValue::Multiple {
        value: &AmountValue::CastSpellManaValue,
        factor: 2,
    },
)]);
const DIVIDED: CardRules = observer!(&[Instruction::LoseLifeScaled(
    PlayerSet::Controller,
    AmountValue::Divided {
        value: &AmountValue::CastSpellManaValue,
        divisor: std::num::NonZeroU16::new(2).unwrap(),
    },
)]);
const SUM: CardRules = observer!(&[Instruction::LoseLifeScaled(
    PlayerSet::Controller,
    AmountValue::Sum(&[
        AmountValue::Multiple {
            value: &AmountValue::CastSpellManaValue,
            factor: 2,
        },
        AmountValue::Divided {
            value: &AmountValue::CastSpellManaValue,
            divisor: std::num::NonZeroU16::new(2).unwrap(),
        },
        AmountValue::Fixed(1),
    ]),
)]);
const MODAL: CardRules = observer!(&[Instruction::Modal(&[
    SpellMode {
        text: "Lose the triggering spell's mana value",
        targets: TargetPlan::NONE,
        program: &[Instruction::LoseLifeScaled(
            PlayerSet::Controller,
            AmountValue::CastSpellManaValue,
        )],
    },
    SpellMode {
        text: "Gain the triggering spell's mana value",
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLifeScaled(
            PlayerSet::Controller,
            AmountValue::CastSpellManaValue,
        )],
    },
])]);
const X_SPELL: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, &[]),
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}
fn reach(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..40 {
        if ready(g) {
            return;
        }
        assert_eq!(g.state.pending, Pending::Priority);
        act(g, Action::Pass);
    }
    panic!("cast scalar decision budget exhausted");
}
fn packet(g: &Game, source: CardId) -> &SpellView {
    g.state
        .stack
        .iter()
        .chain(&g.state.pending_triggers)
        .chain(&g.state.ordering_triggers)
        .chain(g.state.announcing.iter().map(|a| &a.item))
        .find(|item| {
            matches!(item.kind, StackKind::TriggeredAbility { .. })
                && item.source.is_some_and(|s| s.id == source)
        })
        .expect("actual observer trigger")
}

#[test]
fn direct_cast_mana_value_event_is_retained_and_lose_life_uses_it() {
    let _scope = fixture_rules::activate(&[(card("Grizzly Bears"), &LOSS)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let observer = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let spell = put(&mut g, p, "Quick Study", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 2];
        cast(&mut g, spell, None);
        let captured = packet(&g, observer)
            .cast_event
            .as_ref()
            .expect("CastSpellManaValue consumer must retain its cast event");
        assert_eq!(captured.mana_value, 3);
        assert_eq!(captured.object, g.object_ref(spell));
        assert_eq!(captured.card, card("Quick Study"));
        assert_eq!(captured.controller, p);
        assert_eq!(captured.payment, [0, 1, 0, 0, 0, 2]);
        assert_eq!(g.state.players[p.index()].life, 20);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 17);
        assert_eq!(g.state.players[p.other().index()].life, 20);
        assert_eq!(g.hand_count(p), 2);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}

#[test]
fn cast_event_allocation_stays_opt_in_and_opponent_casts_do_not_trigger_you() {
    for p in [Player::P0, Player::P1] {
        {
            let _scope = fixture_rules::activate(&[(card("Grizzly Bears"), &FIXED)]);
            let mut g = scenario(p);
            let observer = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let spell = put(&mut g, p, "Quick Study", Zone::Hand);
            cast(&mut g, spell, None);
            assert!(packet(&g, observer).cast_event.is_none());
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, 19);
        }
        let _scope = fixture_rules::activate(&[(card("Grizzly Bears"), &LOSS)]);
        let mut g = scenario(p.other());
        put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let spell = put(&mut g, p.other(), "Quick Study", Zone::Hand);
        cast(&mut g, spell, None);
        assert!(g.state.stack.iter().all(|s| s.kind == StackKind::Spell));
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 20);
        assert_eq!(g.state.players[p.other().index()].life, 20);
    }
}

#[test]
fn composed_multiple_divided_and_sum_quantities_retain_the_same_cast_fact() {
    for (rules, amount) in [(&MULTIPLE, 6), (&DIVIDED, 1), (&SUM, 8)] {
        let _scope = fixture_rules::activate(&[(card("Grizzly Bears"), rules)]);
        for p in [Player::P0, Player::P1] {
            let mut g = scenario(p);
            let observer = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let spell = put(&mut g, p, "Quick Study", Zone::Hand);
            cast(&mut g, spell, None);
            assert_eq!(
                packet(&g, observer).cast_event.as_ref().unwrap().mana_value,
                3
            );
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, 20 - amount);
        }
    }
}

#[test]
fn selected_stack_x_and_omen_cost_determine_captured_mana_value() {
    let _scope = fixture_rules::activate(&[
        (card("Grizzly Bears"), &LOSS),
        (card("Spectral Denial"), &X_SPELL),
    ]);
    for p in [Player::P0, Player::P1] {
        for x in [0, 3] {
            let mut g = scenario(p);
            let observer = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let spell = put(&mut g, p, "Spectral Denial", Zone::Hand);
            g.state.players[p.index()].mana = [0, 1, 0, 0, 0, x];
            act(&mut g, Action::AnnounceCast { card: spell });
            act(&mut g, Action::ChooseX { value: x });
            pay(&mut g);
            assert_eq!(
                packet(&g, observer).cast_event.as_ref().unwrap().mana_value,
                u64::from(x) + 1
            );
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, 19 - i128::from(x));
            assert_eq!(g.object_mana_value(spell), 1);
        }
        let mut g = scenario(p);
        let observer = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let spell = put(&mut g, p, "Scavenger Regent // Exude Toxin", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 2, 0, 0, 0];
        act(&mut g, Action::AnnounceOmen { card: spell });
        act(&mut g, Action::ChooseX { value: 0 });
        pay(&mut g);
        let captured = packet(&g, observer).cast_event.as_ref().unwrap();
        assert_eq!(captured.mana_value, 2);
        assert_eq!(captured.card, card("Exude Toxin"));
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 18);
        assert_eq!(g.object_view(spell).zone, Zone::Library);
        assert_eq!(g.object_mana_value(spell), 4);
    }
}

#[test]
fn countered_spell_and_departed_reentered_source_keep_captured_value_and_controller() {
    let _scope = fixture_rules::activate(&[(card("Grizzly Bears"), &LOSS)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let observer = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let old_source = g.object_ref(observer);
        let spell = put(&mut g, p, "Quick Study", Zone::Hand);
        cast(&mut g, spell, None);
        let original = *packet(&g, observer).cast_event.as_deref().unwrap();
        act(&mut g, Action::Pass);
        let cancel = put(&mut g, p.other(), "Cancel", Zone::Hand);
        cast(&mut g, cancel, Some(Target::Spell(spell)));
        g.move_object(observer, Zone::Hand);
        g.move_object(observer, Zone::Battlefield);
        let new_source = g.object_ref(observer);
        g.change_controller(new_source, p.other());
        assert_ne!(old_source, new_source);
        assert_eq!(
            *packet(&g, observer).cast_event.as_deref().unwrap(),
            original
        );
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        assert_eq!(g.hand_count(p), 0);
        assert_eq!(g.state.players[p.index()].life, 17);
        assert_eq!(g.state.players[p.other().index()].life, 20);
        assert_eq!(g.object_view(observer).controller, p.other());
    }
}

#[test]
fn modal_cast_fact_is_owned_raw_index_restorable_and_reveals_no_private_cards() {
    let _scope = fixture_rules::activate(&[(card("Grizzly Bears"), &MODAL)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let observer = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let secret = put(&mut g, p, "Cancel", Zone::Library);
        let spell = put(&mut g, p, "Quick Study", Zone::Hand);
        cast(&mut g, spell, None);
        reach(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseMode { mode: 0 })
        });
        assert_eq!(
            packet(&g, observer).cast_event.as_ref().unwrap().mana_value,
            3
        );
        assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == secret));
        reject(&mut g, Action::ChooseMode { mode: 99 });
        let before = g.snapshot();
        assert_eq!(
            g.apply(p.other(), Action::ChooseMode { mode: 0 }),
            Err(Error::WrongActor)
        );
        assert_eq!(g.snapshot(), before);
        let action = Action::ChooseMode { mode: 0 };
        let revision = g.revision();
        let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
        let mut raw = g.clone();
        let mut indexed = g.clone();
        let mut sibling = g.clone();
        drop(g);
        raw.apply(p, action.clone()).unwrap();
        indexed.apply_index(p, revision, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        act(&mut sibling, Action::ChooseMode { mode: 1 });
        for child in [&mut raw, &mut indexed, &mut sibling] {
            settle(child);
            assert_eq!(child.hand_count(p), 2);
            assert!(child.observe(p).objects.iter().any(|o| o.id == secret));
            assert!(
                !child
                    .observe(p.other())
                    .objects
                    .iter()
                    .any(|o| o.id == secret)
            );
        }
        assert_eq!(raw.state.players[p.index()].life, 17);
        assert_eq!(sibling.state.players[p.index()].life, 23);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        indexed.restore(&before).unwrap();
        assert_eq!(
            indexed.apply_index(p, revision, index),
            Err(Error::StaleRevision)
        );
        act(&mut indexed, action);
        settle(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
    }
}

#[test]
fn legacy_direct_mana_value_consumers_keep_counter_token_and_damage_results() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Dancing from Dark to Dawn", Zone::Battlefield);
        let recipient = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Bear Cub", Zone::Hand);
        cast(&mut g, spell, None);
        reach(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(recipient),
            })
        });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(recipient),
            },
        );
        settle(&mut g);
        assert_eq!(g.counter_count(recipient, CounterKind::PlusOne), 2);

        let mut g = scenario(p);
        put(&mut g, p, "Ovika, Enigma Goliath", Zone::Battlefield);
        let spell = put(&mut g, p, "Quick Study", Zone::Hand);
        cast(&mut g, spell, None);
        settle(&mut g);
        let tokens: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, o)| {
                (o.zone == Zone::Battlefield && o.controller == p && o.card.is_token())
                    .then_some(id as CardId)
            })
            .collect();
        assert_eq!(tokens.len(), 3);
        for id in tokens {
            assert_eq!(stats(&g, id), (1, 1));
            assert!(g.has_subtype(id, "Phyrexian") && g.has_subtype(id, "Goblin"));
            assert_eq!(g.object_view(id).colors, 8);
            assert!(g.keywords(id).contains(Keywords::HASTE));
        }

        let mut g = scenario(p);
        put(&mut g, p, "Thor, God of Thunder", Zone::Battlefield);
        let spell = put(&mut g, p, "Quick Study", Zone::Hand);
        cast(&mut g, spell, None);
        reach(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Player(p.other()),
            })
        });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 17);
    }
}

#[test]
fn legacy_printed_symbol_consumer_still_uses_symbols_instead_of_mana_value() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p.other());
        let victim = put(&mut g, p.other(), "Bear Cub", Zone::Hand);
        cast(&mut g, victim, None);
        act(&mut g, Action::Pass);
        let source = put(&mut g, p, "Namor the Sub-Mariner", Zone::Battlefield);
        let cancel = put(&mut g, p, "Cancel", Zone::Hand);
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 1];
        cast(&mut g, cancel, Some(Target::Spell(victim)));
        assert_eq!(
            packet(&g, source).cast_event.as_ref().unwrap().mana_value,
            3
        );
        settle(&mut g);
        let tokens: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, o)| {
                (o.zone == Zone::Battlefield && o.controller == p && o.card.is_token())
                    .then_some(id as CardId)
            })
            .collect();
        assert_eq!(tokens.len(), 2);
        for id in tokens {
            assert_eq!(stats(&g, id), (1, 1));
            assert!(g.has_subtype(id, "Merfolk"));
            assert_eq!(g.object_view(id).colors, 2);
        }
        assert_eq!(stats(&g, source), (3, 4));
        assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
    }
}
