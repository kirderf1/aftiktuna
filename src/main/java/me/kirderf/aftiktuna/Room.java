package me.kirderf.aftiktuna;

import java.util.*;
import java.util.function.Predicate;

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
	
	public Optional<PlacedObject> findNearest(Predicate<GameObject> condition, int position) {
		return objects.stream().filter(placed -> condition.test(placed.gameObj))
				.min(Comparator.comparingInt(placed -> Math.abs(position - placed.pos)));
	}
	
	public void moveObject(GameObject object, int position) {
		removeObject(object);
		addObject(object, position);
	}
	
	public void removeObject(GameObject object) {
		objects.removeIf(placed -> placed.gameObj == object);
	}
	
	public void printRoom() {
		List<List<GameObject>> objectsByPos = new ArrayList<>();
		for (int pos = 0; pos < length; pos++)
			objectsByPos.add(new ArrayList<>());
		for (PlacedObject object : objects)
			objectsByPos.get(object.pos).add(object.gameObj);
		
		int lines = Math.max(1, objectsByPos.stream().map(List::size).max(Integer::compare).orElse(0));
		
		for (int line = lines - 1; line >= 0; line--) {
			StringBuilder builder = new StringBuilder((line == 0 ? "_" : " ").repeat(length));
			for (int pos = 0; pos < length; pos++) {
				if (objectsByPos.get(pos).size() > line)
					builder.setCharAt(pos, objectsByPos.get(pos).get(line).getSymbol());
			}
			System.out.println(builder);
		}
		
		Set<Character> writtenChars = new HashSet<>();
		for (PlacedObject object : objects) {
			if (writtenChars.add(object.gameObj.getSymbol()))
				System.out.println(object.gameObj.getSymbol() + ": " + object.gameObj.getName());
		}
	}
	
	public record PlacedObject(GameObject gameObj, int pos) {}
}
