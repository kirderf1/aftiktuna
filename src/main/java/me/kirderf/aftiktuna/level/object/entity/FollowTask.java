package me.kirderf.aftiktuna.level.object.entity;

import me.kirderf.aftiktuna.ContextPrinter;
import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.level.object.door.Door;

public final class FollowTask extends Task {
	private final Aftik aftik;
	private FollowTarget followTarget;
	
	private static record FollowTarget(Door door, Aftik aftik) {}
	
	public FollowTask(Aftik aftik) {
		this.aftik = aftik;
	}
	
	@Override
	public boolean performAction(ContextPrinter out) {
		if (followTarget != null && followTarget.door.getRoom() == aftik.getRoom()) {
			Aftik.MoveAndEnterResult result = aftik.moveAndEnter(followTarget.door);
			
			if (result.success()) {
				out.printAt(aftik, "%s follows %s into the room.%n", aftik.getName(), followTarget.aftik.getName());
			}
			return true;
		} else
			return false;
	}
	
	@Override
	public void prepare() {
		followTarget = null;
	}
	
	@Override
	public void observeEnteredDoor(Aftik aftik, Door door, EnterResult result) {
		if (result.success()) {
			this.followTarget = new FollowTarget(door, aftik);
		}
	}
}
