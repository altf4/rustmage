//! Chosen-color mana restricted to abilities of land sources.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const BLUE: Mana = [0, 1, 0, 0, 0, 0];
const TWO_BLUE: Mana = [0, 2, 0, 0, 0, 0];
const BASIC: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost {
        tap: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::AddMana(ManaProduction::ChosenColor)],
    sorcery: false,
    once_per_turn: false,
};
const PROVIDER: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[
        BASIC,
        ActivatedAbility {
            program: &[Instruction::AddMana(ManaProduction::Tagged {
                production: &ManaProduction::ChosenColorAmount(2),
                quality: &ManaQuality {
                    restriction: ManaRestriction::LandActivation,
                    spell_bonus: ManaSpellBonus::None,
                    retention: ManaRetention::Step,
                },
            })],
            ..BASIC
        },
    ],
    entry: EntryTapped::Always,
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::Color),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const PAYOFF: ActivatedAbility = ActivatedAbility {
    cost: ActivationCost {
        colored: BLUE,
        mana_component: true,
        ..ActivationCost::ZERO
    },
    program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    ..BASIC
};
const LAND: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[
        PAYOFF,
        ActivatedAbility {
            origin: ActivationOrigin::Hand,
            ..PAYOFF
        },
    ],
    ..CardRules::EMPTY
};
const CREATURE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[PAYOFF],
    ..CardRules::EMPTY
};

fn mana(card: CardId, ability: u16, production: Mana) -> Action {
    Action::ActivateMana {
        card,
        ability,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production,
    }
}

fn activate(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

#[test]
fn chosen_amount_and_restriction_survive_the_lot_branch() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Captivating Cave").unwrap(), &PROVIDER),
        (Card::from_name("Hidden Cataract").unwrap(), &LAND),
        (Card::GrizzlyBears, &CREATURE),
    ]);
    let p = Player::P0;
    let mut game = scenario(p);
    game.state.players[p.index()].mana = [0; 6];
    let source = put(&mut game, p, "Captivating Cave", Zone::Hand);
    let land = spawn(
        &mut game,
        p,
        Card::from_name("Hidden Cataract").unwrap(),
        Zone::Battlefield,
    );
    let in_hand = spawn(
        &mut game,
        p,
        Card::from_name("Hidden Cataract").unwrap(),
        Zone::Hand,
    );
    let creature = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let spell = put(&mut game, p, "Spyglass Siren", Zone::Hand);
    act(&mut game, Action::PlayLand { card: source });
    act(
        &mut game,
        Action::ChooseEntryValue {
            value: EntryChoiceValue::Color(ChoiceColor::Blue),
        },
    );
    assert!(game.object_view(source).tapped);
    reject(&mut game, mana(source, 1, TWO_BLUE));
    game.untap_objects(&[source], untap_rules::UntapCause::Effect);
    reject(&mut game, mana(source, 1, [2, 0, 0, 0, 0, 0]));
    reject(&mut game, mana(source, 1, [0, 1, 0, 1, 0, 0]));

    let choose_two = mana(source, 1, TWO_BLUE);
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == choose_two)
        .unwrap();
    let saved = game.snapshot();
    let revision = game.revision();
    let mut one = game.clone();
    let mut indexed = game.clone();
    act(&mut one, mana(source, 0, BLUE));
    assert_eq!(one.state.players[p.index()].mana, BLUE);
    assert!(one.legal_actions().contains(&activate(creature, 0)));

    act(&mut game, choose_two.clone());
    indexed.apply_index(p, revision, index).unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    let lots = game.observe(p).mana_lots;
    assert_eq!(lots.len(), 1);
    assert_eq!(lots[0].restriction, ManaRestrictionView::LandActivation);
    assert_eq!(lots[0].amounts[1], 2);
    reject(&mut game, activate(creature, 0));
    reject(&mut game, Action::AnnounceCast { card: spell });

    for branch in [&mut game, &mut indexed] {
        act(branch, activate(land, 0));
        pay(branch);
        settle(branch);
        assert_eq!(branch.state.players[p.index()].mana, BLUE);
        act(branch, activate(in_hand, 1));
        pay(branch);
        settle(branch);
        assert_eq!(branch.state.players[p.index()].mana, [0; 6]);
        assert_eq!(branch.state.players[p.index()].life, 22);
    }
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    act(&mut indexed, choose_two);
    assert_eq!(indexed.observe(p).mana_lots[0].amounts[1], 2);
}
