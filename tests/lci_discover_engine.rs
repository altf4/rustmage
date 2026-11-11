//! Discover uses a complete, owned resolution and a committed player event.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const TREASURE: Card = Card(5302);

const fn spell(targets: TargetPlan, program: &'static [Instruction]) -> CardRules {
    CardRules {
        effect: Effect::Spell(targets, program),
        ..CardRules::EMPTY
    }
}

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn advance_until(game: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..48 {
        if ready(game) {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::PayCosts { .. }
                    | Action::FinishTargets
            )
        });
    }
    panic!("discover scenario did not reach its next decision");
}

fn discover_pending(game: &mut Game, player: Player) {
    advance_until(game, |game| {
        game.state.pending == Pending::ResolutionCasts { player }
    });
}

fn quiet(game: &mut Game) {
    advance_until(game, |game| {
        game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.resolving.is_none()
            && game.state.pending_triggers.is_empty()
    });
}

fn offer(game: &Game, id: CardId, face: CastFace) -> Action {
    Action::CastExiledDuringResolution {
        object: game.object_ref(id),
        face,
    }
}

const DISCOVER_FOUR: CardRules = spell(TargetPlan::NONE, &[Instruction::Discover(4)]);

#[test]
fn discover_filters_the_actual_cast_face_and_keeps_public_information_in_owned_branches() {
    let _fixture = fixture_rules::activate(&[(card("Opt"), &DISCOVER_FOUR)]);
    for player in [Player::P0, Player::P1] {
        let mut parent = scenario(player);
        let unseen = put(&mut parent, player, "Shock", Zone::Library);
        let found = put(&mut parent, player, "Galvanic Giant", Zone::Library);
        let expensive = put(&mut parent, player, "Gigantosaurus", Zone::Library);
        let land = put(&mut parent, player, "Forest", Zone::Library);
        let discover = put(&mut parent, player, "Opt", Zone::Hand);
        cast(&mut parent, discover, None);
        discover_pending(&mut parent, player);

        let expected_exiles = [land, expensive, found].map(|id| parent.object_ref(id));
        let opponent = parent.observe(player.other());
        assert_eq!(opponent.cascade.as_ref().unwrap().exiled, expected_exiles);
        assert_eq!(
            opponent.cascade.unwrap().discovered,
            Some(parent.object_ref(found))
        );
        assert!(
            opponent
                .objects
                .iter()
                .any(|object| object.id == found && object.zone == Zone::Exile)
        );
        assert!(opponent.objects.iter().all(|object| object.id != unseen));

        let normal = offer(&parent, found, CastFace::Normal);
        assert!(parent.legal_actions().contains(&normal));
        let oversized_face = offer(&parent, found, CastFace::Adventure);
        reject(&mut parent, oversized_face);

        let saved = parent.snapshot();
        let revision = parent.revision();
        let finish = Action::FinishResolutionCasts;
        let index = parent
            .legal_actions()
            .iter()
            .position(|action| *action == finish)
            .unwrap();
        let mut raw = parent.clone();
        let mut indexed = parent.clone();
        let mut cast_child = parent.clone();
        drop(parent);

        act(&mut raw, finish.clone());
        indexed.apply_index(player, revision, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.object_view(found).zone, Zone::Hand);
        assert!(
            raw.observe(player.other())
                .known_hand
                .contains(&raw.object_ref(found))
        );
        assert_eq!(raw.top_library_card(player), Some(unseen));
        let library = &raw.state.players[player.index()].library;
        assert!(library[..2].contains(&land));
        assert!(library[..2].contains(&expensive));

        act(&mut cast_child, normal);
        discover_pending(&mut cast_child, player);
        assert_eq!(cast_child.object_view(found).zone, Zone::Stack);
        act(&mut cast_child, finish.clone());
        quiet(&mut cast_child);
        assert_eq!(cast_child.object_view(found).zone, Zone::Battlefield);

        cast_child.restore(&saved).unwrap();
        assert_eq!(cast_child.snapshot(), saved);
        act(&mut cast_child, finish);
        assert_eq!(cast_child.snapshot(), raw.snapshot());
    }
}

const DISCOVER_WITH_TREASURE: CardRules = spell(
    TargetPlan::NONE,
    &[
        Instruction::Discover(10),
        Instruction::CreateTokensScaled {
            token: TREASURE,
            amount: AmountValue::DiscoverManaValueShortfall,
            tapped: true,
        },
    ],
);

