package me.kirderf.aftiktuna.level.object.entity;

import me.kirderf.aftiktuna.ContextPrinter;
import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.level.Ship;
import me.kirderf.aftiktuna.level.object.ObjectTypes;
import me.kirderf.aftiktuna.level.object.door.Door;

import java.util.List;
import java.util.Optional;

public final class AftikMind {
	private final Aftik aftik;
	private final Ship ship;
	
	private final List<Task> tasks;
	// The door that the player aftik entered at the same turn. Other aftiks may try to follow along
	private FollowTarget followTarget;
	private Door forceTarget;
	private boolean launchShip;
	
	private static record FollowTarget(Door door, Aftik aftik) {}
	
	public AftikMind(Aftik aftik, Ship ship) {
		this.aftik = aftik;
		this.ship = ship;
		tasks = List.of(new WieldTask(aftik), new FightTask(aftik));
	}
	
	public boolean overridesPlayerInput() {
		return launchShip;
	}
	
	public void observeEnteredDoor(Aftik aftik, Door door, EnterResult result) {
		if (result.success()) {
			this.followTarget = new FollowTarget(door, aftik);
		}
		result.either().getRight().ifPresent(failureType -> {
			if (canForceDoor(failureType))
				forceTarget = door;
		});
	}
	
	public void setLaunchShip(ContextPrinter out) {
		launchShip = true;
		tryLaunchShip(out);
	}
	
	void prepare() {
		followTarget = null;
	}
	
	void performAction(ContextPrinter out) {
		
		if (launchShip) {
			tryLaunchShip(out);
		} else if (followTarget != null && followTarget.door.getRoom() == aftik.getRoom()) {
			Aftik.MoveAndEnterResult result = aftik.moveAndEnter(followTarget.door);
			
			if (result.success()) {
				out.printAt(aftik, "%s follows %s into the room.%n", aftik.getName(), followTarget.aftik.getName());
			}
		} else if (forceTarget != null) {
			aftik.moveAndForce(forceTarget, out);
			forceTarget = null;
		} else {
			for (Task task : tasks) {
				if (task.performAction(out))
					return;
			}
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
	
	private boolean canForceDoor(EnterResult.FailureType type) {
		if (type == EnterResult.FailureType.STUCK && aftik.hasItem(ObjectTypes.CROWBAR))
			return true;
		else
			return aftik.hasItem(ObjectTypes.BLOWTORCH);
	}
}