package me.kirderf.aftiktuna.object.entity;

import me.kirderf.aftiktuna.GameInstance;
import me.kirderf.aftiktuna.action.ActionHandler;
import me.kirderf.aftiktuna.action.result.AttackResult;
import me.kirderf.aftiktuna.location.Area;
import me.kirderf.aftiktuna.location.GameObject;
import me.kirderf.aftiktuna.location.Position;
import me.kirderf.aftiktuna.object.ObjectType;
import me.kirderf.aftiktuna.print.ActionPrinter;
import me.kirderf.aftiktuna.util.OptionalFunction;
import me.kirderf.aftiktuna.util.StreamUtils;

import java.util.Comparator;
import java.util.Optional;
import java.util.OptionalInt;
import java.util.function.Predicate;
import java.util.stream.Stream;

/**
 * Base class for entities that provide stats, health and combat mechanics,
 * without any behavior details.
 */
public abstract class Entity extends GameObject {
	public static final OptionalFunction<GameObject, Entity> CAST = OptionalFunction.cast(Entity.class);
	
	private final Stats stats;
	private float health;
	private int dodgingStamina;
	
	public Entity(ObjectType type, int weight, Stats stats) {
		super(type, weight);
		this.stats = stats;
		restoreStatus();
	}
	
	////// Extendable
	
	protected void onDeath() {}
	
	protected abstract OptionalInt getWeaponPower();
	
	public void prepare() {
		dodgingStamina = Math.min(dodgingStamina + 1, getMaxStamina());
	}
	
	public abstract void performAction(ActionPrinter out);
	
	////// Health and stats
	
	public final Stats getStats() {
		return stats;
	}
	
	public final int getMaxHealth() {
		return 4 + stats.endurance() * 2;
	}
	
	private int getMaxStamina() {
		return 4 + stats.endurance() * 2;
	}
	
	public final float getHealth() {
		return health;
	}
	
	public final void restoreHealth(float fractionRecovered) {
		health = Math.min(getMaxHealth(), health + getMaxHealth() * fractionRecovered);
	}
	
	public final void restoreStatus() {
		this.health = getMaxHealth();
		dodgingStamina = getMaxStamina();
	}
	
	public final boolean isDead() {
		return !isAlive();
	}
	
	public final boolean isAlive() {
		return health > 0;
	}
	
	////// Movement
	
	public final boolean tryMoveNextTo(Position pos, ActionPrinter out) {
		return tryMoveTo(pos.getPosTowards(this.getCoord()), out);
	}
	
	public final boolean tryMoveTo(Position pos, ActionPrinter out) {
		if (pos.area() != this.getArea())
			throw new IllegalArgumentException("Areas must be the same.");
		
		int coord = pos.coord();
		if(coord != this.getCoord()) {
			Optional<GameObject> blocking = findBlockingTo(coord);
			if(blocking.isEmpty()) {
				teleport(coord);
			}
			
			blocking.ifPresent(blockingObject ->
					out.printFor(this, ActionHandler.createBlockingMessage(blockingObject)));
			
			return blocking.isEmpty();
		} else
			return true;
	}
	
	public final boolean isAccessible(Position pos, boolean exactPos) {
		if (!exactPos)
			pos = pos.getPosTowards(this.getCoord());
		return findBlockingTo(pos.coord()).isEmpty();
	}
	
	public final Optional<GameObject> findBlockingTo(int coord) {
		if (coord != this.getCoord())
			return getArea().findBlockingInRange(this, getPosition().getPosTowards(coord).coord(), coord);
		else
			return Optional.empty();
	}
	
	/**
	 * A comparator that places accessible objects before inaccessible ones.
	 */
	protected final Comparator<GameObject> blockingComparator(boolean exactPos) {
		return Comparator.comparing(object -> !isAccessible(object.getPosition(), exactPos), Boolean::compareTo);
	}
	
	////// Combat
	
	public final void moveAndAttack(Entity target, ActionPrinter out) {
		boolean success = tryMoveNextTo(target.getPosition(), out);
		if (success) {
			attack(target, out);
		}
	}
	
