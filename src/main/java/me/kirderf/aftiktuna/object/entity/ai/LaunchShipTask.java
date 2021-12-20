package me.kirderf.aftiktuna.object.entity.ai;

import me.kirderf.aftiktuna.location.Ship;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.print.ActionPrinter;

/**
 * A task that has the character try to enter the ship, and when in the ship, launch it.
 * The task is cancelled if there is no accessible ship entrance in the area,
 * or if the move-and-enter action fails in some way.
 * The task is finished after attempting to launch the ship, independently of the result.
 */
public final class LaunchShipTask extends Task {
	private final Ship ship;
	
	public LaunchShipTask(Ship ship) {
		this.ship = ship;
	}
	
	@Override
	public Status prepare(Aftik aftik) {
		if (aftik.getArea() != ship.getRoom()) {
			if (MoveToAreaTask.findPathTowardsArea(aftik, ship.getRoom()).map(door -> !aftik.isAccessible(door.getPosition(), true)).orElse(true))
				return Status.REMOVE;
		}
		return Status.KEEP;
	}
	
	@Override
	public Status performAction(Aftik aftik, ActionPrinter out) {
		if (aftik.getArea() != ship.getRoom()) {
			return MoveToAreaTask.tryGoToArea(aftik, ship.getRoom().getId(), out);
		} else {
			ship.tryLaunchShip(aftik, out);
			return Status.REMOVE;
		}
	}
}