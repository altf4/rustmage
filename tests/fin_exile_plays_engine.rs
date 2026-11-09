//! Exact exile receipts, normal timing and independent paid/free play routes.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const PLAYER_COMBAT_DAMAGE: TriggerCondition = TriggerCondition::PlayerDamaged {
    subject: DamageSubject::Source,
    combat_only: true,
    opponent_only: false,
    one_or_more: false,
};
const LIGHTNING: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[TriggeredAbility {
        condition: PLAYER_COMBAT_DAMAGE,
        targets: TargetPlan::NONE,
        program: &[Instruction::ExileTopPlayWhileSourceControlled],
    }],
    ..CardRules::EMPTY
};
const RENO: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    triggers: &[TriggeredAbility {
        condition: PLAYER_COMBAT_DAMAGE,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ExileTopLibraryRemember {
                players: PlayerSet::DamagedPlayer,
                amount: AmountValue::Fixed(1),
            },
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::Sacrifice,
                players: PlayerSet::Controller,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 1 | 16,
                    nonland: false,
                    other: true,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
                amount: 1,
                optional_skip: Some(1),
            },
            Instruction::GrantExiledResultFlexiblePlay,
        ],
    }],
    ..CardRules::EMPTY
};
const TRIAD: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::ExileEachTopPlayOwnAndLesserFree],
    ),
    ..CardRules::EMPTY
};
const GRAVE: TargetRule = TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 4 | 8,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const SCHEMES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Union(&[
            GRAVE,
            TargetRule::OwnedExileWithFlashback,
        ]))),
        &[
            Instruction::ReturnToHand,
            Instruction::AddMana(ManaProduction::Fixed([0, 0, 0, 1, 0, 0])),
        ],
    ),
    ..CardRules::EMPTY
};

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn attack_unblocked(game: &mut Game, source: CardId) {
    for _ in 0..8 {
        if game.state.pending == Pending::Attackers {
            break;
        }
        act(game, Action::Pass);
    }
    act(game, Action::Attack { card: source });
    act(game, Action::FinishAttackers);
    pass_pair(game);
    act(game, Action::FinishBlockers);
    pass_pair(game);
    assert_eq!(
        game.state
            .stack
            .last()
            .unwrap()
            .damage_event
            .unwrap()
            .amount,
        2
    );
}