#[test]
fn discover_followup_uses_the_physical_hit_value_and_cannot_fund_its_cast() {
    let _fixture = fixture_rules::activate(&[(card("Opt"), &DISCOVER_WITH_TREASURE)]);
    let player = Player::P0;
    for (candidate, expected) in [(None, 0), (Some("Dusk Rose Reliquary"), 9)] {
        let mut game = scenario(player);
        let found = candidate.map(|name| put(&mut game, player, name, Zone::Library));
        let discover = put(&mut game, player, "Opt", Zone::Hand);
        cast(&mut game, discover, None);
        if let Some(found) = found {
            discover_pending(&mut game, player);
            let actions = game.legal_actions();
            assert!(actions.contains(&Action::FinishResolutionCasts));
            assert!(
                actions
                    .iter()
                    .all(|action| !matches!(action, Action::CastExiledDuringResolution { .. }))
            );
            let unaffordable = offer(&game, found, CastFace::Normal);
            reject(&mut game, unaffordable);
            act(&mut game, Action::FinishResolutionCasts);
            assert_eq!(game.object_view(found).zone, Zone::Hand);
        }
        quiet(&mut game);
        let treasures: Vec<_> = game
            .state
            .objects
            .iter()
            .filter(|object| object.zone == Zone::Battlefield && object.card == TREASURE)
            .collect();
        assert_eq!(treasures.len(), expected);
        assert!(
            treasures
                .iter()
                .all(|token| token.tapped && token.controller == player)
        );
    }

    // An Adventure spell's mana value differs from its physical card in exile.
    let mut game = scenario(player);
    let found = put(&mut game, player, "Galvanic Giant", Zone::Library);
    let discover = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, discover, None);
    discover_pending(&mut game, player);
    let adventure = offer(&game, found, CastFace::Adventure);
    act(&mut game, adventure);
    discover_pending(&mut game, player);
    assert_eq!(game.state.stack.last().unwrap().mana_value(), 7);
    act(&mut game, Action::FinishResolutionCasts);
    assert_eq!(
        game.state
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Battlefield && o.card == TREASURE && o.tapped)
            .count(),
        6
    );
}

const COUNTER_DISCOVER: CardRules = spell(
    TargetPlan::Single(TargetSpec::one(TargetRule::SpellKinds {
        kinds: 1 | 16,
        excluded: 0,
    })),
    &[Instruction::CounterTargetThenDiscover],
);
const UNCOUNTERABLE: CardRules = CardRules {
    effect: Effect::Creature(0, 1),
    extras: &CardExtras {
        uncounterable: true,
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

#[test]
fn counter_then_discover_uses_the_chosen_x_even_if_the_spell_cannot_be_countered() {
    let player = Player::P0;
    for uncounterable in [false, true] {
        let mut rules = vec![(card("Negate"), &COUNTER_DISCOVER)];
        if uncounterable {
            rules.push((card("Ingenious Prodigy"), &UNCOUNTERABLE));
        }
        let _fixture = fixture_rules::activate(&rules);
        let mut game = scenario(player);
        if !uncounterable {
            put(&mut game, player, "Rest in Peace", Zone::Battlefield);
        }
        let counter = put(&mut game, player.other(), "Negate", Zone::Hand);
        let found = put(&mut game, player.other(), "Galvanic Giant", Zone::Library);
        let target = put(&mut game, player, "Ingenious Prodigy", Zone::Hand);
        act(&mut game, Action::AnnounceCast { card: target });
        act(&mut game, Action::ChooseX { value: 3 });
        pay(&mut game);
        assert_eq!(game.state.stack.last().unwrap().mana_value(), 4);
        advance_until(&mut game, |game| game.actor() == Some(player.other()));
        cast(&mut game, counter, Some(Target::Spell(target)));
        discover_pending(&mut game, player.other());

        assert_eq!(
            game.object_view(target).zone,
            if uncounterable {
                Zone::Stack
            } else {
                Zone::Exile
            }
        );
        assert!(
            game.legal_actions()
                .contains(&offer(&game, found, CastFace::Normal))
        );
        act(&mut game, Action::FinishResolutionCasts);
        quiet(&mut game);
        assert_eq!(game.object_view(found).zone, Zone::Hand);
    }
}

const SHUFFLE_DISCOVER: CardRules = spell(
    TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
    &[Instruction::ShuffleTargetThenDiscover],
);

#[test]
fn shuffle_then_discover_uses_the_owner_and_can_find_the_shuffled_card() {
    let _fixture = fixture_rules::activate(&[(card("Shock"), &SHUFFLE_DISCOVER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let target = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    game.state.objects[target as usize].controller = player;
    let initial = game.object_ref(target);
    let spell = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(target)));
    discover_pending(&mut game, player.other());

    let object = game.object_view(target);
    assert_eq!(object.zone, Zone::Exile);
    assert_eq!(object.reference.generation, initial.generation + 2);
    assert!(
        game.legal_actions()
            .contains(&offer(&game, target, CastFace::Normal))
    );
    act(&mut game, Action::FinishResolutionCasts);
    let object = game.object_view(target);
    assert_eq!((object.zone, object.owner), (Zone::Hand, player.other()));
}

const TWO_DISCOVERIES: CardRules = spell(
    TargetPlan::NONE,
    &[Instruction::Discover(2), Instruction::Discover(3)],
);
const DISCOVER_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OncePerTurn(&TriggerCondition::DiscoveredYou),
        targets: TargetPlan::NONE,
        program: &[Instruction::DiscoverScaled(
            AmountValue::DiscoverEventAmount,
        )],
    }],
    ..CardRules::EMPTY
};

