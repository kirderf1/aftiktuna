package me.kirderf.aftiktuna.object.entity.ai;

import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.object.door.Door;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.print.ActionPrinter;

import java.util.List;

public final class AftikMind {
	private final Aftik aftik;
	
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
	
	public void observeEnteredDoor(Aftik aftik, Door door, EnterResult result) {
		staticTasks.forEach(task -> task.observeEnteredDoor(aftik, door, result));
	}
	
	public void setLaunchShip(ActionPrinter out) {
		playerTask = new LaunchShipTask(aftik, aftik.getCrew().getShip());
		performPlayerAction(out);
	}
	
	public void setTakeItems(ActionPrinter out) {
		playerTask = new TakeItemsTask(aftik);
		performPlayerAction(out);
	}
	
	public void setRest(ActionPrinter out) {
		playerTask = new RestTask(aftik);
		performPlayerAction(out);
	}
	
	public void prepare() {
		if (playerTask != null) {
			Task.Status status = playerTask.prepare();
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
		Task.Status status = playerTask.performAction(out);
		if (status == Task.Status.REMOVE)
			playerTask = null;
	}
}