package me.kirderf.aftiktuna.level.object.entity;

import me.kirderf.aftiktuna.ContextPrinter;
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
	private FollowTarget followTarget;
	
	private static record FollowTarget(Door door, Aftik aftik) {}
	
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
		followTarget = null;
	}
	
	@Override
	public void performAction(ContextPrinter out) {
		
		if (followTarget != null && followTarget.door.getRoom() == this.getRoom()) {
			Either<EnterResult, MoveFailure> result = moveAndEnter(followTarget.door);
			
			if (result.getLeft().map(EnterResult::success).orElse(false)) {
				out.printf(this.getRoom(), "%s follows %s into the room.%n", this.getName(), followTarget.aftik.getName());
			}
		} else {
			Optional<WeaponType> weaponType = findWieldableItem();
			
			if (weaponType.isPresent()) {
				wieldFromInventory(weaponType.get(), out);
			}
		}
	}
	
	public Optional<MoveFailure> moveAndTake(Item item) {
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
	
	public Either<Boolean, MoveFailure> moveAndGive(Aftik aftik, ObjectType type) {
		Optional<Entity.MoveFailure> failure = tryMoveTo(aftik.getPosition());
		
		if (failure.isEmpty()) {
			
			if (this.removeItem(type)) {
				aftik.addItem(type);
				return Either.left(true);
			} else
				return Either.left(false);
		} else {
			return Either.right(failure.get());
		}
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
	
	public void wieldFromInventory(WeaponType type, ContextPrinter out) {
		if (type == wielded) {
			out.printf(this, "%s is already wielding a %s.%n", this.getName(), type.name());
		} else {
			if (inventory.remove(type)) {
				wield(type);
				out.printf(this.getRoom(), "%s wielded a %s.%n", this.getName(), type.name());
			} else {
				out.printf(this, "%s does not have a %s.%n", this.getName(), type.name());
			}
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
	
	public void observeEnteredDoor(Aftik aftik, Door door) {
		this.followTarget = new FollowTarget(door, aftik);
	}
	
	public <T> Optional<T> findNearest(OptionalFunction<GameObject, T> mapper) {
		return getRoom().objectStream().sorted(blockingComparator().thenComparing(Room.byProximity(getCoord())))
				.flatMap(mapper.toStream()).findFirst();
	}
	
	private Comparator<GameObject> blockingComparator() {
		return Comparator.comparing(value -> isAccessBlocked(value.getCoord()), Boolean::compareTo);
	}
	
	private Optional<WeaponType> findWieldableItem() {
		return inventory.stream().flatMap(OptionalFunction.cast(WeaponType.class).toStream())
				.max(Comparator.comparingInt(WeaponType::getDamageValue))
				.filter(type -> wielded == null || wielded.getDamageValue() < type.getDamageValue());
	}
}