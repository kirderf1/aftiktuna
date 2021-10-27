package me.kirderf.aftiktuna;

import java.util.ArrayList;
import java.util.List;
import java.util.OptionalInt;

public final class Room {
	private final int length;
	private final List<PlacedObject> objects = new ArrayList<>();
	
	public Room(int length) {
		this.length = length;
	}
	
	public void addObject(GameObject object, int position) {
		if (position < 0 || position >= length)
			throw new IllegalArgumentException("Invalid position %d is not in range [0-%d)".formatted(position, length));
		objects.add(new PlacedObject(object, position));
	}
	
	public OptionalInt getPosition(GameObject object) {
		for (PlacedObject placed : objects) {
			if (placed.gameObj == object)
				return OptionalInt.of(placed.pos);
		}
		return OptionalInt.empty();
	}
	
	public void moveObject(GameObject object, int position) {
		removeObject(object);
		addObject(object, position);
	}
	
	public void removeObject(GameObject object) {
		objects.removeIf(placed -> placed.gameObj == object);
	}
	
	public void printRoom() {
		StringBuilder builder = new StringBuilder("_".repeat(length));
		for (PlacedObject object : objects)
			builder.setCharAt(object.pos, object.gameObj.getSymbol());
		System.out.println(builder);
		for (PlacedObject object : objects)
			System.out.println(object.gameObj.getSymbol() + ": " + object.gameObj.getName());
	}
	
	private record PlacedObject(GameObject gameObj, int pos) {}
}
