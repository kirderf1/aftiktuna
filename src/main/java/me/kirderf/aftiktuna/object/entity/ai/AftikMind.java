package me.kirderf.aftiktuna.object.entity.ai;

import me.kirderf.aftiktuna.Crew;
import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.location.Ship;
import me.kirderf.aftiktuna.object.Item;
import me.kirderf.aftiktuna.object.ObjectTypes;
import me.kirderf.aftiktuna.object.door.Door;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.print.ContextPrinter;

import java.util.List;
import java.util.Optional;

public final class AftikMind {
	private final Aftik aftik;
	private final Crew crew;
	
	private final List<Task> tasks;
	private boolean launchShip;
	private boolean takeItems;
	
	public AftikMind(Aftik aftik, Crew crew) {
		this.aftik = aftik;
		this.crew = crew;
		tasks = List.of(new FollowTask(aftik, crew), new ForceDoorTask(aftik),
				new WieldTask(aftik), new FightTask(aftik));
	}
	
	public boolean overridesPlayerInput() {
		return launchShip || takeItems;
	}
	
	public void observeEnteredDoor(Aftik aftik, Door door, EnterResult result) {
		tasks.forEach(task -> task.observeEnteredDoor(aftik, door, result));
	}
	
	public void setLaunchShip(ContextPrinter out) {
		launchShip = true;
		tryLaunchShip(out);
	}
	
	public void setTakeItems(ContextPrinter out) {
		takeItems = true;
		tryTakeItems(out);
	}
	
	public void performAction(ContextPrinter out) {
		if (launchShip) {
			tryLaunchShip(out);
		} else if (takeItems) {
			tryTakeItems(out);
		} else {
			for (Task task : tasks) {
				if (task.performAction(out))
					return;
			}
		}
	}
	
	private void tryTakeItems(ContextPrinter out) {
		Optional<Item> optionalItem = aftik.findNearestAccessible(Item.CAST, true);
		
		if (optionalItem.isPresent()) {
			Item item = optionalItem.get();
			
			aftik.moveAndTake(item, out);
			
			if (aftik.findNearestAccessible(Item.CAST, true).isEmpty())
				takeItems = false;
		} else {
			out.printFor(aftik, "There are no nearby items to take.%n");
			takeItems = false;
		}
	}
	
	private void tryLaunchShip(ContextPrinter out) {
		Ship ship = crew.getShip();
		if (aftik.getArea() != ship.getRoom()) {
			tryGoToShip(out);
		} else {
			ship.tryLaunchShip(aftik, out);
			launchShip = false;
		}
	}
	
	private void tryGoToShip(ContextPrinter out) {
		Optional<Door> optional = aftik.findNearest(Door.CAST.filter(ObjectTypes.SHIP_ENTRANCE::matching), true);
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