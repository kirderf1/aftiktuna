package me.kirderf.aftiktuna.object.entity.ai;

import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.object.door.Door;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.print.ContextPrinter;

public final class ForceDoorTask extends Task {
	private final Aftik aftik;
	private ForceDoorTaskFragment forceFragment;
	
	public ForceDoorTask(Aftik aftik) {
		this.aftik = aftik;
	}
	
	@Override
	public boolean performAction(ContextPrinter out) {
		if (forceFragment != null) {
			boolean result = forceFragment.performAction(aftik, out);
			forceFragment = null;
			return result;
		} else
			return false;
	}
	
	@Override
	public void observeEnteredDoor(Aftik aftik, Door door, EnterResult result) {
		result.either().getRight().ifPresent(failureType -> forceFragment = new ForceDoorTaskFragment(door, failureType));
	}
}