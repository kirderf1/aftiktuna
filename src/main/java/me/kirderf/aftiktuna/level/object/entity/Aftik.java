package me.kirderf.aftiktuna.level.object.entity;

import me.kirderf.aftiktuna.ContextPrinter;
import me.kirderf.aftiktuna.action.ActionHandler;
import me.kirderf.aftiktuna.action.DoorActions;
import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.action.result.ForceResult;
import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.level.Room;
import me.kirderf.aftiktuna.level.Ship;
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
	
	private final AftikMind mind;
	private final String name;
	
	private final List<ObjectType> inventory = new ArrayList<>();
	private WeaponType wielded = null;
	
	public Aftik(String name, Stats stats, Ship ship) {
		super(ObjectTypes.AFTIK, 10, stats);
		this.name = name;
		mind = new AftikMind(this, ship);
	}
	
	public String getName() {
		return name;
	}
	
	public AftikMind getMind() {
		return mind;
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
	protected OptionalInt getWeaponPower() {
		return wielded != null ? OptionalInt.of(wielded.getDamageValue()) : OptionalInt.empty();
	}
	
	@Override
	public void prepare() {
		super.prepare();
		mind.prepare();
	}
	
	@Override
	public void performAction(ContextPrinter out) {
		mind.performAction(out);
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
	
	public MoveAndEnterResult moveAndEnter(Door door) {
		Optional<Entity.MoveFailure> failure = tryMoveTo(door.getPosition());
		if (failure.isEmpty()) {
			EnterResult result = door.enter(this);
			return new MoveAndEnterResult(result);
		} else
			return new MoveAndEnterResult(failure.get());
	}
	
	public MoveAndEnterResult moveEnterMain(Door door, ContextPrinter out) {
		Room originalRoom = this.getRoom();
		
		MoveAndEnterResult result = moveAndEnter(door);
		
		if (result.success()) {
			originalRoom.objectStream().flatMap(Aftik.CAST.toStream()).forEach(other -> other.getMind().observeEnteredDoor(this, door));
		}
		
		result.either().run(enterResult -> DoorActions.printEnterResult(out, this, enterResult),
				moveFailure -> ActionHandler.printMoveFailure(out, this, moveFailure));
		
		return result;
	}
	
	public static record MoveAndEnterResult(Either<EnterResult, MoveFailure> either) {
		public MoveAndEnterResult(EnterResult result) {
			this(Either.left(result));
		}
		
		public MoveAndEnterResult(MoveFailure failure) {
			this(Either.right(failure));
		}
		
		public boolean success() {
			return either.getLeft().map(EnterResult::success).orElse(false);
		}
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
			out.printFor(this, "%s is already wielding a %s.%n", this.getName(), type.name());
		} else {
			if (inventory.remove(type)) {
				wield(type);
				out.printAt(this, "%s wielded a %s.%n", this.getName(), type.name());
			} else {
				out.printFor(this, "%s does not have a %s.%n", this.getName(), type.name());
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
	
	public <T> Optional<T> findNearest(OptionalFunction<GameObject, T> mapper) {
		return getRoom().objectStream().sorted(blockingComparator().thenComparing(Room.byProximity(getCoord())))
				.flatMap(mapper.toStream()).findFirst();
	}
	
	private Comparator<GameObject> blockingComparator() {
		return Comparator.comparing(value -> isAccessBlocked(value.getCoord()), Boolean::compareTo);
	}
}