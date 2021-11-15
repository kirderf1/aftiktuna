package me.kirderf.aftiktuna.object.entity.ai;

import me.kirderf.aftiktuna.Crew;
import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.object.door.Door;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.print.ContextPrinter;

import java.util.List;

public final class AftikMind {
	private final Aftik aftik;
	private final Crew crew;
	
	private final List<Task> tasks;
	private Command command;
	
	public AftikMind(Aftik aftik, Crew crew) {
		this.aftik = aftik;
		this.crew = crew;
		tasks = List.of(new FollowTask(aftik, crew), new ForceDoorTask(aftik),
				new WieldTask(aftik), new FightTask(aftik));
	}
	
	public boolean overridesPlayerInput() {
		return command != null;
	}
	
	public void observeEnteredDoor(Aftik aftik, Door door, EnterResult result) {
		tasks.forEach(task -> task.observeEnteredDoor(aftik, door, result));
	}
	
	public void setLaunchShip(ContextPrinter out) {
		command = new LaunchShipCommand(aftik, crew.getShip());
		command.performAction(out);
	}
	
	public void setTakeItems(ContextPrinter out) {
		command = new TakeItemsCommand(aftik);
		command.performAction(out);
	}
	
	public void performAction(ContextPrinter out) {
		if (command != null) {
			boolean finished = command.performAction(out);
			if (finished)
				command = null;
		} else {
			for (Task task : tasks) {
				if (task.performAction(out))
					return;
			}
		}
	}
}