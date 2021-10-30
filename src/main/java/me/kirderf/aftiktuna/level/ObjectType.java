package me.kirderf.aftiktuna.level;

import java.util.Collection;
import java.util.List;
import java.util.Optional;
import java.util.function.Function;

public record ObjectType(char symbol, String name) {
	public static final ObjectType AFTIK = new ObjectType('A', "Aftik");
	public static final ObjectType FUEL_CAN = new ObjectType('f', "Fuel can");
	public static final ObjectType DOOR = new ObjectType('^', "Door");
	public static final ObjectType LEFT_DOOR = new ObjectType('<', "Left door");
	public static final ObjectType RIGHT_DOOR = new ObjectType('>', "Right door");
	
	public static final Collection<ObjectType> DOORS = List.of(DOOR, LEFT_DOOR, RIGHT_DOOR);
	
	public <T> Function<GameObject, Optional<T>> matchingAndMapped(Function<GameObject, Optional<T>> mapper) {
		return gameObject -> gameObject.getType() == ObjectType.this ? mapper.apply(gameObject) : Optional.empty();
	}
}
