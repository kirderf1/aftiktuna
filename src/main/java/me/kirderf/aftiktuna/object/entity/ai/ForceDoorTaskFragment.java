package me.kirderf.aftiktuna.object.entity.ai;

import me.kirderf.aftiktuna.action.ForceDoorAction;
import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.object.Identifier;
import me.kirderf.aftiktuna.object.ObjectTypes;
import me.kirderf.aftiktuna.object.door.Door;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.print.ActionPrinter;

import java.util.Optional;

public final class ForceDoorTaskFragment {
	private final Identifier doorId;
	private final EnterResult.FailureType failure;
	
	public ForceDoorTaskFragment(Door door, EnterResult.FailureType failure) {
		this.doorId = door.getId();
		this.failure = failure;
	}
	
	public boolean performAction(Aftik aftik, ActionPrinter out) {
		Optional<Door> doorOptional = aftik.getArea().findById(doorId).flatMap(Door.CAST);
		if (doorOptional.isPresent() && aftik.getArea() == doorOptional.get().getArea() && canForceDoor(aftik)) {
			ForceDoorAction.moveAndForce(aftik, doorOptional.get(), out);
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
