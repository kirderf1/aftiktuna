package me.kirderf.aftiktuna.object.entity.ai;

import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.object.ObjectTypes;
import me.kirderf.aftiktuna.object.door.Door;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.print.ActionPrinter;

public final class ForceDoorTaskFragment {
	private final Door door;
	private final EnterResult.FailureType failure;
	
	public ForceDoorTaskFragment(Door door, EnterResult.FailureType failure) {
		this.door = door;
		this.failure = failure;
	}
	
	public boolean performAction(Aftik aftik, ActionPrinter out) {
		if (aftik.getArea() == door.getArea() && canForceDoor(aftik)) {
			aftik.moveAndForce(door, out);
			return true;
		} else
			return false;
	}
	
	private boolean canForceDoor(Aftik aftik) {
		if (failure == EnterResult.FailureType.STUCK && aftik.hasItem(ObjectTypes.CROWBAR))
			return true;
		else
			return aftik.hasItem(ObjectTypes.BLOWTORCH);
	}
}
