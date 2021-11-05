package me.kirderf.aftiktuna.level.object.entity;

import me.kirderf.aftiktuna.action.EnterResult;
import me.kirderf.aftiktuna.action.ForceResult;
import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.level.Room;
import me.kirderf.aftiktuna.level.object.Item;
import me.kirderf.aftiktuna.level.object.ObjectType;
import me.kirderf.aftiktuna.level.object.ObjectTypes;
import me.kirderf.aftiktuna.level.object.WeaponType;
import me.kirderf.aftiktuna.level.object.door.Door;
import me.kirderf.aftiktuna.util.Either;
import me.kirderf.aftiktuna.util.OptionalFunction;

import java.util.*;

public final class Aftik extends Entity {
	public static final OptionalFunction<GameObject, Aftik> CAST = OptionalFunction.cast(Aftik.class);
	
	private final String name;
	
	private final List<ObjectType> inventory = new ArrayList<>();
	private WeaponType wielded = null;
	
	// The door that the player aftik entered at the same turn. Other aftiks may try to follow along
	private Door targetDoor;
	
	public Aftik(String name) {
		super(ObjectTypes.AFTIK, 10, 5);
		this.name = name;
	}
	
	public String getName() {
		return name;
	}
	
	@Override
	public boolean hasCustomName() {
		return true;
	}
	
	@Override
	public String getDisplayName(boolean definite, boolean capitalized) {
		return getName();
	}
	
	@Override
	protected int getAttackPower() {
		return wielded != null ? wielded.getDamageValue() : 2;
	}
	
	@Override
	public void prepare() {
		super.prepare();
		targetDoor = null;
	}
	
	public Optional<Entity.MoveFailure> moveAndTake(Item item) {
		Optional<Entity.MoveFailure> failure = tryMoveTo(item.getPosition());
		if (failure.isEmpty()) {
			item.remove();
			addItem(item.getType());
		}
		return failure;
	}
	
	public Optional<MoveFailure> moveAndWield(Item item, WeaponType type) {
		if (item.getType() != type)
			throw new IllegalArgumentException("Incorrect type given");
		
		Optional<Entity.MoveFailure> failure = tryMoveTo(item.getPosition());
		if (failure.isEmpty()) {
			item.remove();
			wield(type);
		}
		return failure;
	}
	
	public Either<EnterResult, MoveFailure> moveAndEnter(Door door) {
		Optional<Entity.MoveFailure> failure = tryMoveTo(door.getPosition());
		if (failure.isEmpty()) {
			EnterResult result = door.enter(this);
			return Either.left(result);
		} else
			return Either.right(failure.get());
	}
	
	public Either<ForceResult, MoveFailure> moveAndForce(Door door) {
		Optional<Entity.MoveFailure> failure = tryMoveTo(door.getPosition());
		if (failure.isEmpty()) {
			ForceResult result = door.force(this);
			return Either.left(result);
		} else
			return Either.right(failure.get());
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
	
	public void dropItems() {
		if (wielded != null) {
			getRoom().addObject(new Item(wielded), getPosition());
			wielded = null;
		}
		for (ObjectType item : inventory) {
			getRoom().addObject(new Item(item), getPosition());
		}
		inventory.clear();
	}
	
	public void observeEnteredDoor(Door door) {
		this.targetDoor = door;
	}
	
	public boolean tryFollow() {
		if (targetDoor != null && targetDoor.getRoom() == this.getRoom()) {
			Either<EnterResult, MoveFailure> result = moveAndEnter(targetDoor);
			
			return result.getLeft().map(EnterResult::success).orElse(false);
		}
		return false;
	}
	
	public <T> Optional<T> findNearest(OptionalFunction<GameObject, T> mapper) {
		return getRoom().objectStream().sorted(blockingComparator().thenComparing(Room.byProximity(getCoord())))
				.flatMap(mapper.toStream()).findFirst();
	}
	
	private Comparator<GameObject> blockingComparator() {
		return Comparator.comparing(value -> isAccessBlocked(value.getCoord()), Boolean::compareTo);
	}
}