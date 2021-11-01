package me.kirderf.aftiktuna.level.object;

import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.util.OptionalFunction;

public final class Creature extends Entity {
	public static final OptionalFunction<GameObject, Creature> CAST = OptionalFunction.cast(Creature.class);
	
	private boolean isTargeting = false;
	
	public Creature() {
		super(ObjectType.CREATURE, 5, 5);
	}
	
	@Override
	public boolean isBlocking(Entity entity) {
		return entity instanceof Aftik;
	}
	
	public boolean isTargeting() {
		return isTargeting;
	}
	
	public void prepare() {
		isTargeting = getRoom().objectStream().flatMap(Aftik.CAST.toStream()).anyMatch(Entity::isAlive);
	}
	
	@Override
	protected void onDeath() {
		this.remove();
	}
}