#[test]
fn paid_play_ends_when_the_exact_source_changes_control_or_leaves() {
    let lightning = Card::from_name("Lightning, Security Sergeant").unwrap();
    let _rules = fixture_rules::activate(&[(lightning, &LIGHTNING)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = spawn(&mut game, player, lightning, Zone::Battlefield);
    let island = put(&mut game, player, "Island", Zone::Library);
    let play = Action::PlayLand { card: island };
    attack_unblocked(&mut game, source);
    let exact = game.object_ref(source);
    let mut lost_early = game.clone();
    lost_early.record_control(exact, opponent, None);
    settle(&mut lost_early);
    assert_eq!(lost_early.object_view(island).zone, Zone::Exile);
    assert!(lost_early.observe(player).exile_permissions.is_empty());

    settle(&mut game);
    assert_eq!(game.object_view(island).zone, Zone::Exile);
    assert_eq!(
        game.observe(player).exile_permissions[0].source,
        Some(exact)
    );
    reject(&mut game, play.clone());
    game.state.step = Step::Main2;
    game.give_priority();
    let mut stolen = game.clone();
    stolen.record_control(exact, opponent, None);
    stolen.record_control(exact, player, None);
    reject(&mut stolen, play.clone());

    let mut blinked = game.clone();
    blinked.move_object(source, Zone::Exile);
    blinked.move_object(source, Zone::Battlefield);
    reject(&mut blinked, play.clone());
    act(&mut game, play);
    assert_eq!(game.object_view(island).zone, Zone::Battlefield);
    assert_eq!(game.object_view(island).controller, player);
}

#[test]
fn optional_sacrifice_grants_only_the_actual_damaged_players_top_card() {
    let reno = Card::from_name("Reno and Rude").unwrap();
    let _rules = fixture_rules::activate(&[(reno, &RENO)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = spawn(&mut game, player, reno, Zone::Battlefield);
    let payment = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    let enemy_card = put(&mut game, opponent, "Shock", Zone::Library);
    let own_card = put(&mut game, player, "Shock", Zone::Library);
    let unrelated = put(&mut game, opponent, "Shock", Zone::Exile);
    attack_unblocked(&mut game, source);
    pass_pair(&mut game);
    assert_eq!(game.object_view(enemy_card).zone, Zone::Exile);
    assert_eq!(game.object_view(own_card).zone, Zone::Library);
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineEffectPayment);
    assert_eq!(declined.object_view(payment).zone, Zone::Battlefield);
    assert!(declined.observe(player).exile_permissions.is_empty());
    let source_payment = Action::ChooseEffectObject {
        object: game.object_ref(source),
    };
    reject(&mut game, source_payment);
    let accepted = Action::ChooseEffectObject {
        object: game.object_ref(payment),
    };
    act(&mut game, accepted);
    assert_eq!(game.object_view(payment).zone, Zone::Graveyard);
    assert!(game.observe(player).exile_permissions[0].spend_any_mana);
    assert!(!game.observe(player).exile_permissions[0].cast_only);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];
    reject(&mut game, Action::AnnounceExile { card: unrelated });
    act(&mut game, Action::AnnounceExile { card: enemy_card });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(opponent),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 16);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    assert_eq!(game.object_view(enemy_card).zone, Zone::Graveyard);
    assert_eq!(game.object_view(enemy_card).owner, opponent);
}

#[test]
fn compared_exiles_allow_the_lower_land_and_the_owned_free_spell() {
    let divination = Card::from_name("Divination").unwrap();
    let _rules = fixture_rules::activate(&[(divination, &TRIAD)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let trigger = spawn(&mut game, player, divination, Zone::Hand);
    let own = put(&mut game, player, "Hill Giant", Zone::Library);
    let land = put(&mut game, opponent, "Island", Zone::Library);
    let mut equal = game.clone();
    equal.state.objects[land as usize].card = Card::from_name("Hill Giant").unwrap();

    for branch in [&mut game, &mut equal] {
        cast(branch, trigger, None);
        settle(branch);
        assert_eq!(branch.object_view(own).zone, Zone::Exile);
        assert_eq!(branch.object_view(land).zone, Zone::Exile);
    }
    assert_eq!(equal.observe(player).exile_permissions.len(), 1);
    assert_eq!(
        equal.observe(player).exile_permissions[0].object,
        equal.object_ref(own)
    );
    assert_eq!(game.observe(player).exile_permissions.len(), 2);
    let play = Action::PlayLand { card: land };
    let mut no_plays = game.clone();
    no_plays.state.land_played = true;
    reject(&mut no_plays, play.clone());
    act(&mut game, play);
    assert_eq!(game.object_view(land).owner, opponent);
    assert_eq!(game.object_view(land).controller, player);
    game.state.players[player.index()].mana = [0; 6];
    reject(&mut game, Action::AnnounceExile { card: own });
    act(
        &mut game,
        Action::AnnounceExileFree {
            card: own,
            face: CastFace::Normal,
            flash_surcharge: false,
        },
    );
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(own).zone, Zone::Battlefield);
}

#[test]
fn flashback_target_uses_public_exact_exile_and_the_graveyard_alternative() {
    let schemes = Card::from_name("Sorceress's Schemes").unwrap();
    let _rules = fixture_rules::activate(&[(schemes, &SCHEMES)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = spawn(&mut game, player, schemes, Zone::Hand);
    let returned = put(&mut game, player, "Resentful Revelation", Zone::Exile);
    let hidden = put(&mut game, player, "Resentful Revelation", Zone::Exile);
    game.remember_face_down_exile(game.object_ref(hidden), 1 << player.index(), None);
    let foreign = put(
        &mut game,
        player.other(),
        "Resentful Revelation",
        Zone::Exile,
    );
    let ordinary = put(&mut game, player, "Shock", Zone::Exile);
    let grave = put(&mut game, player, "Shock", Zone::Graveyard);
    let choose_card = |card| Action::ChooseTarget {
        target: Target::Card(card),
    };
    act(&mut game, Action::AnnounceCast { card: spell });
    for illegal in [hidden, foreign, ordinary] {
        reject(&mut game, choose_card(illegal));
    }
    let mut from_graveyard = game.clone();
    for (branch, target) in [(&mut game, returned), (&mut from_graveyard, grave)] {
        act(branch, choose_card(target));
        if branch.legal_actions().contains(&Action::FinishTargets) {
            act(branch, Action::FinishTargets);
        }
        pay(branch);
    }
    let mut stale = game.clone();
    stale.move_object(returned, Zone::Hand);
    stale.move_object(returned, Zone::Exile);
    let mana_before = stale.state.players[player.index()].mana;
    settle(&mut stale);
    assert_eq!(stale.object_view(returned).zone, Zone::Exile);
    assert_eq!(stale.state.players[player.index()].mana, mana_before);
    settle(&mut game);
    assert_eq!(game.object_view(returned).zone, Zone::Hand);
    assert_eq!(
        game.state.players[player.index()].mana[3],
        mana_before[3] + 1
    );
    settle(&mut from_graveyard);
    assert_eq!(from_graveyard.object_view(grave).zone, Zone::Hand);
}
