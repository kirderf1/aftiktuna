package me.kirderf.aftiktuna.level.object;

import me.kirderf.aftiktuna.OptionalFunction;
import me.kirderf.aftiktuna.level.GameObject;

import java.util.Locale;

public class Creature extends GameObject {
	public static final OptionalFunction<GameObject, Creature> CAST = OptionalFunction.cast(Creature.class);
	
	private int health = 3;
	
	public Creature() {
		super(ObjectType.CREATURE, 5);
	}
	
	public void receiveAttack() {
		health--;
		if (health <= 0) {
			remove();
			System.out.printf("You attacked and killed the %s.%n", getType().name().toLowerCase(Locale.ROOT));
		} else System.out.printf("You attacked the %s.%n", getType().name().toLowerCase(Locale.ROOT));
	}
	
	@Override
	public boolean isBlocking() {
		return true;
	}
}