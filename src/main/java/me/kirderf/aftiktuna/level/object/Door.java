package me.kirderf.aftiktuna.level.object;

import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.level.Position;

import java.util.Optional;

public class Door extends GameObject {
	
	private final Position destination;
	
	public Door(ObjectType type, Position destination) {
		super(type, 20);
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