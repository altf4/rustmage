//! The cast owns one exact nontargeted creature choice or private hand reveal.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules, one};
use rustmage_core::{cards::*, engine::*};
use support::*;

const CHOICE: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[Instruction::DamageScaled(
            AmountValue::ChosenCostCreaturePower,
        )],
    ),
    extras: &CardExtras {
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::ChooseControlledOrRevealCreature,
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
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const HAND_POWER: DynamicStats = DynamicStats {
    power: Some(StatExpr {
        value: StatValue::ControllerHandSize,
        multiplier: 1,
        offset: 0,
    }),
    toughness: None,
};
const CDA: CardRules = CardRules {
    effect: Effect::Creature(0, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Cda,
        fragments: &[StaticAbilityFragment::DynamicStats(0, HAND_POWER)],
    }],
    extras: &CardExtras {
        dynamic_stats: Some(HAND_POWER),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const WEAKEN: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[Instruction::Grow {
            subject: Subject::Targets,
            power: -6,
            toughness: 0,
        }],
    ),
    ..CardRules::EMPTY
};

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn target(creature: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(creature),
    }
}

fn choose_clause(game: &mut Game, player: Player, kind: CastCostClause) {
    let index = game
        .observe(player)
        .cast_costs
        .unwrap()
        .clauses
        .iter()
        .position(|row| row.clause == kind && !row.paid)
        .unwrap();
    act(
        game,
        Action::ChooseCastCost {
            clause: index as u16,
        },
    );
}

fn cost(object: ObjectRef) -> Action {
    Action::ChooseAdditionalCost { object }
}

fn visible(game: &Game, viewer: Player, card: CardId) -> bool {
    game.observe(viewer)
        .objects
        .iter()
        .any(|view| view.id == card)
}

