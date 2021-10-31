package me.kirderf.aftiktuna.level.object.door;

import me.kirderf.aftiktuna.OptionalFunction;
import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.level.Position;
import me.kirderf.aftiktuna.level.object.Aftik;
import me.kirderf.aftiktuna.level.object.ObjectType;

import java.util.concurrent.atomic.AtomicReference;

public class Door extends GameObject {
	public static final OptionalFunction<GameObject, Door> CAST = OptionalFunction.cast(Door.class);
	
	private final Position destination;
	private final AtomicReference<DoorProperty> property;
	
	public Door(ObjectType type, Position destination, AtomicReference<DoorProperty> property) {
		super(type, 20);
		this.destination = destination;
		this.property = property;
	}
	
	public void enter(Aftik aftik) {
		if (property.get().checkEntry(aftik)) {
			aftik.teleport(destination);
		}
	}
	
	public void force(Aftik aftik) {
		property.set(property.get().tryForce(aftik));
	}
}