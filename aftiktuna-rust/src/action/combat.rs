use crate::action::{self, Error};
use crate::core::name::{NameData, NameWithAttribute};
use crate::core::position::{OccupiesSpace, Placement, PlacementQuery, Pos};
use crate::core::status::{Health, Killed, Stamina, Stats};
use crate::core::{
    self, AttackKind, Hostile, RepeatingAction, Species, inventory, position, status,
};
use hecs::{Entity, EntityRef, World};
use rand::Rng;
use std::cmp::Ordering;

pub(super) fn attack(
    context: &mut action::Context,
    attacker: Entity,
    targets: Vec<Entity>,
    attack_kind: AttackKind,
) -> action::Result {
    if targets.len() == 1 {
        return attack_single(context, attacker, targets[0], attack_kind);
    }
    let world = &context.state.world;
    let pos = *world.get::<&Pos>(attacker).unwrap();

    let targets = targets
        .into_iter()
        .flat_map(|entity| {
            world
                .query_one::<PlacementQuery>(entity)
                .ok()
                .and_then(|mut query| query.get().map(|query| (entity, Placement::from(query))))
        })
        .filter(|(entity, other_placement)| {
            pos.is_in(other_placement.area()) && status::is_alive(*entity, world)
        })
        // collects the closest targets and also maps them to just the entity in one
        .fold(
            (u32::MAX, vec![]),
            |mut partial, (entity, other_placement)| {
                let distance = other_placement.distance_to(pos);
                match distance.cmp(&partial.0) {
                    Ordering::Less => (distance, vec![entity]),
                    Ordering::Equal => {
                        partial.1.push(entity);
                        partial
                    }
                    Ordering::Greater => partial,
                }
            },
        )
        .1;

    if targets.is_empty() {
        Ok(action::Success)
    } else {
        let target = targets[context.state.rng.random_range(0..targets.len())];
        attack_single(context, attacker, target, attack_kind)
    }
}

fn attack_single(
    context: &mut action::Context,
    attacker: Entity,
    target: Entity,
    attack_kind: AttackKind,
) -> action::Result {
    let world = &mut context.state.world;
    let attacker_pos = *world
        .get::<&Pos>(attacker)
        .expect("Expected attacker to have a position");

    if !status::is_alive(target, world) {
        return Ok(action::Success);
    }
    let target_placement = world
        .query_one_mut::<PlacementQuery>(target)
        .map(Placement::from)
        .map_err(|_| {
            format!(
                "{target_name} disappeared before {attacker_name} could attack.",
                attacker_name = NameWithAttribute::lookup(attacker, world).definite(),
                target_name = NameWithAttribute::lookup(target, world).definite()
            )
        })?;

    if !attacker_pos.is_in(target_placement.area()) {
        return Err(Error::private(format!(
            "{target_name} left before {attacker_name} could attack.",
            attacker_name = NameWithAttribute::lookup(attacker, world).definite(),
            target_name = NameWithAttribute::lookup(target, world).definite(),
        )));
    }

    context
        .view_context
        .capture_unseen_view(attacker_pos.get_area(), context.state);

    let world = &mut context.state.world;
    core::trigger_aggression_in_area(world, attacker_pos.get_area());

    position::move_adjacent_placement(world, attacker, target_placement)?;

    if attack_kind == AttackKind::Charged {
        world
            .insert_one(attacker, RepeatingAction::ChargedAttack(target))
            .unwrap();
        context.view_context.add_message_at(
            attacker_pos.get_area(),
            format!(
                "{attacker_name} readies a powerful attack.",
                attacker_name = NameWithAttribute::lookup(attacker, world).definite()
            ),
            context.state,
        );
        Ok(action::Success)
    } else {
        perform_attack(context, attacker, target, attack_kind)
    }
}

