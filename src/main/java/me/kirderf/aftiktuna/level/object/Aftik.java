package me.kirderf.aftiktuna.level.object;

import me.kirderf.aftiktuna.OptionalFunction;
import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.level.Room;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Optional;
import java.util.stream.Collectors;

public final class Aftik extends Entity {
	private final List<ObjectType> inventory = new ArrayList<>();
	private ObjectType wielded = null;
	
	public Aftik() {
		super(ObjectType.AFTIK, 10, 5);
	}
	
	public int getAttackPower() {
		return wielded != null ? 3 : 2;
	}
	
	public void addItem(ObjectType type) {
		inventory.add(type);
	}
	
	public boolean hasItem(ObjectType type) {
		return type != null && (wielded == type || inventory.contains(type));
	}
	
	public boolean wieldFromInventory(ObjectType type) {
		if (inventory.remove(type)) {
			wield(type);
			return true;
		} else {
			return false;
		}
	}
	
	public void wield(ObjectType item) {
		if (wielded != null)
			addItem(wielded);
		wielded = item;
	}
	
	public void optionallyPrintInventory() {
		if (!inventory.isEmpty()) {
			String itemList = inventory.stream().map(ObjectType::name).collect(Collectors.joining(", "));
			System.out.printf("Inventory: %s%n", itemList);
		}
	}
	
	public void optionallyPrintWieldedItem() {
		if (wielded != null) {
			System.out.printf("Wielded: %s%n", wielded.name());
		}
	}
	
	public MoveResult tryMoveTo(int coord) {
		if(coord != this.getCoord()) {
			Optional<GameObject> blocking = findBlockingTo(coord);
			if(blocking.isPresent()) {
				return new MoveResult(blocking);
			} else {
				teleport(coord);
				return new MoveResult(Optional.empty());
			}
		} else
			return new MoveResult(Optional.empty());
	}
	
	public boolean isAccessBlocked(int coord) {
		return findBlockingTo(coord).isPresent();
	}
	
	private Optional<GameObject> findBlockingTo(int coord) {
		return getRoom().findBlockingInRange(getPosition().getPosTowards(coord).coord(), coord);
	}
	
	public <T> Optional<T> findNearest(OptionalFunction<GameObject, T> mapper) {
		return getRoom().objectStream().sorted(blockingComparator().thenComparing(Room.byProximity(getCoord())))
				.flatMap(mapper.toStream()).findFirst();
	}
	
	private Comparator<GameObject> blockingComparator() {
		return Comparator.comparing(value -> isAccessBlocked(value.getCoord()), Boolean::compareTo);
	}
	
	public static record MoveResult(Optional<GameObject> blocking) {
		public boolean success() {
			return blocking.isEmpty();
		}
	}
}