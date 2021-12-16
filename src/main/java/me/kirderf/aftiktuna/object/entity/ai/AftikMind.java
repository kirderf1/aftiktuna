package me.kirderf.aftiktuna.object.entity.ai;

import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.action.result.ForceResult;
import me.kirderf.aftiktuna.object.door.Door;
import me.kirderf.aftiktuna.object.door.DoorProperty;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.print.ActionPrinter;

import java.util.List;

public final class AftikMind {
	private final Aftik aftik;
	private final Memory memory = new Memory();
	
	private final List<StaticTask> staticTasks;
	private Task playerTask;
	
	public AftikMind(Aftik aftik) {
		this.aftik = aftik;
		staticTasks = List.of(new FollowTask(aftik), new ForceDoorTask(aftik),
				new WieldTask(aftik), new FightTask(aftik));
	}
	
	public boolean overridesPlayerInput() {
		return playerTask != null;
	}
	
	public Memory getMemory() {
		return memory;
	}
	
	public void observeEnteredDoor(Aftik aftik, Door door, EnterResult result) {
		memory.observeDoorProperty(door, result.property());
		
		staticTasks.forEach(task -> task.observeEnteredDoor(aftik, door, result));
	}
	
	public void observeForcedDoor(Door door, ForceResult result) {
		result.propertyResult().either().run(
				success -> memory.observeDoorProperty(door, success.newProperty()),
				status -> {
					if (status == DoorProperty.Status.NOT_STUCK)
						memory.observeDoorProperty(door, DoorProperty.EMPTY);
				});
	}
	
	public void setAndPerformPlayerTask(Task task, ActionPrinter out) {
		playerTask = task;
		performPlayerAction(out);
	}
	
	public void prepare() {
		if (playerTask != null) {
			Task.Status status = playerTask.prepare(aftik);
			if (status == Task.Status.REMOVE)
				playerTask = null;
		}
	}
	
	public void performAction(ActionPrinter out) {
		if (playerTask != null) {
			performPlayerAction(out);
		} else {
			for (StaticTask task : staticTasks) {
				if (task.performAction(out))
					return;
			}
		}
	}
	
	private void performPlayerAction(ActionPrinter out) {
		Task.Status status = playerTask.performAction(aftik, out);
		if (status == Task.Status.REMOVE)
			playerTask = null;
	}
}