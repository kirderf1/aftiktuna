package me.kirderf.aftiktuna.level.object.entity;

import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.level.Room;
import me.kirderf.aftiktuna.level.object.ObjectType;
import me.kirderf.aftiktuna.level.object.ObjectTypes;
import me.kirderf.aftiktuna.level.object.WeaponType;
import me.kirderf.aftiktuna.util.OptionalFunction;

import java.util.*;

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
	
	public boolean removeItem(ObjectType type) {
		if (inventory.remove(type)) {
			return true;
		} else if (wielded == type) {
			wielded = null;
			return true;
		} else
			return false;
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
	
	public Optional<WeaponType> getWieldedItem() {
		return Optional.ofNullable(wielded);
	}
	
	public List<ObjectType> getInventory() {
		return Collections.unmodifiableList(inventory);
	}
	
	public <T> Optional<T> findNearest(OptionalFunction<GameObject, T> mapper) {
		return getRoom().objectStream().sorted(blockingComparator().thenComparing(Room.byProximity(getCoord())))
				.flatMap(mapper.toStream()).findFirst();
	}
	
	private Comparator<GameObject> blockingComparator() {
		return Comparator.comparing(value -> isAccessBlocked(value.getCoord()), Boolean::compareTo);
	}
}