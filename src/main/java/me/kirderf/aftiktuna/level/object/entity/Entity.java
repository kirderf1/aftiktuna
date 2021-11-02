package me.kirderf.aftiktuna.level.object.entity;

import me.kirderf.aftiktuna.GameInstance;
import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.level.Position;
import me.kirderf.aftiktuna.level.object.ObjectType;
import me.kirderf.aftiktuna.util.Either;

import java.util.Optional;

public abstract class Entity extends GameObject {
	
	private int health;
	
	public Entity(ObjectType type, int weight, int initialHealth) {
		super(type, weight);
		this.health = initialHealth;
	}
	
	protected void onDeath() {}
	
	protected abstract int getAttackPower();
	
	public final int getHealth() {
		return health;
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
		if (GameInstance.RANDOM.nextInt(4) == 0) {
			return new AttackResult(this, AttackResult.Type.DODGE);
		} else {
			health -= attackPower;
			if(this.isDead())
				this.onDeath();
			return new AttackResult(this, isDead() ? AttackResult.Type.KILL : AttackResult.Type.HIT);
		}
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