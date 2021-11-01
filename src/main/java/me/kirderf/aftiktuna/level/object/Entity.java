package me.kirderf.aftiktuna.level.object;

import me.kirderf.aftiktuna.level.GameObject;

public abstract class Entity extends GameObject {
	
	private int health;
	
	public Entity(ObjectType type, int weight, int initialHealth) {
		super(type, weight);
		this.health = initialHealth;
	}
	
	public final int getHealth() {
		return health;
	}
	
	public final boolean isDead() {
		return !isAlive();
	}
	
	public final boolean isAlive() {
		return health > 0;
	}
	
	public final Creature.AttackResult receiveAttack(int attackPower) {
		health -= attackPower;
		if (this.isDead())
			this.onDeath();
		return new AttackResult(isDead());
	}
	
	protected void onDeath() {}
	
	public static record AttackResult(boolean death) {}
}