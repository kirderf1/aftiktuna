package me.kirderf.aftiktuna.level.object.door;

import me.kirderf.aftiktuna.OptionalFunction;
import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.level.Position;
import me.kirderf.aftiktuna.level.object.Aftik;
import me.kirderf.aftiktuna.level.object.ObjectType;

import java.util.Locale;
import java.util.Optional;
import java.util.concurrent.atomic.AtomicReference;

public final class Door extends GameObject {
	public static final OptionalFunction<GameObject, Door> CAST = OptionalFunction.cast(Door.class);
	
	private final Position destination;
	private final AtomicReference<DoorProperty> property;
	
	public Door(ObjectType type, Position destination, AtomicReference<DoorProperty> property) {
		super(type, 20);
		this.destination = destination;
		this.property = property;
	}
	
	public void enter(Aftik aftik) {
		Optional<DoorProperty.EnterResult> optionalResult = property.get().checkEntry(aftik);
		optionalResult.ifPresent(result -> {
			result.usedItem().ifPresentOrElse(
					item -> System.out.printf("Using your %s, you entered the door into a new room.%n", item.name().toLowerCase(Locale.ROOT)),
					() -> System.out.printf("You entered the door into a new room.%n"));
			aftik.teleport(destination);
		});
	}
	
	public void force(Aftik aftik) {
		property.set(property.get().tryForce(aftik));
	}
}