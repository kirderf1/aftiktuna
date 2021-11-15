package me.kirderf.aftiktuna.object.entity;

import me.kirderf.aftiktuna.Crew;
import me.kirderf.aftiktuna.action.ActionHandler;
import me.kirderf.aftiktuna.action.DoorActions;
import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.action.result.ForceResult;
import me.kirderf.aftiktuna.location.GameObject;
import me.kirderf.aftiktuna.location.Room;
import me.kirderf.aftiktuna.object.Item;
import me.kirderf.aftiktuna.object.ObjectType;
import me.kirderf.aftiktuna.object.ObjectTypes;
import me.kirderf.aftiktuna.object.WeaponType;
import me.kirderf.aftiktuna.object.door.Door;
import me.kirderf.aftiktuna.object.entity.ai.AftikMind;
import me.kirderf.aftiktuna.print.ContextPrinter;
import me.kirderf.aftiktuna.util.Either;
import me.kirderf.aftiktuna.util.OptionalFunction;

import java.util.*;

import static me.kirderf.aftiktuna.action.ActionHandler.printMoveFailure;

public final class Aftik extends Entity {
	public static final OptionalFunction<GameObject, Aftik> CAST = OptionalFunction.cast(Aftik.class);
	
	private final AftikMind mind;
	private final String name;
	
	private final List<ObjectType> inventory = new ArrayList<>();
	private WeaponType wielded = null;
	
	public Aftik(String name, Stats stats, Crew crew) {
		super(ObjectTypes.AFTIK, 10, stats);
		this.name = name;
		mind = new AftikMind(this, crew);
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
	public char getDisplaySymbol() {
		return name.charAt(0);
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
	public void performAction(ContextPrinter out) {
		mind.performAction(out);
	}
	
	public void moveAndTake(Item item, ContextPrinter out) {
		Optional<Entity.MoveFailure> failure = tryMoveTo(item.getPosition());
		if (failure.isEmpty()) {
			item.remove();
			addItem(item.getType());
			
			out.printAt(this, "%s picked up the %s.%n", this.getName(), item.getType().name());
		} else {
			ActionHandler.printMoveFailure(out, this, failure.get());
		}
	}
	
	public void moveAndWield(Item item, WeaponType type, ContextPrinter out) {
		if (item.getType() != type)
			throw new IllegalArgumentException("Incorrect type given");
		
		Optional<Entity.MoveFailure> failure = tryMoveTo(item.getPosition());
		
		if (failure.isEmpty()) {
			item.remove();
			wield(type);
			
			out.printAt(this, "%s picked up and wielded the %s.%n", this.getName(), type.name());
		} else {
			ActionHandler.printMoveFailure(out, this, failure.get());
		}
	}
	
	public void moveAndGive(Aftik aftik, ObjectType type, ContextPrinter out) {
		Optional<Entity.MoveFailure> failure = tryMoveTo(aftik.getPosition());
		
		if (failure.isEmpty()) {
			
			if (this.removeItem(type)) {
				aftik.addItem(type);
				out.printAt(this, "%s gave %s a %s.%n", this.getName(), aftik.getName(), type.name());
			} else
				out.printFor(this, "%s does not have that item.%n", this.getName());
		} else {
			printMoveFailure(out, this, failure.get());
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
		
		result.either.getLeft().ifPresent(enterResult ->
				originalRoom.objectStream().flatMap(Aftik.CAST.toStream())
						.forEach(other -> other.getMind().observeEnteredDoor(this, door, enterResult)));
		
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
	
	public void moveAndForce(Door door, ContextPrinter out) {
		Optional<Entity.MoveFailure> failure = tryMoveTo(door.getPosition());
		if (failure.isEmpty()) {
			ForceResult result = door.force(this);
			DoorActions.printForceResult(out, this, result);
		} else
			ActionHandler.printMoveFailure(out, this, failure.get());
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
}