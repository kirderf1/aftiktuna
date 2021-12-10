package me.kirderf.aftiktuna.object.entity.ai;

import me.kirderf.aftiktuna.action.EnterDoorAction;
import me.kirderf.aftiktuna.location.Ship;
import me.kirderf.aftiktuna.object.ObjectTypes;
import me.kirderf.aftiktuna.object.door.Door;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.print.ActionPrinter;

import java.util.Optional;

/**
 * A command that has the character try to enter the ship, and when in the ship, launch it.
 * The command is cancelled if there is no accessible ship entrance in the area,
 * or if the move-and-enter action fails in some way.
 * Command is finished after attempting to launch the ship, independently of the result.
 */
public final class LaunchShipTask extends Task {
	private final Aftik aftik;
	private final Ship ship;
	
	public LaunchShipTask(Aftik aftik, Ship ship) {
		this.aftik = aftik;
		this.ship = ship;
	}
	
	@Override
	public Status prepare() {
		if (aftik.getArea() != ship.getRoom()) {
			if (findPathTowardsShip().map(door -> !aftik.isAccessible(door.getPosition(), true)).orElse(true))
				return Status.REMOVE;
		}
		return Status.KEEP;
	}
	
	@Override
	public Status performAction(ActionPrinter out) {
		if (aftik.getArea() != ship.getRoom()) {
			return tryGoToShip(out);
		} else {
			ship.tryLaunchShip(aftik, out);
			return Status.REMOVE;
		}
	}
	
	private Status tryGoToShip(ActionPrinter out) {
		Optional<Door> optional = findPathTowardsShip();
		if (optional.isPresent()) {
			Door door = optional.get();
			
			EnterDoorAction.Result result = EnterDoorAction.moveAndEnter(aftik, door, out);
			
			return result.success() ? Status.KEEP : Status.REMOVE;
		} else {
			out.printFor(aftik, "%s need to be near the ship in order to launch it.", aftik.getName());
			return Status.REMOVE;
		}
	}
	
	private Optional<Door> findPathTowardsShip() {
		return aftik.findNearest(Door.CAST.filter(ObjectTypes.SHIP_ENTRANCE::matching), true);
	}
}