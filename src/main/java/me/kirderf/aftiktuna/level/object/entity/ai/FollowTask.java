package me.kirderf.aftiktuna.level.object.entity.ai;

import me.kirderf.aftiktuna.ContextPrinter;
import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.level.object.door.Door;
import me.kirderf.aftiktuna.level.object.entity.Aftik;

public final class FollowTask extends Task {
	private final Aftik aftik;
	private FollowTarget followTarget;
	private ForceDoorTaskFragment forceFragment;
	
	private static record FollowTarget(Door door, Aftik aftik) {}
	
	public FollowTask(Aftik aftik) {
		this.aftik = aftik;
	}
	
	@Override
	public boolean performAction(ContextPrinter out) {
		if (forceFragment != null) {
			if (forceFragment.performAction(aftik, out)) {
				forceFragment = null;
				return true;
			} else {
				forceFragment = null;
				followTarget = null;
				return false;
			}
		} else if (followTarget != null) {
			
			if (followTarget.door.getRoom() == aftik.getRoom()) {
				Aftik.MoveAndEnterResult result = aftik.moveAndEnter(followTarget.door);
				
				if (result.success()) {
					out.printAt(aftik, "%s follows %s into the room.%n", aftik.getName(), followTarget.aftik.getName());
				}
				
				result.either().getLeft().flatMap(enterResult -> enterResult.either().getRight())
						.ifPresentOrElse(failureType -> forceFragment = new ForceDoorTaskFragment(followTarget.door, failureType),
								() -> followTarget = null);
				
				return true;
			} else {
				followTarget = null;
				return false;
			}
		} else {
			return false;
		}
	}
	
	@Override
	public void observeEnteredDoor(Aftik aftik, Door door, EnterResult result) {
		if (result.success()) {
			this.followTarget = new FollowTarget(door, aftik);
		}
	}
}