	public final void attack(Entity target, ActionPrinter out) {
		if (!target.getPosition().isAdjacent(this.getPosition()))
			throw new IllegalStateException("Expected to be next to target when attacking.");
		
		AttackResult result = target.receiveAttack(getAttackPower(), stats.agility());
		
		Entity attacked = result.attacked();
		switch(result.type()) {
			case DIRECT_HIT -> out.printAt(this, ActionHandler.condition("%s got a direct hit on[ and killed] %s.", result.isKill()),
					getDisplayName(true, true), attacked.getDisplayName(true, false));
			case GRAZING_HIT -> out.printAt(this, ActionHandler.condition("%s's attack grazed[ and killed] %s.", result.isKill()),
					getDisplayName(true, true), attacked.getDisplayName(true, false));
			case DODGE -> out.printAt(this, "%s dodged %s's attack.",
					attacked.getDisplayName(true, true), getDisplayName(true, false));
		}
	}
	
	public final AttackResult receiveAttack(float attackPower, int hitAgility) {
		AttackResult.Type type = tryDodge(hitAgility);
		if (type == AttackResult.Type.GRAZING_HIT) {
			attackPower /= 2;
		}
		
		if (type != AttackResult.Type.DODGE) {
			health -= attackPower;
			if(this.isDead())
				this.onDeath();
		}
		return new AttackResult(this, type);
	}
	
	private float getStrengthModifier() {
		return 1/3F + stats.strength()/6F;
	}
	
	private float getAttackPower() {
		return getStrengthModifier() * getWeaponPower().orElse(2);
	}
	
	private AttackResult.Type tryDodge(int hitAgility) {
		if (dodgingStamina > 0) {
			float dodgeRating = getDodgeFactor(hitAgility) * getDodgeEndurance() - GameInstance.RANDOM.nextInt(20);
			dodgingStamina -= 3;
			
			if (dodgeRating > 5)
				return AttackResult.Type.DODGE;
			else if (dodgeRating > 0)
				return AttackResult.Type.GRAZING_HIT;
			else
				return AttackResult.Type.DIRECT_HIT;
		} else
			return AttackResult.Type.DIRECT_HIT;
	}
	
	private float getDodgeFactor(int hitAgility) {
		return 2*stats.agility() - hitAgility;
	}
	
	private float getDodgeEndurance() {
		return dodgingStamina / (float) getMaxStamina();
	}
	
	////// Utilities
	
	/**
	 * Finds the nearest object in the area that passes the optional function and that this entity can move to.
	 * @param exactPos if the entity need to be able to move to the exact position of the object.
	 */
	public final <T extends GameObject> Optional<T> findNearestAccessible(OptionalFunction<GameObject, T> mapper, boolean exactPos) {
		return findNearestAccessibleFrom(getArea().objectStream().flatMap(mapper.toStream()), exactPos);
	}
	
	/**
	 * Finds the nearest object in the stream that this entity can move to.
	 * Assumes that all objects in the stream is in the same area.
	 * @param exactPos if the entity need to be able to move to the exact position of the object.
	 */
	public final <T extends GameObject> Optional<T> findNearestAccessibleFrom(Stream<T> stream, boolean exactPos) {
		return StreamUtils.findRandomMin(stream.filter(object -> isAccessible(object.getPosition(), exactPos)),
				Area.byProximity(getCoord()));
	}
	
	/**
	 * Finds the nearest accessible object in the area that passes the optional function, or failing that,
	 * the nearest inaccessible object.
	 * @param exactPos if the entity need to be able to move to the exact position of the object for it to count as accessible.
	 */
	public final <T extends GameObject> Optional<T> findNearest(OptionalFunction<GameObject, T> mapper, boolean exactPos) {
		return StreamUtils.findRandomMin(getArea().objectStream().flatMap(mapper.toStream()),
				blockingComparator(exactPos).thenComparing(Area.byProximity(getCoord())));
	}
	
	public final boolean isAnyNearAccessible(Predicate<GameObject> predicate, boolean exactPos) {
		return isAnyNear(predicate.and(object -> isAccessible(object.getPosition(), exactPos)));
	}
	
	public final boolean isAnyNear(Predicate<GameObject> predicate) {
		return getArea().objectStream().anyMatch(predicate);
	}
}