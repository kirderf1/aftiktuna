package me.kirderf.aftiktuna.level.object;

import me.kirderf.aftiktuna.OptionalFunction;
import me.kirderf.aftiktuna.level.GameObject;

public class Creature extends GameObject {
	public static final OptionalFunction<GameObject, Creature> CAST = OptionalFunction.cast(Creature.class);
	
	public Creature() {
		super(ObjectType.CREATURE, 5);
	}
	
	@Override
	public boolean isBlocking() {
		return true;
	}
}