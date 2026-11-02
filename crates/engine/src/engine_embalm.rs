//! Acquired graveyard activations use ordinary costs and frozen immutable recipes.
use super::*;
use ability_world::CharacteristicWorld;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(crate) const ACTIVATION_INDEX: u16 = crate::cards::activation_slots::EMBALM;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(crate) const TEMPLATE: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Graveyard,
    cycling: false,
    cost: ActivationCost {
        mana_component: true,
        exile_self: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::CreateEmbalmedSourceCopy],
    sorcery: true,
    once_per_turn: false,
};
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct EmbalmAbilityView {
    pub object: ObjectRef,
    pub acquisition: u64,
    pub expires_turn: u32,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) type Rows = Option<SharedVec<EmbalmAbilityView>>;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn owned_bytes(rows: &Rows) -> usize {
    rows.as_ref().map_or(0, |rows| {
        std::mem::size_of::<Vec<EmbalmAbilityView>>()
            + rows.len() * std::mem::size_of::<EmbalmAbilityView>()
    })
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn abilities<W: CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
) -> Vec<EmbalmAbilityView> {
    let object = world.object(id);
    if object.zone != Zone::Graveyard || !object.is_card() {
        return Vec::new();
    }
    world
        .game()
        .state
        .embalm_abilities
        .as_ref()
        .map_or_else(Vec::new, |rows| {
            rows.iter()
                .filter(|r| {
                    r.object == world.reference(id) && r.expires_turn >= world.game().state.turn
                })
                .copied()
                .collect()
        })
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn grant_embalm_targets(&mut self, item: &SpellView) {
        for bound in &item.targets {
            if !self.same_target_object(*bound) {
                continue;
            }
            let Some(object) = bound.object else { continue };
            let card = &self.state.objects[object.card as usize];
            if card.zone != Zone::Graveyard || !card.is_card() || !card.card.is_creature() {
                continue;
            }
            let acquisition = self.next_role_timestamp();
            let expires_turn = self.state.turn;
            self.state
                .embalm_abilities
                .get_or_insert_default()
                .push(EmbalmAbilityView {
                    object,
                    acquisition,
                    expires_turn,
                });
        }
    }
    pub(super) fn visit_embalm_activations(
        &self,
        id: CardId,
        actor: Player,
        sorcery: bool,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let card = &self.state.objects[id as usize];
        if !sorcery
            || card.owner != actor
            || self.nonmana_actions_prohibited()
            || self.named_activation_prohibited(id)
            || CharacteristicWorld::activation_prohibited(self, actor, id)
        {
            return false;
        }
        let rows = abilities(self, id);
        if rows.is_empty() {
            return false;
        }
        if !self
            .activation_mana_costs(id, card.card, ACTIVATION_INDEX, 0)
            .any(|cost| {
                self.activation_can_fund(
                    FundingGoal {
                        actor,
                        required: cost.colored,
                        generic: cost.generic,
                        reserve: None,
                        source: Some((
                            self.object_ref(id),
                            ACTIVATION_INDEX,
                            Counters::default(),
                            card.card,
                            None,
                        )),
                    },
                    None,
                )
            })
        {
            return false;
        }
        rows.into_iter().any(|row| {
            emit(Action::AnnounceEmbalm {
                source: row.object,
                acquisition: row.acquisition,
            })
        })
    }
    pub(super) fn announce_embalm(&mut self, source: ObjectRef, acquisition: u64) {
        let actor = self.state.objects[source.card as usize].owner;
        let mut item = self.activation_item(actor, source.card, ACTIVATION_INDEX);
        item.source_ability_origin = Some(acquisition);
        self.open_payment_announcement(item, Counters::default(), None);
    }
    pub(super) fn create_embalmed_source_copy(&mut self, item: &SpellView) {
        let Some(source) = item.source else { return };
        let recipe = CopyRecipe::new(source.card).with_exceptions(CopyExceptions {
            embalm: true,
            ..CopyExceptions::NONE
        });
        // The created token is not cast; its entry X is zero, independent of the
        // X chosen to pay the graveyard activation's mana cost.
        self.stage_copy_creation(item.controller, recipe, 1);
    }
    pub(super) fn forget_embalm(&mut self, object: ObjectRef) {
        if let Some(rows) = self.state.embalm_abilities.as_mut() {
            rows.retain(|row| row.object != object);
        }
        if self
            .state
            .embalm_abilities
            .as_ref()
            .is_some_and(|rows| rows.is_empty())
        {
            self.state.embalm_abilities = None;
        }
    }
    pub(super) fn expire_embalm(&mut self) {
        if let Some(rows) = self.state.embalm_abilities.as_mut() {
            rows.retain(|row| row.expires_turn > self.state.turn);
        }
        if self
            .state
            .embalm_abilities
            .as_ref()
            .is_some_and(|rows| rows.is_empty())
        {
            self.state.embalm_abilities = None;
        }
    }
    pub(super) fn embalm_views(&self) -> Option<Vec<EmbalmAbilityView>> {
        let rows = self
            .state
            .embalm_abilities
            .as_ref()?
            .iter()
            .filter(|row| {
                self.object_ref(row.object.card) == row.object
                    && self.state.objects[row.object.card as usize].zone == Zone::Graveyard
                    && row.expires_turn >= self.state.turn
            })
            .copied()
            .collect::<Vec<_>>();
        (!rows.is_empty()).then_some(rows)
    }
}
