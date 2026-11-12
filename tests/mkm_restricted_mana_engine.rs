//! Restricted mana follows the actual spell face and exact face-up special action.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const GOSSIP_MANA: Mana = [0, 0, 0, 1, 1, 0];
const GOSSIP: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        timing: ActivationTiming::Any,
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Tagged {
            production: &ManaProduction::Fixed(GOSSIP_MANA),
            quality: &ManaQuality {
                restriction: ManaRestriction::FaceDownSpellOrTurnCreatureFaceUp,
                spell_bonus: ManaSpellBonus::None,
                retention: ManaRetention::Step,
            },
        })],
    }],
    ..CardRules::EMPTY
};

fn activate(card: CardId) -> Action {
    Action::ActivateMana {
        card,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production: GOSSIP_MANA,
    }
}

#[test]
fn restricted_lots_fund_disguise_from_an_untapped_source_and_reject_face_up_spells() {
    let source = Card::from_name("Grizzly Bears").unwrap();
    let _rules = fixture_rules::activate(&[(source, &GOSSIP)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let producer = spawn(&mut game, player, source, Zone::Battlefield);
        let disguise = put(&mut game, player, "Bubble Smuggler", Zone::Hand);
        let shock = put(&mut game, player, "Shock", Zone::Hand);
        game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];

        reject(&mut game, Action::AnnounceCast { card: shock });
        let announce = Action::AnnounceDisguise { card: disguise };
        let snapshot = game.snapshot();
        let mut indexed = game.clone();
        let index = game
            .legal_actions()
            .iter()
            .position(|action| *action == announce)
            .unwrap();
        act(&mut game, announce);
        indexed
            .apply_index(player, indexed.revision(), index)
            .unwrap();
        assert_eq!(indexed.snapshot(), game.snapshot());

        act(&mut game, activate(producer));
        assert_eq!(game.observe(player).mana_lots.len(), 1);
        let payment = [0, 0, 0, 1, 1, 1];
        act(&mut game, Action::PayCosts { payment });
        assert_eq!(
            game.state.stack.last().unwrap().cast.unwrap().payment,
            payment
        );
        assert_eq!(
            game.observe(player.other()).face_down_spells[0].underlying,
            None
        );
        settle(&mut game);

        indexed.restore(&snapshot).unwrap();
        act(&mut indexed, activate(producer));
        reject(&mut indexed, Action::AnnounceCast { card: shock });
        reject(
            &mut indexed,
            Action::Cast {
                card: shock,
                target: Some(Target::Player(player.other())),
                payment: [0, 0, 0, 1, 0, 0],
            },
        );
    }
}

#[test]
fn the_face_down_creature_can_use_the_lots_to_turn_up_as_equipment() {
    let source = Card::from_name("Grizzly Bears").unwrap();
    let _rules = fixture_rules::activate(&[(source, &GOSSIP)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let producer = spawn(&mut game, player, source, Zone::Battlefield);
    let weapon = Card::from_name("Concealed Weapon").unwrap();
    let hidden = spawn(&mut game, player, weapon, Zone::Hand);
    act(&mut game, Action::AnnounceDisguise { card: hidden });
    pay(&mut game);
    settle(&mut game);
    assert!(game.object_view(hidden).is_creature());

    let object = game.object_ref(hidden);
    let cost = ManaCost {
        colored: [0, 0, 0, 1, 0, 0],
        generic: 2,
    };
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];
    act(&mut game, Action::BeginFaceUp { object, cost });
    act(&mut game, activate(producer));
    act(
        &mut game,
        Action::PayFaceUp {
            payment: [0, 0, 0, 1, 1, 1],
        },
    );
    assert_eq!(game.object_ref(hidden), object);
    assert_eq!(game.object_view(hidden).card, weapon);
    assert!(!game.object_view(hidden).is_creature());
}
