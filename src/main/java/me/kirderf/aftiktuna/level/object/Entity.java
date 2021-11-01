package me.kirderf.aftiktuna.level.object;

import me.kirderf.aftiktuna.level.GameObject;

public abstract class Entity extends GameObject {
	
	private int health;
	
	public Entity(ObjectType type, int weight, int initialHealth) {
		super(type, weight);
		this.health = initialHealth;
	}
	
	public boolean isDead() {
		return health <= 0;
	}
	
	public Creature.AttackResult receiveAttack(int attackPower) {
		health -= attackPower;
		if (this.isDead()) {
			remove();
			return new Creature.AttackResult(true);
		} else
			return new Creature.AttackResult(false);
	}
	
	public static record AttackResult(boolean death) {}
}