package me.kirderf.aftiktuna.object.entity.ai;

import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.object.door.Door;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.print.ActionPrinter;

/**
 * A task where the character follows the controlled character through doors.
 * If the door cannot be entered, an attempt might be made to force the door.
 */
public final class FollowTask extends Task {
	private final Aftik aftik;
	
	private FollowTarget followTarget;
	private ForceDoorTaskFragment forceFragment;
	
	private static record FollowTarget(Door door, Aftik aftik) {}
	
	public FollowTask(Aftik aftik) {
		this.aftik = aftik;
	}
	
	@Override
	public boolean performAction(ActionPrinter out) {
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
		if (aftik == aftik.getCrew().getAftik() && result.success()) {
			this.followTarget = new FollowTarget(door, aftik);
		}
	}
}
