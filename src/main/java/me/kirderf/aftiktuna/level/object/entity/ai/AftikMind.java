package me.kirderf.aftiktuna.level.object.entity.ai;

import me.kirderf.aftiktuna.ContextPrinter;
import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.level.Ship;
import me.kirderf.aftiktuna.level.object.ObjectTypes;
import me.kirderf.aftiktuna.level.object.door.Door;
import me.kirderf.aftiktuna.level.object.entity.Aftik;

import java.util.List;
import java.util.Optional;

public final class AftikMind {
	private final Aftik aftik;
	private final Ship ship;
	
	private final List<Task> tasks;
	private boolean launchShip;
	
	public AftikMind(Aftik aftik, Ship ship) {
		this.aftik = aftik;
		this.ship = ship;
		tasks = List.of(new FollowTask(aftik), new ForceDoorTask(aftik),
				new WieldTask(aftik), new FightTask(aftik));
	}
	
	public boolean overridesPlayerInput() {
		return launchShip;
	}
	
	public void observeEnteredDoor(Aftik aftik, Door door, EnterResult result) {
		tasks.forEach(task -> task.observeEnteredDoor(aftik, door, result));
	}
	
	public void setLaunchShip(ContextPrinter out) {
		launchShip = true;
		tryLaunchShip(out);
	}
	
	public void prepare() {
		tasks.forEach(Task::prepare);
	}
	
	public void performAction(ContextPrinter out) {
		if (launchShip) {
			tryLaunchShip(out);
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
}