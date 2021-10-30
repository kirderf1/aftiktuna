package me.kirderf.aftiktuna.level;

import java.util.*;
import java.util.function.Function;
import java.util.function.Predicate;

public final class Room {
	final int length;
	private final List<GameObject> objects = new ArrayList<>();
	
	public Room(int length) {
		this.length = length;
	}
	
	public Position getPosAt(int coord) {
		return new Position(this, coord);
	}
	
	public void addObject(GameObject object, int coord) {
		addObject(object, getPosAt(coord));
	}
	
	public void addObject(GameObject object, Position position) {
		if (position.room() == this) {
			object.setRoom(position);
			objects.add(object);
		}
	}
	
	public void verifyValidPosition(int position) {
		if (position < 0 || position >= length)
			throw new IllegalArgumentException("Invalid position %d is not in range [0-%d)".formatted(position, length));
	}
	
	public Optional<GameObject> findNearest(Predicate<GameObject> condition, int position) {
		return objects.stream().filter(condition)
				.min(Comparator.comparingInt(object -> Math.abs(position - object.getCoord())));
	}
	
	public <T> Optional<T> findNearest(Function<GameObject, Optional<T>> mapper, int position) {
		return objects.stream().sorted(Comparator.comparingInt(object -> Math.abs(position - object.getCoord())))
				.flatMap(object -> mapper.apply(object).stream())
				.findFirst();
	}
	
	public void removeObject(GameObject object) {
		objects.remove(object);
	}
	
	public void printRoom() {
		List<List<GameObject>> objectsByPos = new ArrayList<>();
		for (int pos = 0; pos < length; pos++)
			objectsByPos.add(new ArrayList<>());
		for (GameObject object : objects)
			objectsByPos.get(object.getCoord()).add(object);
		for (List<GameObject> objectStack : objectsByPos)
			objectStack.sort(Comparator.comparingInt(GameObject::getWeight).reversed());
		
		int lines = Math.max(1, objectsByPos.stream().map(List::size).max(Integer::compare).orElse(0));
		
		for (int line = lines - 1; line >= 0; line--) {
			StringBuilder builder = new StringBuilder((line == 0 ? "_" : " ").repeat(length));
			for (int pos = 0; pos < length; pos++) {
				if (objectsByPos.get(pos).size() > line)
					builder.setCharAt(pos, objectsByPos.get(pos).get(line).getType().symbol());
			}
			System.out.println(builder);
		}
		
		Set<ObjectType> writtenChars = new HashSet<>();
		for (GameObject object : objects) {
			if (writtenChars.add(object.getType()))
				System.out.printf("%s: %s%n", object.getType().symbol(), object.getType().name());
		}
	}
}
