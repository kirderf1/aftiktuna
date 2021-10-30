package me.kirderf.aftiktuna.level.object;

import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.level.Position;

import java.util.Optional;

public class Door extends GameObject {
	
	private final Position destination;
	private final DoorStuckProperty property;
	
	public Door(ObjectType type, Position destination, DoorStuckProperty property) {
		super(type, 20);
		this.destination = destination;
		this.property = property;
	}
	
	public void enter(Aftik aftik) {
		if (property.checkEntry()) {
			aftik.moveTo(destination);
			System.out.println("You entered the door into a new room.");
		}
	}
	
	public void force(Aftik aftik) {
		property.tryForce(aftik.hasItem(ObjectType.CROWBAR));
	}
	
	@Override
	public Optional<Door> getAsDoor() {
		return Optional.of(this);
	}
}