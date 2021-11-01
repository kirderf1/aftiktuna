package me.kirderf.aftiktuna.level.object;

import me.kirderf.aftiktuna.OptionalFunction;
import me.kirderf.aftiktuna.level.GameObject;

public class Creature extends GameObject {
	public static final OptionalFunction<GameObject, Creature> CAST = OptionalFunction.cast(Creature.class);
	
	private int health = 5;
	
	public Creature() {
		super(ObjectType.CREATURE, 5);
	}
	
	public AttackResult receiveAttack(int attackPower) {
		health -= attackPower;
		if (health <= 0) {
			remove();
			return new AttackResult(true);
		} else
			return new AttackResult(false);
	}
	
	public static record AttackResult(boolean death) {}
	
	@Override
	public boolean isBlocking() {
		return true;
	}
}