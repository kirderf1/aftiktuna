package me.kirderf.aftiktuna.object.entity.ai;

import me.kirderf.aftiktuna.action.EnterDoorAction;
import me.kirderf.aftiktuna.location.Ship;
import me.kirderf.aftiktuna.object.door.Door;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.print.ActionPrinter;

import java.util.Optional;

public final class EnterShipTask extends Task {
	private final Ship ship;
	
	public EnterShipTask(Ship ship) {
		this.ship = ship;
	}
	
	@Override
	public Status prepare(Aftik aftik) {
		if (aftik.getArea() != ship.getRoom()) {
			if (findPathTowardsShip(aftik, ship).map(door -> !aftik.isAccessible(door.getPosition(), true)).orElse(true))
				return Status.REMOVE;
			else
				return Status.KEEP;
		} else
			return Status.REMOVE;
	}
	
	@Override
	public Status performAction(Aftik aftik, ActionPrinter out) {
		if (aftik.getArea() != ship.getRoom()) {
			return tryGoToShip(aftik, ship, out);
		} else {
			return Status.REMOVE;
		}
	}
	
	static Status tryGoToShip(Aftik aftik, Ship ship, ActionPrinter out) {
		Optional<Door> optional = findPathTowardsShip(aftik, ship);
		if (optional.isPresent()) {
			Door door = optional.get();
			
			EnterDoorAction.Result result = EnterDoorAction.moveAndEnter(aftik, door, out);
			
			return result.success() ? Status.KEEP : Status.REMOVE;
		} else {
			out.printFor(aftik, "%s need to be near the ship in order to launch it.", aftik.getName());
			return Status.REMOVE;
		}
	}
	
	public static Optional<Door> findPathTowardsShip(Aftik aftik, Ship ship) {
		return aftik.getMind().getMemory().findDoorTowards(aftik.getArea(), ship.getRoom().getId());
	}
}