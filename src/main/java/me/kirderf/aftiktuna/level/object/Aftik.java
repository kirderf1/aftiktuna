package me.kirderf.aftiktuna.level.object;

import me.kirderf.aftiktuna.OptionalFunction;
import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.level.Room;

import java.util.*;
import java.util.stream.Collectors;

public class Aftik extends GameObject {
	private final List<ObjectType> inventory = new ArrayList<>();
	
	public Aftik() {
		super(ObjectType.AFTIK, 10);
	}
	
	public void addItem(ObjectType type) {
		inventory.add(type);
	}
	
	public boolean hasItem(ObjectType type) {
		return inventory.contains(type);
	}
	
	public void optionallyPrintInventory() {
		if (!inventory.isEmpty()) {
			String itemList = inventory.stream().map(ObjectType::name).collect(Collectors.joining(", "));
			System.out.printf("Inventory: %s%n", itemList);
		}
	}
	
	public final boolean tryMoveTo(int coord) {
		if(coord != this.getCoord()) {
			Optional<GameObject> blocking = findBlockingTo(coord);
			if(blocking.isPresent()) {
				System.out.printf("The %s is blocking the way.%n", blocking.get().getType().name().toLowerCase(Locale.ROOT));
				return false;
			} else {
				teleport(coord);
				return true;
			}
		} else return true;
	}
	
	public final boolean isAccessBlocked(int coord) {
		return findBlockingTo(coord).isPresent();
	}
	
	protected final Optional<GameObject> findBlockingTo(int coord) {
		return getRoom().findBlockingInRange(getPosition().getPosTowards(coord).coord(), coord);
	}
	
	public final <T> Optional<T> findNearest(OptionalFunction<GameObject, T> mapper) {
		return getRoom().objectStream().sorted(blockingComparator().thenComparing(Room.byProximity(getCoord())))
				.flatMap(mapper.toStream()).findFirst();
	}
	
	private Comparator<GameObject> blockingComparator() {
		return Comparator.comparing(value -> isAccessBlocked(value.getCoord()), Boolean::compareTo);
	}
}