package me.kirderf.aftiktuna.object.entity.ai;

import me.kirderf.aftiktuna.Crew;
import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.object.door.Door;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.print.ContextPrinter;

public final class FollowTask extends Task {
	private final Aftik aftik;
	private final Crew crew;
	
	private FollowTarget followTarget;
	private ForceDoorTaskFragment forceFragment;
	
	private static record FollowTarget(Door door, Aftik aftik) {}
	
	public FollowTask(Aftik aftik, Crew crew) {
		this.aftik = aftik;
		this.crew = crew;
	}
	
	@Override
	public boolean performAction(ContextPrinter out) {
		if (followTarget != null &&
				(followTarget.door.getRoom() != this.aftik.getRoom() || followTarget.aftik.getRoom() == this.aftik.getRoom())) {
			followTarget = null;
			forceFragment = null;
			return false;
		}
		
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
			
			Aftik.MoveAndEnterResult result = aftik.moveAndEnter(followTarget.door);
			
			if (result.success()) {
				out.printAt(aftik, "%s follows %s into the room.%n", aftik.getName(), followTarget.aftik.getName());
			}
			
			result.either().getLeft().flatMap(enterResult -> enterResult.either().getRight())
					.ifPresentOrElse(failureType -> forceFragment = new ForceDoorTaskFragment(followTarget.door, failureType),
							() -> followTarget = null);
			
			return true;
		} else {
			return false;
		}
	}
	
	@Override
	public void observeEnteredDoor(Aftik aftik, Door door, EnterResult result) {
		if (aftik == crew.getAftik() && result.success()) {
			this.followTarget = new FollowTarget(door, aftik);
		}
	}
}
