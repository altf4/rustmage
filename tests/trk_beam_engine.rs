//! Beam Me Up graveyard casting, cost payment, and stack-exit rules.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules, one};
use rustmage_core::{cards::*, engine::*};
use support::*;

const BEAM_COST: ManaCost = ManaCost {
    colored: [0, 1, 0, 0, 0, 0],
    generic: 1,
};
const BEAM_SPELL: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Draw(PlayerSet::Controller, 1)],
    ),
    extras: &CardExtras {
        beam_me_up_cost: Some(BEAM_COST),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const TAXER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::OpponentNoncreatureSpellTax(1)],
    }],
    ..CardRules::EMPTY
};
const GRANTER: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::GrantGraveyardBeamMeUp],
    }],
    ..CardRules::EMPTY
};
const VANILLA_SORCERY: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, &[]),
    ..CardRules::EMPTY
};
const X_SORCERY: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, &[]),
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const MAROONED_AURA: CardRules = CardRules {
    effect: Effect::Aura(one(TargetRule::Creature)),
    aura: Some(AuraSpec {
        kinds: 1,
        relation: -1,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        },
    }),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::AttachedCannotBeBeamedUp],
    }],
    ..CardRules::EMPTY
};

fn beam(card: CardId) -> Action {
    Action::AnnounceBeamMeUp {
        card,
        face: CastFace::Normal,
        cost: BEAM_COST,
        flash_surcharge: false,
        provider: None,
    }
}

fn clause(game: &mut Game, want: impl Fn(CastCostClause) -> bool) {
    let clause = game
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|row| !row.paid && want(row.clause))
        .unwrap()
        .try_into()
        .unwrap();
    act(game, Action::ChooseCastCost { clause });
}

fn pay_beam(game: &mut Game, creature: ObjectRef) {
    act(game, Action::BeginOrderedCosts);
    clause(game, |row| row == CastCostClause::Mana);
    pay(game);
    clause(game, |row| {
        matches!(row, CastCostClause::BeamMeUpReturn(_, _))
    });
    act(game, Action::PayBeamMeUpReturn { object: creature });
}

#[test]
fn graveyard_permission_timing_tax_and_return_cost() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Opt").unwrap(), &BEAM_SPELL),
        (Card::from_name("Hill Giant").unwrap(), &TAXER),
    ]);
    for actor in [Player::P0, Player::P1] {
        let mut game = scenario(actor);
        let spell = put(&mut game, actor, "Opt", Zone::Graveyard);
        let enemy_spell = put(&mut game, actor.other(), "Opt", Zone::Graveyard);
        let hand_spell = put(&mut game, actor, "Opt", Zone::Hand);
        admit(&game, beam(spell));
        reject(&mut game, beam(enemy_spell));
        reject(&mut game, beam(hand_spell));

        let creature = put(&mut game, actor.other(), "Grizzly Bears", Zone::Battlefield);
        let original = game.object_ref(creature);
        game.change_controller(original, actor);
        let taxer = put(&mut game, actor.other(), "Hill Giant", Zone::Battlefield);
        assert!(game.legal_actions().contains(&beam(spell)));
        // Beam Me Up grants cast permission, but does not change printed timing.
        game.state.step = Step::Draw;
        assert!(game.legal_actions().contains(&beam(spell))); // Opt is instant.
        game.state.step = Step::Main1;
        game.state.players[actor.index()].mana = [0, 1, 0, 0, 0, 2];
        act(&mut game, beam(spell));
        pay_beam(&mut game, original);
        let paid = game.state.stack[0].cast.unwrap();
        assert_eq!(paid.method, CastMethod::BeamMeUp);
        assert_eq!(paid.payment, [0, 1, 0, 0, 0, 2]);
        assert_eq!(game.state.objects[creature as usize].zone, Zone::Hand);
        assert_eq!(game.state.objects[creature as usize].owner, actor.other());
        assert_ne!(game.object_ref(creature), original);
        assert_eq!(game.state.objects[spell as usize].zone, Zone::Stack);
        assert_eq!(game.state.players[actor.index()].mana, [0; 6]);
        settle(&mut game);
        assert_eq!(game.state.objects[spell as usize].zone, Zone::Exile);
        assert_eq!(game.state.objects[taxer as usize].zone, Zone::Battlefield);
    }
}

