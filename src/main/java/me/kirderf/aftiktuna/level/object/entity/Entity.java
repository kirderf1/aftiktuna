package me.kirderf.aftiktuna.level.object.entity;

import me.kirderf.aftiktuna.GameInstance;
import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.level.Position;
import me.kirderf.aftiktuna.level.object.ObjectType;
import me.kirderf.aftiktuna.util.Either;
import me.kirderf.aftiktuna.util.OptionalFunction;

import java.util.Optional;

public abstract class Entity extends GameObject {
	public static final OptionalFunction<GameObject, Entity> CAST = OptionalFunction.cast(Entity.class);
	
	private static final int STAMINA_MAX = 10;
	
	private final int maxHealth;
	private int health;
	private int dodgingStamina = STAMINA_MAX;
	
	public Entity(ObjectType type, int weight, int initialHealth) {
		super(type, weight);
		this.maxHealth = initialHealth;
		restoreHealth();
	}
	
	protected void onDeath() {}
	
	protected abstract int getAttackPower();
	
	public void prepare() {
		dodgingStamina = Math.min(dodgingStamina + 1, STAMINA_MAX);
	}
	
	public int getMaxHealth() {
		return maxHealth;
	}
	
	public final int getHealth() {
		return health;
	}
	
	public final void restoreHealth() {
		this.health = this.maxHealth;
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
	
	public final AttackResult attack(Entity target) {
		if (!target.getPosition().isAdjacent(this.getPosition()))
			throw new IllegalStateException("Expected to be next to target when attacking.");
		return target.receiveAttack(getAttackPower());
	}
	
	public final AttackResult receiveAttack(int attackPower) {
		AttackResult.Type type = tryDodge();
		if (type == AttackResult.Type.GRAZING_HIT) {
			attackPower /= 2;
			if (attackPower <= 0)
				type = AttackResult.Type.DODGE;
		}
		
		if (type != AttackResult.Type.DODGE) {
			health -= attackPower;
			if(this.isDead())
				this.onDeath();
		}
		return new AttackResult(this, type);
	}
	
	private AttackResult.Type tryDodge() {
		if (dodgingStamina > 0) {
			int dodgeRating = dodgingStamina - GameInstance.RANDOM.nextInt(20);
			dodgingStamina -= 2;
			
			if (dodgeRating > 5)
				return AttackResult.Type.DODGE;
			else if (dodgeRating > 0)
				return AttackResult.Type.GRAZING_HIT;
			else
				return AttackResult.Type.DIRECT_HIT;
		} else
			return AttackResult.Type.DIRECT_HIT;
	}
	
	public final MoveAndAttackResult moveAndAttack(Entity target) {
		Optional<MoveFailure> move = tryMoveNextTo(target.getPosition());
		if (move.isEmpty()) {
			AttackResult result = attack(target);
			return new MoveAndAttackResult(result);
		} else
			return new MoveAndAttackResult(move.get());
	}
	
	public static record MoveFailure(GameObject blocking) {
	}
	
	public static record MoveAndAttackResult(Either<AttackResult, MoveFailure> either) {
		public MoveAndAttackResult(AttackResult result) {
			this(Either.left(result));
		}
		
		public MoveAndAttackResult(MoveFailure result) {
			this(Either.right(result));
		}
		
		public boolean success() {
			return either.isLeft();
		}
	}
}