package me.kirderf.aftiktuna.level.object;

import me.kirderf.aftiktuna.level.GameObject;

public class Creature extends GameObject {
	
	public Creature() {
		super(ObjectType.CREATURE, 5);
	}
	
	@Override
	public boolean isBlocking() {
		return true;
	}
}