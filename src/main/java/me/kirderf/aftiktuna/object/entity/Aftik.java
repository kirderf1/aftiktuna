package me.kirderf.aftiktuna.object.entity;

import me.kirderf.aftiktuna.Crew;
import me.kirderf.aftiktuna.location.GameObject;
import me.kirderf.aftiktuna.object.Item;
import me.kirderf.aftiktuna.object.entity.ai.AftikMind;
import me.kirderf.aftiktuna.object.type.ItemType;
import me.kirderf.aftiktuna.object.type.ObjectType;
import me.kirderf.aftiktuna.object.type.ObjectTypes;
import me.kirderf.aftiktuna.object.type.WeaponType;
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
	
	public int getItemCount(ItemType item) {
		int count = (int) inventory.stream().filter(invItem -> invItem == item).count();
		if (item != null && wielded == item)
			return count + 1;
		else
			return count;
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