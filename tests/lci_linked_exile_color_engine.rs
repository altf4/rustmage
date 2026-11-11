//! Only the live public exile of the exact linked ability contributes colors.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const BLUE: Mana = [0, 1, 0, 0, 0, 0];
const GREEN: Mana = [0, 0, 0, 0, 1, 0];
const RED: Mana = [0, 0, 0, 1, 0, 0];
const WHITE: Mana = [1, 0, 0, 0, 0, 0];
const COLORLESS: Mana = [0, 0, 0, 0, 0, 1];
const TAP: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost {
        tap: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::AddMana(ManaProduction::Fixed(COLORLESS))],
    sorcery: false,
    once_per_turn: false,
};
const LINKER: CardRules = CardRules {
    effect: Effect::Land,
    entry: EntryTapped::Always,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 0,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            min: 0,
            max: 3,
        }),
        program: &[Instruction::ExileGraveTargetsLinked],
    }],
    activated: &[
        TAP,
        ActivatedAbility {
            program: &[Instruction::AddMana(ManaProduction::LinkedExileCardColor)],
            ..TAP
        },
    ],
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

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn enter(game: &mut Game, source: CardId, targets: &[CardId]) {
    act(game, Action::PlayLand { card: source });
    for _ in 0..8 {
        if matches!(game.state.pending, Pending::Targets { .. }) {
            break;
        }
        choose(game, |action| {
            matches!(action, Action::PutTrigger { .. } | Action::Pass)
        });
    }
    for &card in targets {
        act(game, target(card));
    }
    if matches!(game.state.pending, Pending::Targets { .. }) {
        act(game, Action::FinishTargets);
    }
    settle(game);
    assert!(game.object_view(source).tapped);
    game.untap_objects(&[source], untap_rules::UntapCause::Effect);
}

#[test]
fn the_trigger_exiles_from_both_graves_and_funds_a_cast_with_one_chosen_color() {
    let _rules =
        fixture_rules::activate(&[(Card::from_name("Captivating Cave").unwrap(), &LINKER)]);
    let p = Player::P0;
    let mut game = scenario(p);
    game.state.players[p.index()].mana = [0; 6];
    let source = put(&mut game, p, "Captivating Cave", Zone::Hand);
    let multicolor = put(&mut game, p, "Pond Prophet", Zone::Graveyard);
    let red = put(&mut game, p.other(), "Shock", Zone::Graveyard);
    let colorless = put(&mut game, p.other(), "Forest", Zone::Graveyard);
    let spell = put(&mut game, p, "Spyglass Siren", Zone::Hand);
    enter(&mut game, source, &[multicolor, red, colorless]);
    for id in [multicolor, red, colorless] {
        assert_eq!(game.object_view(id).zone, Zone::Exile);
    }
    for production in [BLUE, GREEN, RED] {
        assert!(game.legal_actions().contains(&mana(source, 1, production)));
    }
    assert!(game.legal_actions().contains(&mana(source, 0, COLORLESS)));
    reject(&mut game, mana(source, 1, COLORLESS));
    reject(&mut game, mana(source, 1, WHITE));
    reject(&mut game, mana(source, 1, [0, 1, 0, 0, 1, 0]));

    act(&mut game, Action::AnnounceCast { card: spell });
    let action = mana(source, 1, BLUE);
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let saved = game.snapshot();
    let revision = game.revision();
    let mut indexed = game.clone();
    act(&mut game, action.clone());
    indexed.apply_index(p, revision, index).unwrap();
    for branch in [&mut game, &mut indexed] {
        pay(branch);
        assert_eq!(branch.object_view(spell).zone, Zone::Stack);
    }
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    act(&mut indexed, action);
    pay(&mut indexed);
    assert_eq!(game.snapshot(), indexed.snapshot());
}

#[test]
fn changed_incarnations_and_hidden_or_unlinked_cards_do_not_supply_colors() {
    let definition = Card::from_name("Captivating Cave").unwrap();
    let _rules = fixture_rules::activate(&[(definition, &LINKER)]);
    let p = Player::P1;
    let mut game = scenario(p);
    game.state.players[p.index()].mana = [0; 6];
    let source = put(&mut game, p, "Captivating Cave", Zone::Hand);
    let donor = put(&mut game, p.other(), "Pond Prophet", Zone::Graveyard);
    let colorless = put(&mut game, p, "Forest", Zone::Graveyard);
    let hidden = put(&mut game, p.other(), "Savannah Lions", Zone::Exile);
    let red = put(&mut game, p, "Shock", Zone::Exile);
    enter(&mut game, source, &[donor, colorless]);
    let object = game.object_ref(source);
    let hidden = game.object_ref(hidden);
    game.remember_face_down_exile(hidden, 1 << p.other().index(), None);
    game.link_exile_to_ability(object, None, hidden);
    assert!(!game.legal_actions().contains(&mana(source, 1, WHITE)));
    assert!(!game.legal_actions().contains(&mana(source, 1, RED)));
    assert_eq!(game.object_view(red).zone, Zone::Exile);

    let mut copied = game.clone();
    copied.apply_temporary_copy(object, CopyRecipe::new(definition), CopyDeadline::EndOfTurn);
    assert_eq!(copied.object_ref(source), object);
    reject(&mut copied, mana(source, 1, BLUE));
    act(&mut copied, mana(source, 1, [0; 6]));
    assert!(copied.object_view(source).tapped);
    assert_eq!(copied.state.players[p.index()].mana, [0; 6]);

    let spell = put(&mut game, p, "Spyglass Siren", Zone::Hand);
    assert!(
        game.legal_actions()
            .contains(&Action::AnnounceCast { card: spell })
    );
    game.move_object(donor, Zone::Graveyard);
    game.move_object(donor, Zone::Exile);
    reject(&mut game, Action::AnnounceCast { card: spell });
    reject(&mut game, mana(source, 1, BLUE));
    act(&mut game, mana(source, 1, [0; 6]));
    assert_eq!(game.state.players[p.index()].mana, [0; 6]);
}