#[test]
fn return_choice_is_transactional_and_owned_branches_restore() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &BEAM_SPELL)]);
    for actor in [Player::P0, Player::P1] {
        let mut game = scenario(actor);
        let spell = put(&mut game, actor, "Opt", Zone::Graveyard);
        let creature = put(&mut game, actor, "Grizzly Bears", Zone::Battlefield);
        game.state.objects[creature as usize].tapped = true;
        let returned = game.object_ref(creature);
        game.state.players[actor.index()].mana = [0, 1, 0, 0, 0, 1];
        act(&mut game, beam(spell));
        act(&mut game, Action::BeginOrderedCosts);
        clause(&mut game, |row| row == CastCostClause::Mana);
        pay(&mut game);
        clause(&mut game, |row| {
            matches!(row, CastCostClause::BeamMeUpReturn(_, _))
        });
        reject(
            &mut game,
            Action::PayBeamMeUpReturn {
                object: ObjectRef {
                    generation: returned.generation + 1,
                    ..returned
                },
            },
        );
        let action = Action::PayBeamMeUpReturn { object: returned };
        let before = game.snapshot();
        let revision = game.revision();
        let index = game
            .legal_actions()
            .iter()
            .position(|a| a == &action)
            .unwrap();
        let mut indexed = game.clone();
        indexed.apply_index(actor, revision, index).unwrap();
        assert_eq!(game.snapshot(), before);
        assert_eq!(
            game.apply(actor.other(), action.clone()),
            Err(Error::WrongActor)
        );
        act(&mut game, action.clone());
        assert_eq!(game.snapshot(), indexed.snapshot());
        game.restore(&before).unwrap();
        assert_eq!(
            game.apply_index(actor, revision, index),
            Err(Error::StaleRevision)
        );
        act(&mut game, action);
        assert_eq!(game.snapshot(), indexed.snapshot());
        assert_eq!(game.state.objects[creature as usize].zone, Zone::Hand);
        assert_eq!(game.state.objects[spell as usize].zone, Zone::Stack);
    }
}

#[test]
fn beam_exiles_on_every_stack_exit_and_failed_announcement_rolls_back() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &BEAM_SPELL)]);
    let actor = Player::P0;
    let mut game = scenario(actor);
    let spell = put(&mut game, actor, "Opt", Zone::Graveyard);
    let creature = put(&mut game, actor, "Grizzly Bears", Zone::Battlefield);
    let original = game.object_ref(creature);
    game.state.players[actor.index()].mana = [0, 1, 0, 0, 0, 1];
    act(&mut game, beam(spell));
    assert!(game.state.objects[spell as usize].flashback);
    // A failed transaction must restore the original graveyard object and flag.
    let snapshot = game.snapshot();
    reject(&mut game, Action::PayBeamMeUpReturn { object: original });
    assert_eq!(game.snapshot(), snapshot);
    pay_beam(&mut game, original);
    game.move_object(spell, Zone::Hand);
    assert_eq!(game.state.objects[spell as usize].zone, Zone::Exile);
}

#[test]
fn granted_beam_uses_printed_cost_only_in_main_phase_and_excludes_provider() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Duress").unwrap(), &VANILLA_SORCERY),
        (Card::from_name("Hill Giant").unwrap(), &GRANTER),
    ]);
    let actor = Player::P0;
    let mut game = scenario(actor);
    let spell = put(&mut game, actor, "Duress", Zone::Graveyard);
    let provider = put(&mut game, actor, "Hill Giant", Zone::Battlefield);
    let other = put(&mut game, actor, "Grizzly Bears", Zone::Battlefield);
    let provider_ref = game.object_ref(provider);
    let other_ref = game.object_ref(other);
    let cost = ManaCost {
        colored: [0, 0, 1, 0, 0, 0],
        generic: 0,
    };
    let action = Action::AnnounceBeamMeUp {
        card: spell,
        face: CastFace::Normal,
        cost,
        flash_surcharge: false,
        provider: Some(provider_ref),
    };
    game.state.step = Step::Draw;
    reject(&mut game, action.clone());
    game.state.step = Step::Main1;
    assert!(game.legal_actions().contains(&action));
    game.state.players[actor.index()].mana = [0, 0, 1, 0, 0, 0];
    act(&mut game, action);
    act(&mut game, Action::BeginOrderedCosts);
    clause(&mut game, |row| row == CastCostClause::Mana);
    pay(&mut game);
    clause(&mut game, |row| {
        matches!(row, CastCostClause::BeamMeUpReturn(_, _))
    });
    reject(
        &mut game,
        Action::PayBeamMeUpReturn {
            object: provider_ref,
        },
    );
    act(&mut game, Action::PayBeamMeUpReturn { object: other_ref });
    assert_eq!(
        game.state.stack[0].cast.unwrap().method,
        CastMethod::BeamMeUpGranted(provider_ref)
    );
    assert_eq!(
        game.state.objects[provider as usize].zone,
        Zone::Battlefield
    );
    assert_eq!(game.state.objects[other as usize].zone, Zone::Hand);
    settle(&mut game);
    assert_eq!(game.state.objects[spell as usize].zone, Zone::Exile);
}

