//! Copiable exceptions and exact delayed cleanup for temporary token creation.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardRules, one};
use rustmage_core::cards::*;
use rustmage_core::engine::characteristic_layers::AnimationTypeMode;
use rustmage_core::engine::*;
use support::*;

const PLAYER: Player = Player::P0;
const BLUE: u8 = 1 << 1;
const RED: u8 = 1 << 3;
const SOURCE: &str = "Grizzly Bears";
const OWN_OTHER_CREATURE: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 1,
    relation: 0,
    other: true,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const CREATOR: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::HASTE,
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        targets: one(OWN_OTHER_CREATURE),
        program: &[Instruction::CreateCopyWithExceptionsAndDelayedSacrifice(
            CopyExceptions::BALLOON,
        )],
        cycling: false,
        exhaust: false,
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const COPY: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[Instruction::CreateCopyToken {
            subject: Subject::Targets,
            amount: 1,
        }],
    ),
    ..CardRules::EMPTY
};

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn create_balloon(game: &mut Game, source: CardId, target: CardId) {
    act(
        game,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(
        game,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    pay(game);
    settle(game);
}

fn token_views(game: &Game) -> Vec<ObjectView> {
    game.observe(PLAYER)
        .objects
        .into_iter()
        .filter(|object| object.zone == Zone::Battlefield && object.is_token())
        .collect()
}

#[test]
fn doubled_tokens_and_a_copy_of_a_token_keep_characteristics_with_separate_cleanup() {
    let source_card = card(SOURCE);
    let copy_spell = card("Think Twice");
    let _rules = fixture_rules::activate(&[(source_card, &CREATOR), (copy_spell, &COPY)]);
    let mut game = scenario(PLAYER);
    let source = put(&mut game, PLAYER, SOURCE, Zone::Battlefield);
    let donor = put(&mut game, PLAYER, "Oaken Siren", Zone::Battlefield);
    let doubler = put(&mut game, PLAYER, "Doubling Season", Zone::Battlefield);
    game.state.objects[source as usize].summoning_sick = true;
    game.state.objects[donor as usize].tapped = true;
    game.add_counter(donor, CounterKind::PlusOne, 1);
    create_balloon(&mut game, source, donor);
    let tokens = token_views(&game);
    assert_eq!(tokens.len(), 2);
    let created: Vec<_> = tokens.iter().map(|token| token.reference).collect();
    assert_eq!(game.observe(PLAYER).delayed_sacrifices[0].tokens, created);
    game.move_object(doubler, Zone::Exile);
    let spell = spawn(&mut game, PLAYER, copy_spell, Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(created[0].card)));
    settle(&mut game);
    let copies = token_views(&game);
    assert_eq!(copies.len(), 3);
    let ordinary_copy = copies
        .iter()
        .find(|token| !created.contains(&token.reference))
        .unwrap()
        .reference;
    for token in copies {
        assert_eq!(token.name, "Oaken Siren");
        assert_eq!(token.kinds, 1 | 16);
        assert_eq!(token.colors, BLUE | RED);
        assert!(
            token.has_subtype("Balloon")
                && token.has_subtype("Siren")
                && token.has_subtype("Pirate")
        );
        assert!(
            token.keywords.contains(
                Keywords::FLYING
                    .union(Keywords::HASTE)
                    .union(Keywords::VIGILANCE)
            )
        );
        assert_eq!((token.power, token.toughness), (1, 1));
        assert!(!token.tapped);
        assert_eq!(game.counter_count(token.id, CounterKind::PlusOne), 0);
    }
    game.move_object(source, Zone::Graveyard);
    game.begin_step(Step::End);
    game.prepare_triggers();
    settle(&mut game);
    assert!(created.iter().all(|token| !game.on_battlefield(token.card)));
    assert_eq!(
        game.object_view(ordinary_copy.card).reference,
        ordinary_copy
    );
    assert!(game.on_battlefield(ordinary_copy.card));
}

#[test]
fn a_temporary_land_creature_is_copied_as_a_permanent_balloon_land_creature() {
    let _rules = fixture_rules::activate(&[(card(SOURCE), &CREATOR)]);
    let mut game = scenario(PLAYER);
    let source = put(&mut game, PLAYER, SOURCE, Zone::Battlefield);
    let land = put(&mut game, PLAYER, "Forest", Zone::Battlefield);
    game.change_object_form(
        game.object_ref(land),
        AnimationSpec {
            artifact: false,
            creature_types: AnimationCreatureTypes::Additional(&["Bear"]),
            base: Some((6, 6)),
            keywords: Keywords::TRAMPLE,
            duration: AnimationDuration::EndOfTurn,
        },
        AnimationTypeMode::AddCreature,
    );
    assert_eq!(stats(&game, land), (6, 6));
    create_balloon(&mut game, source, land);
    let token = token_views(&game).pop().unwrap();
    assert_eq!(token.kinds, 1 | 2);
    assert_eq!(token.colors, RED);
    assert!(token.has_subtype("Forest") && token.has_subtype("Balloon"));
    assert!(!token.has_subtype("Bear") && !token.keywords.contains(Keywords::TRAMPLE));
    assert_eq!((token.power, token.toughness), (1, 1));
    assert!(
        game.legal_actions()
            .contains(&Action::TapMana { card: token.id })
    );
}

#[test]
fn copied_front_and_back_each_keep_their_own_copiable_types_and_colors() {
    let _rules = fixture_rules::activate(&[(card(SOURCE), &CREATOR)]);
    let mut game = scenario(PLAYER);
    let source = put(&mut game, PLAYER, SOURCE, Zone::Battlefield);
    let donor = put(&mut game, PLAYER, "Inverted Iceberg", Zone::Battlefield);
    let donor_ref = game.object_ref(donor);
    let front = game.object_view(donor);
    game.transform_permanent(donor_ref);
    let back = game.object_view(donor);
    assert_ne!(front.card, back.card);
    assert_eq!(back.kinds & 1, 1);
    create_balloon(&mut game, source, donor);
    let token = token_views(&game).pop().unwrap();
    assert_eq!(token.card, back.card);
    assert_eq!(token.colors, back.colors | RED);
    assert_eq!(token.kinds, back.kinds | 1);
    game.transform_permanent(token.reference);
    let flipped = game.object_view(token.id);
    assert_eq!(flipped.reference, token.reference);
    assert_eq!(flipped.card, front.card);
    assert_eq!(flipped.colors, front.colors | RED);
    assert_eq!(flipped.kinds, front.kinds | 1);
    assert!(flipped.has_subtype("Balloon"));
    assert!(
        flipped
            .keywords
            .contains(Keywords::FLYING.union(Keywords::HASTE))
    );
    assert_eq!((flipped.power, flipped.toughness), (1, 1));
}

#[test]
fn the_fixed_copiable_base_takes_precedence_over_a_printed_power_toughness_cda() {
    let _rules = fixture_rules::activate(&[(card(SOURCE), &CREATOR)]);
    let mut game = scenario(PLAYER);
    let source = put(&mut game, PLAYER, SOURCE, Zone::Battlefield);
    let donor = put(&mut game, PLAYER, "Regal Bunnicorn", Zone::Battlefield);
    assert_eq!(stats(&game, donor), (2, 2));
    create_balloon(&mut game, source, donor);
    let token = token_views(&game).pop().unwrap();
    assert_eq!(stats(&game, donor), (3, 3));
    assert_eq!(stats(&game, token.id), (1, 1));
    assert!(token.has_subtype("Rabbit") && token.has_subtype("Balloon"));
    assert_eq!(token.colors, 1 | RED);
}