pub(super) fn charged_attack(
    context: &mut action::Context,
    attacker: Entity,
    target: Entity,
) -> action::Result {
    let world = &mut context.state.world;
    let attacker_pos = *world
        .get::<&Pos>(attacker)
        .expect("Expected attacker to have a position");

    if !status::is_alive(target, world) {
        return Ok(action::Success);
    }
    let target_placement = world
        .query_one_mut::<PlacementQuery>(target)
        .map(Placement::from)
        .map_err(|_| {
            format!(
                "{target_name} disappeared before {attacker_name} could attack.",
                attacker_name = NameWithAttribute::lookup(attacker, world).definite(),
                target_name = NameWithAttribute::lookup(target, world).definite()
            )
        })?;

    if !attacker_pos.is_in(target_placement.area()) {
        return Err(Error::private(format!(
            "{target_name} left before {attacker_name} could attack.",
            attacker_name = NameWithAttribute::lookup(attacker, world).definite(),
            target_name = NameWithAttribute::lookup(target, world).definite(),
        )));
    }

    context
        .view_context
        .capture_unseen_view(attacker_pos.get_area(), context.state);

    let world = &mut context.state.world;
    core::trigger_aggression_in_area(world, attacker_pos.get_area());

    position::move_adjacent_placement(world, attacker, target_placement)?;

    perform_attack(context, attacker, target, AttackKind::Charged)
}

fn perform_attack(
    context: &mut action::Context<'_>,
    attacker: Entity,
    target: Entity,
    attack_kind: AttackKind,
) -> Result<action::Success, Error> {
    let world = &mut context.state.world;
    let attacker_area = world.get::<&Pos>(attacker).unwrap().get_area();
    let attacker_name = NameWithAttribute::lookup(attacker, world).definite();
    let target_name = NameWithAttribute::lookup(target, world).definite();

    let attack_kind_text = match attack_kind {
        AttackKind::Light => "",
        AttackKind::Rash => "With uncontrolled force, ",
        AttackKind::Charged => "With power, ",
    };
    let (attack_text, hit_verb) = if let Some(weapon) = inventory::get_wielded(world, attacker) {
        let weapon_name = NameData::find(world, weapon);
        let weapon_name = weapon_name.base();
        (
            format!(
                "{attack_kind_text}{attacker_name} swings their {weapon_name} at {target_name}"
            ),
            "hits",
        )
    } else if let Ok(unarmed_type) = world
        .get::<&Species>(attacker)
        .map(|species| species.unarmed_type())
    {
        let attack_verb = unarmed_type.attack_verb();
        (
            format!("{attack_kind_text}{attacker_name} {attack_verb} {target_name}"),
            unarmed_type.hit_verb(),
        )
    } else {
        (
            format!("{attack_kind_text}{attacker_name} attacks {target_name}"),
            "hits",
        )
    };

    let hit_type = roll_hit(world, attacker, target, attack_kind, &mut context.state.rng);

    if attack_kind == AttackKind::Rash {
        world.insert_one(attacker, status::IsStunned).unwrap();
    }

    match hit_type {
        HitType::Dodge => context.view_context.add_message_at(
            attacker_area,
            format!("{attack_text}, but {target_name} dodges the attack."),
            context.state,
        ),
        HitType::GrazingHit => {
            let effect = perform_attack_hit(
                false,
                attacker,
                target,
                attack_kind,
                world,
                &mut context.state.rng,
            );
            let effect_text = effect
                .map(AttackEffect::verb)
                .map_or("".to_string(), |effect| format!(", {effect} {target_name}"));

            context.view_context.add_message_at(
                attacker_area,
                format!("{attack_text} and narrowly {hit_verb} them{effect_text}."),
                context.state,
            );
        }
        HitType::DirectHit => {
            let effect = perform_attack_hit(
                true,
                attacker,
                target,
                attack_kind,
                world,
                &mut context.state.rng,
            );
            let effect_text = effect
                .map(AttackEffect::verb)
                .map_or("".to_string(), |effect| format!(", {effect} {target_name}"));

            context.view_context.add_message_at(
                attacker_area,
                format!("{attack_text} and directly {hit_verb} them{effect_text}."),
                context.state,
            );
        }
    }

    context
        .view_context
        .make_noise_at(&[attacker_area], context.state);

    Ok(action::Success)
}

#[derive(Debug, Clone, Copy)]
enum AttackEffect {
    Stunned,
    Killed,
}

