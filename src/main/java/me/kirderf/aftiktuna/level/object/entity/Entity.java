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
	
	private static final int STAMINA_MAX = 5;
	
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
	
	private Optional<GameObject> findBlockingTo(int coord) {
		return getRoom().findBlockingInRange(this, getPosition().getPosTowards(coord).coord(), coord);
	}
	
	public final AttackResult attack(Entity target) {
		if (!target.getPosition().isAdjacent(this.getPosition()))
			throw new IllegalStateException("Expected to be next to target when attacking.");
		return target.receiveAttack(getAttackPower());
	}
	
	public final AttackResult receiveAttack(int attackPower) {
		if (tryDodge()) {
			return new AttackResult(this, AttackResult.Type.DODGE);
		} else {
			health -= attackPower;
			if(this.isDead())
				this.onDeath();
			return new AttackResult(this, isDead() ? AttackResult.Type.KILL : AttackResult.Type.HIT);
		}
	}
	
	private boolean tryDodge() {
		if (dodgingStamina > 0) {
			boolean dodged = GameInstance.RANDOM.nextInt(20) < dodgingStamina;
			dodgingStamina -= 2;
			return dodged;
		} else
			return false;
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