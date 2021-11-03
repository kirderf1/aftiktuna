package me.kirderf.aftiktuna.level.object.entity;

import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.level.Room;
import me.kirderf.aftiktuna.level.object.ObjectTypes;
import me.kirderf.aftiktuna.util.OptionalFunction;

import java.util.Collection;
import java.util.Collections;
import java.util.Optional;
import java.util.stream.Collectors;

public final class Creature extends Entity {
	public static final OptionalFunction<GameObject, Creature> CAST = OptionalFunction.cast(Creature.class);
	
	private final boolean isMoving;
	
	private Collection<Aftik> targets = Collections.emptyList();
	
	public Creature(boolean isMoving) {
		super(ObjectTypes.CREATURE, 5, 5);
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
	
	@Override
	public void prepare() {
		super.prepare();
		targets = getRoom().objectStream().flatMap(Aftik.CAST.toStream()).filter(Entity::isAlive).collect(Collectors.toList());
	}
	
	public Optional<AttackResult> doAction() {
		
		Optional<Aftik> target = targets.stream().filter(Entity::isAlive)
				.filter(aftik -> aftik.getRoom() == this.getRoom()).min(Room.byProximity(this.getCoord()));
		if(target.isPresent()) {
			Aftik aftik = target.get();
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