fn resolve_response(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn battlefield_choice_is_untargeted_and_reads_signed_live_or_departed_power() {
    let _rules = fixture_rules::activate(&[(card("Divination"), &CHOICE), (Card::Shock, &WEAKEN)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let chosen = put(&mut game, player, "Goblin Piker", Zone::Battlefield);
    game.state.objects[chosen as usize].gained_keywords = Keywords::PROTECTION_BLUE
        .union(Keywords::LIFELINK)
        .union(Keywords::DEATHTOUCH);
    let exact = game.object_ref(chosen);
    let victim = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    game.state.objects[victim as usize].counters.plus_one = 4;
    let spell = put(&mut game, player, "Divination", Zone::Hand);
    let growth = put(&mut game, player, "Giant Growth", Zone::Hand);
    let weaken = put(&mut game, player, "Shock", Zone::Hand);

    act(&mut game, Action::AnnounceCast { card: spell });
    reject(&mut game, target(chosen));
    act(&mut game, target(victim));
    pay(&mut game);
    choose_clause(&mut game, player, CastCostClause::Additional);
    let candidates = game.observe(player).additional_cost.unwrap().candidates;
    assert_eq!(candidates, [exact]);
    assert!(game.observe(player.other()).additional_cost.is_none());
    act(&mut game, cost(exact));
    assert_eq!(game.object_ref(chosen), exact);
    assert_eq!(game.object_view(chosen).zone, Zone::Battlefield);
    assert_eq!(game.observe(player).stack.last().unwrap().targets.len(), 1);

    let mut negative = game.clone();
    cast(&mut negative, weaken, Some(Target::Card(chosen)));
    resolve_response(&mut negative);
    assert_eq!(stats(&negative, chosen).0, -4);
    settle(&mut negative);
    assert_eq!(negative.object_view(victim).damage, 0);

    cast(&mut game, growth, Some(Target::Card(chosen)));
    resolve_response(&mut game);
    assert_eq!(stats(&game, chosen).0, 5);
    let mut departed = game.clone();
    departed.move_object(chosen, Zone::Graveyard);
    departed.move_object(chosen, Zone::Battlefield);
    assert_ne!(departed.object_ref(chosen), exact);
    assert_eq!(stats(&departed, chosen).0, 2);
    for branch in [&mut game, &mut departed] {
        settle(branch);
        assert_eq!(branch.object_view(victim).zone, Zone::Battlefield);
        assert_eq!(branch.object_view(victim).damage, 5);
        assert_eq!(branch.state.players[player.index()].life, 20);
    }
}

#[test]
fn hand_payment_projects_exact_candidates_and_tracks_revealed_card_lki() {
    let _rules =
        fixture_rules::activate(&[(card("Divination"), &CHOICE), (card("Bear Cub"), &CDA)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let victim = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    game.state.objects[victim as usize].counters.plus_one = 5;
    let foreign = put(&mut game, player.other(), "Bear Cub", Zone::Hand);
    let exiled = put(&mut game, player, "Bear Cub", Zone::Exile);
    let land = put(&mut game, player, "Forest", Zone::Hand);
    let draw = put(&mut game, player, "Quick Study", Zone::Hand);
    let spell = put(&mut game, player, "Divination", Zone::Hand);
    reject(&mut game, Action::AnnounceCast { card: spell });

    let revealed = put(&mut game, player, "Bear Cub", Zone::Hand);
    let stale = game.object_ref(revealed);
    act(&mut game, Action::AnnounceCast { card: spell });
    act(&mut game, target(victim));
    pay(&mut game);
    choose_clause(&mut game, player, CastCostClause::Additional);

    game.move_object(revealed, Zone::Graveyard);
    game.move_object(revealed, Zone::Hand);
    let exact = game.object_ref(revealed);
    assert_ne!(exact, stale);
    reject(&mut game, cost(stale));
    for invalid in [victim, foreign, exiled, land, draw] {
        reject(&mut game, cost(game.object_ref(invalid)));
    }
    assert_eq!(
        game.observe(player).additional_cost.unwrap().candidates,
        [exact]
    );
    assert!(!visible(&game, player.other(), revealed));
    assert!(game.observe(player.other()).additional_cost.is_none());

    let saved = game.snapshot();
    let action = cost(exact);
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let mut indexed = game.clone();
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    act(&mut game, action);
    assert_eq!(indexed.snapshot(), game.snapshot());
    game.restore(&saved).unwrap();
    assert!(!visible(&game, player.other(), revealed));
    act(&mut game, cost(exact));
    let public = game.observe(player.other());
    assert!(public.known_hand.contains(&exact));
    assert!(visible(&game, player.other(), revealed));
    assert!(!visible(&game, player.other(), land));
    let event = public
        .stack
        .last()
        .unwrap()
        .event_object
        .as_deref()
        .unwrap();
    assert_eq!(event.object, exact);
    assert!(matches!(
        event.facts,
        EventObjectFacts::RevealedCostCreature { card: chosen, power: 3 }
            if chosen == card("Bear Cub")
    ));

    cast(&mut game, draw, None);
    resolve_response(&mut game);
    assert_eq!(stats(&game, revealed).0, 4);
    let mut departed = game.clone();
    departed.move_object(revealed, Zone::Graveyard);
    departed.move_object(revealed, Zone::Hand);
    put(&mut departed, player, "Island", Zone::Hand);
    assert_ne!(departed.object_ref(revealed), exact);
    assert_eq!(stats(&departed, revealed).0, 5);
    for branch in [&mut game, &mut departed] {
        settle(branch);
        assert_eq!(branch.object_view(victim).damage, 4);
    }
}

#[test]
fn a_partially_paid_cast_keeps_the_selected_hand_card_private() {
    let _rules =
        fixture_rules::activate(&[(card("Divination"), &CHOICE), (card("Bear Cub"), &CDA)]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(
        &mut game,
        player,
        "Silverquill, the Disputant",
        Zone::Battlefield,
    );
    let sacrifice = put(&mut game, player, "Goblin Piker", Zone::Battlefield);
    let victim = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let revealed = put(&mut game, player, "Bear Cub", Zone::Hand);
    let spell = put(&mut game, player, "Divination", Zone::Hand);
    let exact = game.object_ref(revealed);

    act(&mut game, Action::AnnounceCast { card: spell });
    act(&mut game, Action::ChooseCasualty { pay: true });
    act(&mut game, target(victim));
    pay(&mut game);
    choose_clause(&mut game, player, CastCostClause::Additional);
    act(&mut game, cost(exact));
    let private = game.observe(player);
    assert_eq!(
        private.announcing.unwrap().event_object.unwrap().object,
        exact
    );
    let public = game.observe(player.other());
    assert!(public.announcing.unwrap().event_object.is_none());
    assert!(public.stack.is_empty());
    assert!(!public.known_hand.contains(&exact));
    assert!(!visible(&game, player.other(), revealed));

    let mut cancelled = game.clone();
    act(&mut cancelled, Action::Concede);
    assert!(!visible(&cancelled, player.other(), revealed));
    assert!(
        !cancelled
            .observe(player.other())
            .known_hand
            .contains(&exact)
    );

    choose_clause(&mut game, player, CastCostClause::Casualty(0));
    let object = game.object_ref(sacrifice);
    act(&mut game, Action::PayCasualty { object });
    assert!(visible(&game, player.other(), revealed));
    assert_eq!(game.object_view(sacrifice).zone, Zone::Graveyard);

    put(&mut game, player, "Forest", Zone::Hand);
    game.move_object(revealed, Zone::Graveyard);
    let public = game.observe(player.other());
    let spell_copy = public
        .stack
        .iter()
        .chain(&public.waiting_triggers)
        .find(|item| item.kind == StackKind::CasualtyCopy)
        .unwrap()
        .copy_event
        .as_ref()
        .unwrap();
    let receipt = spell_copy.spell.event_object.as_ref().unwrap();
    assert_eq!(receipt.object, exact);
    assert!(matches!(
        receipt.facts,
        EventObjectFacts::RevealedCostCreature { power: 2, .. }
    ));
}
