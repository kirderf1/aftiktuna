package me.kirderf.aftiktuna.level.object;

import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.util.OptionalFunction;

import java.util.Optional;

public final class Creature extends Entity {
	public static final OptionalFunction<GameObject, Creature> CAST = OptionalFunction.cast(Creature.class);
	
	private final boolean isMoving;
	
	private boolean isTargeting = false;
	
	public Creature(boolean isMoving) {
		super(ObjectType.CREATURE, 5, 5);
		this.isMoving = isMoving;
	}
	
	@Override
	protected int getAttackPower() {
		return 1;
	}
	
	@Override
	public boolean isBlocking(Entity entity) {
		return entity instanceof Aftik;
	}
	
	public void prepare() {
		isTargeting = getRoom().objectStream().flatMap(Aftik.CAST.toStream()).anyMatch(Entity::isAlive);
	}
	
	public Optional<AttackResult> doAction(Aftik aftik) {
		if(isTargeting && aftik.isAlive()) {
			if (isMoving) {
				tryMoveNextTo(aftik.getPosition());
			}
			if (aftik.getPosition().isAdjacent(this.getPosition())) {
				AttackResult result = attack(aftik);
				return Optional.of(result);
			}
		}
		return Optional.empty();
	}
	
	@Override
	protected void onDeath() {
		this.remove();
	}
}