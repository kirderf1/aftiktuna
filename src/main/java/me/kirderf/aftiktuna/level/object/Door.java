package me.kirderf.aftiktuna.level.object;

import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.level.Position;

import java.util.Optional;
import java.util.concurrent.atomic.AtomicBoolean;

public class Door extends GameObject {
	
	private final Position destination;
	private final AtomicBoolean stuck;
	
	public Door(ObjectType type, Position destination, AtomicBoolean stuck) {
		super(type, 20);
		this.destination = destination;
		this.stuck = stuck;
	}
	
	public void enter(Aftik aftik) {
		if (isStuck() && !aftik.hasItem(ObjectType.CROWBAR)) {
			System.out.println("The door is stuck. You need a crowbar to force it open.");
		} else {
			if (isStuck()) {
				System.out.println("The door is stuck, so you use your crowbar to force it open.");
				stuck.set(false);
			}
			aftik.moveTo(destination);
			System.out.println("You entered the door into a new room.");
		}
	}
	
	private boolean isStuck() {
		return stuck != null && stuck.get();
	}
	
	@Override
	public Optional<Door> getAsDoor() {
		return Optional.of(this);
	}
}