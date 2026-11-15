//! Four printed Star Trek T–Z cards after the reviewed shared engine checkpoint.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const ADD_LORE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield, kinds: 0, relation: 0,
            other: false, nonland: false, tapped: false,
            min_mana: 0, max_mana: u32::MAX,
        })),
        &[Instruction::PutCounters { subject: Subject::Targets, kind: CounterKind::Lore, amount: 1 }],
    ),
    ..CardRules::EMPTY
};

fn drive(game: &mut Game) {
    for _ in 0..48 {
        if game.state.pending == Pending::Priority && game.state.stack.is_empty()
            && game.state.resolving.is_none() && game.state.pending_triggers.is_empty() { return; }
        choose(game, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("resolution exceeded 48 actions: {:?}", game.state.pending);
}

fn advance_turn(game: &mut Game, current: Player) {
    game.state.step = Step::End;
    game.state.priority = current;
    for _ in 0..16 {
        if game.state.active != current { return; }
        choose(game, |action| matches!(action, Action::Pass));
    }
    panic!("turn did not advance");
}

fn lore(game: &mut Game, saga: CardId) {
    let spell = put(game, Player::P0, "Opt", Zone::Hand);
    game.state.priority = Player::P0;
    cast(game, spell, Some(Target::Card(saga)));
}

#[test]
fn praxis_uses_one_six_name_count_and_only_preexisting_creatures_shrink() {
    let mut game = scenario(Player::P0);
    for name in ["Forest", "Island", "Mountain", "Plains", "Swamp", "Vulcan"] {
        put(&mut game, Player::P0, name, Zone::Battlefield);
    }
    let ours = put(&mut game, Player::P0, "Hill Giant", Zone::Battlefield);
    let theirs = put(&mut game, Player::P1, "Hill Giant", Zone::Battlefield);
    let spell = put(&mut game, Player::P0, "The Destruction of Praxis", Zone::Hand);
    cast(&mut game, spell, None);
    drive(&mut game);
    assert_eq!(game.object_view(ours).zone, Zone::Graveyard);
    assert_eq!(game.object_view(theirs).zone, Zone::Graveyard);
    let tokens: Vec<_> = game.controlled_token_ids(Player::P0)
        .filter(|&id| game.object_view(id).card == Card(6010)).collect();
    assert_eq!(tokens.len(), 2);
    for token in tokens {
        assert_eq!(stats(&game, token), (2, 1));
        assert!(game.object_view(token).keywords.contains(Keywords::HASTE));
    }
}

#[test]
fn needs_sacrifices_as_cast_cost_then_protects_creatures_and_player() {
    let mut empty = scenario(Player::P0);
    let spell = put(&mut empty, Player::P0, "The Needs of the Many", Zone::Hand);
    reject(&mut empty, Action::AnnounceCast { card: spell });

    let mut game = scenario(Player::P0);
    let paid = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let protected = put(&mut game, Player::P0, "Hill Giant", Zone::Battlefield);
    let opposing = put(&mut game, Player::P1, "Hill Giant", Zone::Battlefield);
    let spell = put(&mut game, Player::P0, "The Needs of the Many", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    let paid_ref = game.object_ref(paid);
    act(&mut game, Action::ChooseAdditionalCost { object: paid_ref });
    pay(&mut game);
    assert_eq!(game.object_view(paid).zone, Zone::Graveyard);
    drive(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Exile);
    for keyword in [Keywords::HEXPROOF, Keywords::INDESTRUCTIBLE] {
        assert!(game.object_view(protected).keywords.contains(keyword));
        assert!(!game.object_view(opposing).keywords.contains(keyword));
    }
    assert!(game.state.players[0].hexproof_until_end_turn);
    assert_eq!(game.observe(Player::P0).player_damage_shields.len(), 1);
}

#[test]
fn wrath_second_chapter_draws_only_for_actual_hand_exile_and_third_rejects_tokens() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &ADD_LORE)]);
    let mut game = scenario(Player::P0);
    let victim = put(&mut game, Player::P1, "V'Ger, the Intruder", Zone::Battlefield);
    let saga = put(&mut game, Player::P0, "The Wrath of Khan", Zone::Hand);
    cast(&mut game, saga, None);
    for _ in 0..24 {
        if game.legal_actions().contains(&Action::ChooseTarget { target: Target::Card(victim) }) {
            act(&mut game, Action::ChooseTarget { target: Target::Card(victim) });
        } else if game.legal_actions().contains(&Action::FinishStackTargets) {
            act(&mut game, Action::FinishStackTargets);
        } else if game.state.pending == Pending::Priority && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty() && game.state.resolving.is_none() { break; }
        else { choose(&mut game, |a| matches!(a, Action::Pass | Action::PutTrigger { .. })); }
    }
    assert_eq!(game.object_view(victim).zone, Zone::Battlefield);
    let card = put(&mut game, Player::P0, "Shock", Zone::Hand);
    put(&mut game, Player::P0, "Forest", Zone::Library);
    lore(&mut game, saga);
    for _ in 0..24 {
        if matches!(game.state.pending, Pending::EffectChoice { .. }) { break; }
        choose(&mut game, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    let card_ref = game.object_ref(card);
    act(&mut game, Action::SelectResolutionExile { object: card_ref });
    drive(&mut game);
    assert_eq!(game.object_view(card).zone, Zone::Exile);
    assert_eq!(game.state.players[0].cards_drawn_this_turn, 1);

    let token = spawn(&mut game, Player::P1, Card(6010), Zone::Battlefield);
    let permanent = put(&mut game, Player::P1, "Forest", Zone::Battlefield);
    lore(&mut game, saga);
    for _ in 0..24 {
        if game.legal_actions().contains(&Action::Sacrifice { object: game.object_ref(permanent) }) { break; }
        choose(&mut game, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    let token_ref = game.object_ref(token);
    let permanent_ref = game.object_ref(permanent);
    reject(&mut game, Action::Sacrifice { object: token_ref });
    act(&mut game, Action::Sacrifice { object: permanent_ref });
    drive(&mut game);
    assert_eq!(game.object_view(permanent).zone, Zone::Graveyard);
    assert_eq!(game.object_view(token).zone, Zone::Battlefield);
}

#[test]
fn wrath_first_chapter_takes_only_its_damaged_creature_at_next_combat_this_turn() {
    let mut game = scenario(Player::P0);
    let victim = put(&mut game, Player::P1, "V'Ger, the Intruder", Zone::Battlefield);
    game.state.objects[victim as usize].tapped = true;
    let saga = put(&mut game, Player::P0, "The Wrath of Khan", Zone::Hand);
    cast(&mut game, saga, None);
    for _ in 0..24 {
        if game.legal_actions().contains(&Action::ChooseTarget { target: Target::Card(victim) }) {
            act(&mut game, Action::ChooseTarget { target: Target::Card(victim) });
        } else if game.legal_actions().contains(&Action::FinishStackTargets) {
            act(&mut game, Action::FinishStackTargets);
        } else if game.state.pending == Pending::Priority && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty() && game.state.resolving.is_none() { break; }
        else { choose(&mut game, |action| matches!(action, Action::Pass | Action::PutTrigger { .. })); }
    }
    assert_eq!(game.object_view(victim).controller, Player::P1);
    assert!(game.object_view(victim).tapped);
    assert_eq!(game.object_view(victim).damage, 4);
    assert!(!game.object_view(victim).keywords.contains(Keywords::HASTE));

    game.begin_step(Step::BeginCombat);
    game.prepare_triggers();
    drive(&mut game);
    assert_eq!(game.object_view(victim).controller, Player::P0);
    assert!(!game.object_view(victim).tapped);
    assert!(game.object_view(victim).keywords.contains(Keywords::HASTE));
    advance_turn(&mut game, Player::P0);
    assert_eq!(game.object_view(victim).controller, Player::P1);
    assert!(!game.object_view(victim).keywords.contains(Keywords::HASTE));
}

#[test]
fn vger_modes_offer_private_hand_look_or_opponent_power_loss() {
    for mode in [0, 1] {
        let mut game = scenario(Player::P0);
        let opposing = put(&mut game, Player::P1, "Hill Giant", Zone::Battlefield);
        let own = put(&mut game, Player::P0, "Hill Giant", Zone::Battlefield);
        let secret = put(&mut game, Player::P1, "Shock", Zone::Hand);
        put(&mut game, Player::P0, "Forest", Zone::Library);
        let vger = put(&mut game, Player::P0, "V'Ger, the Intruder", Zone::Hand);
        cast(&mut game, vger, None);
        for _ in 0..24 {
            if game.legal_actions().contains(&Action::ChooseMode { mode }) { break; }
            choose(&mut game, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
        }
        act(&mut game, Action::ChooseMode { mode });
        if mode == 0 {
            act(&mut game, Action::ChooseTarget { target: Target::Player(Player::P1) });
            if game.legal_actions().contains(&Action::FinishStackTargets) { act(&mut game, Action::FinishStackTargets); }
            for _ in 0..24 {
                if matches!(game.state.pending, Pending::HandCardChoice { .. }) { break; }
                choose(&mut game, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
            }
            let actor = game.observe(Player::P0).hand_choice.unwrap();
            assert_eq!(actor.kind, HandChoiceKind::LookOnly);
            assert!(actor.inspected_cards.unwrap().iter().any(|o| o.id == secret));
            assert!(game.observe(Player::P1).hand_choice.is_none());
            act(&mut game, Action::ChooseHandCard { object: None });
            drive(&mut game);
            assert_eq!(game.state.players[0].cards_drawn_this_turn, 1);
        } else {
            drive(&mut game);
            assert_eq!(stats(&game, opposing), (2, 3));
            assert_eq!(stats(&game, own), (3, 3));
            advance_turn(&mut game, Player::P0);
            assert_eq!(stats(&game, opposing), (2, 3));
            advance_turn(&mut game, Player::P1);
            assert_eq!(stats(&game, opposing), (3, 3));
        }
    }
}
