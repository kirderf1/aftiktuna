package me.kirderf.aftiktuna.level.object.entity.ai;

import me.kirderf.aftiktuna.ContextPrinter;
import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.level.object.ObjectTypes;
import me.kirderf.aftiktuna.level.object.door.Door;
import me.kirderf.aftiktuna.level.object.entity.Aftik;

public final class ForceDoorTask extends Task {
	private final Aftik aftik;
	private Door forceTarget;
	
	public ForceDoorTask(Aftik aftik) {
		this.aftik = aftik;
	}
	
	@Override
	public boolean performAction(ContextPrinter out) {
		if (forceTarget != null) {
			aftik.moveAndForce(forceTarget, out);
			forceTarget = null;
			return true;
		} else
			return false;
	}
	
	@Override
	public void observeEnteredDoor(Aftik aftik, Door door, EnterResult result) {
		result.either().getRight().ifPresent(failureType -> {
			if (canForceDoor(failureType))
				forceTarget = door;
		});
	}
	
	private boolean canForceDoor(EnterResult.FailureType type) {
		if (type == EnterResult.FailureType.STUCK && aftik.hasItem(ObjectTypes.CROWBAR))
			return true;
		else
			return aftik.hasItem(ObjectTypes.BLOWTORCH);
	}
}