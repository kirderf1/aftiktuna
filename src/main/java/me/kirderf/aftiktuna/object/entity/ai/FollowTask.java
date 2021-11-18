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
				(followTarget.door.getArea() != this.aftik.getArea() || followTarget.aftik.getArea() == this.aftik.getArea())) {
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
			
			Aftik.MoveAndEnterResult result = aftik.moveAndEnter(followTarget.door, followTarget.aftik, out);
			
			result.optional().flatMap(enterResult -> enterResult.either().getRight())
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
