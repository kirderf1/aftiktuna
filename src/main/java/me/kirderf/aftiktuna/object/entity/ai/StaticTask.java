package me.kirderf.aftiktuna.object.entity.ai;

import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.object.door.Door;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.print.ActionPrinter;

public abstract class StaticTask {
	public abstract boolean performAction(ActionPrinter out);
	
	public void observeEnteredDoor(Aftik aftik, Door door, EnterResult result) {}
}