package me.kirderf.aftiktuna.level.object;

import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.level.Room;
import me.kirderf.aftiktuna.util.OptionalFunction;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Optional;
import java.util.stream.Collectors;

public final class Aftik extends Entity {
	public static final OptionalFunction<GameObject, Aftik> CAST = OptionalFunction.cast(Aftik.class);
	
	private final List<ObjectType> inventory = new ArrayList<>();
	private WeaponType wielded = null;
	
	public Aftik() {
		super(ObjectTypes.AFTIK, 10, 5);
	}
	
	@Override
	protected int getAttackPower() {
		return wielded != null ? wielded.getDamageValue() : 2;
	}
	
	public void addItem(ObjectType type) {
		inventory.add(type);
	}
	
	public boolean hasItem(ObjectType type) {
		return type != null && (wielded == type || inventory.contains(type));
	}
	
	public boolean wieldFromInventory(WeaponType type) {
		if (inventory.remove(type)) {
			wield(type);
			return true;
		} else {
			return false;
		}
	}
	
	public void wield(WeaponType item) {
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
	
	public <T> Optional<T> findNearest(OptionalFunction<GameObject, T> mapper) {
		return getRoom().objectStream().sorted(blockingComparator().thenComparing(Room.byProximity(getCoord())))
				.flatMap(mapper.toStream()).findFirst();
	}
	
	private Comparator<GameObject> blockingComparator() {
		return Comparator.comparing(value -> isAccessBlocked(value.getCoord()), Boolean::compareTo);
	}
}