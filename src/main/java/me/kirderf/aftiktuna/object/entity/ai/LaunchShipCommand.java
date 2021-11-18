package me.kirderf.aftiktuna.object.entity.ai;

import me.kirderf.aftiktuna.location.Ship;
import me.kirderf.aftiktuna.object.ObjectTypes;
import me.kirderf.aftiktuna.object.door.Door;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.print.ContextPrinter;

import java.util.Optional;

public final class LaunchShipCommand extends Command {
	private final Aftik aftik;
	private final Ship ship;
	
	public LaunchShipCommand(Aftik aftik, Ship ship) {
		this.aftik = aftik;
		this.ship = ship;
	}
	
	@Override
	public boolean performAction(ContextPrinter out) {
		if (aftik.getArea() != ship.getRoom()) {
			return tryGoToShip(out);
		} else {
			ship.tryLaunchShip(aftik, out);
			return true;
		}
	}
	
	private boolean tryGoToShip(ContextPrinter out) {
		Optional<Door> optional = aftik.findNearest(Door.CAST.filter(ObjectTypes.SHIP_ENTRANCE::matching), true);
		if (optional.isPresent()) {
			Door door = optional.get();
			
			Aftik.MoveAndEnterResult result = aftik.moveAndEnter(door, out);
			
			return !result.success();
		} else {
			out.printFor(aftik, "%s need to be near the ship in order to launch it.", aftik.getName());
			return true;
		}
	}
}