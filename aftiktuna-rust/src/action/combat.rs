use crate::action::{self, Error};
use crate::core::name::NameWithAttribute;
use crate::core::position::{OccupiesSpace, Pos};
use crate::core::status::{Health, Stamina, Stats};
use crate::core::{self, inventory, item, position, status, Hostile};
use crate::game_loop::GameState;
use hecs::{Entity, EntityRef, World};
use rand::Rng;
use std::cmp::Ordering;

pub(super) fn attack(
    state: &mut GameState,
    attacker: Entity,
    targets: Vec<Entity>,
) -> action::Result {
    if targets.len() == 1 {
        return attack_single(state, attacker, targets[0]);
    }
    let pos = *state.world.get::<&Pos>(attacker).unwrap();

    let targets = targets
        .into_iter()
        .flat_map(|entity| {
            state
                .world
                .get::<&Pos>(entity)
                .ok()
                .map(|pos| (entity, *pos))
        })
        .filter(|(entity, other_pos)| {
            other_pos.is_in(pos.get_area()) && status::is_alive(*entity, &state.world)
        })
        // collects the closest targets and also maps them to just the entity in one
        .fold((usize::MAX, vec![]), |mut partial, (entity, other_pos)| {
            let distance = other_pos.distance_to(pos);
            match distance.cmp(&partial.0) {
                Ordering::Less => (distance, vec![entity]),
                Ordering::Equal => {
                    partial.1.push(entity);
                    partial
                }
                Ordering::Greater => partial,
            }
        })
        .1;

    if targets.is_empty() {
        action::silent_ok()
    } else {
        let target = targets[state.rng.gen_range(0..targets.len())];
        attack_single(state, attacker, target)
    }
}

fn attack_single(state: &mut GameState, attacker: Entity, target: Entity) -> action::Result {
    let world = &mut state.world;
    let attacker_name = NameWithAttribute::lookup(attacker, world).definite();
    let target_name = NameWithAttribute::lookup(target, world).definite();
    let attacker_pos = *world
        .get::<&Pos>(attacker)
        .expect("Expected attacker to have a position");

    if !status::is_alive(target, world) {
        return action::silent_ok();
    }
    let target_pos = *world
        .get::<&Pos>(target)
        .map_err(|_| format!("{target_name} disappeared before {attacker_name} could attack.",))?;

    if attacker_pos.get_area() != target_pos.get_area() {
        return Err(Error::private(format!(
            "{target_name} left before {attacker_name} could attack."
        )));
    }

    core::trigger_aggression_in_area(world, attacker_pos.get_area());

    position::move_adjacent(world, attacker, target_pos)?;

    let hit_type = roll_hit(world, attacker, target, &mut state.rng);

    match hit_type {
        HitType::Dodge => action::ok(format!("{target_name} dodged {attacker_name}'s attack.")),
        HitType::GrazingHit => {
            let effect = perform_attack_hit(false, attacker, target, world, &mut state.rng);
            let effect = effect.descriptor();

            action::ok(format!(
                "{attacker_name}'s attack grazed {effect}{target_name}."
            ))
        }
        HitType::DirectHit => {
            let effect = perform_attack_hit(true, attacker, target, world, &mut state.rng);
            let effect = effect.descriptor();

            action::ok(format!(
                "{attacker_name} got a direct hit on {effect}{target_name}."
            ))
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum AttackEffect {
    None,
    Stunned,
    Killed,
}

impl AttackEffect {
    fn descriptor(self) -> &'static str {
        match self {
            AttackEffect::None => "",
            AttackEffect::Stunned => "and stunned ",
            AttackEffect::Killed => "and killed ",
        }
    }
}

fn perform_attack_hit(
    is_direct_hit: bool,
    attacker: Entity,
    target: Entity,
    world: &mut World,
    rng: &mut impl Rng,
) -> AttackEffect {
    let damage_factor = if is_direct_hit { 1.0 } else { 0.5 };

    let killed = deal_damage(
        world.entity(target).unwrap(),
        damage_factor * get_attack_damage(world, attacker),
    );

    if killed {
        let _ = world.remove_one::<OccupiesSpace>(target);
        let _ = world.remove_one::<Hostile>(target);
        return AttackEffect::Killed;
    }
    if is_direct_hit
        && !world.satisfies::<&status::IsStunned>(target).unwrap()
        && has_stun_attack_weapon(attacker, world)
    {
        let successful_stun = roll_stun(
            world.entity(attacker).unwrap(),
            world.entity(target).unwrap(),
            rng,
        );
        if successful_stun {
            world.insert_one(target, status::IsStunned).unwrap();
            return AttackEffect::Stunned;
        }
    }
    AttackEffect::None
}

fn deal_damage(target_ref: EntityRef, damage: f32) -> bool {
    target_ref
        .get::<&mut Health>()
        .map_or(false, |mut health| health.take_damage(damage, target_ref))
}

fn get_attack_damage(world: &World, attacker: Entity) -> f32 {
    let strength = world
        .get::<&Stats>(attacker)
        .expect("Expected attacker to have stats attached")
        .strength;
    let strength_mod = f32::from(strength + 2) / 6.0;
    core::get_wielded_weapon_modifier(world, attacker) * strength_mod
}

fn has_stun_attack_weapon(attacker: Entity, world: &World) -> bool {
    inventory::get_wielded(world, attacker)
        .and_then(|wielded| world.satisfies::<&item::StunAttack>(wielded).ok())
        .unwrap_or(false)
}

fn roll_hit(world: &mut World, attacker: Entity, target: Entity, rng: &mut impl Rng) -> HitType {
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
    let hit_difficulty = hit_difficulty.ceil() as i16;

    // Yes, this looks slightly odd. This is meant to act as a d20 integer roll,
    // which is converted to a float only to be compared against the float factor.
    let hit_roll = rng.gen_range::<i16, _>(1..=20);

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

fn roll_stun(attacker: EntityRef, target: EntityRef, rng: &mut impl Rng) -> bool {
    let attacker_strength = attacker
        .get::<&Stats>()
        .expect("Expected attacker to have stats attached")
        .strength;
    let target_endurance = target
        .get::<&Stats>()
        .expect("Expected target to have stats attached")
        .endurance;

    let stun_difficulty = 15 + 2 * (target_endurance - attacker_strength);
    let stun_roll = rng.gen_range::<i16, _>(1..=20);
    stun_roll >= stun_difficulty
}
