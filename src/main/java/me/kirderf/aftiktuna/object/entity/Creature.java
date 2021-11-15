package me.kirderf.aftiktuna.object.entity;

import me.kirderf.aftiktuna.location.GameObject;
import me.kirderf.aftiktuna.object.CreatureType;
import me.kirderf.aftiktuna.object.ObjectTypes;
import me.kirderf.aftiktuna.print.ContextPrinter;
import me.kirderf.aftiktuna.util.OptionalFunction;

import java.util.Collection;
import java.util.Collections;
import java.util.Optional;
import java.util.OptionalInt;
import java.util.stream.Collectors;

public final class Creature extends Entity {
	public static final OptionalFunction<GameObject, Creature> CAST = OptionalFunction.cast(Creature.class);
	
	private final boolean isMoving;
	
	private Collection<Aftik> targets = Collections.emptyList();
	
	public Creature(CreatureType type) {
		this(type, true);
	}
	
	public Creature(CreatureType type, boolean isMoving) {
		super(type, 5, type.getStats());
		this.isMoving = isMoving;
		
		if (!ObjectTypes.CREATURES.contains(type))
			throw new IllegalArgumentException("Invalid creature type %s".formatted(type.name()));
	}
	
	@Override
	protected OptionalInt getWeaponPower() {
		return OptionalInt.empty();
	}
	
	@Override
	public boolean isBlocking(Entity entity) {
		return entity instanceof Aftik;
	}
	
	@Override
	public void prepare() {
		super.prepare();
		targets = getArea().objectStream().flatMap(Aftik.CAST.toStream()).filter(Entity::isAlive).collect(Collectors.toList());
	}
	
	@Override
	public void performAction(ContextPrinter out) {
		
		Optional<Aftik> target = findNearestAccessibleFrom(targets.stream().filter(Entity::isAlive)
				.filter(aftik -> aftik.getArea() == this.getArea()), false);
		if(target.isPresent()) {
			Aftik aftik = target.get();
			
			if (isMoving) {
				this.moveAndAttack(aftik, out);
			} else if (aftik.getPosition().isAdjacent(this.getPosition())) {
				this.attack(aftik, out);
			}
		}
	}
	
	@Override
	protected void onDeath() {
		this.remove();
	}
}