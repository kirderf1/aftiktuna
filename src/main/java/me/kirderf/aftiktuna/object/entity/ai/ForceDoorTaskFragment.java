package me.kirderf.aftiktuna.object.entity.ai;

import me.kirderf.aftiktuna.action.ForceDoorAction;
import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.object.Reference;
import me.kirderf.aftiktuna.object.door.Door;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.object.type.ObjectTypes;
import me.kirderf.aftiktuna.print.ActionPrinter;

import java.util.Optional;

public final class ForceDoorTaskFragment {
	private final Reference<Door> doorRef;
	
	public ForceDoorTaskFragment(Door door) {
		this.doorRef = new Reference<>(door, Door.class);
	}
	
	public boolean performAction(Aftik aftik, ActionPrinter out) {
		Optional<Door> doorOptional = doorRef.find(aftik.getArea());
		if (doorOptional.isPresent() && canForceDoor(aftik, doorOptional.get())) {
			ForceDoorAction.moveAndForce(aftik, doorOptional.get(), out);
			return true;
		} else
			return false;
	}
	
	private boolean canForceDoor(Aftik aftik, Door door) {
		EnterResult.FailureType failureType = aftik.getMind().getMemory().getObservedFailureType(door);
		if (failureType == EnterResult.FailureType.STUCK && aftik.hasItem(ObjectTypes.CROWBAR))
			return true;
		else
			return aftik.hasItem(ObjectTypes.BLOWTORCH);
	}
}
