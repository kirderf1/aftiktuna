package me.kirderf.aftiktuna.level.object;

import me.kirderf.aftiktuna.OptionalFunction;
import me.kirderf.aftiktuna.level.GameObject;

public final class Creature extends Entity {
	public static final OptionalFunction<GameObject, Creature> CAST = OptionalFunction.cast(Creature.class);
	
	public Creature() {
		super(ObjectType.CREATURE, 5, 5);
	}
	
	@Override
	public boolean isBlocking() {
		return true;
	}
}