package me.kirderf.aftiktuna.level;

import java.util.Optional;
import java.util.function.Function;

public record ObjectType(char symbol, String name) {
	public static final ObjectType AFTIK = new ObjectType('A', "Aftik");
	public static final ObjectType FUEL_CAN = new ObjectType('f', "Fuel can");
	public static final ObjectType DOOR = new ObjectType('^', "Door");
	
	public <T> Function<GameObject, Optional<T>> matchingAndMapped(Function<GameObject, Optional<T>> mapper) {
		return gameObject -> gameObject.getType() == ObjectType.this ? mapper.apply(gameObject) : Optional.empty();
	}
}