#[test]
fn enchanted_creature_cannot_pay_beam_return() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Opt").unwrap(), &BEAM_SPELL),
        (Card::from_name("Pacifism").unwrap(), &MAROONED_AURA),
    ]);
    let mut game = scenario(Player::P0);
    let spell = put(&mut game, Player::P0, "Opt", Zone::Graveyard);
    let enchanted = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let aura = put(&mut game, Player::P1, "Pacifism", Zone::Battlefield);
    let enchanted_ref = game.object_ref(enchanted);
    game.state.objects[aura as usize].attached_to = Some(enchanted_ref);
    admit(&game, beam(spell));

    let other = put(&mut game, Player::P0, "Bear Cub", Zone::Battlefield);
    let other_ref = game.object_ref(other);
    game.state.players[Player::P0.index()].mana = [0, 1, 0, 0, 0, 1];
    act(&mut game, beam(spell));
    act(&mut game, Action::BeginOrderedCosts);
    clause(&mut game, |row| row == CastCostClause::Mana);
    pay(&mut game);
    clause(&mut game, |row| {
        matches!(row, CastCostClause::BeamMeUpReturn(_, _))
    });
    assert!(!game.legal_actions().contains(&Action::PayBeamMeUpReturn {
        object: enchanted_ref,
    }));
    reject(
        &mut game,
        Action::PayBeamMeUpReturn {
            object: enchanted_ref,
        },
    );
    act(&mut game, Action::PayBeamMeUpReturn { object: other_ref });
    assert_eq!(
        game.state.objects[enchanted as usize].zone,
        Zone::Battlefield
    );
    assert_eq!(game.state.objects[other as usize].zone, Zone::Hand);
}

#[test]
fn granted_beam_keeps_printed_x_and_uses_the_declared_value() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Exsanguinate").unwrap(), &X_SORCERY),
        (Card::from_name("Hill Giant").unwrap(), &GRANTER),
    ]);
    let actor = Player::P0;
    let mut game = scenario(actor);
    let spell = put(&mut game, actor, "Exsanguinate", Zone::Graveyard);
    let provider = put(&mut game, actor, "Hill Giant", Zone::Battlefield);
    let creature = put(&mut game, actor, "Grizzly Bears", Zone::Battlefield);
    let returned = game.object_ref(creature);
    let action = Action::AnnounceBeamMeUp {
        card: spell,
        face: CastFace::Normal,
        cost: ManaCost {
            colored: [0, 0, 2, 0, 0, 0],
            generic: 0,
        },
        flash_surcharge: false,
        provider: Some(game.object_ref(provider)),
    };
    game.state.players[actor.index()].mana = [0, 0, 2, 0, 0, 3];
    assert!(game.legal_actions().contains(&action));
    act(&mut game, action);
    act(&mut game, Action::ChooseX { value: 3 });
    pay_beam(&mut game, returned);
    let item = &game.state.stack[0];
    assert_eq!(item.x, Some(3));
    assert_eq!(item.mana_value(), 5);
    assert_eq!(item.cast.unwrap().payment, [0, 0, 2, 0, 0, 3]);
    settle(&mut game);
    assert_eq!(game.state.objects[spell as usize].zone, Zone::Exile);
}

#[test]
fn granted_beam_casts_the_instant_adventure_face_at_its_own_cost() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Hill Giant").unwrap(), &GRANTER)]);
    let actor = Player::P0;
    let mut game = scenario(actor);
    let front = Card::from_name("Questing Druid").unwrap();
    let adventure = front.adventure_spell().unwrap();
    let spell = put(&mut game, actor, "Questing Druid", Zone::Graveyard);
    let provider = put(&mut game, actor, "Hill Giant", Zone::Battlefield);
    let creature = put(&mut game, actor, "Grizzly Bears", Zone::Battlefield);
    let returned = game.object_ref(creature);
    let action = Action::AnnounceBeamMeUp {
        card: spell,
        face: CastFace::Adventure,
        cost: ManaCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 1,
        },
        flash_surcharge: false,
        provider: Some(game.object_ref(provider)),
    };
    game.state.players[actor.index()].mana = [0, 0, 0, 1, 0, 1];
    assert!(game.legal_actions().contains(&action));
    act(&mut game, action);
    pay_beam(&mut game, returned);
    let item = &game.state.stack[0];
    assert_eq!(item.cast.unwrap().face, CastFace::Adventure);
    assert_eq!(item.cast.unwrap().physical_card, Some(front));
    assert_eq!(item.source.unwrap().card, adventure);
    assert_eq!(item.cast.unwrap().payment, [0, 0, 0, 1, 0, 1]);
    game.move_object(spell, Zone::Hand);
    assert_eq!(game.state.objects[spell as usize].zone, Zone::Exile);
}
