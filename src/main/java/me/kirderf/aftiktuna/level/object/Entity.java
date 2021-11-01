package me.kirderf.aftiktuna.level.object;

import me.kirderf.aftiktuna.level.GameObject;

import java.util.Optional;

public abstract class Entity extends GameObject {
	
	private int health;
	
	public Entity(ObjectType type, int weight, int initialHealth) {
		super(type, weight);
		this.health = initialHealth;
	}
	
	protected void onDeath() {}
	
	public final int getHealth() {
		return health;
	}
	
	public final boolean isDead() {
		return !isAlive();
	}
	
	public final boolean isAlive() {
		return health > 0;
	}
	
	public final MoveResult tryMoveTo(int coord) {
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
	
	public static record MoveResult(Optional<GameObject> blocking) {
		public boolean success() {
			return blocking.isEmpty();
		}
	}
	
	public static record AttackResult(Entity attacked, boolean death) {}
}