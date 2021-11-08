package me.kirderf.aftiktuna.level.object.entity;

import me.kirderf.aftiktuna.ContextPrinter;
import me.kirderf.aftiktuna.level.Ship;
import me.kirderf.aftiktuna.level.object.ObjectTypes;
import me.kirderf.aftiktuna.level.object.WeaponType;
import me.kirderf.aftiktuna.level.object.door.Door;
import me.kirderf.aftiktuna.util.OptionalFunction;

import java.util.Comparator;
import java.util.Optional;

public final class AftikMind {
	private final Aftik aftik;
	private final Ship ship;
	
	// The door that the player aftik entered at the same turn. Other aftiks may try to follow along
	private FollowTarget followTarget;
	
	private boolean launchShip;
	
	private static record FollowTarget(Door door, Aftik aftik) {}
	
	public AftikMind(Aftik aftik, Ship ship) {
		this.aftik = aftik;
		this.ship = ship;
	}
	
	public boolean overridesPlayerInput() {
		return launchShip;
	}
	
	public void observeEnteredDoor(Aftik aftik, Door door) {
		this.followTarget = new FollowTarget(door, aftik);
	}
	
	public void setLaunchShip(ContextPrinter out) {
		launchShip = true;
		tryLaunchShip(out);
	}
	
	void prepare() {
		followTarget = null;
	}
	
	void performAction(ContextPrinter out) {
		Optional<WeaponType> weaponType = findWieldableInventoryItem();
		
		if (launchShip) {
			tryLaunchShip(out);
		} else if (followTarget != null && followTarget.door.getRoom() == aftik.getRoom()) {
			Aftik.MoveAndEnterResult result = aftik.moveAndEnter(followTarget.door);
			
			if (result.success()) {
				out.printAt(aftik, "%s follows %s into the room.%n", aftik.getName(), followTarget.aftik.getName());
			}
		} else if (weaponType.isPresent()) {
			aftik.wieldFromInventory(weaponType.get(), out);
		} else {
			Optional<Creature> target = aftik.findNearest(Creature.CAST);
			target.ifPresent(creature -> aftik.moveAndAttack(creature, out));
		}
	}
	
	private void tryLaunchShip(ContextPrinter out) {
		if (aftik.getRoom() != ship.getRoom()) {
			tryGoToShip(out);
		} else {
			ship.tryLaunchShip(aftik, out);
			launchShip = false;
		}
	}
	
	private void tryGoToShip(ContextPrinter out) {
		Optional<Door> optional = aftik.findNearest(Door.CAST.filter(ObjectTypes.SHIP_ENTRANCE::matching));
		if (optional.isPresent()) {
			Door door = optional.get();
			
			Aftik.MoveAndEnterResult result = aftik.moveEnterMain(door, out);
			
			if (!result.success())
				launchShip = false;
		} else {
			out.printFor(aftik, "%s need to be near the ship in order to launch it.%n", aftik.getName());
			launchShip = false;
		}
	}
	
	private Optional<WeaponType> findWieldableInventoryItem() {
		int currentWeaponValue = aftik.getWieldedItem().map(WeaponType::getDamageValue).orElse(0);
		return aftik.getInventory().stream().flatMap(OptionalFunction.cast(WeaponType.class).toStream())
				.max(Comparator.comparingInt(WeaponType::getDamageValue))
				.filter(type -> currentWeaponValue < type.getDamageValue());
	}
}