impl AttackEffect {
    fn verb(self) -> &'static str {
        match self {
            Self::Stunned => "stunning",
            Self::Killed => "killing",
        }
    }
}

fn perform_attack_hit(
    is_direct_hit: bool,
    attacker: Entity,
    target: Entity,
    attack_kind: AttackKind,
    world: &mut World,
    rng: &mut impl Rng,
) -> Option<AttackEffect> {
    let damage_factor = if is_direct_hit { 1.0 } else { 0.5 } * attack_kind.damage_modifier();

    let damage_result = deal_damage(
        world.entity(target).unwrap(),
        damage_factor * get_attack_damage(world, attacker),
    );

    if matches!(damage_result, Some(Killed)) {
        let _ = world.remove_one::<OccupiesSpace>(target);
        let _ = world.remove_one::<Hostile>(target);
        return Some(AttackEffect::Killed);
    }
    if is_direct_hit
        && !world.satisfies::<&status::IsStunned>(target).unwrap()
        && core::get_active_weapon_properties(world, attacker).stun_attack
    {
        let successful_stun = roll_stun(
            world.entity(attacker).unwrap(),
            world.entity(target).unwrap(),
            attack_kind,
            rng,
        );
        if successful_stun {
            world.insert_one(target, status::IsStunned).unwrap();
            return Some(AttackEffect::Stunned);
        }
    }
    None
}

fn deal_damage(target_ref: EntityRef, damage: f32) -> Option<Killed> {
    target_ref
        .get::<&mut Health>()
        .and_then(|mut health| health.take_damage(damage, target_ref))
}

fn get_attack_damage(world: &World, attacker: Entity) -> f32 {
    let strength = world
        .get::<&Stats>(attacker)
        .expect("Expected attacker to have stats attached")
        .strength;
    let strength_mod = f32::from(strength + 2) / 6.0;
    core::get_active_weapon_properties(world, attacker).damage_mod * strength_mod
}

fn roll_hit(
    world: &mut World,
    attacker: Entity,
    target: Entity,
    attack_kind: AttackKind,
    rng: &mut impl Rng,
) -> HitType {
    let attacker_stats = world.get::<&Stats>(attacker).unwrap();
    let target_ref = world.entity(target).unwrap();
    let target_stats = target_ref.get::<&Stats>().unwrap();
    let mut stamina = target_ref.get::<&mut Stamina>().unwrap();
    let stamina_factor = stamina.as_fraction();

    let mut hit_difficulty = f32::from(target_stats.luck);
    if stamina_factor > 0.0 {
        stamina.on_dodge_attempt();
        hit_difficulty +=
            2. * stamina_factor * f32::from(target_stats.agility_for_dodging(target_ref));
    }
    hit_difficulty -= f32::from(attacker_stats.agility) + 0.5 * f32::from(attacker_stats.luck);
    let hit_difficulty = hit_difficulty.ceil() as i16 - attack_kind.hit_modifier();

    // Yes, this looks slightly odd. This is meant to act as a d20 integer roll,
    // which is converted to a float only to be compared against the float factor.
    let hit_roll = rng.random_range::<i16, _>(1..=20);

    if hit_roll < hit_difficulty - 5 {
        HitType::Dodge
    } else if hit_roll < hit_difficulty {
        HitType::GrazingHit
    } else {
        HitType::DirectHit
    }
}

#[derive(Debug, Eq, PartialEq)]
enum HitType {
    DirectHit,
    GrazingHit,
    Dodge,
}

fn roll_stun(
    attacker: EntityRef,
    target: EntityRef,
    attack_kind: AttackKind,
    rng: &mut impl Rng,
) -> bool {
    let attacker_strength = attacker
        .get::<&Stats>()
        .expect("Expected attacker to have stats attached")
        .strength;
    let target_endurance = target
        .get::<&Stats>()
        .expect("Expected target to have stats attached")
        .endurance;

    let stun_difficulty =
        15 + 2 * (target_endurance - attacker_strength) - attack_kind.stun_modifier();
    let stun_roll = rng.random_range::<i16, _>(1..=20);
    stun_roll >= stun_difficulty
}
