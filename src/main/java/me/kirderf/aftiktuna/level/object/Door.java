package me.kirderf.aftiktuna.level.object;

import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.level.Position;

import java.util.Optional;

public class Door extends GameObject {
	
	private final Position destination;
	private final boolean stuck;
	
	public Door(ObjectType type, Position destination, boolean stuck) {
		super(type, 20);
		this.destination = destination;
		this.stuck = stuck;
	}
	
	public void enter(Aftik aftik) {
		if (stuck && !aftik.hasItem(ObjectType.CROWBAR)) {
			System.out.println("The door is stuck. You need a crowbar to force it open.");
		} else {
			aftik.moveTo(destination);
			System.out.println("You entered the door into a new room.");
		}
	}
	
	@Override
	public Optional<Door> getAsDoor() {
		return Optional.of(this);
	}
}