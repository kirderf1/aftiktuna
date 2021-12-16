package me.kirderf.aftiktuna.location;

import me.kirderf.aftiktuna.object.Identifier;
import me.kirderf.aftiktuna.object.Item;
import me.kirderf.aftiktuna.object.entity.Creature;
import me.kirderf.aftiktuna.object.entity.Entity;
import me.kirderf.aftiktuna.object.type.CreatureType;
import me.kirderf.aftiktuna.object.type.ItemType;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Optional;
import java.util.stream.Stream;

public final class Area {
	private final Identifier id = Identifier.newId();
	private final String label;
	private final int length;
	private final List<GameObject> objects = new ArrayList<>();
	
	Area(String label, int length) {
		this.label = label;
		this.length = length;
	}
	
	public Identifier getId() {
		return id;
	}
	
	public int getLength() {
		return length;
	}
	
	public String getLabel() {
		return label;
	}
	
	public Position getPosAt(int coord) {
		return new Position(this, coord);
	}
	
	public void addItem(ItemType type, int coord) {
		addObject(new Item(type), coord);
	}
	
	public void addCreature(CreatureType type, int coord) {
		addObject(new Creature(type), coord);
	}
	
	public void addObject(GameObject object, int coord) {
		addObject(object, getPosAt(coord));
	}
	
	public void addObject(GameObject object, Position position) {
		if (position.area() == this) {
			object.setArea(position);
			objects.add(object);
		}
	}
	
	public void verifyValidPosition(int position) {
		if (position < 0 || position >= length)
			throw new IllegalArgumentException("Invalid position %d is not in range [0-%d)".formatted(position, length));
	}
	
	public Stream<GameObject> objectStream() {
		return objects.stream();
	}
	
	public Optional<GameObject> findBlockingInRange(Entity entity, int from, int to) {
		int start = Math.min(from, to), end = Math.max(from, to);
		return objects.stream().sorted(byProximity(from)).filter(object -> object.isBlocking(entity))
				.filter(object -> start <= object.getCoord() && object.getCoord() <= end).findFirst();
	}
	
	public void removeObject(GameObject object) {
		objects.remove(object);
	}
	
	public static Comparator<GameObject> byProximity(int position) {
		return Comparator.comparingInt(object -> Math.abs(position - object.getCoord()));
	}
}