#[test]
fn each_observer_queues_once_for_a_completed_no_hit_and_retains_that_discover_value() {
    let _fixture = fixture_rules::activate(&[
        (card("Opt"), &TWO_DISCOVERIES),
        (card("Hill Giant"), &DISCOVER_OBSERVER),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let first = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let second = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let lands = game.state.players[player.index()].library.clone();
    let found = put(&mut game, player, "Giant Cindermaw", Zone::Library);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, spell, None);
    discover_pending(&mut game, player);
    let second_scan = game.observe(player).cascade.unwrap();
    assert_eq!(second_scan.discovered, Some(game.object_ref(found)));
    act(&mut game, Action::FinishResolutionCasts);
    advance_until(&mut game, |game| {
        game.state.pending == Pending::Priority && game.state.stack.len() == 2
    });
    for observer in [first, second] {
        let trigger = game
            .state
            .stack
            .iter()
            .find(|packet| packet.source.unwrap().id == observer)
            .unwrap();
        assert!(matches!(
            trigger.event_object.as_ref().map(|event| &event.facts),
            Some(EventObjectFacts::DiscoverAmount(2))
        ));
    }
    quiet(&mut game);
    assert_eq!(game.object_view(found).zone, Zone::Hand);
    // The first attempt and both observers scan every land. The successful
    // second attempt only scans the cards it finds before the creature.
    for land in lands {
        let extra = u32::from(second_scan.exiled.iter().any(|object| object.card == land)) * 2;
        assert_eq!(game.object_ref(land).generation, 6 + u64::from(extra));
    }
}

#[test]
fn cascade_does_not_emit_discover_and_a_discovered_x_is_zero() {
    const CASCADE_ONLY: CardRules = spell(TargetPlan::NONE, &[Instruction::Cascade]);
    let _fixture = fixture_rules::activate(&[
        (card("Opt"), &DISCOVER_FOUR),
        (card("Shock"), &CASCADE_ONLY),
        (card("Hill Giant"), &DISCOVER_OBSERVER),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let cascade = put(&mut game, player, "Shock", Zone::Hand);
    let discover = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, cascade, None);
    quiet(&mut game);
    assert!(game.state.triggered_this_turn.is_empty());

    let found = put(&mut game, player, "Ingenious Prodigy", Zone::Library);
    cast(&mut game, discover, None);
    discover_pending(&mut game, player);
    let cast_free = offer(&game, found, CastFace::Normal);
    act(&mut game, cast_free);
    discover_pending(&mut game, player);
    let spell = game.state.stack.last().unwrap();
    assert_eq!(spell.source.unwrap().id, found);
    assert_eq!(spell.x, Some(0));
    act(&mut game, Action::FinishResolutionCasts);
    quiet(&mut game);
}
