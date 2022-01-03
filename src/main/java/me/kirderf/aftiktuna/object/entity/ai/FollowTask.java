package me.kirderf.aftiktuna.object.entity.ai;

import me.kirderf.aftiktuna.action.EnterDoorAction;
import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.object.Reference;
import me.kirderf.aftiktuna.object.door.Door;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.print.ActionPrinter;

/**
 * A task where the character follows the controlled character through doors.
 * If the door cannot be entered, an attempt might be made to force the door.
 */
public final class FollowTask extends StaticTask {
	private final Aftik aftik;
	
	private FollowTarget followTarget;
	private ForceDoorTaskFragment forceFragment;
	
	private record FollowTarget(Reference<Door> door, Reference<Aftik> aftik) {}
	
	public FollowTask(Aftik aftik) {
		this.aftik = aftik;
	}
	
	@Override
	public boolean performAction(ActionPrinter out) {
		if (followTarget != null &&
				(!followTarget.door.isIn(this.aftik.getArea()) || followTarget.aftik.isIn(this.aftik.getArea()))) {
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
			Door door = followTarget.door.getOrThrow(aftik.getArea());
			EnterDoorAction.Result result = EnterDoorAction.moveAndEnter(aftik, door, out);
			
			result.optional().flatMap(enterResult -> enterResult.either().getRight())
					.ifPresentOrElse(failureType -> forceFragment = new ForceDoorTaskFragment(door),
							() -> followTarget = null);
			
			return true;
		} else {
			return false;
		}
	}
	
	@Override
	public void observeEnteredDoor(Aftik aftik, Door door, EnterResult result) {
		if (aftik == aftik.getCrew().getAftik() && result.success()) {
			this.followTarget = new FollowTarget(new Reference<>(door, Door.class), new Reference<>(aftik, Aftik.class));
		}
	}
}
