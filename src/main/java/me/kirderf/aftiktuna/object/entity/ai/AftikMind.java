package me.kirderf.aftiktuna.object.entity.ai;

import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.object.door.Door;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.print.ActionPrinter;

import java.util.List;

public final class AftikMind {
	private final Aftik aftik;
	
	private final List<Task> tasks;
	private Command command;
	
	public AftikMind(Aftik aftik) {
		this.aftik = aftik;
		tasks = List.of(new FollowTask(aftik), new ForceDoorTask(aftik),
				new WieldTask(aftik), new FightTask(aftik));
	}
	
	public boolean overridesPlayerInput() {
		return command != null;
	}
	
	public void observeEnteredDoor(Aftik aftik, Door door, EnterResult result) {
		tasks.forEach(task -> task.observeEnteredDoor(aftik, door, result));
	}
	
	public void setLaunchShip(ActionPrinter out) {
		command = new LaunchShipCommand(aftik, aftik.getCrew().getShip());
		performCommandAction(out);
	}
	
	public void setTakeItems(ActionPrinter out) {
		command = new TakeItemsCommand(aftik);
		performCommandAction(out);
	}
	
	public void setRest(ActionPrinter out) {
		command = new RestCommand(aftik);
		performCommandAction(out);
	}
	
	public void prepare() {
		if (command != null) {
			Command.Status status = command.prepare();
			if (status == Command.Status.REMOVE)
				command = null;
		}
	}
	
	public void performAction(ActionPrinter out) {
		if (command != null) {
			performCommandAction(out);
		} else {
			for (Task task : tasks) {
				if (task.performAction(out))
					return;
			}
		}
	}
	
	private void performCommandAction(ActionPrinter out) {
		Command.Status status = command.performAction(out);
		if (status == Command.Status.REMOVE)
			command = null;
	}
}