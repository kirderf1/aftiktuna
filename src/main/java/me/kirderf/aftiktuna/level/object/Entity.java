package me.kirderf.aftiktuna.level.object;

import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.level.Position;
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
	
	public final MoveResult tryMoveTo(Position pos) {
		if (pos.room() != this.getRoom())
			throw new IllegalArgumentException("Rooms must be the same.");
		int coord = pos.coord();
		if(coord != this.getCoord()) {
			Optional<GameObject> blocking = findBlockingTo(coord);
			if(blocking.isPresent()) {
				return new MoveResult(blocking);
			} else {
				teleport(coord);
				return new MoveResult(Optional.empty());
			}
		} else
			return new MoveResult(Optional.empty());
	}
	
	public final boolean isAccessBlocked(int coord) {
		return findBlockingTo(coord).isPresent();
	}
	
	private Optional<GameObject> findBlockingTo(int coord) {
		return getRoom().findBlockingInRange(this, getPosition().getPosTowards(coord).coord(), coord);
	}
	
	public final Creature.AttackResult receiveAttack(int attackPower) {
		health -= attackPower;
		if (this.isDead())
			this.onDeath();
		return new AttackResult(this, isDead());
	}
	
	public final MoveAndAttackResult moveAndAttack(Creature creature) {
		Entity.MoveResult move = this.tryMoveTo(creature.getPosition().getPosTowards(this.getCoord()));
		if (move.success()) {
			Entity.AttackResult result = creature.receiveAttack(this.getAttackPower());
			return new MoveAndAttackResult(result);
		} else
			return new MoveAndAttackResult(move);
	}
	
	public static record MoveResult(Optional<GameObject> blocking) {
		public boolean success() {
			return blocking.isEmpty();
		}
	}
	
	public static record AttackResult(Entity attacked, boolean death) {}
	
	public static record MoveAndAttackResult(Either<AttackResult, MoveResult> either) {
		public MoveAndAttackResult(AttackResult result) {
			this(Either.left(result));
		}
		
		public MoveAndAttackResult(MoveResult result) {
			this(Either.right(result));
		}
		
		public boolean success() {
			return either.isLeft();
		}
	}
}