//! Bargain's optional target plan follows the declared payment, including copies.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const BARGAIN: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::IfBargained { skip: 1 },
            Instruction::Tap,
            Instruction::GainLife(PlayerSet::Controller, 2),
        ],
    ),
    extras: &CardExtras {
        bargain_reduction: Some(2),
        bargain_targets: Some(TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 2,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }))),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const ARTIFACT_LAND: CardRules = CardRules {
    effect: Effect::Land,
    ..CardRules::EMPTY
};
const COPY_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::InstantSorcery,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CopyCapturedSpell {
            count: SpellCopyCount::One,
            retarget: true,
        }],
    }],
    ..CardRules::EMPTY
};

const DECLINE: Action = Action::ChooseBargain { sacrifice: None };
const BLUE: Mana = [0, 1, 0, 0, 0, 0];
const PRINTED: Mana = [0, 3, 0, 0, 0, 0];

fn target(id: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(id),
    }
}

#[test]
fn only_fundable_bargain_routes_with_available_targets_start_casting() {
    let card = Card::from_name("Quick Study").unwrap();
    let _rules = fixture_rules::activate(&[(card, &BARGAIN)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let spell = spawn(&mut game, player, card, Zone::Hand);
        let offering = put(&mut game, player, "Short Bow", Zone::Battlefield);
        let foreign = put(&mut game, player.other(), "Forest", Zone::Battlefield);
        game.state.players[player.index()].mana = BLUE;
        let start = Action::AnnounceCast { card: spell };
        // Only the discounted cost would be payable, but no land can be targeted.
        reject(&mut game, start.clone());

        let own = put(&mut game, player, "Forest", Zone::Battlefield);
        act(&mut game, start);
        reject(&mut game, DECLINE);
        let sacrifice = game.object_ref(offering);
        act(
            &mut game,
            Action::ChooseBargain {
                sacrifice: Some(sacrifice),
            },
        );
        assert_eq!(game.object_view(offering).zone, Zone::Battlefield);
        reject(&mut game, target(foreign));
        act(&mut game, target(own));
        pay(&mut game);
        let packet = game.state.stack.last().unwrap();
        assert_eq!(packet.cast.unwrap().bargain, Some(true));
        assert_eq!(packet.targets.len(), 1);
        assert_eq!(game.object_view(offering).zone, Zone::Graveyard);
        settle(&mut game);
        assert!(game.object_view(own).tapped);
        assert!(!game.object_view(foreign).tapped);
        assert_eq!(game.state.players[player.index()].life, 22);
    }
}

#[test]
fn free_cast_can_decline_bargain_without_a_target_or_payment() {
    let card = Card::from_name("Quick Study").unwrap();
    let _rules = fixture_rules::activate(&[(card, &BARGAIN)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let spell = spawn(&mut game, player, card, Zone::Exile);
    let offering = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let object = game.object_ref(spell);
    game.grant_exile_free_permission(player, object);
    game.state.players[player.index()].mana = [0; 6];
    act(
        &mut game,
        Action::AnnounceExileFree {
            card: spell,
            face: CastFace::Normal,
            flash_surcharge: false,
        },
    );
    let sacrifice = game.object_ref(offering);
    reject(
        &mut game,
        Action::ChooseBargain {
            sacrifice: Some(sacrifice),
        },
    );
    act(&mut game, DECLINE);
    assert_eq!(game.state.pending, Pending::PayCosts { player });
    pay(&mut game);
    let packet = game.state.stack.last().unwrap();
    assert_eq!(packet.cast.unwrap().bargain, Some(false));
    assert!(packet.targets.is_empty());
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 22);
    assert_eq!(game.object_view(offering).zone, Zone::Battlefield);
}

#[test]
fn paid_and_unpaid_bargain_targets_survive_raw_index_and_restore() {
    let card = Card::from_name("Quick Study").unwrap();
    let _rules = fixture_rules::activate(&[(card, &BARGAIN)]);
    let player = Player::P0;
    let mut parent = scenario(player);
    let spell = spawn(&mut parent, player, card, Zone::Hand);
    let offering = put(&mut parent, player, "Short Bow", Zone::Battlefield);
    let land = put(&mut parent, player, "Forest", Zone::Battlefield);
    parent.state.players[player.index()].mana = PRINTED;
    act(&mut parent, Action::AnnounceCast { card: spell });
    let saved = parent.snapshot();
    let sacrifice = parent.object_ref(offering);
    let paid = Action::ChooseBargain {
        sacrifice: Some(sacrifice),
    };
    let index = parent
        .legal_actions()
        .iter()
        .position(|action| *action == paid)
        .unwrap();
    let revision = parent.revision();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let mut unpaid = parent.clone();
    assert_eq!(parent.snapshot(), saved);
    drop(parent);

    act(&mut raw, paid);
    indexed.apply_index(player, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(unpaid.snapshot(), saved);
    for branch in [&mut raw, &mut indexed] {
        reject(branch, Action::FinishTargets);
        act(branch, target(land));
        pay(branch);
        settle(branch);
        assert_eq!(branch.object_view(offering).zone, Zone::Graveyard);
        assert!(branch.object_view(land).tapped);
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());

    indexed.restore(&saved).unwrap();
    for branch in [&mut indexed, &mut unpaid] {
        act(branch, DECLINE);
        reject(branch, target(land));
        pay(branch);
        settle(branch);
        assert_eq!(branch.object_view(offering).zone, Zone::Battlefield);
        assert!(!branch.object_view(land).tapped);
    }
    assert_eq!(unpaid.snapshot(), indexed.snapshot());
    assert_eq!(raw.object_view(offering).zone, Zone::Graveyard);
}

#[test]
fn a_bargain_sacrifice_can_invalidate_the_spells_own_target() {
    let card = Card::from_name("Quick Study").unwrap();
    let artifact_land = Card::from_name("Scene of the Crime").unwrap();
    let _rules = fixture_rules::activate(&[(card, &BARGAIN), (artifact_land, &ARTIFACT_LAND)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = spawn(&mut game, player, card, Zone::Hand);
    let land = spawn(&mut game, player, artifact_land, Zone::Battlefield);
    game.state.players[player.index()].mana = BLUE;
    act(&mut game, Action::AnnounceCast { card: spell });
    let sacrifice = game.object_ref(land);
    act(
        &mut game,
        Action::ChooseBargain {
            sacrifice: Some(sacrifice),
        },
    );
    act(&mut game, target(land));
    pay(&mut game);
    assert_eq!(game.object_view(land).zone, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 20);
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
}

#[test]
fn a_bargained_copy_can_retarget_the_paid_land_group_without_sacrificing() {
    let card = Card::from_name("Quick Study").unwrap();
    let observer = Card::from_name("Bear Cub").unwrap();
    let _rules = fixture_rules::activate(&[(card, &BARGAIN), (observer, &COPY_OBSERVER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = spawn(&mut game, player, card, Zone::Hand);
    spawn(&mut game, player, observer, Zone::Battlefield);
    let first = put(&mut game, player, "Forest", Zone::Battlefield);
    let second = put(&mut game, player, "Island", Zone::Battlefield);
    let offering = put(&mut game, player, "Short Bow", Zone::Battlefield);
    game.state.players[player.index()].mana = BLUE;

    act(&mut game, Action::AnnounceCast { card: spell });
    let sacrifice = game.object_ref(offering);
    act(
        &mut game,
        Action::ChooseBargain {
            sacrifice: Some(sacrifice),
        },
    );
    act(&mut game, target(first));
    pay(&mut game);
    assert_eq!(game.state.stack.len(), 2);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.state.pending, Pending::Retarget { player });
    act(
        &mut game,
        Action::SetStackTarget {
            index: 0,
            target: Target::Card(second),
        },
    );
    act(&mut game, Action::FinishStackTargets);
    settle(&mut game);
    assert!(game.object_view(first).tapped);
    assert!(game.object_view(second).tapped);
    assert_eq!(game.state.players[player.index()].life, 24);
    assert_eq!(game.object_view(offering).zone, Zone::Graveyard);
}
