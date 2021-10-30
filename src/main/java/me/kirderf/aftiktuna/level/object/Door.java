package me.kirderf.aftiktuna.level.object;

import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.level.ObjectType;
import me.kirderf.aftiktuna.level.Position;

import java.util.Optional;

public class Door extends GameObject {
	
	private final Position destination;
	
	public Door(Position destination) {
		super(ObjectType.DOOR, 20);
		this.destination = destination;
	}
	
	public Position getDestination() {
		return destination;
	}
	
	@Override
	public Optional<Door> getAsDoor() {
		return Optional.of(this);
	}
}