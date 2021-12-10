package me.kirderf.aftiktuna.object.entity;

import me.kirderf.aftiktuna.Crew;
import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.action.result.ForceResult;
import me.kirderf.aftiktuna.location.Area;
import me.kirderf.aftiktuna.location.GameObject;
import me.kirderf.aftiktuna.object.*;
import me.kirderf.aftiktuna.object.door.Door;
import me.kirderf.aftiktuna.object.door.DoorPair;
import me.kirderf.aftiktuna.object.entity.ai.AftikMind;
import me.kirderf.aftiktuna.print.ActionPrinter;
import me.kirderf.aftiktuna.util.OptionalFunction;

import java.util.*;

public final class Aftik extends Entity {
	public static final OptionalFunction<GameObject, Aftik> CAST = OptionalFunction.cast(Aftik.class);
	
	private final Crew crew;
	private final AftikMind mind;
	private final String name;
	
	private final List<ItemType> inventory = new ArrayList<>();
	private WeaponType wielded = null;
	
	public Aftik(String name, Stats stats, Crew crew) {
		super(ObjectTypes.AFTIK, 10, stats);
		this.crew = crew;
		this.name = name;
		mind = new AftikMind(this);
	}
	
	public Crew getCrew() {
		return crew;
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
	public void prepare() {
		super.prepare();
		mind.prepare();
	}
	
	@Override
	public void performAction(ActionPrinter out) {
		mind.performAction(out);
	}
	
	public void moveAndTake(Item item, ActionPrinter out) {
		boolean success = tryMoveTo(item.getPosition(), out);
		if (success) {
			item.remove();
			addItem(item.getType());
			
			out.printAt(this, "%s picked up the %s.", this.getName(), item.getType().name());
		}
	}
	
	public void moveAndWield(Item item, WeaponType type, ActionPrinter out) {
		if (item.getType() != type)
			throw new IllegalArgumentException("Incorrect type given");
		
		boolean success = tryMoveTo(item.getPosition(), out);
		
		if (success) {
			item.remove();
			wield(type);
			
			out.printAt(this, "%s picked up and wielded the %s.", this.getName(), type.name());
		}
	}
	
	public void moveAndGive(Aftik aftik, ItemType type, ActionPrinter out) {
		boolean success = tryMoveTo(aftik.getPosition(), out);
		
		if (success) {
			
			if (this.removeItem(type)) {
				aftik.addItem(type);
				out.printAt(this, "%s gave %s a %s.", this.getName(), aftik.getName(), type.name());
			} else
				out.printFor(this, "%s does not have that item.", this.getName());
		}
	}
	
	public MoveAndEnterResult moveAndEnter(Door door, ActionPrinter out) {
		return moveAndEnter(door, null, out);
	}
	
	public MoveAndEnterResult moveAndEnter(Door door, Aftik followTarget, ActionPrinter out) {
		boolean success = tryMoveTo(door.getPosition(), out);
		if (success) {
			Area originalArea = this.getArea();
			
			EnterResult result = door.enter(this);
			
			originalArea.objectStream().flatMap(Aftik.CAST.toStream())
					.forEach(other -> other.getMind().observeEnteredDoor(this, door, result));
			
			if (followTarget != null) {
				out.printAt(this, "%s follows %s into the area.", this.getName(), followTarget.getName());
			} else {
				printEnterResult(out, this, door, result);
			}
			
			return new MoveAndEnterResult(result);
		} else
			return new MoveAndEnterResult();
	}
	
	public record MoveAndEnterResult(Optional<EnterResult> optional) {
		public MoveAndEnterResult(EnterResult result) {
			this(Optional.of(result));
		}
		
		public MoveAndEnterResult() {
			this(Optional.empty());
		}
		
		public boolean success() {
			return optional.map(EnterResult::success).orElse(false);
		}
	}
	
	public void moveAndForce(Door door, ActionPrinter out) {
		boolean success = tryMoveTo(door.getPosition(), out);
		if (success) {
			ForceResult result = door.force(this);
			printForceResult(out, this, door, result);
		}
	}
	
	private static void printEnterResult(ActionPrinter out, Aftik aftik, Door door, EnterResult result) {
		result.either().run(success -> printEnterSuccess(out, aftik, door, success),
				failureType -> out.printFor(aftik, "The %s is %s.", door.getType().getCategoryName(), failureType.adjective()));
	}
	
	private static void printEnterSuccess(ActionPrinter out, Aftik aftik, Door door, EnterResult.Success result) {
		result.usedItem().ifPresentOrElse(
				item -> out.printFor(aftik, "Using their %s, %s entered the %s into a new area.", item.name(), aftik.getName(), door.getType().getCategoryName()),
				() -> out.printFor(aftik, "%s entered the %s into a new area.", aftik.getName(), door.getType().getCategoryName()));
	}
	
	private static void printForceResult(ActionPrinter out, Aftik aftik, Door door, ForceResult result) {
		result.propertyResult().either().run(success -> printForceSuccess(out, aftik, result.pair(), success), status -> printForceStatus(out, aftik, door, status));
	}
	
	private static void printForceSuccess(ActionPrinter out, Aftik aftik, DoorPair pair, ForceResult.Success result) {
		out.printAt(pair.targetDoor(), "%s used their %s and %s the %s.", aftik.getName(), result.item().name(), result.method().text(), pair.targetDoor().getType().getCategoryName());
		out.printAt(pair.destination(), "A %s was %s from the other side.", pair.targetDoor().getType().getCategoryName(), result.method().text());
	}
	
	private static void printForceStatus(ActionPrinter out, Aftik aftik, Door door, ForceResult.Status status) {
		switch(status) {
			case NOT_STUCK -> out.printFor(aftik, "The %s does not seem to be stuck.", door.getType().getCategoryName());
			case NEED_TOOL -> out.printFor(aftik, "%s need some sort of tool to force the %s open.", aftik.getName(), door.getType().getCategoryName());
			case NEED_BREAK_TOOL -> out.printFor(aftik, "%s need some sort of tool to break the %s open.", aftik.getName(), door.getType().getCategoryName());
		}
	}
	
	public void addItem(ItemType type) {
		inventory.add(type);
	}
	
	public boolean removeItem(ItemType type) {
		if (inventory.remove(type)) {
			return true;
		} else if (wielded == type) {
			wielded = null;
			return true;
		} else
			return false;
	}
	
	public boolean hasItem(ItemType type) {
		return type != null && (wielded == type || inventory.contains(type));
	}
	
	public void wieldFromInventory(WeaponType type, ActionPrinter out) {
		if (type == wielded) {
			out.printFor(this, "%s is already wielding a %s.", this.getName(), type.name());
		} else {
			if (inventory.remove(type)) {
				wield(type);
				out.printAt(this, "%s wielded a %s.", this.getName(), type.name());
			} else {
				out.printFor(this, "%s does not have a %s.", this.getName(), type.name());
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
			getArea().addItem(wielded, getCoord());
			wielded = null;
		}
		for (ItemType item : inventory) {
			getArea().addItem(item, getCoord());
		}
		inventory.clear();
	}
}