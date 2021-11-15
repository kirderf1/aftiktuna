package me.kirderf.aftiktuna.object.entity;

import me.kirderf.aftiktuna.GameInstance;
import me.kirderf.aftiktuna.action.ActionHandler;
import me.kirderf.aftiktuna.action.result.AttackResult;
import me.kirderf.aftiktuna.location.GameObject;
import me.kirderf.aftiktuna.location.Position;
import me.kirderf.aftiktuna.object.ObjectType;
import me.kirderf.aftiktuna.print.ContextPrinter;
import me.kirderf.aftiktuna.util.OptionalFunction;

import java.util.Optional;
import java.util.OptionalInt;

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
	
	protected void onDeath() {}
	
	protected abstract OptionalInt getWeaponPower();
	
	public void prepare() {
		dodgingStamina = Math.min(dodgingStamina + 1, getMaxStamina());
	}
	
	public abstract void performAction(ContextPrinter out);
	
	public final int getMaxHealth() {
		return 4 + stats.endurance() * 2;
	}
	
	private int getMaxStamina() {
		return 4 + stats.endurance() * 2;
	}
	
	public final float getHealth() {
		return health;
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
	
	public final Optional<MoveFailure> tryMoveNextTo(Position pos) {
		return tryMoveTo(pos.getPosTowards(this.getCoord()));
	}
	
	public final Optional<MoveFailure> tryMoveTo(Position pos) {
		if (pos.room() != this.getRoom())
			throw new IllegalArgumentException("Rooms must be the same.");
		
		int coord = pos.coord();
		if(coord != this.getCoord()) {
			Optional<GameObject> blocking = findBlockingTo(coord);
			if(blocking.isEmpty()) {
				teleport(coord);
			}
			
			return blocking.map(MoveFailure::new);
		} else
			return Optional.empty();
	}
	
	public final boolean isAccessBlocked(int coord) {
		return findBlockingTo(coord).isPresent();
	}
	
	public final Optional<GameObject> findBlockingTo(int coord) {
		if (coord != this.getCoord())
			return getRoom().findBlockingInRange(this, getPosition().getPosTowards(coord).coord(), coord);
		else
			return Optional.empty();
	}
	
	public final void attack(Entity target, ContextPrinter out) {
		if (!target.getPosition().isAdjacent(this.getPosition()))
			throw new IllegalStateException("Expected to be next to target when attacking.");
		
		AttackResult result = target.receiveAttack(getAttackPower(), stats.agility());
		
		ActionHandler.printAttackAction(out, this, result);
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
	
	public final void moveAndAttack(Entity target, ContextPrinter out) {
		Optional<MoveFailure> move = tryMoveNextTo(target.getPosition());
		if (move.isEmpty()) {
			attack(target, out);
		} else
			ActionHandler.printMoveFailure(out, this, move.get());
	}
	
	public static record MoveFailure(GameObject blocking) {
	}
}