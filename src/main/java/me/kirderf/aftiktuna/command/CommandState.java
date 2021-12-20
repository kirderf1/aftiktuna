package me.kirderf.aftiktuna.command;

import me.kirderf.aftiktuna.location.Area;
import me.kirderf.aftiktuna.object.Identifier;
import me.kirderf.aftiktuna.object.entity.Aftik;

public final class CommandState {
	private Aftik aftik;
	private Identifier<Area> currentArea, previousArea;
	
	public CommandState() {}
	
	public Aftik getAftik() {
		return aftik;
	}
	
	// Meant to be called right before user input
	public void inputPrepare(Aftik aftik) {
		if (aftik != this.aftik) {
			this.aftik = aftik;
			currentArea = aftik.getArea().getId();
			previousArea = null;
		} else {
			if (!aftik.getArea().getId().equals(currentArea)) {
				previousArea = currentArea;
				currentArea = aftik.getArea().getId();
			}
		}
	}
	
	public Identifier<Area> getPreviousArea() {
		return previousArea;
	}
}